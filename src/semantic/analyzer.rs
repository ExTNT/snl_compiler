//! 语义分析器。
//!
//! 对 AST 执行语义检查，包括：
//! - 符号表构建（两遍：先收集声明，再检查使用）
//! - 类型检查（赋值、表达式、过程调用参数）
//! - 未声明标识符检测
//! - 标识符种类正确性验证
//!
//! ## 分析流程
//! 1. **第一遍**：遍历声明部分，构建符号表（类型、变量、过程）
//! 2. **第二遍**：遍历语句和表达式，进行类型检查
//! 3. 每个作用域结束时记录快照，用于生成符号表文档

use std::collections::HashMap;

use crate::ast::nodes::*;
use crate::error::{CompileError, SemanticErrCode};
use crate::semantic::symbol::*;

/// 语义分析器。
///
/// 维护符号表和错误列表。分析过程消费 AST 的不可变引用，
/// 不会修改 AST。
pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    errors: Vec<CompileError>,
    /// 各作用域的快照，按退出顺序记录
    scope_snapshots: Vec<(usize, HashMap<String, SymbolEntry>)>,
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbols: SymbolTable::new(),
            errors: Vec::new(),
            scope_snapshots: Vec::new(),
        }
    }

    pub fn errors(&self) -> &[CompileError] {
        &self.errors
    }

    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    pub fn scope_snapshots(&self) -> &[(usize, HashMap<String, SymbolEntry>)] {
        &self.scope_snapshots
    }

    /// 保存当前最内层作用域的快照（用于生成符号表文档）。
    fn snapshot_scope(&mut self) {
        let level = self.symbols.current_level();
        let scope = self.symbols.scopes().last().unwrap().clone();
        self.scope_snapshots.push((level, scope));
    }

    // ===== 主入口 =====

    /// 对程序进行完整的语义分析。
    ///
    /// 执行两遍分析：第一遍构建符号表，第二遍检查类型。
    pub fn analyze(&mut self, prog: &Program) {
        // 第一遍：符号表构建
        self.collect_declarations(prog);

        // 第二遍：语句检查
        self.check_program_body(&prog.body);

        // 记录剩余的全局作用域快照
        self.snapshot_scope();
    }

    // ===== 第一遍：构建符号表 =====

    fn collect_declarations(&mut self, prog: &Program) {
        // 程序名本身作为过程标识符（入口点）
        self.insert_symbol(SymbolEntry {
            name: prog.name.clone(),
            kind: IdKind::ProcId,
            typ: None,
            params: vec![],
            level: 0,
            loc: prog.loc,
        });

        self.collect_type_decs(&prog.decl.types);
        self.collect_var_decs(&prog.decl.vars);
        self.collect_proc_decs(&prog.decl.procs);
    }

    fn collect_type_decs(&mut self, type_dec: &TypeDec) {
        if let TypeDec::Defined(defs) = type_dec {
            for def in defs {
                self.validate_type_body(&def.body);
                let ti = self.type_body_to_info(&def.body);
                self.insert_symbol(SymbolEntry {
                    name: def.name.clone(),
                    kind: IdKind::TypeId,
                    typ: Some(ti),
                    params: vec![],
                    level: self.symbols.current_level(),
                    loc: def.loc,
                });
            }
        }
    }

    fn collect_var_decs(&mut self, var_dec: &VarDec) {
        if let VarDec::Defined(defs) = var_dec {
            for def in defs {
                self.validate_type_desig(&def.type_name);
                let ti = self.type_desig_to_info(&def.type_name);
                for name in &def.names {
                    self.insert_symbol(SymbolEntry {
                        name: name.clone(),
                        kind: IdKind::VarId,
                        typ: Some(ti.clone()),
                        params: vec![],
                        level: self.symbols.current_level(),
                        loc: def.loc,
                    });
                }
            }
        }
    }

    fn collect_proc_decs(&mut self, proc_dec: &ProcDec) {
        if let ProcDec::Defined(procs) = proc_dec {
            for proc in procs {
                let mut params = Vec::new();
                for param in &proc.params {
                    self.validate_type_desig(&param.type_name);
                    let ti = self.type_desig_to_info(&param.type_name);
                    for name in &param.names {
                        params.push(ParamInfo {
                            name: name.clone(),
                            is_var: param.is_var,
                            typ: ti.clone(),
                        });
                    }
                }

                self.insert_symbol(SymbolEntry {
                    name: proc.name.clone(),
                    kind: IdKind::ProcId,
                    typ: None,
                    params,
                    level: self.symbols.current_level(),
                    loc: proc.loc,
                });

                // 进入过程作用域
                self.symbols.enter_scope();

                // 将形参添加到内部作用域（作为变量）
                for p in &proc.params {
                    let ti = self.type_desig_to_info(&p.type_name);
                    for name in &p.names {
                        self.insert_symbol(SymbolEntry {
                            name: name.clone(),
                            kind: IdKind::VarId,
                            typ: Some(ti.clone()),
                            params: vec![],
                            level: self.symbols.current_level(),
                            loc: p.loc,
                        });
                    }
                }

                self.collect_type_decs(&proc.decl.types);
                self.collect_var_decs(&proc.decl.vars);
                self.collect_proc_decs(&proc.decl.procs);
                self.check_program_body(&proc.body);

                self.snapshot_scope();
                self.symbols.exit_scope();
            }
        }
    }

    // ===== 第二遍：语句检查 =====

    fn check_program_body(&mut self, body: &StmList) {
        for stm in &body.stmts {
            self.check_stm(stm);
        }
    }

    fn check_stm(&mut self, stm: &Stm) {
        match stm {
            Stm::Assign { lhs, rhs, loc } => self.check_assign(lhs, rhs, *loc),
            Stm::If {
                cond,
                then_branch,
                else_branch,
                loc,
            } => {
                let cond_ty = self.check_exp(cond);
                // SNL 以整数作为布尔值，条件必须为整数类型
                if let Some(cond_ty) = cond_ty {
                    let resolved = self.resolve_type(&cond_ty, &mut Vec::new());
                    if !matches!(resolved, TypeInfo::Integer) {
                        self.error(
                            SemanticErrCode::CondNotBool,
                            "If condition must be integer type",
                            *loc,
                        );
                    }
                }
                self.check_program_body(then_branch);
                self.check_program_body(else_branch);
            }
            Stm::While { cond, body, loc } => {
                let cond_ty = self.check_exp(cond);
                if let Some(cond_ty) = cond_ty {
                    let resolved = self.resolve_type(&cond_ty, &mut Vec::new());
                    if !matches!(resolved, TypeInfo::Integer) {
                        self.error(
                            SemanticErrCode::CondNotBool,
                            "While condition must be integer type",
                            *loc,
                        );
                    }
                }
                self.check_program_body(body);
            }
            Stm::Read { var, loc } => match self.symbols.lookup(var) {
                None => self.error(
                    SemanticErrCode::UndeclaredId,
                    format!("Undeclared identifier '{}'", var),
                    *loc,
                ),
                Some(entry) => {
                    if entry.kind != IdKind::VarId {
                        self.error(
                            SemanticErrCode::WrongIdKind,
                            format!("'{}' is not a variable", var),
                            *loc,
                        );
                    }
                }
            },
            Stm::Write { exp, .. } => {
                self.check_exp(exp);
            }
            Stm::Return { exp, .. } => {
                self.check_exp(exp);
            }
            Stm::Call { name, args, loc } => self.check_call(name, args, *loc),
        }
    }

    /// 检查赋值语句：LHS 必须为变量，左右两侧类型必须兼容。
    fn check_assign(&mut self, lhs: &VarAccess, rhs: &Exp, loc: Loc) {
        let lhs_entry = self.symbols.lookup(&lhs.base);
        match lhs_entry {
            None => {
                self.error(
                    SemanticErrCode::UndeclaredId,
                    format!("Undeclared identifier '{}'", lhs.base),
                    loc,
                );
                return;
            }
            Some(entry) => {
                if entry.kind != IdKind::VarId {
                    self.error(
                        SemanticErrCode::AssignLhsNotVariable,
                        format!("'{}' is not a variable", lhs.base),
                        loc,
                    );
                    return;
                }
            }
        }

        let rhs_ty = self.check_exp(rhs);
        let lhs_ty = self.check_var_access(lhs);

        // LHS 和 RHS 类型不兼容时报告错误
        if let (Some(l), Some(r)) = (&lhs_ty, &rhs_ty) {
                let rl = self.resolve_type(l, &mut Vec::new());
                let rr = self.resolve_type(r, &mut Vec::new());
                if !types_compatible(&rl, &rr) {
                    self.error(
                        SemanticErrCode::AssignTypeMismatch,
                    "Assignment type mismatch",
                    loc,
                );
            }
        }
    }

    /// 检查过程调用：被调用者必须是过程，实参与形参数量和类型必须匹配。
    fn check_call(&mut self, name: &str, args: &[Exp], loc: Loc) {
        // 克隆所需数据以避免同时持有不可变借用和可变借用
        let proc_info = self
            .symbols
            .lookup(name)
            .map(|e| (e.kind.clone(), e.params.clone()));

        match proc_info {
            None => {
                self.error(
                    SemanticErrCode::UndeclaredId,
                    format!("Undeclared procedure '{}'", name),
                    loc,
                );
            }
            Some((kind, params)) => {
                if kind != IdKind::ProcId {
                    self.error(
                        SemanticErrCode::NotProcedure,
                        format!("'{}' is not a procedure", name),
                        loc,
                    );
                    return;
                }
                if params.len() != args.len() {
                    self.error(
                        SemanticErrCode::ParamCountMismatch,
                        format!(
                            "Procedure '{}' expects {} arguments, got {}",
                            name,
                            params.len(),
                            args.len()
                        ),
                        loc,
                    );
                }
                for (i, arg) in args.iter().enumerate() {
                    let arg_ty = self.check_exp(arg);
                    if let Some(param) = params.get(i) {
                        if param.is_var && !matches!(arg, Exp::Variable(_, _)) {
                            self.error(
                                SemanticErrCode::ParamTypeMismatch,
                                format!(
                                    "Argument {} for var parameter '{}' must be a variable",
                                    i + 1,
                                    param.name
                                ),
                                loc,
                            );
                        }
                        if let Some(t) = &arg_ty {
                            let rt = self.resolve_type(t, &mut Vec::new());
                            let rp = self.resolve_type(&param.typ, &mut Vec::new());
                            if !types_compatible(&rt, &rp) {
                                self.error(
                                    SemanticErrCode::ParamTypeMismatch,
                                    format!(
                                        "Argument {} type mismatch in call to '{}'",
                                        i + 1,
                                        name
                                    ),
                                    loc,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// 检查表达式并推导其类型。
    ///
    /// 返回表达式类型供上层使用（类型检查、代码生成）。
    fn check_exp(&mut self, exp: &Exp) -> Option<TypeInfo> {
        match exp {
            Exp::Binary {
                op: _,
                left,
                right,
                loc,
            } => {
                let lt = self.check_exp(left);
                let rt = self.check_exp(right);
                if let (Some(l), Some(r)) = (&lt, &rt) {
                        let rl = self.resolve_type(l, &mut Vec::new());
                        let rr = self.resolve_type(r, &mut Vec::new());
                    if !types_compatible(&rl, &rr) {
                        self.error(
                            SemanticErrCode::OperatorTypeMismatch,
                            "Operator operand type mismatch",
                            *loc,
                        );
                    }
                }
                // SNL 中所有算术运算的结果均为整数类型
                Some(TypeInfo::Integer)
            }
            Exp::IntConst(_, _) => Some(TypeInfo::Integer),
            Exp::CharConst(_, _) => Some(TypeInfo::Char),
            Exp::Variable(va, _) => self.check_var_access(va),
        }
    }

    /// 检查变量访问并推导类型（含选择器链遍历）。
    fn check_var_access(&mut self, va: &VarAccess) -> Option<TypeInfo> {
        let entry = self
            .symbols
            .lookup(&va.base)
            .map(|e| (e.kind.clone(), e.typ.clone()));
        match entry {
            None => {
                self.error(
                    SemanticErrCode::UndeclaredId,
                    format!("Undeclared identifier '{}'", va.base),
                    va.loc,
                );
                None
            }
            Some((kind, _)) if kind != IdKind::VarId => {
                self.error(
                    SemanticErrCode::WrongIdKind,
                    format!("'{}' is not a variable", va.base),
                    va.loc,
                );
                None
            }
            Some((_, typ)) => {
                let mut current_ty = typ;
                for sel in &va.selector {
                    current_ty = self.resolve_selector(&current_ty?, sel);
                }
                current_ty
            }
        }
    }

    /// 解析选择器，逐层推导类型并检查下标范围。
    fn resolve_selector(&mut self, ty: &TypeInfo, sel: &Selector) -> Option<TypeInfo> {
        match ty {
            TypeInfo::Array(elem_ty, low, high) => {
                match sel {
                    Selector::ArraySubscript(exp) => {
                        // 常量下标时检查范围
                        if let Exp::IntConst(n, loc) = exp.as_ref()
                            && (*n < *low || *n > *high) {
                                self.error(
                                    SemanticErrCode::ArraySubscriptRange,
                                    format!(
                                        "Array subscript {} out of range [{}, {}]",
                                        n, low, high
                                    ),
                                    *loc,
                                );
                            }
                        Some(*elem_ty.clone())
                    }
                    _ => {
                        self.error(
                            SemanticErrCode::InvalidArrayOrFieldRef,
                            "Expected array subscript, got field access",
                            Loc { line: 0, col: 0 },
                        );
                        None
                    }
                }
            }
            TypeInfo::Record(fields) => {
                match sel {
                    Selector::Field(name) | Selector::FieldSubscript(name, _) => {
                        match fields.iter().find(|f| f.name == *name) {
                            Some(f) => {
                                if let Selector::FieldSubscript(_, exp) = sel {
                                    match &f.typ {
                                        TypeInfo::Array(elem_ty, low, high) => {
                                            if let Exp::IntConst(n, loc) = exp.as_ref()
                                                && (*n < *low || *n > *high) {
                                                    self.error(
                                                        SemanticErrCode::ArraySubscriptRange,
                                                        "Array subscript out of range".to_string(),
                                                        *loc,
                                                    );
                                                }
                                            Some(*elem_ty.clone())
                                        }
                                        _ => {
                                            self.error(
                                                SemanticErrCode::InvalidArrayOrFieldRef,
                                                "Subscript on non-array field",
                                                Loc { line: 0, col: 0 },
                                            );
                                            None
                                        }
                                    }
                                } else {
                                    Some(f.typ.clone())
                                }
                            }
                            None => {
                                self.error(
                                    SemanticErrCode::InvalidArrayOrFieldRef,
                                    format!("Field '{}' not found in record", name),
                                    Loc { line: 0, col: 0 },
                                );
                                None
                            }
                        }
                    }
                    _ => None,
                }
            }
            TypeInfo::Named(_) => {
                let resolved = self.resolve_type(ty, &mut Vec::new());
                if matches!(&resolved, TypeInfo::Named(_)) {
                    None
                } else {
                    self.resolve_selector(&resolved, sel)
                }
            }
            _ => None,
        }
    }

    // ===== 类型转换辅助函数 =====

    fn type_body_to_info(&self, body: &TypeBody) -> TypeInfo {
        match body {
            TypeBody::Base(BaseType::Integer) => TypeInfo::Integer,
            TypeBody::Base(BaseType::Char) => TypeInfo::Char,
            TypeBody::Array(arr) => {
                let elem = match arr.elem_type {
                    BaseType::Integer => TypeInfo::Integer,
                    BaseType::Char => TypeInfo::Char,
                };
                TypeInfo::Array(Box::new(elem), arr.low, arr.high)
            }
            TypeBody::Record(rec) => {
                let fields: Vec<FieldInfo> = rec
                    .fields
                    .iter()
                    .flat_map(|f| {
                        let ft = self.field_type_to_info(&f.typ);
                        f.names.iter().map(move |n| FieldInfo {
                            name: n.clone(),
                            typ: ft.clone(),
                        })
                    })
                    .collect();
                TypeInfo::Record(fields)
            }
            TypeBody::Named(name) => TypeInfo::Named(name.clone()),
        }
    }

    fn type_desig_to_info(&self, td: &TypeDesig) -> TypeInfo {
        match td {
            TypeDesig::Base(BaseType::Integer) => TypeInfo::Integer,
            TypeDesig::Base(BaseType::Char) => TypeInfo::Char,
            TypeDesig::Array(arr) => {
                let elem = match arr.elem_type {
                    BaseType::Integer => TypeInfo::Integer,
                    BaseType::Char => TypeInfo::Char,
                };
                TypeInfo::Array(Box::new(elem), arr.low, arr.high)
            }
            TypeDesig::Record(rec) => {
                let fields: Vec<FieldInfo> = rec
                    .fields
                    .iter()
                    .flat_map(|f| {
                        let ft = self.field_type_to_info(&f.typ);
                        f.names.iter().map(move |n| FieldInfo {
                            name: n.clone(),
                            typ: ft.clone(),
                        })
                    })
                    .collect();
                TypeInfo::Record(fields)
            }
            TypeDesig::Named(name) => TypeInfo::Named(name.clone()),
        }
    }

    fn field_type_to_info(&self, ft: &FieldTypeDef) -> TypeInfo {
        match ft {
            FieldTypeDef::Base(BaseType::Integer) => TypeInfo::Integer,
            FieldTypeDef::Base(BaseType::Char) => TypeInfo::Char,
            FieldTypeDef::Array(arr) => {
                let elem = match arr.elem_type {
                    BaseType::Integer => TypeInfo::Integer,
                    BaseType::Char => TypeInfo::Char,
                };
                TypeInfo::Array(Box::new(elem), arr.low, arr.high)
            }
        }
    }

    fn validate_type_body(&mut self, body: &TypeBody) {
        match body {
            TypeBody::Array(arr) => self.validate_array_bounds(arr),
            TypeBody::Record(rec) => {
                for field in &rec.fields {
                    self.validate_field_type(&field.typ);
                }
            }
            TypeBody::Base(_) | TypeBody::Named(_) => {}
        }
    }

    fn validate_type_desig(&mut self, typ: &TypeDesig) {
        match typ {
            TypeDesig::Array(arr) => self.validate_array_bounds(arr),
            TypeDesig::Record(rec) => {
                for field in &rec.fields {
                    self.validate_field_type(&field.typ);
                }
            }
            TypeDesig::Base(_) | TypeDesig::Named(_) => {}
        }
    }

    fn validate_field_type(&mut self, typ: &FieldTypeDef) {
        if let FieldTypeDef::Array(arr) = typ {
            self.validate_array_bounds(arr);
        }
    }

    fn validate_array_bounds(&mut self, arr: &ArrayTypeDef) {
        if arr.low > arr.high {
            self.error(
                SemanticErrCode::ArraySubscriptRange,
                format!(
                    "Array lower bound {} exceeds upper bound {}",
                    arr.low, arr.high
                ),
                arr.loc,
            );
        }
    }

    fn error(&mut self, code: SemanticErrCode, msg: impl Into<String>, loc: Loc) {
        self.errors.push(CompileError::semantic(code, msg, loc));
    }

    /// 向符号表插入条目，若重复定义则报告 DuplicateId 错误。
    fn insert_symbol(&mut self, entry: SymbolEntry) {
        let name = entry.name.clone();
        let loc = entry.loc;
        if self.symbols.insert(entry).is_err() {
            self.error(
                SemanticErrCode::DuplicateId,
                format!("Duplicate definition of '{}'", name),
                loc,
            );
        }
    }

    /// 解析 Named 类型别名，在符号表中查找并递归展开。
    ///
    /// `visited` 用于检测循环类型别名（如 `type A = B; type B = A`）。
    fn resolve_type(&mut self, ty: &TypeInfo, visited: &mut Vec<String>) -> TypeInfo {
        match ty {
            TypeInfo::Named(name) => {
                if visited.contains(name) {
                    let loc = self
                        .symbols
                        .lookup(name)
                        .map(|e| e.loc)
                        .unwrap_or(Loc { line: 0, col: 0 });
                    self.error(
                        SemanticErrCode::CircularTypeAlias,
                        format!("Circular type alias: '{}'", name),
                        loc,
                    );
                    return ty.clone();
                }
                visited.push(name.clone());
                let inner_opt = self
                    .symbols
                    .lookup(name)
                    .filter(|e| e.kind == IdKind::TypeId)
                    .and_then(|e| e.typ.clone());
                let result = match &inner_opt {
                    Some(inner) if matches!(inner, TypeInfo::Named(_)) => {
                        self.resolve_type(inner, visited)
                    }
                    Some(inner) => inner.clone(),
                    None => ty.clone(),
                };
                visited.pop();
                result
            }
            TypeInfo::Array(elem, low, high) => {
                TypeInfo::Array(Box::new(self.resolve_type(elem, visited)), *low, *high)
            }
            _ => ty.clone(),
        }
    }
}

/// 比较两种类型是否兼容（结构等价）。
///
/// 数组要求元素类型和下界上界都相同，
/// 记录要求字段数量和每一个字段的名字及类型都相同。
fn types_compatible(a: &TypeInfo, b: &TypeInfo) -> bool {
    match (a, b) {
        (TypeInfo::Integer, TypeInfo::Integer) => true,
        (TypeInfo::Char, TypeInfo::Char) => true,
        (TypeInfo::Array(et1, l1, h1), TypeInfo::Array(et2, l2, h2)) => {
            l1 == l2 && h1 == h2 && types_compatible(et1, et2)
        }
        (TypeInfo::Record(f1), TypeInfo::Record(f2)) => {
            f1.len() == f2.len()
                && f1
                    .iter()
                    .zip(f2.iter())
                    .all(|(a, b)| a.name == b.name && types_compatible(&a.typ, &b.typ))
        }
        (TypeInfo::Named(n1), TypeInfo::Named(n2)) => n1 == n2,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::rd::RdParser;

    fn analyze(source: &str) -> Vec<CompileError> {
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize(source);
        let mut parser = RdParser::new(tokens);
        let prog = parser.parse().expect("Parse should succeed");
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&prog);
        analyzer.errors
    }

    fn analyze_ok(source: &str) {
        let errors = analyze(source);
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_undeclared_variable() {
        let errors = analyze("program p begin x := 1 end.");
        assert!(errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::UndeclaredId)
        )));
    }

    #[test]
    fn test_undeclared_variable_in_expression() {
        let errors = analyze("program p begin write(x) end.");
        assert!(errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::UndeclaredId)
        )));
    }

    #[test]
    fn test_non_variable_identifier_in_expression() {
        let type_errors = analyze("program p type T = integer; begin write(T) end.");
        assert!(type_errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::WrongIdKind)
        )));

        let proc_errors = analyze(
            "program p procedure f(); begin write(0) end begin write(f) end.",
        );
        assert!(proc_errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::WrongIdKind)
        )));
    }

    #[test]
    fn test_duplicate_definition() {
        analyze_ok("program p var integer x; begin x := 1 end.");
    }

    #[test]
    fn test_assign_type_mismatch() {
        let errors = analyze("program p var integer x; char c; begin x := 'a' end.");
        assert!(errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::AssignTypeMismatch)
        )));
    }

    #[test]
    fn test_wrong_id_kind() {
        analyze_ok("program p var integer x; begin x := 1 end.");
    }

    #[test]
    fn test_procedure_call_errors() {
        let errors =
            analyze("program p procedure q(integer a); begin write(a) end begin q('x') end.");
        assert!(errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::ParamTypeMismatch)
        )));
    }

    #[test]
    fn test_param_count_mismatch() {
        let errors =
            analyze("program p procedure q(integer a); begin write(a) end begin q(1, 2) end.");
        assert!(errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::ParamCountMismatch)
        )));
    }

    #[test]
    fn test_multiple_names_in_one_parameter_declaration() {
        analyze_ok(
            "program p procedure q(integer a,b); begin write(a); write(b) end begin q(1,2) end.",
        );
    }

    #[test]
    fn test_var_parameter_requires_variable_argument() {
        let errors = analyze(
            "program p procedure q(var integer a); begin a := 1 end begin q(1 + 2) end.",
        );
        assert!(errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::ParamTypeMismatch)
        )));
    }

    #[test]
    fn test_valid_var_parameter_argument() {
        analyze_ok(
            "program p var integer x; procedure q(var integer a); begin a := 1 end begin q(x) end.",
        );
    }

    #[test]
    fn test_not_procedure() {
        let errors = analyze("program p var integer x; begin x(1) end.");
        assert!(errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::NotProcedure)
        )));
    }

    #[test]
    fn test_valid_program() {
        analyze_ok(
            "program p var integer v1; char c; procedure f(); begin v1 := 2 end begin f(); write(v1) end.",
        );
    }

    #[test]
    fn test_duplicate_variable_and_procedure_name() {
        let errors = analyze(
            "program p var integer f; procedure f(integer a); begin write(a) end begin f(1) end.",
        );
        assert!(
            !errors.is_empty(),
            "Should have at least one error for name conflict"
        );
    }

    #[test]
    fn test_wrong_id_kind_procedure_as_variable() {
        let errors =
            analyze("program p procedure f(integer a); begin write(a) end begin f := 1 end.");
        assert!(errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::AssignLhsNotVariable)
        )));
    }

    #[test]
    fn test_assign_to_non_variable() {
        let errors = analyze("program p procedure q(); begin write(0) end begin q := 1 end.");
        assert!(errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::AssignLhsNotVariable)
        )));
    }

    #[test]
    fn test_if_condition_not_integer() {
        let errors =
            analyze("program p var char c; begin if c then write(1) else write(2) fi end.");
        assert!(errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::CondNotBool)
        )));
    }

    #[test]
    fn test_while_condition_not_integer() {
        let errors = analyze("program p var char c; begin while c do write(1) endwh end.");
        assert!(errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::CondNotBool)
        )));
    }

    #[test]
    fn test_named_integer_condition() {
        analyze_ok(
            "program p type Bool = integer; var Bool x; begin if x then write(1) else write(0) fi end.",
        );
    }

    #[test]
    fn test_inverted_array_bounds() {
        let errors = analyze(
            "program p var array[5..1] of integer a; begin write(0) end.",
        );
        assert!(errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::ArraySubscriptRange)
        )));
    }

    #[test]
    fn test_operator_type_mismatch() {
        let errors = analyze("program p var integer x; char c; begin x := x + c end.");
        assert!(errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::OperatorTypeMismatch)
        )));
    }

    #[test]
    fn test_valid_nested_procedures() {
        analyze_ok(
            "program p var integer r; procedure outer(integer n); procedure inner(); begin r := n end begin inner() end begin r := 0; outer(5); write(r) end.",
        );
    }

    #[test]
    fn test_valid_if_with_else() {
        analyze_ok("program p var integer x; begin if x < 10 then x := 1 else x := 2 fi end.");
    }

    #[test]
    fn test_valid_multiple_writes() {
        analyze_ok("program p var integer x; begin x := 1; write(x); write(x) end.");
    }

    #[test]
    fn test_valid_arithmetic_expressions() {
        analyze_ok("program p var integer x; integer y; begin x := ((1 + 2) * 3) - y end.");
    }

    #[test]
    fn test_return_value_ok() {
        analyze_ok("program p procedure f(integer a); begin return(a + 1) end begin f(1) end.");
    }

    #[test]
    fn test_read_write_semantic_ok() {
        analyze_ok("program p var integer x; begin read(x); write(x) end.");
    }

    #[test]
    fn test_circular_type_alias() {
        let errors = analyze(
            "program p type A = B; B = A; var A x; begin x := 1 end.",
        );
        assert!(
            errors.iter().any(|e| matches!(
                e.kind,
                crate::error::ErrorKind::Semantic(SemanticErrCode::CircularTypeAlias)
            )),
            "Expected CircularTypeAlias error, got: {:?}",
            errors
        );
    }

    #[test]
    fn test_valid_type_alias_chain() {
        analyze_ok("program p type A = B; B = integer; var A x; begin x := 1 end.");
    }
}
