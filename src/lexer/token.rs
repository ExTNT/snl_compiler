#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // Reserved words
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

    // Multi-character operators
    Assign, // :=
    Range,  // ..

    // Single-character operators / delimiters
    Plus,      // +
    Minus,     // -
    Times,     // *
    Divide,    // /
    LParent,   // (
    RParent,   // )
    LBracket,  // [
    RBracket,  // ]
    Semicolon, // ;
    Dot,       // .
    Comma,     // ,
    Less,      // <
    Equal,     // =

    // Literals
    Ident(String),
    IntConst(i64),
    CharConst(char),

    // Special
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}
