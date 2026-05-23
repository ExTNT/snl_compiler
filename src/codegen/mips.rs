use std::collections::HashMap;

use crate::ast::nodes::*;

// ===== Codegen Type Representation =====

#[derive(Clone, Debug, PartialEq)]
pub enum CodegenType {
    Integer,
    Char,
    Array(Box<CodegenType>, i64, i64),  // elem_type, low, high
    Record(Vec<(String, CodegenType)>), // (field_name, field_type) — multi-name FieldDefs expanded
}

impl CodegenType {
    /// Byte size for allocating a variable of this type.
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

    /// Byte stride for subscript arithmetic. Integer→4, Char→1.
    fn element_byte_size(&self) -> i32 {
        match self {
            CodegenType::Integer => 4,
            CodegenType::Char => 1,
            CodegenType::Array(elem, _, _) => elem.element_byte_size(),
            CodegenType::Record(_) => panic!("Record has no element byte size"),
        }
    }

    /// Precompute (offset, type) for each record field.
    fn field_offsets(&self) -> HashMap<String, (i32, CodegenType)> {
        let mut offsets = HashMap::new();
        if let CodegenType::Record(fields) = self {
            let mut offset = 0i32;
            for (name, ft) in fields {
                offsets.insert(name.clone(), (offset, ft.clone()));
                offset += ft.size_of();
            }
        }
        offsets
    }
}

// ===== Type Resolution from AST =====

fn build_type_alias_map(type_dec: &TypeDec) -> HashMap<String, TypeBody> {
    let mut aliases = HashMap::new();
    if let TypeDec::Defined(defs) = type_dec {
        for def in defs {
            aliases.insert(def.name.clone(), def.body.clone());
        }
    }
    aliases
}

fn type_body_to_codegen(
    body: &TypeBody,
    aliases: &HashMap<String, TypeBody>,
    visited: &mut Vec<String>,
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
                panic!("Circular type alias: {}", name);
            }
            visited.push(name.clone());
            let resolved = aliases
                .get(name)
                .unwrap_or_else(|| panic!("Undefined type alias: {}", name));
            let result = type_body_to_codegen(resolved, aliases, visited);
            visited.pop();
            result
        }
    }
}

fn type_desig_to_codegen(td: &TypeDesig, aliases: &HashMap<String, TypeBody>) -> CodegenType {
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
            let body = aliases
                .get(name)
                .unwrap_or_else(|| panic!("Undefined type alias: {}", name));
            type_body_to_codegen(body, aliases, &mut visited)
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

// ===== MIPS Context =====

pub struct MipsContext {
    pub code: String,
    pub data: String,
    label_counter: usize,
    /// Maps variable name to (stack offset, nesting level)
    var_offsets: Vec<HashMap<String, (i32, usize)>>,
    /// Maps variable name to its resolved CodegenType
    var_types: Vec<HashMap<String, CodegenType>>,
    /// Current stack frame size (accumulated)
    frame_sizes: Vec<i32>,
    /// Current scope nesting level (0 = global/main)
    nesting_level: usize,
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
        }
    }

    pub fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    fn current_scope(&self) -> &HashMap<String, (i32, usize)> {
        self.var_offsets.last().unwrap()
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<String, (i32, usize)> {
        self.var_offsets.last_mut().unwrap()
    }

    pub fn alloc_var(&mut self, name: &str, typ: &CodegenType) {
        if !self.current_scope().contains_key(name) {
            let offset = *self.frame_sizes.last().unwrap();
            let level = self.nesting_level;
            self.current_scope_mut()
                .insert(name.to_string(), (offset, level));
            self.var_types
                .last_mut()
                .unwrap()
                .insert(name.to_string(), typ.clone());
            *self.frame_sizes.last_mut().unwrap() += typ.size_of();
        }
    }

    pub fn get_var_offset(&self, name: &str) -> Option<(i32, usize)> {
        for scope in self.var_offsets.iter().rev() {
            if let Some(&val) = scope.get(name) {
                return Some(val);
            }
        }
        None
    }

    pub fn get_var_type(&self, name: &str) -> Option<CodegenType> {
        for scope in self.var_types.iter().rev() {
            if let Some(typ) = scope.get(name) {
                return Some(typ.clone());
            }
        }
        None
    }

    pub fn enter_proc(&mut self) {
        self.nesting_level += 1;
        self.var_offsets.push(HashMap::new());
        self.var_types.push(HashMap::new());
        self.frame_sizes.push(8); // $fp save + $ra save
    }

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
        *self.frame_sizes.last().unwrap()
    }

    pub fn emit(&mut self, s: &str) {
        self.code.push_str(s);
        self.code.push('\n');
    }

    pub fn emit_label(&mut self, label: &str) {
        self.code.push_str(&format!("{}:\n", label));
    }

    pub fn emit_data(&mut self, s: &str) {
        self.data.push_str(s);
        self.data.push('\n');
    }
}

fn fp_offset(offset: i32) -> String {
    if offset >= 0 {
        format!("-{}($fp)", offset)
    } else {
        format!("{}($fp)", -offset)
    }
}

/// Globals (level 0) use direct label addressing in .data.
/// Locals use $fp-relative stack access.
fn emit_load(ctx: &mut MipsContext, offset: i32, var_level: usize, name: &str, typ: &CodegenType) {
    let instr = if *typ == CodegenType::Char {
        "lb"
    } else {
        "lw"
    };
    if var_level == 0 {
        ctx.emit(&format!("  la $t8, var_{}", name));
        ctx.emit(&format!(
            "  {} $v0, 0($t8)         # load global {}",
            instr, name
        ));
    } else {
        ctx.emit(&format!(
            "  {} $v0, {}       # load {}",
            instr,
            fp_offset(offset),
            name
        ));
    }
}

fn emit_store(ctx: &mut MipsContext, offset: i32, var_level: usize, name: &str, typ: &CodegenType) {
    let instr = if *typ == CodegenType::Char {
        "sb"
    } else {
        "sw"
    };
    if var_level == 0 {
        ctx.emit(&format!("  la $t8, var_{}", name));
        ctx.emit(&format!(
            "  {} $v0, 0($t8)         # store to global {}",
            instr, name
        ));
    } else {
        ctx.emit(&format!(
            "  {} $v0, {}       # store to {}",
            instr,
            fp_offset(offset),
            name
        ));
    }
}

// ===== Main compilation entry point =====

pub fn compile(prog: &Program) -> String {
    let mut ctx = MipsContext::new();

    // Build global type alias map
    let global_aliases = build_type_alias_map(&prog.decl.types);

    // Data section: allocate global variables
    ctx.emit_data("newline: .asciiz \"\\n\"");
    for var_dec in var_decs(&prog.decl.vars) {
        let resolved = type_desig_to_codegen(&var_dec.type_name, &global_aliases);
        let size = resolved.size_of();
        for name in &var_dec.names {
            if size == 4 {
                ctx.emit_data(&format!("var_{}: .word 0", name));
            } else {
                ctx.emit_data("  .align 2");
                ctx.emit_data(&format!("var_{}: .space {}", name, size));
            }
            ctx.current_scope_mut().insert(name.to_string(), (0, 0));
            ctx.var_types
                .last_mut()
                .unwrap()
                .insert(name.to_string(), resolved.clone());
        }
    }

    // Emit program as "main" procedure
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

    // Compile global body
    compile_stm_list(&prog.body.stmts, &mut ctx);

    // Epilogue
    ctx.emit("  li $v0, 10             # exit syscall");
    ctx.emit("  syscall");
    ctx.emit("");

    // Compile procedures
    compile_procs(&prog.decl.procs, &mut ctx, &global_aliases);

    format!(".data\n{}\n.text\n.globl main\n{}", ctx.data, ctx.code)
}

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

            // Prologue: save $fp (callee-saved) + $ra
            ctx.emit("  addiu $sp, $sp, -8     # space for $fp + $ra");
            ctx.emit("  sw $fp, 0($sp)         # save old $fp");
            ctx.emit("  sw $ra, 4($sp)         # save return address");
            ctx.emit("  move $fp, $sp          # frame pointer");

            // Merge local type definitions with inherited ones
            let mut proc_aliases = parent_aliases.clone();
            for (name, body) in build_type_alias_map(&proc.decl.types) {
                proc_aliases.insert(name, body);
            }

            // Allocate params (in caller's frame, above $fp)
            let proc_level = ctx.nesting_level();
            for (i, param) in proc.params.iter().enumerate() {
                let param_type = type_desig_to_codegen(&param.type_name, &proc_aliases);
                for name in &param.names {
                    let offset = -(i as i32 * 4 + 8);
                    ctx.current_scope_mut()
                        .insert(name.clone(), (offset, proc_level));
                    ctx.var_types
                        .last_mut()
                        .unwrap()
                        .insert(name.clone(), param_type.clone());
                }
            }

            // Local declarations
            for var_dec in var_decs(&proc.decl.vars) {
                let resolved = type_desig_to_codegen(&var_dec.type_name, &proc_aliases);
                for name in &var_dec.names {
                    ctx.alloc_var(name, &resolved);
                }
            }

            let frame = ctx.frame_size();
            if frame > 0 {
                ctx.emit(&format!("  addiu $sp, $sp, -{}     # locals", frame));
            }

            // Body
            compile_stm_list(&proc.body.stmts, ctx);

            // Epilogue
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

            // Nested procedures
            compile_procs(&proc.decl.procs, ctx, &proc_aliases);

            ctx.exit_proc();
        }
    }
}

// ===== Selector address computation =====

/// Emit MIPS code to compute the runtime address of a VarAccess into $t0.
/// Handles base variable (global/local), array subscripts, and record fields.
/// Returns the scalar CodegenType of the final element.
fn emit_var_address(va: &VarAccess, ctx: &mut MipsContext) -> CodegenType {
    let (offset, var_level) = ctx
        .get_var_offset(&va.base)
        .unwrap_or_else(|| panic!("Unknown variable '{}'", va.base));
    let current_typ = ctx
        .get_var_type(&va.base)
        .unwrap_or_else(|| panic!("No type for '{}'", va.base));

    // Load base address into $t0
    if var_level == 0 {
        ctx.emit(&format!("  la $t0, var_{}", va.base));
    } else {
        ctx.emit(&format!("  addiu $t0, $fp, {}", -offset));
    }

    walk_selectors(&va.selector, ctx, current_typ)
}

/// Walk a chain of selectors, emitting code to update $t0 to point to the
/// final element. Returns the scalar type of the selected element.
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
                    _ => panic!("Array subscript on non-array type"),
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
            Selector::Field(name) => {
                let offsets = current_typ.field_offsets();
                let (field_offset, field_type) = offsets
                    .get(name)
                    .unwrap_or_else(|| panic!("Field '{}' not found in record", name))
                    .clone();
                ctx.emit(&format!("  addiu $t0, $t0, {}", field_offset));
                current_typ = field_type;
            }
            Selector::FieldSubscript(name, exp) => {
                let offsets = current_typ.field_offsets();
                let (field_offset, field_type) = offsets
                    .get(name)
                    .unwrap_or_else(|| panic!("Field '{}' not found in record", name))
                    .clone();
                ctx.emit(&format!("  addiu $t0, $t0, {}", field_offset));
                let (elem_type, low_val) = match &field_type {
                    CodegenType::Array(elem, low, _) => (*elem.clone(), *low),
                    _ => panic!("FieldSubscript on non-array field"),
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

// ===== Statement compilation =====

fn compile_stm_list(stmts: &[Stm], ctx: &mut MipsContext) {
    for stm in stmts {
        compile_stm(stm, ctx);
    }
}

fn compile_stm(stm: &Stm, ctx: &mut MipsContext) {
    match stm {
        Stm::Assign { lhs, rhs, .. } => {
            compile_exp(rhs, ctx);
            if lhs.selector.is_empty() {
                if let Some((offset, var_level)) = ctx.get_var_offset(&lhs.base) {
                    if let Some(typ) = ctx.get_var_type(&lhs.base) {
                        emit_store(ctx, offset, var_level, &lhs.base, &typ);
                    }
                }
            } else {
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
            let typ = ctx.get_var_type(var).unwrap_or(CodegenType::Integer);
            if typ == CodegenType::Char {
                ctx.emit("  li $v0, 12             # read char syscall");
            } else {
                ctx.emit("  li $v0, 5              # read int syscall");
            }
            ctx.emit("  syscall");
            if let Some((offset, var_level)) = ctx.get_var_offset(var) {
                emit_store(ctx, offset, var_level, var, &typ);
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

// ===== Expression compilation =====

fn compile_exp(exp: &Exp, ctx: &mut MipsContext) -> CodegenType {
    match exp {
        Exp::Binary {
            op, left, right, ..
        } => {
            let _ = compile_exp(right, ctx);
            ctx.emit("  addiu $sp, $sp, -4");
            ctx.emit("  sw $v0, 0($sp)          # push right");
            let _ = compile_exp(left, ctx);
            ctx.emit("  lw $t0, 0($sp)          # pop right");
            ctx.emit("  addiu $sp, $sp, 4");
            match op {
                BinOp::Add => ctx.emit("  addu $v0, $v0, $t0"),
                BinOp::Sub => ctx.emit("  subu $v0, $v0, $t0"),
                BinOp::Mul => ctx.emit("  mul $t7, $v0, $t0\n  move $v0, $t7"),
                BinOp::Div => {
                    ctx.emit("  div $v0, $v0, $t0");
                    ctx.emit("  mflo $v0");
                }
                BinOp::Lt => ctx.emit("  slt $v0, $v0, $t0"),
                BinOp::Eq => {
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
                    if let Some(typ) = ctx.get_var_type(&va.base) {
                        emit_load(ctx, offset, var_level, &va.base, &typ);
                        return typ;
                    }
                }
                CodegenType::Integer
            } else {
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
        compile(&prog)
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
        // a at offset 0, b at offset 4
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
        // Field a at offset 0, then array subscript with sll
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
        // Should save rhs, compute LHS address, restore rhs, store
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
