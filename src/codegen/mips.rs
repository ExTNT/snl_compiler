//! MIPS 汇编代码生成器。
//!
//! 将经过语法和语义分析的 AST 编译为 MIPS I 汇编代码。
//! 生成的代码可在 SPIM/MARS 等 MIPS 模拟器上运行。
//!
//! ## 运行时约定
//! - **全局变量**: 分配在 `.data` 段，通过 `la` + 标签名访问
//! - **局部变量**: 分配在栈上，通过 `$fp`（帧指针）相对寻址
//! - **过程调用**: 使用 `jal`/`jr $ra`，栈帧保存 `$fp` 和 `$ra`
//! - **参数传递**: 调用者将实参压栈（从右到左），被调用者通过 `$fp + offset` 读取
//! - **返回值**: 统一通过 `$v0` 返回
//! - **临时寄存器**: `$t0` 用于地址计算，`$t7` 用于乘法中间结果，`$t8` 用于全局变量加载
//!
//! ## 类型到 MIPS 的映射
//! - `integer` → 4 字节，`lw`/`sw`
//! - `char` → 4 字节（栈对齐），`lb`/`sb`，元素步长为 1
//! - 数组 → 连续分配 `size * count` 字节，下标访问包含指针运算
//! - 记录 → 字段按声明顺序连续分配，带字段偏移量计算

use std::collections::HashMap;

use crate::ast::nodes::*;
use crate::error::CompileError;

// ===== 代码生成类型表示 =====

/// 代码生成阶段的类型表示。
///
/// 与语义分析阶段的 `TypeInfo` 独立，便于代码生成器
/// 按需扩展。支持类型别名解析。
#[derive(Clone, Debug, PartialEq)]
pub enum CodegenType {
    Integer,
    Char,
    /// 数组类型：元素类型、下界、上界
    Array(Box<CodegenType>, i64, i64),
    /// 记录类型：(字段名, 字段类型) 列表，多名字段已展开
    Record(Vec<(String, CodegenType)>),
}

impl CodegenType {
    /// 计算该类型变量的分配大小（字节）。
    fn size_of(&self) -> i32 {
        match self {
            CodegenType::Integer => 4,
            CodegenType::Char => 4,
            CodegenType::Array(elem, low, high) => {
                let count = (high - low + 1) as i32;
                count * elem.element_byte_size()
            }
            CodegenType::Record(fields) => fields.iter().map(|(_, t)| t.size_of()).sum(),
        }
    }

    /// 数组下标运算时的元素步长（字节）。
    fn element_byte_size(&self) -> i32 {
        match self {
            CodegenType::Integer => 4,
            CodegenType::Char => 1,
            CodegenType::Array(elem, _, _) => elem.element_byte_size(),
            CodegenType::Record(_) => {
                // Record types should never reach this; fallback
                0
            }
        }
    }

    /// 查找记录字段的偏移量和类型。
    fn field_offset(&self, name: &str) -> Option<(i32, CodegenType)> {
        if let CodegenType::Record(fields) = self {
            let mut offset = 0i32;
            for (fname, ft) in fields {
                if fname == name {
                    return Some((offset, ft.clone()));
                }
                offset += ft.size_of();
            }
        }
        None
    }
}

// ===== AST 类型 → 代码生成类型映射 =====

/// 构建类型别名映射表。
///
/// 将 TypeDec 中的命名类型展平为 `名字 → TypeBody` 的映射。
fn build_type_alias_map(type_dec: &TypeDec) -> HashMap<String, TypeBody> {
    let mut aliases = HashMap::new();
    if let TypeDec::Defined(defs) = type_dec {
        for def in defs {
            aliases.insert(def.name.clone(), def.body.clone());
        }
    }
    aliases
}

/// 将 AST 的 TypeBody 转换为代码生成类型。
///
/// `visited` 用于检测循环类型别名（例如 `type A = B; type B = A`）。
fn type_body_to_codegen(
    body: &TypeBody,
    aliases: &HashMap<String, TypeBody>,
    visited: &mut Vec<String>,
    errors: &mut Vec<CompileError>,
) -> CodegenType {
    match body {
        TypeBody::Base(BaseType::Integer) => CodegenType::Integer,
        TypeBody::Base(BaseType::Char) => CodegenType::Char,
        TypeBody::Array(arr) => {
            let elem = match arr.elem_type {
                BaseType::Integer => CodegenType::Integer,
                BaseType::Char => CodegenType::Char,
            };
            CodegenType::Array(Box::new(elem), arr.low, arr.high)
        }
        TypeBody::Record(rec) => {
            let fields: Vec<(String, CodegenType)> = rec
                .fields
                .iter()
                .flat_map(|f| {
                    let ft = field_type_to_codegen(&f.typ);
                    f.names.iter().map(move |n| (n.clone(), ft.clone()))
                })
                .collect();
            CodegenType::Record(fields)
        }
        TypeBody::Named(name) => {
            if visited.contains(name) {
                errors.push(CompileError::codegen(format!("Circular type alias: {}", name)));
                return CodegenType::Integer;
            }
            visited.push(name.clone());
            let resolved = match aliases.get(name) {
                Some(body) => body,
                None => {
                    errors.push(CompileError::codegen(format!("Undefined type alias: {}", name)));
                    visited.pop();
                    return CodegenType::Integer;
                }
            };
            let result = type_body_to_codegen(resolved, aliases, visited, errors);
            visited.pop();
            result
        }
    }
}

fn type_desig_to_codegen(td: &TypeDesig, aliases: &HashMap<String, TypeBody>, errors: &mut Vec<CompileError>) -> CodegenType {
    match td {
        TypeDesig::Base(BaseType::Integer) => CodegenType::Integer,
        TypeDesig::Base(BaseType::Char) => CodegenType::Char,
        TypeDesig::Array(arr) => {
            let elem = match arr.elem_type {
                BaseType::Integer => CodegenType::Integer,
                BaseType::Char => CodegenType::Char,
            };
            CodegenType::Array(Box::new(elem), arr.low, arr.high)
        }
        TypeDesig::Record(rec) => {
            let fields: Vec<(String, CodegenType)> = rec
                .fields
                .iter()
                .flat_map(|f| {
                    let ft = field_type_to_codegen(&f.typ);
                    f.names.iter().map(move |n| (n.clone(), ft.clone()))
                })
                .collect();
            CodegenType::Record(fields)
        }
        TypeDesig::Named(name) => {
            let mut visited = vec![name.clone()];
            let body = match aliases.get(name) {
                Some(body) => body,
                None => {
                    errors.push(CompileError::codegen(format!("Undefined type alias: {}", name)));
                    return CodegenType::Integer;
                }
            };
            type_body_to_codegen(body, aliases, &mut visited, errors)
        }
    }
}

fn field_type_to_codegen(ft: &FieldTypeDef) -> CodegenType {
    match ft {
        FieldTypeDef::Base(BaseType::Integer) => CodegenType::Integer,
        FieldTypeDef::Base(BaseType::Char) => CodegenType::Char,
        FieldTypeDef::Array(arr) => {
            let elem = match arr.elem_type {
                BaseType::Integer => CodegenType::Integer,
                BaseType::Char => CodegenType::Char,
            };
            CodegenType::Array(Box::new(elem), arr.low, arr.high)
        }
    }
}

// ===== MIPS 上下文 =====

/// MIPS 代码生成上下文。
///
/// 维护汇编代码字符串、数据段字符串、标签计数器、
/// 变量偏移量表（嵌套作用域）、变量类型表以及帧大小栈。
pub struct MipsContext {
    pub code: String,
    pub data: String,
    label_counter: usize,
    var_offsets: Vec<HashMap<String, (i32, usize)>>,
    var_types: Vec<HashMap<String, CodegenType>>,
    frame_sizes: Vec<i32>,
    nesting_level: usize,
    errors: Vec<CompileError>,
}

impl Default for MipsContext {
    fn default() -> Self {
        Self::new()
    }
}

impl MipsContext {
    pub fn new() -> Self {
        MipsContext {
            code: String::new(),
            data: String::new(),
            label_counter: 0,
            var_offsets: vec![HashMap::new()],
            var_types: vec![HashMap::new()],
            frame_sizes: vec![4],
            nesting_level: 0,
            errors: Vec::new(),
        }
    }

    fn error(&mut self, msg: impl Into<String>) {
        self.errors.push(CompileError::codegen(msg));
    }

    /// 生成唯一标签（如 `else_0`、`endif_1`）。
    pub fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    fn current_scope(&self) -> &HashMap<String, (i32, usize)> {
        self.var_offsets.last().expect("var_offsets should never be empty")
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<String, (i32, usize)> {
        self.var_offsets.last_mut().expect("var_offsets should never be empty")
    }

    /// 在当前作用域中为变量分配栈空间。
    ///
    /// 若变量已在当前作用域声明则跳过。
    pub fn alloc_var(&mut self, name: &str, typ: &CodegenType) {
        if !self.current_scope().contains_key(name) {
            let offset = *self.frame_sizes.last().expect("frame_sizes should never be empty");
            let level = self.nesting_level;
            self.current_scope_mut()
                .insert(name.to_string(), (offset, level));
            self.var_types
                .last_mut()
                .expect("var_types should never be empty")
                .insert(name.to_string(), typ.clone());
            *self.frame_sizes.last_mut().expect("frame_sizes should never be empty") += typ.size_of();
        }
    }

    /// 查找变量的栈偏移和声明层级。
    ///
    /// 从最内层作用域向外搜索。
    pub fn get_var_offset(&self, name: &str) -> Option<(i32, usize)> {
        for scope in self.var_offsets.iter().rev() {
            if let Some(&val) = scope.get(name) {
                return Some(val);
            }
        }
        None
    }

    /// 查找变量的代码生成类型。
    pub fn get_var_type(&self, name: &str) -> Option<&CodegenType> {
        for scope in self.var_types.iter().rev() {
            if let Some(typ) = scope.get(name) {
                return Some(typ);
            }
        }
        None
    }

    /// 进入过程作用域：初始化新的偏移量和类型映射。
    pub fn enter_proc(&mut self) {
        self.nesting_level += 1;
        self.var_offsets.push(HashMap::new());
        self.var_types.push(HashMap::new());
        // 预留 $fp 和 $ra 各 4 字节
        self.frame_sizes.push(8);
    }

    /// 退出过程作用域。
    pub fn exit_proc(&mut self) {
        self.nesting_level -= 1;
        self.var_offsets.pop();
        self.var_types.pop();
        self.frame_sizes.pop();
    }

    pub fn nesting_level(&self) -> usize {
        self.nesting_level
    }

    pub fn frame_size(&self) -> i32 {
        *self.frame_sizes.last().expect("frame_sizes should never be empty")
    }

    /// 向代码段追加一条指令。
    pub fn emit(&mut self, s: &str) {
        self.code.push_str(s);
        self.code.push('\n');
    }

    /// 向代码段追加一个标签。
    pub fn emit_label(&mut self, label: &str) {
        self.code.push_str(&format!("{}:\n", label));
    }

    /// 向数据段追加一行。
    pub fn emit_data(&mut self, s: &str) {
        self.data.push_str(s);
        self.data.push('\n');
    }
}

/// 生成加载变量值到 `$v0` 的 MIPS 代码。
fn emit_load(ctx: &mut MipsContext, offset: i32, var_level: usize, name: &str, typ: &CodegenType) {
    let instr = if matches!(typ, CodegenType::Char) { "lb" } else { "lw" };
    if var_level == 0 {
        ctx.emit(&format!("  la $t8, var_{}", name));
        ctx.emit(&format!("  {} $v0, 0($t8)         # load global {}", instr, name));
    } else {
        let off_str = if offset >= 0 {
            format!("-{}", offset)
        } else {
            format!("{}", -offset)
        };
        ctx.emit(&format!("  {} $v0, {}({})       # load {}", instr, off_str, "$fp", name));
    }
}

/// 生成将 `$v0` 存回变量的 MIPS 代码。
fn emit_store(ctx: &mut MipsContext, offset: i32, var_level: usize, name: &str, typ: &CodegenType) {
    let instr = if matches!(typ, CodegenType::Char) { "sb" } else { "sw" };
    if var_level == 0 {
        ctx.emit(&format!("  la $t8, var_{}", name));
        ctx.emit(&format!("  {} $v0, 0($t8)         # store to global {}", instr, name));
    } else {
        let off_str = if offset >= 0 {
            format!("-{}", offset)
        } else {
            format!("{}", -offset)
        };
        ctx.emit(&format!("  {} $v0, {}({})       # store to {}", instr, off_str, "$fp", name));
    }
}

// ===== 主编译入口 =====

/// 将程序编译为 MIPS 汇编代码字符串。
///
/// 返回完整的 `.data` + `.text` 段汇编代码，若代码生成过程中
/// 遇到错误则返回错误列表。
pub fn compile(prog: &Program) -> Result<String, Vec<CompileError>> {
    let mut ctx = MipsContext::new();

    // 构建全局类型别名映射
    let global_aliases = build_type_alias_map(&prog.decl.types);

    // 数据段：分配全局变量
    ctx.emit_data("newline: .asciiz \"\\n\"");
    for var_dec in var_decs(&prog.decl.vars) {
        let resolved = type_desig_to_codegen(&var_dec.type_name, &global_aliases, &mut ctx.errors);
        let size = resolved.size_of();
        for name in &var_dec.names {
            if size == 4 {
                ctx.emit_data(&format!("var_{}: .word 0", name));
            } else {
                ctx.emit_data("  .align 2");
                ctx.emit_data(&format!("var_{}: .space {}", name, size));
            }
            // 全局变量偏移量统一为 0，层级为 0
            ctx.current_scope_mut().insert(name.to_string(), (0, 0));
            ctx.var_types
                .last_mut()
                .expect("var_types should never be empty")
                .insert(name.to_string(), resolved.clone());
        }
    }

    // 生成 main 标号及序言
    ctx.emit("main:");
    ctx.emit("  addiu $sp, $sp, -4     # space for $ra");
    ctx.emit("  sw $ra, 0($sp)         # save return address");
    ctx.emit("  move $fp, $sp          # frame pointer");

    let frame = ctx.frame_size();
    if frame > 0 {
        ctx.emit(&format!(
            "  addiu $sp, $sp, -{}     # local variables",
            frame
        ));
    }

    // 编译全局过程体
    compile_stm_list(&prog.body.stmts, &mut ctx);

    // 尾言
    ctx.emit("  li $v0, 10             # exit syscall");
    ctx.emit("  syscall");
    ctx.emit("");

    // 编译过程声明
    compile_procs(&prog.decl.procs, &mut ctx, &global_aliases);

    if ctx.errors.is_empty() {
        Ok(format!(
            ".data\n{}\n.text\n.globl main\n{}",
            ctx.data, ctx.code
        ))
    } else {
        Err(ctx.errors)
    }
}

/// 递归编译过程声明。
///
/// 每个过程生成独立的标号（`proc_<name>`）、栈帧管理序言/尾言和过程体。
/// 类型别名与父作用域合并，支持嵌套过程。
fn compile_procs(
    proc_dec: &ProcDec,
    ctx: &mut MipsContext,
    parent_aliases: &HashMap<String, TypeBody>,
) {
    if let ProcDec::Defined(procs) = proc_dec {
        for proc in procs {
            let label = format!("proc_{}", proc.name);
            ctx.emit("");
            ctx.emit_label(&label);

            ctx.enter_proc();

            // 序言
            ctx.emit("  addiu $sp, $sp, -8     # space for $fp + $ra");
            ctx.emit("  sw $fp, 0($sp)         # save old $fp");
            ctx.emit("  sw $ra, 4($sp)         # save return address");
            ctx.emit("  move $fp, $sp          # frame pointer");

            // 合并局部类型定义与继承的类型别名
            let mut proc_aliases = parent_aliases.clone();
            for (name, body) in build_type_alias_map(&proc.decl.types) {
                proc_aliases.insert(name, body);
            }

            // 分配形参（在调用者的帧中，位于 $fp 之上）
            let proc_level = ctx.nesting_level();
            for (i, param) in proc.params.iter().enumerate() {
                let param_type = type_desig_to_codegen(&param.type_name, &proc_aliases, &mut ctx.errors);
                for name in &param.names {
                    // 形参位于前 8 字节（$fp + $ra）之上
                    let offset = -(i as i32 * 4 + 8);
                    ctx.current_scope_mut()
                        .insert(name.clone(), (offset, proc_level));
                    ctx.var_types
                        .last_mut()
                        .expect("var_types should never be empty")
                        .insert(name.clone(), param_type.clone());
                }
            }

            // 局部变量声明
            for var_dec in var_decs(&proc.decl.vars) {
                let resolved = type_desig_to_codegen(&var_dec.type_name, &proc_aliases, &mut ctx.errors);
                for name in &var_dec.names {
                    ctx.alloc_var(name, &resolved);
                }
            }

            let frame = ctx.frame_size();
            if frame > 0 {
                ctx.emit(&format!("  addiu $sp, $sp, -{}     # locals", frame));
            }

            // 过程体
            compile_stm_list(&proc.body.stmts, ctx);

            // 尾言
            if frame > 0 {
                ctx.emit(&format!(
                    "  addiu $sp, $sp, {}      # deallocate locals",
                    frame
                ));
            }
            ctx.emit("  lw $fp, 0($sp)         # restore old $fp");
            ctx.emit("  lw $ra, 4($sp)         # restore $ra");
            ctx.emit("  addiu $sp, $sp, 8      # deallocate $fp + $ra slots");
            ctx.emit("  jr $ra                  # return");

            // 嵌套过程
            compile_procs(&proc.decl.procs, ctx, &proc_aliases);

            ctx.exit_proc();
        }
    }
}

// ===== 选择器地址计算 =====

/// 生成 MIPS 代码以计算 VarAccess 的运行时地址到 `$t0`。
///
/// 处理基础变量（全局/局部）、数组下标和记录字段。
/// 返回最终元素的标量 CodegenType。
fn emit_var_address(va: &VarAccess, ctx: &mut MipsContext) -> CodegenType {
    let (offset, var_level) = ctx
        .get_var_offset(&va.base)
        .unwrap_or_else(|| {
            ctx.error(format!("Unknown variable '{}'", va.base));
            (0, 0)
        });
    let current_typ = ctx
        .get_var_type(&va.base)
        .cloned()
        .unwrap_or_else(|| {
            ctx.error(format!("No type for '{}'", va.base));
            CodegenType::Integer
        });

    // 将基础地址加载到 $t0
    if var_level == 0 {
        ctx.emit(&format!("  la $t0, var_{}", va.base));
    } else {
        ctx.emit(&format!("  addiu $t0, $fp, {}", -offset));
    }

    walk_selectors(&va.selector, ctx, current_typ)
}

/// 遍历选择器链，生成代码更新 `$t0` 指向最终元素。
///
/// 返回所选元素的标量类型。
fn walk_selectors(
    selectors: &[Selector],
    ctx: &mut MipsContext,
    mut current_typ: CodegenType,
) -> CodegenType {
    for sel in selectors {
        match sel {
            Selector::ArraySubscript(exp) => {
                let (elem_type, low_val) = match &current_typ {
                    CodegenType::Array(elem, low, _) => (*elem.clone(), *low),
                    _ => {
                        ctx.error("Array subscript on non-array type");
                        return CodegenType::Integer;
                    }
                };
                ctx.emit("  addiu $sp, $sp, -4");
                ctx.emit("  sw $t0, 0($sp)          # save base address");
                compile_exp(exp, ctx);
                // 下标从下界开始，若下界非零则减去偏移
                if low_val != 0 {
                    ctx.emit(&format!("  addiu $v0, $v0, {}", -low_val));
                }
                let elem_size = elem_type.element_byte_size();
                if elem_size == 4 {
                    ctx.emit("  sll $v0, $v0, 2");
                }
                ctx.emit("  lw $t0, 0($sp)          # restore base address");
                ctx.emit("  addiu $sp, $sp, 4");
                ctx.emit("  addu $t0, $t0, $v0");
                current_typ = elem_type;
            }
            Selector::Field(name) => {
                let (field_offset, field_type) = current_typ
                    .field_offset(name)
                    .unwrap_or_else(|| {
                        ctx.error(format!("Field '{}' not found in record", name));
                        (0, CodegenType::Integer)
                    });
                ctx.emit(&format!("  addiu $t0, $t0, {}", field_offset));
                current_typ = field_type;
            }
            Selector::FieldSubscript(name, exp) => {
                let (field_offset, field_type) = current_typ
                    .field_offset(name)
                    .unwrap_or_else(|| {
                        ctx.error(format!("Field '{}' not found in record", name));
                        (0, CodegenType::Integer)
                    });
                ctx.emit(&format!("  addiu $t0, $t0, {}", field_offset));
                let (elem_type, low_val) = match &field_type {
                    CodegenType::Array(elem, low, _) => (*elem.clone(), *low),
                    _ => {
                        ctx.error("FieldSubscript on non-array field");
                        return CodegenType::Integer;
                    }
                };
                ctx.emit("  addiu $sp, $sp, -4");
                ctx.emit("  sw $t0, 0($sp)          # save base address");
                compile_exp(exp, ctx);
                if low_val != 0 {
                    ctx.emit(&format!("  addiu $v0, $v0, {}", -low_val));
                }
                let elem_size = elem_type.element_byte_size();
                if elem_size == 4 {
                    ctx.emit("  sll $v0, $v0, 2");
                }
                ctx.emit("  lw $t0, 0($sp)          # restore base address");
                ctx.emit("  addiu $sp, $sp, 4");
                ctx.emit("  addu $t0, $t0, $v0");
                current_typ = elem_type;
            }
        }
    }
    current_typ
}

// ===== 语句编译 =====

/// 编译语句列表。
fn compile_stm_list(stmts: &[Stm], ctx: &mut MipsContext) {
    for stm in stmts {
        compile_stm(stm, ctx);
    }
}

/// 编译单条语句。
fn compile_stm(stm: &Stm, ctx: &mut MipsContext) {
    match stm {
        Stm::Assign { lhs, rhs, .. } => {
            compile_exp(rhs, ctx);
            // 无选择器的简单赋值
            if lhs.selector.is_empty() {
                if let Some((offset, var_level)) = ctx.get_var_offset(&lhs.base) {
                    let is_char = ctx
                        .get_var_type(&lhs.base)
                        .is_some_and(|t| matches!(t, CodegenType::Char));
                    let store_typ = if is_char { &CodegenType::Char } else { &CodegenType::Integer };
                    emit_store(ctx, offset, var_level, &lhs.base, store_typ);
                }
            } else {
                // 带选择器（数组下标/记录字段）的赋值：
                // 1. 保存 RHS 值
                // 2. 计算 LHS 地址
                // 3. 恢复 RHS 值并存入计算出的地址
                ctx.emit("  addiu $sp, $sp, -4");
                ctx.emit("  sw $v0, 0($sp)          # save rhs value");
                let lhs_type = emit_var_address(lhs, ctx);
                ctx.emit("  lw $v0, 0($sp)          # restore rhs value");
                ctx.emit("  addiu $sp, $sp, 4");
                if lhs_type == CodegenType::Char {
                    ctx.emit("  sb $v0, 0($t0)");
                } else {
                    ctx.emit("  sw $v0, 0($t0)");
                }
            }
        }
        Stm::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            let else_label = ctx.new_label("else");
            let end_label = ctx.new_label("endif");
            let _ = compile_exp(cond, ctx);
            ctx.emit(&format!("  beqz $v0, {}", else_label));
            compile_stm_list(&then_branch.stmts, ctx);
            ctx.emit(&format!("  j {}", end_label));
            ctx.emit_label(&else_label);
            compile_stm_list(&else_branch.stmts, ctx);
            ctx.emit_label(&end_label);
        }
        Stm::While { cond, body, .. } => {
            let loop_label = ctx.new_label("loop");
            let end_label = ctx.new_label("endloop");
            ctx.emit_label(&loop_label);
            let _ = compile_exp(cond, ctx);
            ctx.emit(&format!("  beqz $v0, {}", end_label));
            compile_stm_list(&body.stmts, ctx);
            ctx.emit(&format!("  j {}", loop_label));
            ctx.emit_label(&end_label);
        }
        Stm::Read { var, .. } => {
            let is_char = ctx
                .get_var_type(var)
                .is_some_and(|t| matches!(t, CodegenType::Char));
            if is_char {
                ctx.emit("  li $v0, 12             # read char syscall");
            } else {
                ctx.emit("  li $v0, 5              # read int syscall");
            }
            ctx.emit("  syscall");
            if let Some((offset, var_level)) = ctx.get_var_offset(var) {
                let store_typ = if is_char { &CodegenType::Char } else { &CodegenType::Integer };
                emit_store(ctx, offset, var_level, var, store_typ);
            }
        }
        Stm::Write { exp, .. } => {
            let typ = compile_exp(exp, ctx);
            ctx.emit("  move $a0, $v0          # value to print");
            if typ == CodegenType::Char {
                ctx.emit("  li $v0, 11             # print char syscall");
            } else {
                ctx.emit("  li $v0, 1              # print int syscall");
            }
            ctx.emit("  syscall");
            ctx.emit("  la $a0, newline");
            ctx.emit("  li $v0, 4              # print string syscall");
            ctx.emit("  syscall");
        }
        Stm::Return { exp, .. } => {
            let _ = compile_exp(exp, ctx);
        }
        Stm::Call { name, args, .. } => {
            // 实参从右到左压栈（第一个实参最后压入，在栈顶）
            for arg in args.iter().rev() {
                let _ = compile_exp(arg, ctx);
                ctx.emit("  addiu $sp, $sp, -4");
                ctx.emit("  sw $v0, 0($sp)");
            }
            ctx.emit(&format!("  jal proc_{}", name));
            if !args.is_empty() {
                ctx.emit(&format!("  addiu $sp, $sp, {}", args.len() as i32 * 4));
            }
        }
    }
}

// ===== 表达式编译 =====

/// 编译表达式，结果存入 `$v0`。
///
/// 返回表达式结果类型的 `CodegenType`。
fn compile_exp(exp: &Exp, ctx: &mut MipsContext) -> CodegenType {
    match exp {
        Exp::Binary {
            op, left, right, ..
        } => {
            // 先求值右操作数，压栈保存
            let _ = compile_exp(right, ctx);
            ctx.emit("  addiu $sp, $sp, -4");
            ctx.emit("  sw $v0, 0($sp)          # push right");
            let _ = compile_exp(left, ctx);
            ctx.emit("  lw $t0, 0($sp)          # pop right");
            ctx.emit("  addiu $sp, $sp, 4");
            match op {
                BinOp::Add => ctx.emit("  addu $v0, $v0, $t0"),
                BinOp::Sub => ctx.emit("  subu $v0, $v0, $t0"),
                // mul 结果在 $t7 中，需再 move 到 $v0
                BinOp::Mul => ctx.emit("  mul $v0, $v0, $t0"),
                BinOp::Div => {
                    ctx.emit("  div $v0, $v0, $t0");
                    ctx.emit("  mflo $v0");
                }
                BinOp::Lt => ctx.emit("  slt $v0, $v0, $t0"),
                BinOp::Eq => {
                    // 相等比较使用条件分支生成 0 或 1
                    let true_label = ctx.new_label("eq_true");
                    let end_label = ctx.new_label("eq_end");
                    ctx.emit(&format!("  beq $v0, $t0, {}", true_label));
                    ctx.emit("  li $v0, 0");
                    ctx.emit(&format!("  j {}", end_label));
                    ctx.emit_label(&true_label);
                    ctx.emit("  li $v0, 1");
                    ctx.emit_label(&end_label);
                }
            }
            CodegenType::Integer
        }
        Exp::IntConst(n, _) => {
            ctx.emit(&format!("  li $v0, {}", n));
            CodegenType::Integer
        }
        Exp::CharConst(c, _) => {
            ctx.emit(&format!("  li $v0, {}", *c as i32));
            CodegenType::Char
        }
        Exp::Variable(va, _) => {
            if va.selector.is_empty() {
                if let Some((offset, var_level)) = ctx.get_var_offset(&va.base) {
                    let is_char = ctx
                        .get_var_type(&va.base)
                        .is_some_and(|t| matches!(t, CodegenType::Char));
                    let load_typ = if is_char { &CodegenType::Char } else { &CodegenType::Integer };
                    emit_load(ctx, offset, var_level, &va.base, load_typ);
                    return if is_char {
                        CodegenType::Char
                    } else {
                        CodegenType::Integer
                    };
                }
                CodegenType::Integer
            } else {
                // 带选择器的变量：先计算地址，再加载值
                let final_typ = emit_var_address(va, ctx);
                if final_typ == CodegenType::Char {
                    ctx.emit("  lb $v0, 0($t0)");
                } else {
                    ctx.emit("  lw $v0, 0($t0)");
                }
                final_typ
            }
        }
    }
}

/// 提取 VarDec 中的 VarDef 切片。
fn var_decs(var_dec: &VarDec) -> &[VarDef] {
    match var_dec {
        VarDec::Empty => &[],
        VarDec::Defined(defs) => defs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::rd::RdParser;

    fn compile_source(source: &str) -> String {
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize(source);
        let mut parser = RdParser::new(tokens);
        let prog = parser.parse().expect("Parse should succeed");
        compile(&prog).expect("Codegen should succeed")
    }

    #[test]
    fn test_simple_mips_output() {
        let asm = compile_source("program p var integer x; begin x := 1; write(x) end.");
        assert!(asm.contains("main:"), "Should have main label");
        assert!(asm.contains("syscall"), "Should have syscall instructions");
    }

    #[test]
    fn test_if_statement_codegen() {
        let asm = compile_source(
            "program p var integer x; begin if x < 10 then x := 1 else x := 2 fi end.",
        );
        assert!(asm.contains("beqz"), "Should have branch");
    }

    #[test]
    fn test_while_statement_codegen() {
        let asm = compile_source("program p begin while 1 do write(0) endwh end.");
        assert!(asm.contains("loop_"), "Should have loop label");
    }

    #[test]
    fn test_arithmetic_expression_codegen() {
        let asm =
            compile_source("program p var integer x; integer y; begin x := 10; y := x + 5 end.");
        assert!(asm.contains("addu"), "Should have add instruction");
        assert!(asm.contains("main:"), "Should have main label");
    }

    #[test]
    fn test_subtraction_codegen() {
        let asm = compile_source("program p var integer x; begin x := 10 - 3 end.");
        assert!(asm.contains("subu"), "Should have subu instruction");
    }

    #[test]
    fn test_multiplication_codegen() {
        let asm = compile_source("program p var integer x; begin x := 3 * 5 end.");
        assert!(asm.contains("mul"), "Should have mul instruction");
    }

    #[test]
    fn test_char_const_codegen() {
        let asm = compile_source("program p var char c; begin c := 'a' end.");
        assert!(asm.contains("97"), "Should have ASCII value 97 for 'a'");
        assert!(asm.contains("sb"), "Should use sb for char store");
    }

    #[test]
    fn test_char_load_store() {
        let asm = compile_source("program p var char c; char d; begin c := 'x'; d := c end.");
        assert!(asm.contains("lb"), "Should use lb for char load");
        assert!(asm.contains("sb"), "Should use sb for char store");
    }

    #[test]
    fn test_char_read_write_syscalls() {
        let asm = compile_source("program p var char c; begin read(c); write(c) end.");
        assert!(asm.contains("li $v0, 12"), "Should use read char syscall");
        assert!(asm.contains("li $v0, 11"), "Should use print char syscall");
    }

    #[test]
    fn test_char_const_write() {
        let asm = compile_source("program p begin write('a') end.");
        assert!(
            asm.contains("li $v0, 11"),
            "Should use print char syscall for char const"
        );
    }

    #[test]
    fn test_int_array_allocation_and_access() {
        let asm = compile_source(
            "program p var array[1..5] of integer a; begin a[1] := 42; write(a[1]) end.",
        );
        assert!(asm.contains(".space 20"), "Should allocate 5*4=20 bytes");
        assert!(asm.contains("sll"), "Should shift for int array index");
        assert!(asm.contains("lw"), "Should use lw for int array load");
        assert!(asm.contains("sw"), "Should use sw for int array store");
    }

    #[test]
    fn test_int_array_nonzero_low_bound() {
        let asm = compile_source("program p var array[5..10] of integer a; begin a[5] := 1 end.");
        assert!(
            asm.contains("addiu $v0, $v0, -5"),
            "Should subtract low bound 5"
        );
    }

    #[test]
    fn test_char_array_allocation_and_access() {
        let asm = compile_source("program p var array[0..9] of char s; begin s[0] := 'a' end.");
        assert!(asm.contains(".space 10"), "Should allocate 10 bytes");
        assert!(asm.contains("sb"), "Should use sb for char array store");
    }

    #[test]
    fn test_record_field_access() {
        let asm = compile_source(
            "program p var record integer a; char b end r; begin r.a := 1; r.b := 'z'; write(r.a) end.",
        );
        assert!(
            asm.contains("addiu $t0, $t0, 0"),
            "Should have field offset for a"
        );
        assert!(
            asm.contains("addiu $t0, $t0, 4"),
            "Should have field offset 4 for b"
        );
        assert!(asm.contains("sw"), "Should use sw for int field");
        assert!(asm.contains("sb"), "Should use sb for char field");
    }

    #[test]
    fn test_record_with_array_field() {
        let asm = compile_source(
            "program p var record array[0..4] of integer a end r; begin r.a[2] := 99; write(r.a[2]) end.",
        );
        assert!(
            asm.contains("addiu $t0, $t0, 0"),
            "Should have field offset"
        );
        assert!(asm.contains("sll"), "Should shift for int array");
        assert!(asm.contains("lw"), "Should use lw for int array in record");
    }

    #[test]
    fn test_type_alias_to_integer() {
        let asm = compile_source("program p type T = integer; var T x; begin x := 5 end.");
        assert!(
            asm.contains(".word 0"),
            "Should allocate word for aliased integer"
        );
        assert!(asm.contains("sw"), "Should use sw for aliased integer");
    }

    #[test]
    fn test_type_alias_to_char() {
        let asm = compile_source("program p type C = char; var C c; begin c := 'a' end.");
        assert!(asm.contains("sb"), "Should use sb for aliased char");
    }

    #[test]
    fn test_type_alias_to_array() {
        let asm = compile_source(
            "program p type Arr = array[0..9] of char; var Arr s; begin s[0] := 'a' end.",
        );
        assert!(asm.contains(".space 10"), "Should allocate 10 bytes");
        assert!(asm.contains("sb"), "Should use sb for aliased char array");
    }

    #[test]
    fn test_nested_proc_with_local_array() {
        let asm = compile_source(
            "program p procedure f(); var array[0..3] of char buf; begin buf[0] := 'a' end begin f() end.",
        );
        assert!(asm.contains("proc_f:"), "Should have procedure label");
        assert!(asm.contains("sb"), "Should use sb for local char array");
    }

    #[test]
    fn test_record_multi_name_fields() {
        let asm = compile_source(
            "program p var record integer x, y end r; begin r.x := 1; r.y := 2 end.",
        );
        assert!(
            asm.contains("addiu $t0, $t0, 0"),
            "Should have field offset 0 for x"
        );
        assert!(
            asm.contains("addiu $t0, $t0, 4"),
            "Should have field offset 4 for y"
        );
    }

    #[test]
    fn test_integer_write_still_uses_syscall_1() {
        let asm = compile_source("program p var integer x; begin x := 42; write(x) end.");
        assert!(
            asm.contains("li $v0, 1"),
            "Should use print int syscall for integer"
        );
    }

    #[test]
    fn test_integer_read_still_uses_syscall_5() {
        let asm = compile_source("program p var integer x; begin read(x); write(x) end.");
        assert!(
            asm.contains("li $v0, 5"),
            "Should use read int syscall for integer"
        );
    }

    #[test]
    fn test_array_assignment_with_subscript_lhs() {
        let asm = compile_source("program p var array[0..5] of integer a; begin a[3] := 99 end.");
        assert!(asm.contains("save rhs value"), "Should save rhs");
        assert!(asm.contains("restore rhs value"), "Should restore rhs");
    }

    #[test]
    fn test_equality_in_condition_codegen() {
        let asm = compile_source(
            "program p var integer x; begin if x = 0 then write(1) else write(2) fi end.",
        );
        assert!(asm.contains("beq"), "Should have beq for equality");
    }

    #[test]
    fn test_less_than_in_condition_codegen() {
        let asm = compile_source(
            "program p var integer x; begin if x < 2 then write(1) else write(2) fi end.",
        );
        assert!(asm.contains("slt"), "Should have slt for less-than");
    }

    #[test]
    fn test_read_statement_codegen() {
        let asm = compile_source("program p var integer x; begin read(x) end.");
        assert!(asm.contains("li $v0, 5"), "Should have read int syscall");
    }

    #[test]
    fn test_write_statement_codegen() {
        let asm = compile_source("program p begin write(42) end.");
        assert!(asm.contains("li $v0, 1"), "Should have print int syscall");
        assert!(
            asm.contains("li $v0, 4"),
            "Should have print string syscall"
        );
    }

    #[test]
    fn test_return_statement_codegen() {
        let asm = compile_source("program p procedure f(); begin return(1) end begin f() end.");
        assert!(asm.contains("proc_f:"), "Should have procedure label");
    }

    #[test]
    fn test_procedure_call_codegen() {
        let asm =
            compile_source("program p procedure f(integer a); begin write(a) end begin f(42) end.");
        assert!(asm.contains("jal proc_f"), "Should have jal instruction");
        assert!(asm.contains("proc_f:"), "Should have procedure label");
    }

    #[test]
    fn test_global_variable_in_procedure_codegen() {
        let asm = compile_source(
            "program p var integer r; procedure set(); begin r := 42 end begin set(); write(r) end.",
        );
        assert!(
            asm.contains("var_r"),
            "Should have global variable in .data"
        );
        assert!(asm.contains("proc_set:"), "Should have procedure label");
        assert!(asm.contains("jal proc_set"), "Should have jal");
    }

    #[test]
    fn test_fp_save_restore_codegen() {
        let asm = compile_source(
            "program p var integer r; procedure inc(integer n); begin r := n + 1 end begin inc(5); write(r) end.",
        );
        assert!(asm.contains("sw $fp, 0($sp)"), "Should save $fp");
        assert!(asm.contains("lw $fp, 0($sp)"), "Should restore $fp");
    }

    #[test]
    fn test_data_section_has_newline() {
        let asm = compile_source("program p begin write(1) end.");
        assert!(asm.contains("newline:"), "Should have newline string");
        assert!(asm.contains(".asciiz"), "Should have asciiz directive");
    }

    #[test]
    fn test_program_exit_syscall() {
        let asm = compile_source("program p begin write(1) end.");
        assert!(asm.contains("li $v0, 10"), "Should have exit syscall");
    }

    #[test]
    fn test_nested_if_else_codegen() {
        let asm = compile_source(
            "program p var integer x; begin if x < 10 then if x < 5 then x := 1 else x := 2 fi else x := 3 fi end.",
        );
        assert!(asm.contains("beqz"), "Should have branch instructions");
    }

    #[test]
    fn test_global_variable_data_section() {
        let asm =
            compile_source("program p var integer result; begin result := 0; write(result) end.");
        assert!(
            asm.contains("var_result:"),
            "Should allocate result in .data"
        );
        assert!(asm.contains(".word 0"), "Should initialize to zero");
    }
}
