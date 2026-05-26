//! 编译错误类型定义。
//!
//! 提供统一的错误表示，覆盖词法、语法、语义三个阶段的错误。

use crate::ast::nodes::Loc;

/// 语义错误码，标识具体的语义违规类型。
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticErrCode {
    /// 重复定义的标识符
    DuplicateId,
    /// 未声明的标识符
    UndeclaredId,
    /// 标识符种类错误（如将过程名作为变量使用）
    WrongIdKind,
    /// 数组下标越界
    ArraySubscriptRange,
    /// 无效的数组或记录字段访问
    InvalidArrayOrFieldRef,
    /// 赋值类型不匹配
    AssignTypeMismatch,
    /// 赋值左侧不是变量
    AssignLhsNotVariable,
    /// 参数类型不匹配
    ParamTypeMismatch,
    /// 参数数量不匹配
    ParamCountMismatch,
    /// 调用目标不是过程
    NotProcedure,
    /// 条件表达式不是整数类型（SNL 以整数作为布尔值）
    CondNotBool,
    /// 运算符操作数类型不匹配
    OperatorTypeMismatch,
}

/// 错误阶段/类别。
#[derive(Debug, Clone)]
pub enum ErrorKind {
    /// 词法错误
    Lexical,
    /// 语法错误
    Syntax,
    /// 语义错误，携带具体错误码
    Semantic(SemanticErrCode),
}

/// 编译错误。
///
/// 包含错误类别、消息以及源位置信息。
#[derive(Debug, Clone)]
pub struct CompileError {
    /// 错误类别
    pub kind: ErrorKind,
    /// 错误描述
    pub msg: String,
    /// 源位置
    pub loc: Loc,
}

impl CompileError {
    /// 创建词法错误。
    pub fn lexical(msg: impl Into<String>, line: usize, col: usize) -> Self {
        CompileError {
            kind: ErrorKind::Lexical,
            msg: msg.into(),
            loc: Loc { line, col },
        }
    }

    /// 创建语法错误。
    pub fn syntax(msg: impl Into<String>, loc: Loc) -> Self {
        CompileError {
            kind: ErrorKind::Syntax,
            msg: msg.into(),
            loc,
        }
    }

    /// 创建语义错误。
    pub fn semantic(code: SemanticErrCode, msg: impl Into<String>, loc: Loc) -> Self {
        CompileError {
            kind: ErrorKind::Semantic(code),
            msg: msg.into(),
            loc,
        }
    }
}
