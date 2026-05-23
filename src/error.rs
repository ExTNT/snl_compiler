use crate::ast::nodes::Loc;

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticErrCode {
    DuplicateId,
    UndeclaredId,
    WrongIdKind,
    ArraySubscriptRange,
    InvalidArrayOrFieldRef,
    AssignTypeMismatch,
    AssignLhsNotVariable,
    ParamTypeMismatch,
    ParamCountMismatch,
    NotProcedure,
    CondNotBool,
    OperatorTypeMismatch,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Lexical,
    Syntax,
    Semantic(SemanticErrCode),
}

#[derive(Debug, Clone)]
pub struct CompileError {
    pub kind: ErrorKind,
    pub msg: String,
    pub loc: Loc,
}

impl CompileError {
    pub fn lexical(msg: impl Into<String>, line: usize, col: usize) -> Self {
        CompileError {
            kind: ErrorKind::Lexical,
            msg: msg.into(),
            loc: Loc { line, col },
        }
    }

    pub fn syntax(msg: impl Into<String>, loc: Loc) -> Self {
        CompileError {
            kind: ErrorKind::Syntax,
            msg: msg.into(),
            loc,
        }
    }

    pub fn semantic(code: SemanticErrCode, msg: impl Into<String>, loc: Loc) -> Self {
        CompileError {
            kind: ErrorKind::Semantic(code),
            msg: msg.into(),
            loc,
        }
    }
}
