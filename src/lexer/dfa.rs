use super::token::TokenKind;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DfaState {
    Start,
    InIdent,
    InNumber,
    InAssign,
    InComment,
    InChar,
    InCharEnd,
    InRange,
    Done,
}

pub struct Dfa {
    pub(crate) state: DfaState,
    pub(crate) lexeme: String,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

#[derive(Debug, Clone)]
pub struct DfaResult {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
    /// If true, the character that caused this result should be re-processed (backtrack).
    /// If false, it was consumed as part of this token.
    pub backtrack: bool,
}

impl Dfa {
    pub fn new(line: usize, col: usize) -> Self {
        Dfa {
            state: DfaState::Start,
            lexeme: String::new(),
            line,
            col,
        }
    }

    pub fn reset(&mut self, line: usize, col: usize) {
        self.state = DfaState::Start;
        self.lexeme.clear();
        self.line = line;
        self.col = col;
    }

    /// Returns `Some(DfaResult)` when a token is complete, `None` if more chars are needed.
    /// `DfaResult.backtrack` is true when the current char is a terminator (lookahead)
    /// that should be re-processed for the next token.
    pub fn advance(&mut self, ch: char) -> Option<DfaResult> {
        match self.state {
            DfaState::Start => self.start_state(ch),
            DfaState::InIdent => self.in_ident(ch),
            DfaState::InNumber => self.in_number(ch),
            DfaState::InAssign => self.in_assign(ch),
            DfaState::InComment => self.in_comment(ch),
            DfaState::InChar => self.in_char(ch),
            DfaState::InCharEnd => self.in_char_end(ch),
            DfaState::InRange => self.in_range(ch),
            DfaState::Done => None,
        }
    }

    pub fn finish(&self) -> Option<DfaResult> {
        match self.state {
            DfaState::Start => None,
            DfaState::InIdent => {
                let kind = super::keyword::lookup_keyword(&self.lexeme);
                Some(DfaResult {
                    kind,
                    line: self.line,
                    col: self.col,
                    backtrack: false,
                })
            }
            DfaState::InNumber => {
                let val: i64 = self.lexeme.parse().unwrap_or(0);
                Some(DfaResult {
                    kind: TokenKind::IntConst(val),
                    line: self.line,
                    col: self.col,
                    backtrack: false,
                })
            }
            DfaState::InAssign => {
                // Stray ':' — treat as error, emit Eof as placeholder
                None
            }
            DfaState::InComment => None,
            DfaState::InChar | DfaState::InCharEnd => None,
            DfaState::InRange => Some(DfaResult {
                kind: TokenKind::Dot,
                line: self.line,
                col: self.col,
                backtrack: false,
            }),
            DfaState::Done => None,
        }
    }

    fn start_state(&mut self, ch: char) -> Option<DfaResult> {
        match ch {
            c if c.is_ascii_alphabetic() => {
                self.lexeme.push(ch);
                self.state = DfaState::InIdent;
                None
            }
            c if c.is_ascii_digit() => {
                self.lexeme.push(ch);
                self.state = DfaState::InNumber;
                None
            }
            ':' => {
                self.lexeme.push(ch);
                self.state = DfaState::InAssign;
                None
            }
            '{' => {
                self.state = DfaState::InComment;
                None
            }
            '\'' => {
                self.state = DfaState::InChar;
                None
            }
            '.' => {
                self.lexeme.push(ch);
                self.state = DfaState::InRange;
                None
            }
            '+' => Some(self.result(TokenKind::Plus)),
            '-' => Some(self.result(TokenKind::Minus)),
            '*' => Some(self.result(TokenKind::Times)),
            '/' => Some(self.result(TokenKind::Divide)),
            '(' => Some(self.result(TokenKind::LParent)),
            ')' => Some(self.result(TokenKind::RParent)),
            '[' => Some(self.result(TokenKind::LBracket)),
            ']' => Some(self.result(TokenKind::RBracket)),
            ';' => Some(self.result(TokenKind::Semicolon)),
            ',' => Some(self.result(TokenKind::Comma)),
            '<' => Some(self.result(TokenKind::Less)),
            '=' => Some(self.result(TokenKind::Equal)),
            _ => None,
        }
    }

    fn result(&self, kind: TokenKind) -> DfaResult {
        DfaResult {
            kind,
            line: self.line,
            col: self.col,
            backtrack: false,
        }
    }

    fn in_ident(&mut self, ch: char) -> Option<DfaResult> {
        if ch.is_ascii_alphanumeric() {
            self.lexeme.push(ch);
            None
        } else {
            let kind = super::keyword::lookup_keyword(&self.lexeme);
            self.state = DfaState::Done;
            Some(DfaResult {
                kind,
                line: self.line,
                col: self.col,
                backtrack: true,
            })
        }
    }

    fn in_number(&mut self, ch: char) -> Option<DfaResult> {
        if ch.is_ascii_digit() {
            self.lexeme.push(ch);
            None
        } else {
            let val: i64 = self.lexeme.parse().unwrap_or(0);
            self.state = DfaState::Done;
            Some(DfaResult {
                kind: TokenKind::IntConst(val),
                line: self.line,
                col: self.col,
                backtrack: true,
            })
        }
    }

    fn in_assign(&mut self, ch: char) -> Option<DfaResult> {
        self.state = DfaState::Done;
        if ch == '=' {
            // ":-" → consumed as part of token, no backtrack
            Some(DfaResult {
                kind: TokenKind::Assign,
                line: self.line,
                col: self.col,
                backtrack: false,
            })
        } else {
            // Stray ':' — invalid in SNL; backtrack needed
            Some(DfaResult {
                kind: TokenKind::Assign,
                line: self.line,
                col: self.col,
                backtrack: true,
            })
        }
    }

    fn in_comment(&mut self, ch: char) -> Option<DfaResult> {
        if ch == '}' {
            self.state = DfaState::Start;
            None // Comment ended, resume scanning
        } else {
            None // Still in comment
        }
    }

    fn in_char(&mut self, ch: char) -> Option<DfaResult> {
        self.lexeme.push(ch);
        self.state = DfaState::InCharEnd;
        None
    }

    fn in_char_end(&mut self, ch: char) -> Option<DfaResult> {
        self.state = DfaState::Done;
        if ch == '\'' {
            let c = self.lexeme.chars().next().unwrap_or('\0');
            Some(DfaResult {
                kind: TokenKind::CharConst(c),
                line: self.line,
                col: self.col,
                backtrack: false,
            })
        } else {
            // Malformed: multiple chars or missing closing quote
            let c = self.lexeme.chars().next().unwrap_or('\0');
            Some(DfaResult {
                kind: TokenKind::CharConst(c),
                line: self.line,
                col: self.col,
                backtrack: true,
            })
        }
    }

    fn in_range(&mut self, ch: char) -> Option<DfaResult> {
        self.state = DfaState::Done;
        if ch == '.' {
            // ".." → second dot consumed, no backtrack
            Some(DfaResult {
                kind: TokenKind::Range,
                line: self.line,
                col: self.col,
                backtrack: false,
            })
        } else {
            // Single "." → backtrack needed for the next char
            Some(DfaResult {
                kind: TokenKind::Dot,
                line: self.line,
                col: self.col,
                backtrack: true,
            })
        }
    }
}
