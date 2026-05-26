//! Token 类型定义。
//!
//! 词法分析器输出的基本单元，包括关键字、运算符、字面量和标识符。

/// Token（词法单元）种类。
///
/// 其中 `Ident`、`IntConst`、`CharConst` 携带具体值，
/// 其余变体为无数据的符号标记。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // ---- 保留字 ----
    Program,
    Type,
    Var,
    Procedure,
    Begin,
    End,
    Integer,
    Char,
    Array,
    Record,
    Of,
    While,
    Do,
    EndWh,
    If,
    Then,
    Else,
    Fi,
    Return,
    Read,
    Write,

    // ---- 多字符运算符 ----
    /// `:=`
    Assign,
    /// `..`
    Range,

    // ---- 单字符运算符/分隔符 ----
    Plus,
    Minus,
    Times,
    Divide,
    LParent,
    RParent,
    LBracket,
    RBracket,
    Semicolon,
    Dot,
    Comma,
    Less,
    Equal,

    // ---- 字面量 ----
    Ident(String),
    IntConst(i64),
    CharConst(char),

    // ---- 特殊 ----
    /// 输入结束标记
    Eof,
}

/// 一个完整的 Token，包含种类和源码位置。
///
/// 行号和列号从 1 开始计数。
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    /// 行号（从 1 开始）
    pub line: usize,
    /// 列号（从 1 开始）
    pub col: usize,
}
