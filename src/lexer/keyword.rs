//! 关键字识别。
//!
//! SNL 关键字不区分大小写。词法分析器在完成标识符/关键字的词素收集后，
//! 通过二分查找在关键字表中匹配，未命中则视为普通标识符。

use super::token::TokenKind;

type KeywordEntry = (&'static str, TokenKind);

/// SNL 关键字表，按字母序排列以保证二分查找正确。
///
/// 该表为静态常量，编译期确定，无需运行时构造。
const KEYWORDS: &[KeywordEntry] = &[
    ("array", TokenKind::Array),
    ("begin", TokenKind::Begin),
    ("char", TokenKind::Char),
    ("do", TokenKind::Do),
    ("else", TokenKind::Else),
    ("end", TokenKind::End),
    ("endwh", TokenKind::EndWh),
    ("fi", TokenKind::Fi),
    ("if", TokenKind::If),
    ("integer", TokenKind::Integer),
    ("of", TokenKind::Of),
    ("procedure", TokenKind::Procedure),
    ("program", TokenKind::Program),
    ("read", TokenKind::Read),
    ("record", TokenKind::Record),
    ("return", TokenKind::Return),
    ("then", TokenKind::Then),
    ("type", TokenKind::Type),
    ("var", TokenKind::Var),
    ("while", TokenKind::While),
    ("write", TokenKind::Write),
];

/// 在关键字表中查找标识符。
///
/// 匹配成功返回对应的关键字 TokenKind，否则返回 `Ident` 变体。
/// 查找前会将输入转为小写以实现大小写不敏感。
///
/// # 参数
/// - `ident`: 待查的标识符字符串
///
/// # 返回
/// 匹配的 TokenKind（关键字或 `Ident`）
pub fn lookup_keyword(ident: &str) -> TokenKind {
    let lower = ident.to_ascii_lowercase();
    KEYWORDS
        .binary_search_by(|(kw, _)| kw.cmp(&lower.as_str()))
        .map(|i| KEYWORDS[i].1.clone())
        .unwrap_or_else(|_| TokenKind::Ident(ident.to_string()))
}
