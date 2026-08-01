//! MIPS 汇编代码生成器。
//!
//! 将经过语法和语义分析的 AST 编译为 MIPS I 汇编代码。
//! 生成的代码可在 SPIM/MARS 等 MIPS 模拟器上运行。
//!
//! ## 运行时约定
//! - **全局变量**: 分配在 `.data` 段，通过 `la` + 标签名访问
//! - **局部变量**: 分配在栈上，通过 `$fp`（帧指针）相对寻址
//! - **过程调用**: 使用 `jal`/`jr $ra`，栈帧保存 `$fp`、`$ra` 和静态链
//! - **参数传递**: 标量按值、复合值完整复制、`var` 传地址，调用者从右到左压栈
//! - **返回值**: 统一通过 `$v0` 返回
//! - **临时寄存器**: `$t0` 用于地址计算，`$t1`/`$t2` 用于复合值复制
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
    fn size_of(&self) -> Result<i32, String> {
        match self {
            CodegenType::Integer | CodegenType::Char => Ok(4),
            CodegenType::Array(elem, low, high) => {
                let count = high
                    .checked_sub(*low)
                    .and_then(|n| n.checked_add(1))
                    .ok_or_else(|| "Array bound arithmetic overflow".to_string())?;
                if count <= 0 {
                    return Err(format!("Invalid array bounds [{}..{}]", low, high));
                }
                let count = i32::try_from(count)
                    .map_err(|_| "Array element count exceeds MIPS limits".to_string())?;
                count
                    .checked_mul(elem.element_byte_size())
                    .ok_or_else(|| "Array storage size overflow".to_string())
            }
            CodegenType::Record(fields) => {
                let mut size = 0i32;
                for (_, typ) in fields {
                    size = size
                        .checked_add(typ.size_of()?)
                        .ok_or_else(|| "Record storage size overflow".to_string())?;
                }
                Ok(size)
            }
        }
    }

    fn slot_size(&self) -> Result<i32, String> {
        let size = self.size_of()?;
        size.checked_add(3)
            .map(|n| n & !3)
            .ok_or_else(|| "Aligned storage size overflow".to_string())
    }

    fn is_aggregate(&self) -> bool {
        matches!(self, CodegenType::Array(_, _, _) | CodegenType::Record(_))
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
    fn field_offset(&self, name: &str) -> Result<Option<(i32, CodegenType)>, String> {
        if let CodegenType::Record(fields) = self {
            let mut offset = 0i32;
            for (fname, ft) in fields {
                if fname == name {
                    return Ok(Some((offset, ft.clone())));
                }
                offset = offset
                    .checked_add(ft.size_of()?)
                    .ok_or_else(|| "Record field offset overflow".to_string())?;
            }
        }
        Ok(None)
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

#[derive(Clone, Debug)]
enum Storage {
    Global(String),
    Local { level: usize, offset: i32 },
    ValueParam { level: usize, offset: i32 },
    RefParam { level: usize, offset: i32 },
}

#[derive(Clone, Debug)]
struct VarBinding {
    storage: Storage,
    typ: CodegenType,
}

#[derive(Clone, Debug)]
struct ParamLayout {
    name: String,
    is_var: bool,
    typ: CodegenType,
    size: i32,
    slot_size: i32,
    offset: i32,
}

#[derive(Clone, Debug)]
struct ProcMeta {
    label: String,
    path: Vec<String>,
    level: usize,
    parent_level: usize,
    params: Vec<ParamLayout>,
}

/// MIPS 代码生成上下文。
///
/// 维护汇编代码字符串、数据段字符串、标签计数器、
/// 变量绑定、过程词法作用域以及帧大小栈。
pub struct MipsContext {
    pub code: String,
    pub data: String,
    label_counter: usize,
    bindings: Vec<HashMap<String, VarBinding>>,
    proc_scopes: Vec<HashMap<String, ProcMeta>>,
    frame_sizes: Vec<i32>,
    nesting_level: usize,
    epilogue_labels: Vec<String>,
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
            bindings: vec![HashMap::new()],
            proc_scopes: Vec::new(),
            frame_sizes: vec![0],
            nesting_level: 0,
            epilogue_labels: Vec::new(),
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

    fn current_scope(&self) -> &HashMap<String, VarBinding> {
        self.bindings.last().expect("bindings should never be empty")
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<String, VarBinding> {
        self.bindings
            .last_mut()
            .expect("bindings should never be empty")
    }

    /// 在当前作用域中为变量分配栈空间。
    ///
    /// 若变量已在当前作用域声明则跳过。
    pub fn alloc_var(&mut self, name: &str, typ: &CodegenType) {
        if !self.current_scope().contains_key(name) {
            let slot_size = match typ.slot_size() {
                Ok(size) => size,
                Err(msg) => {
                    self.error(msg);
                    return;
                }
            };
            let current_size = *self
                .frame_sizes
                .last()
                .expect("frame_sizes should never be empty");
            let offset = match current_size.checked_add(slot_size) {
                Some(offset) => offset,
                None => {
                    self.error("Procedure frame size overflow");
                    return;
                }
            };
            let level = self.nesting_level;
            self.current_scope_mut().insert(
                name.to_string(),
                VarBinding {
                    storage: Storage::Local { level, offset },
                    typ: typ.clone(),
                },
            );
            *self
                .frame_sizes
                .last_mut()
                .expect("frame_sizes should never be empty") = offset;
        }
    }

    /// 查找变量的栈偏移和声明层级。
    ///
    /// 从最内层作用域向外搜索。
    pub fn get_var_offset(&self, name: &str) -> Option<(i32, usize)> {
        for scope in self.bindings.iter().rev() {
            if let Some(binding) = scope.get(name) {
                return match binding.storage {
                    Storage::Global(_) => Some((0, 0)),
                    Storage::Local { level, offset } => Some((offset, level)),
                    Storage::ValueParam { level, offset }
                    | Storage::RefParam { level, offset } => Some((-offset, level)),
                };
            }
        }
        None
    }

    /// 查找变量的代码生成类型。
    pub fn get_var_type(&self, name: &str) -> Option<&CodegenType> {
        for scope in self.bindings.iter().rev() {
            if let Some(binding) = scope.get(name) {
                return Some(&binding.typ);
            }
        }
        None
    }

    fn get_binding(&self, name: &str) -> Option<&VarBinding> {
        self.bindings
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
    }

    fn resolve_proc(&self, name: &str) -> Option<&ProcMeta> {
        self.proc_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
    }

    /// 进入过程作用域：初始化新的偏移量和类型映射。
    pub fn enter_proc(&mut self) {
        self.nesting_level += 1;
        self.bindings.push(HashMap::new());
        self.frame_sizes.push(0);
    }

    /// 退出过程作用域。
    pub fn exit_proc(&mut self) {
        self.nesting_level -= 1;
        self.bindings.pop();
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

fn emit_add_to_reg(ctx: &mut MipsContext, reg: &str, delta: i32) {
    if let Ok(imm) = i16::try_from(delta) {
        ctx.emit(&format!("  addiu {}, {}, {}", reg, reg, imm));
    } else {
        ctx.emit(&format!("  li $t9, {}", delta));
        ctx.emit(&format!("  addu {}, {}, $t9", reg, reg));
    }
}

fn emit_adjust_sp(ctx: &mut MipsContext, delta: i32) {
    emit_add_to_reg(ctx, "$sp", delta);
}

fn emit_frame_for_level(ctx: &mut MipsContext, target_level: usize) {
    ctx.emit("  move $t0, $fp");
    if target_level > ctx.nesting_level {
        ctx.error("Invalid lexical level while following static chain");
        return;
    }
    for _ in target_level..ctx.nesting_level {
        ctx.emit("  lw $t0, 8($t0)          # follow static link");
    }
}

fn emit_binding_address(name: &str, ctx: &mut MipsContext) -> CodegenType {
    let binding = ctx.get_binding(name).cloned().unwrap_or_else(|| {
        ctx.error(format!("Unknown variable '{}'", name));
        VarBinding {
            storage: Storage::Global(format!("var_{}", name)),
            typ: CodegenType::Integer,
        }
    });

    match binding.storage {
        Storage::Global(label) => ctx.emit(&format!("  la $t0, {}", label)),
        Storage::Local { level, offset } => {
            emit_frame_for_level(ctx, level);
            emit_add_to_reg(ctx, "$t0", -offset);
        }
        Storage::ValueParam { level, offset } => {
            emit_frame_for_level(ctx, level);
            emit_add_to_reg(ctx, "$t0", offset);
        }
        Storage::RefParam { level, offset } => {
            emit_frame_for_level(ctx, level);
            emit_add_to_reg(ctx, "$t0", offset);
            ctx.emit("  lw $t0, 0($t0)          # dereference var parameter");
        }
    }
    binding.typ
}

fn build_proc_scope(
    proc_dec: &ProcDec,
    parent_path: &[String],
    parent_level: usize,
    aliases: &HashMap<String, TypeBody>,
    ctx: &mut MipsContext,
) -> HashMap<String, ProcMeta> {
    let mut scope = HashMap::new();
    if let ProcDec::Defined(procs) = proc_dec {
        for proc in procs {
            let mut path = parent_path.to_vec();
            path.push(proc.name.clone());
            let label = format!("proc_{}", path.join("__"));
            let level = parent_level + 1;
            let mut params = Vec::new();
            let mut offset = 12i32;

            for param in &proc.params {
                let typ = type_desig_to_codegen(&param.type_name, aliases, &mut ctx.errors);
                let size = match typ.size_of() {
                    Ok(size) => size,
                    Err(msg) => {
                        ctx.error(msg);
                        4
                    }
                };
                let slot_size = if param.is_var {
                    4
                } else {
                    match typ.slot_size() {
                        Ok(size) => size,
                        Err(msg) => {
                            ctx.error(msg);
                            4
                        }
                    }
                };

                for name in &param.names {
                    params.push(ParamLayout {
                        name: name.clone(),
                        is_var: param.is_var,
                        typ: typ.clone(),
                        size,
                        slot_size,
                        offset,
                    });
                    offset = match offset.checked_add(slot_size) {
                        Some(next) => next,
                        None => {
                            ctx.error("Procedure parameter area overflow");
                            offset
                        }
                    };
                }
            }

            scope.insert(
                proc.name.clone(),
                ProcMeta {
                    label,
                    path,
                    level,
                    parent_level,
                    params,
                },
            );
        }
    }
    scope
}

fn bind_params(meta: &ProcMeta, ctx: &mut MipsContext) {
    for param in &meta.params {
        let storage = if param.is_var {
            Storage::RefParam {
                level: meta.level,
                offset: param.offset,
            }
        } else {
            Storage::ValueParam {
                level: meta.level,
                offset: param.offset,
            }
        };
        ctx.current_scope_mut().insert(
            param.name.clone(),
            VarBinding {
                storage,
                typ: param.typ.clone(),
            },
        );
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
        let size = match resolved.size_of() {
            Ok(size) => size,
            Err(msg) => {
                ctx.error(msg);
                4
            }
        };
        for name in &var_dec.names {
            if size == 4 {
                ctx.emit_data(&format!("var_{}: .word 0", name));
            } else {
                ctx.emit_data("  .align 2");
                ctx.emit_data(&format!("var_{}: .space {}", name, size));
            }
            ctx.current_scope_mut().insert(
                name.to_string(),
                VarBinding {
                    storage: Storage::Global(format!("var_{}", name)),
                    typ: resolved.clone(),
                },
            );
        }
    }

    let global_proc_scope =
        build_proc_scope(&prog.decl.procs, &[], 0, &global_aliases, &mut ctx);
    ctx.proc_scopes.push(global_proc_scope);

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

    let main_epilogue = "main_exit".to_string();
    ctx.epilogue_labels.push(main_epilogue.clone());
    compile_stm_list(&prog.body.stmts, &mut ctx);

    ctx.emit_label(&main_epilogue);
    ctx.emit("  li $v0, 10             # exit syscall");
    ctx.emit("  syscall");
    ctx.emit("");
    ctx.epilogue_labels.pop();

    // 编译过程声明
    compile_procs(&prog.decl.procs, &mut ctx, &global_aliases);
    ctx.proc_scopes.pop();

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
/// 每个过程生成词法作用域限定的标号、栈帧管理序言/尾言和过程体。
/// 类型别名与父作用域合并，支持嵌套过程。
fn compile_procs(
    proc_dec: &ProcDec,
    ctx: &mut MipsContext,
    parent_aliases: &HashMap<String, TypeBody>,
) {
    if let ProcDec::Defined(procs) = proc_dec {
        for proc in procs {
            let meta = ctx
                .proc_scopes
                .last()
                .and_then(|scope| scope.get(&proc.name))
                .cloned()
                .unwrap_or_else(|| {
                    ctx.error(format!("Missing procedure metadata for '{}'", proc.name));
                    ProcMeta {
                        label: format!("proc_{}", proc.name),
                        path: vec![proc.name.clone()],
                        level: ctx.nesting_level + 1,
                        parent_level: ctx.nesting_level,
                        params: Vec::new(),
                    }
                });
            ctx.emit("");
            ctx.emit_label(&meta.label);

            ctx.enter_proc();
            if ctx.nesting_level != meta.level {
                ctx.error("Procedure lexical level mismatch");
            }
            bind_params(&meta, ctx);

            // 合并局部类型定义与继承的类型别名
            let mut proc_aliases = parent_aliases.clone();
            for (name, body) in build_type_alias_map(&proc.decl.types) {
                proc_aliases.insert(name, body);
            }

            let child_scope = build_proc_scope(
                &proc.decl.procs,
                &meta.path,
                meta.level,
                &proc_aliases,
                ctx,
            );
            ctx.proc_scopes.push(child_scope);

            // 局部变量声明
            for var_dec in var_decs(&proc.decl.vars) {
                let resolved = type_desig_to_codegen(&var_dec.type_name, &proc_aliases, &mut ctx.errors);
                for name in &var_dec.names {
                    ctx.alloc_var(name, &resolved);
                }
            }

            let frame = ctx.frame_size();

            ctx.emit("  addiu $sp, $sp, -8     # space for $fp + $ra");
            ctx.emit("  sw $fp, 0($sp)         # save old $fp");
            ctx.emit("  sw $ra, 4($sp)         # save return address");
            ctx.emit("  move $fp, $sp          # frame pointer");
            if frame > 0 {
                emit_adjust_sp(ctx, -frame);
            }

            let epilogue = ctx.new_label("__snl_epilogue");
            ctx.epilogue_labels.push(epilogue.clone());
            compile_stm_list(&proc.body.stmts, ctx);

            ctx.emit_label(&epilogue);
            ctx.emit("  move $sp, $fp          # discard locals");
            ctx.emit("  lw $fp, 0($sp)         # restore old $fp");
            ctx.emit("  lw $ra, 4($sp)         # restore $ra");
            ctx.emit("  addiu $sp, $sp, 8      # deallocate $fp + $ra slots");
            ctx.emit("  jr $ra                  # return");
            ctx.epilogue_labels.pop();

            compile_procs(&proc.decl.procs, ctx, &proc_aliases);

            ctx.proc_scopes.pop();
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
    let current_typ = emit_binding_address(&va.base, ctx);
    walk_selectors(&va.selector, ctx, current_typ)
}

fn emit_subtract_low_bound(ctx: &mut MipsContext, low: i64) {
    if low == 0 {
        return;
    }
    match i32::try_from(low) {
        Ok(low) => emit_add_to_reg(ctx, "$v0", -low),
        Err(_) => ctx.error("Array lower bound exceeds MIPS integer range"),
    }
}

fn lookup_field(
    typ: &CodegenType,
    name: &str,
    ctx: &mut MipsContext,
) -> (i32, CodegenType) {
    match typ.field_offset(name) {
        Ok(Some(field)) => field,
        Ok(None) => {
            ctx.error(format!("Field '{}' not found in record", name));
            (0, CodegenType::Integer)
        }
        Err(msg) => {
            ctx.error(msg);
            (0, CodegenType::Integer)
        }
    }
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
                emit_subtract_low_bound(ctx, low_val);
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
                let (field_offset, field_type) = lookup_field(&current_typ, name, ctx);
                emit_add_to_reg(ctx, "$t0", field_offset);
                current_typ = field_type;
            }
            Selector::FieldSubscript(name, exp) => {
                let (field_offset, field_type) = lookup_field(&current_typ, name, ctx);
                emit_add_to_reg(ctx, "$t0", field_offset);
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
                emit_subtract_low_bound(ctx, low_val);
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

fn var_access_type(va: &VarAccess, ctx: &mut MipsContext) -> CodegenType {
    let mut typ = ctx.get_var_type(&va.base).cloned().unwrap_or_else(|| {
        ctx.error(format!("No type for '{}'", va.base));
        CodegenType::Integer
    });
    for selector in &va.selector {
        typ = match selector {
            Selector::ArraySubscript(_) => match typ {
                CodegenType::Array(elem, _, _) => *elem,
                _ => {
                    ctx.error("Array subscript on non-array type");
                    CodegenType::Integer
                }
            },
            Selector::Field(name) | Selector::FieldSubscript(name, _) => {
                let (_, field_type) = lookup_field(&typ, name, ctx);
                if matches!(selector, Selector::FieldSubscript(_, _)) {
                    match field_type {
                        CodegenType::Array(elem, _, _) => *elem,
                        _ => {
                            ctx.error("FieldSubscript on non-array field");
                            CodegenType::Integer
                        }
                    }
                } else {
                    field_type
                }
            }
        };
    }
    typ
}

fn exp_type(exp: &Exp, ctx: &mut MipsContext) -> CodegenType {
    match exp {
        Exp::Binary { .. } | Exp::IntConst(_, _) => CodegenType::Integer,
        Exp::CharConst(_, _) => CodegenType::Char,
        Exp::Variable(va, _) => var_access_type(va, ctx),
    }
}

fn emit_copy_bytes(ctx: &mut MipsContext, size: i32) {
    if size <= 0 {
        ctx.error("Compound value has invalid storage size");
        return;
    }
    let loop_label = ctx.new_label("copy_loop");
    let end_label = ctx.new_label("copy_end");
    ctx.emit(&format!("  li $t3, {}", size));
    ctx.emit_label(&loop_label);
    ctx.emit(&format!("  beqz $t3, {}", end_label));
    ctx.emit("  lb $t4, 0($t1)");
    ctx.emit("  sb $t4, 0($t2)");
    ctx.emit("  addiu $t1, $t1, 1");
    ctx.emit("  addiu $t2, $t2, 1");
    ctx.emit("  addiu $t3, $t3, -1");
    ctx.emit(&format!("  j {}", loop_label));
    ctx.emit_label(&end_label);
}

fn compile_assign(lhs: &VarAccess, rhs: &Exp, ctx: &mut MipsContext) {
    let rhs_type = exp_type(rhs, ctx);
    if rhs_type.is_aggregate() {
        let Exp::Variable(rhs_var, _) = rhs else {
            ctx.error("Compound assignment requires a variable source");
            return;
        };
        let source_type = emit_var_address(rhs_var, ctx);
        let size = match source_type.size_of() {
            Ok(size) => size,
            Err(msg) => {
                ctx.error(msg);
                return;
            }
        };
        ctx.emit("  addiu $sp, $sp, -4");
        ctx.emit("  sw $t0, 0($sp)          # save compound rhs address");
        let lhs_type = emit_var_address(lhs, ctx);
        if lhs_type != source_type {
            ctx.error("Compound assignment type mismatch during code generation");
        }
        ctx.emit("  move $t2, $t0          # destination address");
        ctx.emit("  lw $t1, 0($sp)          # source address");
        ctx.emit("  addiu $sp, $sp, 4");
        emit_copy_bytes(ctx, size);
        return;
    }

    compile_exp(rhs, ctx);
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

fn compile_call(name: &str, args: &[Exp], ctx: &mut MipsContext) {
    let meta = ctx.resolve_proc(name).cloned().unwrap_or_else(|| {
        ctx.error(format!("Unknown procedure '{}'", name));
        ProcMeta {
            label: format!("proc_{}", name),
            path: vec![name.to_string()],
            level: ctx.nesting_level + 1,
            parent_level: ctx.nesting_level,
            params: Vec::new(),
        }
    });

    if meta.params.len() != args.len() {
        ctx.error(format!(
            "Procedure '{}' expects {} arguments, got {}",
            name,
            meta.params.len(),
            args.len()
        ));
    }

    let mut argument_bytes = 0i32;
    for (arg, param) in args.iter().zip(meta.params.iter()).rev() {
        if param.is_var {
            if let Exp::Variable(var, _) = arg {
                emit_var_address(var, ctx);
            } else {
                ctx.error(format!(
                    "Argument for var parameter '{}' is not a variable",
                    param.name
                ));
                ctx.emit("  move $t0, $zero");
            }
            ctx.emit("  addiu $sp, $sp, -4");
            ctx.emit("  sw $t0, 0($sp)          # var parameter address");
        } else if param.typ.is_aggregate() {
            if let Exp::Variable(var, _) = arg {
                emit_var_address(var, ctx);
            } else {
                ctx.error(format!(
                    "Value parameter '{}' requires a compound variable",
                    param.name
                ));
                ctx.emit("  move $t0, $zero");
            }
            ctx.emit("  move $t1, $t0          # compound argument source");
            emit_adjust_sp(ctx, -param.slot_size);
            ctx.emit("  move $t2, $sp          # compound argument copy");
            emit_copy_bytes(ctx, param.size);
        } else {
            let arg_type = compile_exp(arg, ctx);
            ctx.emit("  addiu $sp, $sp, -4");
            if arg_type == CodegenType::Char {
                ctx.emit("  sb $v0, 0($sp)");
            } else {
                ctx.emit("  sw $v0, 0($sp)");
            }
        }
        argument_bytes = match argument_bytes.checked_add(param.slot_size) {
            Some(total) => total,
            None => {
                ctx.error("Call argument area overflow");
                argument_bytes
            }
        };
    }

    if meta.parent_level == 0 {
        ctx.emit("  move $t0, $zero          # top-level static link");
    } else {
        emit_frame_for_level(ctx, meta.parent_level);
    }
    ctx.emit("  addiu $sp, $sp, -4");
    ctx.emit("  sw $t0, 0($sp)          # static link");
    ctx.emit(&format!("  jal {}", meta.label));

    let cleanup = match argument_bytes.checked_add(4) {
        Some(size) => size,
        None => {
            ctx.error("Call cleanup size overflow");
            4
        }
    };
    emit_adjust_sp(ctx, cleanup);
}

/// 编译单条语句。
fn compile_stm(stm: &Stm, ctx: &mut MipsContext) {
    match stm {
        Stm::Assign { lhs, rhs, .. } => compile_assign(lhs, rhs, ctx),
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
            let is_char = ctx.get_var_type(var) == Some(&CodegenType::Char);
            if is_char {
                ctx.emit("  li $v0, 12             # read char syscall");
            } else {
                ctx.emit("  li $v0, 5              # read int syscall");
            }
            ctx.emit("  syscall");
            ctx.emit("  addiu $sp, $sp, -4");
            ctx.emit("  sw $v0, 0($sp)          # save input value");
            emit_binding_address(var, ctx);
            ctx.emit("  lw $v0, 0($sp)");
            ctx.emit("  addiu $sp, $sp, 4");
            if is_char {
                ctx.emit("  sb $v0, 0($t0)");
            } else {
                ctx.emit("  sw $v0, 0($t0)");
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
            if let Some(epilogue) = ctx.epilogue_labels.last().cloned() {
                ctx.emit(&format!("  j {}", epilogue));
            } else {
                ctx.error("Return statement has no active epilogue");
            }
        }
        Stm::Call { name, args, .. } => compile_call(name, args, ctx),
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
            match i32::try_from(*n) {
                Ok(value) => ctx.emit(&format!("  li $v0, {}", value)),
                Err(_) => {
                    ctx.error("Integer constant exceeds MIPS 32-bit range");
                    ctx.emit("  li $v0, 0");
                }
            }
            CodegenType::Integer
        }
        Exp::CharConst(c, _) => {
            ctx.emit(&format!("  li $v0, {}", *c as i32));
            CodegenType::Char
        }
        Exp::Variable(va, _) => {
            let final_typ = emit_var_address(va, ctx);
            if final_typ.is_aggregate() {
                ctx.error("Compound value used where a scalar expression is required");
            } else if final_typ == CodegenType::Char {
                ctx.emit("  lb $v0, 0($t0)");
            } else {
                ctx.emit("  lw $v0, 0($t0)");
            }
            final_typ
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

    #[test]
    fn test_parameter_group_uses_distinct_slots() {
        let asm = compile_source(
            "program p procedure emit(integer a,b); begin write(a); write(b) end begin emit(1,2) end.",
        );
        assert!(asm.contains("addiu $t0, $t0, 12"));
        assert!(asm.contains("addiu $t0, $t0, 16"));
    }

    #[test]
    fn test_var_parameter_is_indirect() {
        let asm = compile_source(
            "program p var integer x; procedure inc(var integer a); begin a := a + 1 end begin inc(x) end.",
        );
        assert!(asm.contains("# var parameter address"));
        assert!(asm.contains("# dereference var parameter"));
    }

    #[test]
    fn test_compound_value_parameter_emits_full_copy() {
        let asm = compile_source(
            "program p var array[0..1] of integer x; procedure emit(array[0..1] of integer a); begin write(a[1]) end begin emit(x) end.",
        );
        assert!(asm.contains("# compound argument copy"));
        assert!(asm.contains("copy_loop_"));
        assert!(asm.contains("addiu $t0, $t0, 12"));
    }

    #[test]
    fn test_nested_access_follows_static_link() {
        let asm = compile_source(
            "program p procedure outer(); var integer x; procedure inner(); begin x := x + 1 end begin inner() end begin outer() end.",
        );
        assert!(asm.contains("# follow static link"));
    }

    #[test]
    fn test_nested_procedure_labels_include_lexical_path() {
        let asm = compile_source(
            "program p procedure a(); procedure helper(); begin write(1) end begin helper() end procedure b(); procedure helper(); begin write(2) end begin helper() end begin a(); b() end.",
        );
        assert!(asm.contains("proc_a__helper:"));
        assert!(asm.contains("proc_b__helper:"));
        assert!(asm.contains("jal proc_a__helper"));
        assert!(asm.contains("jal proc_b__helper"));
    }

    #[test]
    fn test_epilogue_label_does_not_collide_with_nested_procedure() {
        let asm = compile_source(
            "program p procedure outer(); procedure epilogue(); begin write(1) end begin epilogue() end begin outer() end.",
        );
        assert_eq!(asm.matches("proc_outer__epilogue:").count(), 1);
    }

    #[test]
    fn test_compound_assignment_emits_full_copy() {
        let asm = compile_source(
            "program p var array[0..1] of integer a,b; begin b := a end.",
        );
        assert!(asm.contains("copy_loop_"));
        assert!(asm.contains("save compound rhs address"));
    }

    #[test]
    fn test_return_jumps_to_shared_epilogue() {
        let asm = compile_source(
            "program p procedure f(); begin return(1); write(2) end begin f() end.",
        );
        assert!(
            asm.lines()
                .any(|line| line.trim_start().starts_with("j __snl_epilogue_"))
        );
        assert_eq!(
            asm.lines()
                .filter(|line| line.starts_with("__snl_epilogue_") && line.ends_with(':'))
                .count(),
            1
        );
    }

    #[test]
    fn test_invalid_array_bounds_fail_codegen() {
        let mut lexer = Lexer::new();
        let (tokens, errors) = lexer.tokenize(
            "program p var array[5..1] of integer a; begin write(0) end.",
        );
        assert!(errors.is_empty());
        let mut parser = RdParser::new(tokens);
        let prog = parser.parse().expect("Parse should succeed");
        assert!(compile(&prog).is_err());
    }

    #[test]
    fn test_array_size_overflow_fails_codegen() {
        let mut lexer = Lexer::new();
        let (tokens, errors) = lexer.tokenize(
            "program p var array[0..9223372036854775807] of integer a; begin write(0) end.",
        );
        assert!(errors.is_empty());
        let mut parser = RdParser::new(tokens);
        let prog = parser.parse().expect("Parse should succeed");
        assert!(compile(&prog).is_err());
    }
}
