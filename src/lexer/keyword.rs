use super::token::TokenKind;

type KeywordEntry = (&'static str, TokenKind);

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

pub fn lookup_keyword(ident: &str) -> TokenKind {
    let lower = ident.to_lowercase();
    KEYWORDS
        .binary_search_by(|(kw, _)| kw.cmp(&lower.as_str()))
        .map(|i| KEYWORDS[i].1.clone())
        .unwrap_or_else(|_| TokenKind::Ident(ident.to_string()))
}
