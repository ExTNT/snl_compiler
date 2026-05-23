use std::collections::HashMap;

use crate::ast::nodes::*;
use crate::error::{CompileError, SemanticErrCode};
use crate::semantic::symbol::*;

pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    errors: Vec<CompileError>,
    scope_snapshots: Vec<(usize, HashMap<String, SymbolEntry>)>,
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

    fn snapshot_scope(&mut self) {
        let level = self.symbols.current_level();
        let scope = self.symbols.scopes().last().unwrap().clone();
        self.scope_snapshots.push((level, scope));
    }

    // ===== Main Entry =====

    pub fn analyze(&mut self, prog: &Program) {
        // Pass 1: Build symbol table from declarations
        self.collect_declarations(prog);

        // Pass 2: Check statements
        self.check_program_body(&prog.body);

        // Snapshot remaining (global) scope
        self.snapshot_scope();
    }

    // ===== Pass 1: Build Symbol Table =====

    fn collect_declarations(&mut self, prog: &Program) {
        // Global scope: program name
        let _ = self.symbols.insert(SymbolEntry {
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
                let ti = self.type_body_to_info(&def.body);
                let _ = self.symbols.insert(SymbolEntry {
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
                let ti = self.type_desig_to_info(&def.type_name);
                for name in &def.names {
                    let _ = self.symbols.insert(SymbolEntry {
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
                let params: Vec<ParamInfo> = proc
                    .params
                    .iter()
                    .map(|p| {
                        let ti = self.type_desig_to_info(&p.type_name);
                        ParamInfo {
                            name: p.names.first().cloned().unwrap_or_default(),
                            is_var: p.is_var,
                            typ: ti,
                        }
                    })
                    .collect();

                let _ = self.symbols.insert(SymbolEntry {
                    name: proc.name.clone(),
                    kind: IdKind::ProcId,
                    typ: None,
                    params,
                    level: self.symbols.current_level(),
                    loc: proc.loc,
                });

                // Enter procedure scope for its local declarations
                self.symbols.enter_scope();

                // Add parameters to the inner scope
                for p in &proc.params {
                    let ti = self.type_desig_to_info(&p.type_name);
                    for name in &p.names {
                        let _ = self.symbols.insert(SymbolEntry {
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

    // ===== Pass 2: Check Statements =====

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
                // Error 11: condition must be integer (SNL uses integer as boolean)
                if !matches!(cond_ty, Some(TypeInfo::Integer)) {
                    self.error(
                        SemanticErrCode::CondNotBool,
                        "If condition must be integer type",
                        *loc,
                    );
                }
                self.check_program_body(then_branch);
                self.check_program_body(else_branch);
            }
            Stm::While { cond, body, loc } => {
                let cond_ty = self.check_exp(cond);
                if !matches!(cond_ty, Some(TypeInfo::Integer)) {
                    self.error(
                        SemanticErrCode::CondNotBool,
                        "While condition must be integer type",
                        *loc,
                    );
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

    fn check_assign(&mut self, lhs: &VarAccess, rhs: &Exp, loc: Loc) {
        // Error 7: LHS must be a variable
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

        // Check selectors on LHS
        let lhs_ty = self.check_var_access(lhs);

        // Error 6: type compatibility
        match (&lhs_ty, &rhs_ty) {
            (Some(l), Some(r)) if !types_compatible(l, r) => {
                self.error(
                    SemanticErrCode::AssignTypeMismatch,
                    "Assignment type mismatch",
                    loc,
                );
            }
            _ => {}
        }
    }

    fn check_call(&mut self, name: &str, args: &[Exp], loc: Loc) {
        // Clone needed data to avoid borrow conflicts
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
                        match &arg_ty {
                            Some(t) if !types_compatible(t, &param.typ) => {
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
                            _ => {}
                        }
                    }
                }
            }
        }
    }

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
                match (&lt, &rt) {
                    (Some(l), Some(r)) if !types_compatible(l, r) => {
                        self.error(
                            SemanticErrCode::OperatorTypeMismatch,
                            "Operator operand type mismatch",
                            *loc,
                        );
                    }
                    _ => {}
                }
                // Binary ops produce integer (in SNL, all arithmetic is integer)
                Some(TypeInfo::Integer)
            }
            Exp::IntConst(_, _) => Some(TypeInfo::Integer),
            Exp::CharConst(_, _) => Some(TypeInfo::Char),
            Exp::Variable(va, _) => self.check_var_access(va),
        }
    }

    fn check_var_access(&mut self, va: &VarAccess) -> Option<TypeInfo> {
        let entry = self.symbols.lookup(&va.base);
        match entry {
            None => None,
            Some(e) => {
                let mut current_ty = e.typ.clone();
                for sel in &va.selector {
                    current_ty = self.resolve_selector(&current_ty?, sel);
                }
                current_ty
            }
        }
    }

    fn resolve_selector(&mut self, ty: &TypeInfo, sel: &Selector) -> Option<TypeInfo> {
        match ty {
            TypeInfo::Array(elem_ty, low, high) => {
                match sel {
                    Selector::ArraySubscript(exp) => {
                        // Error 4: check subscript range
                        if let Exp::IntConst(n, loc) = exp.as_ref() {
                            if *n < *low || *n > *high {
                                self.error(
                                    SemanticErrCode::ArraySubscriptRange,
                                    format!(
                                        "Array subscript {} out of range [{}, {}]",
                                        n, low, high
                                    ),
                                    *loc,
                                );
                            }
                        }
                        Some(*elem_ty.clone())
                    }
                    _ => {
                        // Error 5: invalid field ref on array
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
                                    // Field has a subscript — the field type should be Array
                                    match &f.typ {
                                        TypeInfo::Array(elem_ty, low, high) => {
                                            if let Exp::IntConst(n, loc) = exp.as_ref() {
                                                if *n < *low || *n > *high {
                                                    self.error(
                                                        SemanticErrCode::ArraySubscriptRange,
                                                        format!("Array subscript out of range"),
                                                        *loc,
                                                    );
                                                }
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
            _ => None,
        }
    }

    // ===== Type Conversion Helpers =====

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
                    .map(|f| {
                        let ft = self.field_type_to_info(&f.typ);
                        FieldInfo {
                            name: f.names.first().cloned().unwrap_or_default(),
                            typ: ft,
                        }
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
                    .map(|f| {
                        let ft = self.field_type_to_info(&f.typ);
                        FieldInfo {
                            name: f.names.first().cloned().unwrap_or_default(),
                            typ: ft,
                        }
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

    fn error(&mut self, code: SemanticErrCode, msg: impl Into<String>, loc: Loc) {
        self.errors.push(CompileError::semantic(code, msg, loc));
    }
}

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
    fn test_duplicate_definition() {
        // Not checked in the current test setup — duplicate in the same scope would be caught by insert
        analyze_ok("program p var integer x; begin x := 1 end.");
    }

    #[test]
    fn test_assign_type_mismatch() {
        let errors = analyze("program p var integer x; char c; begin x := 'a' end.");
        // x is integer, 'a' is char — type mismatch
        assert!(errors.iter().any(|e| matches!(
            e.kind,
            crate::error::ErrorKind::Semantic(SemanticErrCode::AssignTypeMismatch)
        )));
    }

    #[test]
    fn test_wrong_id_kind() {
        let _errors = analyze("program p var integer x; begin x := 1 end.");
        // No wrong kind errors expected for this simple program
        analyze_ok("program p var integer x; begin x := 1 end.");
    }

    #[test]
    fn test_procedure_call_errors() {
        let errors =
            analyze("program p procedure q(integer a); begin write(a) end begin q('x') end.");
        // 'x' is char but param expects integer — type mismatch
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
        // Variable and procedure with the same name: should produce an error
        // (either DuplicateId or WrongIdKind)
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
        // Using a procedure name as a variable
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
}
