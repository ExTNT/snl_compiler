//! 确定性有限自动机（DFA）实现的词法分析引擎。
//!
//! 以字符为驱动单位，在有限状态之间迁移，当识别出一个完整的 Token
//! 或到达终止状态时返回结果。支持回溯：当当前字符属于下一个 Token
//! 的前缀时，通过 `backtrack` 标记通知调用方不要消费该字符。
//!
//! ## 状态说明
//! - `Start` — 初始状态，等待一个 Token 的起始字符
//! - `InIdent` — 正在收集标识符（字母开头，后跟字母或数字）
//! - `InNumber` — 正在收集整数
//! - `InAssign` — 已读到 `:`，等待 `=` 以形成 `:=`
//! - `InComment` — 已读到 `{`，等待 `}` 结束注释
//! - `InChar` — 已读到 `'`，等待字符内容
//! - `InCharEnd` — 已读到字符内容，等待闭合 `'`
//! - `InRange` — 已读到一个 `.`，等待第二个 `.` 以形成 `..`
//! - `Done` — 本 Token 已完成，等待调用方 reset

use super::token::TokenKind;

/// DFA 内部状态。
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

/// DFA 引擎，负责逐字符推进状态并产出 Token。
///
/// 每个 Token 识别周期为：
/// 1. 调用 `advance(ch)` 逐字符推进
/// 2. 当 `advance` 返回 `Some(DfaResult)` 时，Token 已完整识别
/// 3. 调用 `reset()` 回到 `Start` 状态以开始下一个 Token
pub struct Dfa {
    pub(crate) state: DfaState,
    pub(crate) lexeme: String,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

/// DFA 识别完成后的输出。
#[derive(Debug, Clone)]
pub struct DfaResult {
    /// 识别到的 Token 种类
    pub kind: Result<TokenKind, String>,
    /// Token 的起始行号
    pub line: usize,
    /// Token 的起始列号
    pub col: usize,
    /// 若为 true，导致本结果的当前字符需要留待下一次重新处理
    pub backtrack: bool,
}

impl Dfa {
    /// 创建新的 DFA，初始状态为 `Start`。
    pub fn new(line: usize, col: usize) -> Self {
        Dfa {
            state: DfaState::Start,
            lexeme: String::new(),
            line,
            col,
        }
    }

    /// 重置 DFA 到初始状态，准备识别下一个 Token。
    pub fn reset(&mut self, line: usize, col: usize) {
        self.state = DfaState::Start;
        self.lexeme.clear();
        self.line = line;
        self.col = col;
    }

    /// 推进一个字符。
    ///
    /// 返回 `Some(DfaResult)` 表示一个 Token 已完成；
    /// 返回 `None` 表示需要更多字符才能完成当前 Token。
    ///
    /// # 参数
    /// - `ch`: 当前处理的字符
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

    /// 强制结束当前 Token 的识别（用于到达 EOF 时）。
    ///
    /// 根据当前状态，将半成品 Token 冲刷为最终结果或丢弃。
    pub fn finish(&self) -> Option<DfaResult> {
        match self.state {
            DfaState::Start => None,
            DfaState::InIdent => {
                let kind = super::keyword::lookup_keyword(&self.lexeme);
                Some(DfaResult {
                    kind: Ok(kind),
                    line: self.line,
                    col: self.col,
                    backtrack: false,
                })
            }
            DfaState::InNumber => {
                Some(DfaResult {
                    kind: self
                        .lexeme
                        .parse::<i64>()
                        .map(TokenKind::IntConst)
                        .map_err(|_| "Integer literal out of range".to_string()),
                    line: self.line,
                    col: self.col,
                    backtrack: false,
                })
            }
            DfaState::InAssign => {
                Some(DfaResult {
                    kind: Err("Unexpected ':'; expected ':='".to_string()),
                    line: self.line,
                    col: self.col,
                    backtrack: false,
                })
            }
            DfaState::InComment => None,
            DfaState::InChar | DfaState::InCharEnd => None,
            DfaState::InRange => Some(DfaResult {
                kind: Ok(TokenKind::Dot),
                line: self.line,
                col: self.col,
                backtrack: false,
            }),
            DfaState::Done => None,
        }
    }

    /// `Start` 状态：根据输入字符决定进入哪个子状态。
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
            // 单字符 Token：直接从 Start 生成结果
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
            _ => Some(DfaResult {
                kind: Err(format!("Invalid character '{}'", ch)),
                line: self.line,
                col: self.col,
                backtrack: false,
            }),
        }
    }

    /// 创建不含回溯的简单结果。
    fn result(&self, kind: TokenKind) -> DfaResult {
        DfaResult {
            kind: Ok(kind),
            line: self.line,
            col: self.col,
            backtrack: false,
        }
    }

    /// `InIdent` 状态：继续收集字母/数字，遇到其他字符则结束。
    fn in_ident(&mut self, ch: char) -> Option<DfaResult> {
        if ch.is_ascii_alphanumeric() {
            self.lexeme.push(ch);
            None
        } else {
            let kind = super::keyword::lookup_keyword(&self.lexeme);
            self.state = DfaState::Done;
            Some(DfaResult {
                kind: Ok(kind),
                line: self.line,
                col: self.col,
                backtrack: true,
            })
        }
    }

    /// `InNumber` 状态：继续收集数字，遇到非数字字符则结束。
    fn in_number(&mut self, ch: char) -> Option<DfaResult> {
        if ch.is_ascii_digit() {
            self.lexeme.push(ch);
            None
        } else {
            self.state = DfaState::Done;
            Some(DfaResult {
                kind: self
                    .lexeme
                    .parse::<i64>()
                    .map(TokenKind::IntConst)
                    .map_err(|_| "Integer literal out of range".to_string()),
                line: self.line,
                col: self.col,
                backtrack: true,
            })
        }
    }

    fn in_assign(&mut self, ch: char) -> Option<DfaResult> {
        if ch == '=' {
            // 完整赋值符 ":="，已消费 = 
            self.state = DfaState::Done;
            Some(DfaResult {
                kind: Ok(TokenKind::Assign),
                line: self.line,
                col: self.col,
                backtrack: false,
            })
        } else {
            self.state = DfaState::Done;
            Some(DfaResult {
                kind: Err("Unexpected ':'; expected ':='".to_string()),
                line: self.line,
                col: self.col,
                backtrack: true,
            })
        }
    }

    /// `InComment` 状态：忽略所有字符直到遇到 `}`。
    fn in_comment(&mut self, ch: char) -> Option<DfaResult> {
        if ch == '}' {
            self.state = DfaState::Start;
            None
        } else {
            None
        }
    }

    /// `InChar` 状态：读取单个字符内容。
    fn in_char(&mut self, ch: char) -> Option<DfaResult> {
        self.lexeme.push(ch);
        self.state = DfaState::InCharEnd;
        None
    }

    /// `InCharEnd` 状态：期望闭合引号 `'`。
    fn in_char_end(&mut self, ch: char) -> Option<DfaResult> {
        self.state = DfaState::Done;
        if ch == '\'' {
            let c = self.lexeme.chars().next().unwrap_or('\0');
            Some(DfaResult {
                kind: Ok(TokenKind::CharConst(c)),
                line: self.line,
                col: self.col,
                backtrack: false,
            })
        } else {
            Some(DfaResult {
                kind: Err("Unterminated character literal".to_string()),
                line: self.line,
                col: self.col,
                backtrack: true,
            })
        }
    }

    /// `InRange` 状态：第一个 `.` 后，检查是否为 `..`。
    fn in_range(&mut self, ch: char) -> Option<DfaResult> {
        self.state = DfaState::Done;
        if ch == '.' {
            // ".." — Range 运算符，第二个点已消费
            Some(DfaResult {
                kind: Ok(TokenKind::Range),
                line: self.line,
                col: self.col,
                backtrack: false,
            })
        } else {
            // 单个 "." — Dot 运算符，当前字符需回溯
            Some(DfaResult {
                kind: Ok(TokenKind::Dot),
                line: self.line,
                col: self.col,
                backtrack: true,
            })
        }
    }
}
