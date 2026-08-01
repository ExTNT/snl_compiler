//! 词法分析器（Lexer）。
//!
//! 将 SNL 源代码分割为 Token 序列。采用 DFA 驱动的逐字符扫描方式，
//! 基于最长匹配原则识别 Token。注释（`{ ... }`）在词法阶段被丢弃，
//! 空白字符用于行号/列号追踪。
//!
//! ## 错误处理
//! 词法错误（如未闭合的注释或字符字面量）记录在 `errors` 列表中，
//! 不会中断扫描——词法分析器会尝试恢复并继续处理剩余源码。

mod dfa;
mod keyword;
pub mod token;

use dfa::{Dfa, DfaState};
pub use token::{Token, TokenKind};

/// 词法错误。
///
/// 记录错误消息及发生位置，不中断扫描流程。
#[derive(Debug, Clone)]
pub struct LexerError {
    pub msg: String,
    pub line: usize,
    pub col: usize,
}

/// 词法分析器。
///
/// 维护 Token 列表和错误列表，通过 `tokenize()` 一次完成整个源码的扫描。
pub struct Lexer {
    tokens: Vec<Token>,
    errors: Vec<LexerError>,
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}

impl Lexer {
    /// 创建新的词法分析器实例。
    pub fn new() -> Self {
        Lexer {
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// 对源码字符串进行分词。
    ///
    /// 该函数消费 `&mut self`，将结果写入内部列表。
    /// 返回 Token 和错误的引用切片，避免所有权转移。
    ///
    /// # 参数
    /// - `source`: 待分词的 SNL 源代码
    ///
    /// # 返回
    /// `(&[Token], &[LexerError])` — Token 序列和词法错误的引用
    pub fn tokenize(&mut self, source: &str) -> (&[Token], &[LexerError]) {
        let chars: Vec<char> = source.chars().collect();
        let mut dfa = Dfa::new(1, 1);
        let mut i = 0;
        let mut line: usize = 1;
        let mut col: usize = 1;

        while i < chars.len() {
            let ch = chars[i];

            // 在 Start 状态跳过空白字符和换行符
            if dfa.state == DfaState::Start {
                if ch == '\n' {
                    line += 1;
                    col = 1;
                    i += 1;
                    dfa.reset(line, col);
                    continue;
                }
                if ch.is_whitespace() {
                    col += 1;
                    i += 1;
                    dfa.reset(line, col);
                    continue;
                }
            }

            let result = dfa.advance(ch);

            match result {
                Some(r) if dfa.state == DfaState::Done => {
                    match r.kind {
                        Ok(kind) => self.tokens.push(Token {
                            kind,
                            line: r.line,
                            col: r.col,
                        }),
                        Err(msg) => self.errors.push(LexerError {
                            msg,
                            line: r.line,
                            col: r.col,
                        }),
                    }
                    if r.backtrack {
                        dfa.reset(line, col);
                    } else {
                        i += 1;
                        col += 1;
                        dfa.reset(line, col);
                    }
                    // 若 backtrack=true，当前字符将作为下一个 Token 的起始重新处理
                }
                Some(r) => {
                    match r.kind {
                        Ok(kind) => self.tokens.push(Token {
                            kind,
                            line: r.line,
                            col: r.col,
                        }),
                        Err(msg) => self.errors.push(LexerError {
                            msg,
                            line: r.line,
                            col: r.col,
                        }),
                    }
                    i += 1;
                    col += 1;
                    dfa.reset(line, col);
                }
                None => {
                    i += 1;
                    col += 1;
                    // 注释内的换行符需要追踪行号但不产出 Token
                    if ch == '\n' && dfa.state == DfaState::InComment {
                        line += 1;
                        col = 1;
                        dfa.line = line;
                        dfa.col = col;
                    } else if dfa.state == DfaState::Start {
                        dfa.reset(line, col);
                    }
                }
            }
        }

        // 到达 EOF 时冲刷未完成的状态
        match dfa.state {
            DfaState::Start => {}
            DfaState::InComment => {
                self.errors.push(LexerError {
                    msg: "Unterminated comment".to_string(),
                    line: dfa.line,
                    col: dfa.col,
                });
            }
            DfaState::InChar | DfaState::InCharEnd => {
                self.errors.push(LexerError {
                    msg: "Unterminated character literal".to_string(),
                    line: dfa.line,
                    col: dfa.col,
                });
            }
            _ => {
                if let Some(result) = dfa.finish() {
                    match result.kind {
                        Ok(kind) => self.tokens.push(Token {
                            kind,
                            line: result.line,
                            col: result.col,
                        }),
                        Err(msg) => self.errors.push(LexerError {
                            msg,
                            line: result.line,
                            col: result.col,
                        }),
                    }
                }
            }
        }

        // 追加 Eof 作为结尾标记
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            line,
            col,
        });
        (&self.tokens, &self.errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(source: &str) -> Vec<Token> {
        let mut lexer = Lexer::new();
        let (tokens, errors) = lexer.tokenize(source);
        assert!(errors.is_empty(), "Unexpected lexer errors: {:?}", errors);
        tokens.to_vec()
    }

    fn token_kinds(source: &str) -> Vec<TokenKind> {
        tokenize(source).iter().map(|t| t.kind.clone()).collect()
    }

    #[test]
    fn test_single_char_operators() {
        let kinds = token_kinds("+ - * / ( ) [ ] ; . , < =");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Times,
                TokenKind::Divide,
                TokenKind::LParent,
                TokenKind::RParent,
                TokenKind::LBracket,
                TokenKind::RBracket,
                TokenKind::Semicolon,
                TokenKind::Dot,
                TokenKind::Comma,
                TokenKind::Less,
                TokenKind::Equal,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_assign_operator() {
        let kinds = token_kinds(":=");
        assert_eq!(kinds, vec![TokenKind::Assign, TokenKind::Eof]);
    }

    #[test]
    fn test_range_operator() {
        let kinds = token_kinds("..");
        assert_eq!(kinds, vec![TokenKind::Range, TokenKind::Eof]);
    }

    #[test]
    fn test_dot_alone() {
        let kinds = token_kinds(".");
        assert_eq!(kinds, vec![TokenKind::Dot, TokenKind::Eof]);
    }

    #[test]
    fn test_keywords() {
        let source = "program var procedure begin end integer char array record type while do endwh if then else fi return read write of";
        let kinds = token_kinds(source);
        let expected = vec![
            TokenKind::Program,
            TokenKind::Var,
            TokenKind::Procedure,
            TokenKind::Begin,
            TokenKind::End,
            TokenKind::Integer,
            TokenKind::Char,
            TokenKind::Array,
            TokenKind::Record,
            TokenKind::Type,
            TokenKind::While,
            TokenKind::Do,
            TokenKind::EndWh,
            TokenKind::If,
            TokenKind::Then,
            TokenKind::Else,
            TokenKind::Fi,
            TokenKind::Return,
            TokenKind::Read,
            TokenKind::Write,
            TokenKind::Of,
            TokenKind::Eof,
        ];
        assert_eq!(kinds, expected);
    }

    #[test]
    fn test_identifiers() {
        let kinds = token_kinds("v1 f pp myVar");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident("v1".into()),
                TokenKind::Ident("f".into()),
                TokenKind::Ident("pp".into()),
                TokenKind::Ident("myVar".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_integers() {
        let kinds = token_kinds("0 123 456");
        assert_eq!(
            kinds,
            vec![
                TokenKind::IntConst(0),
                TokenKind::IntConst(123),
                TokenKind::IntConst(456),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_char_literal() {
        let kinds = token_kinds("'a' 'Z'");
        assert_eq!(
            kinds,
            vec![
                TokenKind::CharConst('a'),
                TokenKind::CharConst('Z'),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_comment() {
        let kinds = token_kinds("{ this is a comment } program");
        assert_eq!(kinds, vec![TokenKind::Program, TokenKind::Eof,]);
    }

    #[test]
    fn test_simple_program() {
        let source = "program pp var integer v1; char c; begin v1 := 2 end.";
        let kinds = token_kinds(source);
        assert_eq!(
            kinds,
            vec![
                TokenKind::Program,
                TokenKind::Ident("pp".into()),
                TokenKind::Var,
                TokenKind::Integer,
                TokenKind::Ident("v1".into()),
                TokenKind::Semicolon,
                TokenKind::Char,
                TokenKind::Ident("c".into()),
                TokenKind::Semicolon,
                TokenKind::Begin,
                TokenKind::Ident("v1".into()),
                TokenKind::Assign,
                TokenKind::IntConst(2),
                TokenKind::End,
                TokenKind::Dot,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_nested_procedure() {
        let source = "program pp procedure f(); begin v1 := 2 end begin f(); write(v1) end.";
        let kinds = token_kinds(source);
        assert_eq!(
            kinds,
            vec![
                TokenKind::Program,
                TokenKind::Ident("pp".into()),
                TokenKind::Procedure,
                TokenKind::Ident("f".into()),
                TokenKind::LParent,
                TokenKind::RParent,
                TokenKind::Semicolon,
                TokenKind::Begin,
                TokenKind::Ident("v1".into()),
                TokenKind::Assign,
                TokenKind::IntConst(2),
                TokenKind::End,
                TokenKind::Begin,
                TokenKind::Ident("f".into()),
                TokenKind::LParent,
                TokenKind::RParent,
                TokenKind::Semicolon,
                TokenKind::Write,
                TokenKind::LParent,
                TokenKind::Ident("v1".into()),
                TokenKind::RParent,
                TokenKind::End,
                TokenKind::Dot,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_empty_source() {
        let kinds = token_kinds("");
        assert_eq!(kinds, vec![TokenKind::Eof]);
    }

    #[test]
    fn test_comment_with_newlines() {
        let kinds = token_kinds("{ multi\nline\ncomment } ident");
        assert_eq!(
            kinds,
            vec![TokenKind::Ident("ident".into()), TokenKind::Eof,]
        );
    }

    #[test]
    fn test_unterminated_comment_error() {
        let mut lexer = Lexer::new();
        let (_, errors) = lexer.tokenize("{ unterminated comment");
        assert!(
            errors
                .iter()
                .any(|e| e.msg.contains("Unterminated comment"))
        );
    }

    #[test]
    fn test_unterminated_char_error() {
        let mut lexer = Lexer::new();
        let (_, errors) = lexer.tokenize("'a");
        assert!(
            errors
                .iter()
                .any(|e| e.msg.contains("Unterminated character"))
        );
    }

    #[test]
    fn test_comment_between_tokens() {
        let kinds = token_kinds("x { comment } := { another } 1");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident("x".into()),
                TokenKind::Assign,
                TokenKind::IntConst(1),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_mixed_case_identifiers() {
        let kinds = token_kinds("myVar CamelCase Case1 Var1");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident("myVar".into()),
                TokenKind::Ident("CamelCase".into()),
                TokenKind::Ident("Case1".into()),
                TokenKind::Ident("Var1".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_underscore_is_error() {
        let mut lexer = Lexer::new();
        let (_, errors) = lexer.tokenize("snake_case");
        assert!(errors.iter().any(|e| e.msg.contains("Invalid character")));
    }

    #[test]
    fn test_keyword_vs_identifier_distinction() {
        // 'programa' 是标识符，'program' 是关键字
        let kinds = token_kinds("programa program programb");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident("programa".into()),
                TokenKind::Program,
                TokenKind::Ident("programb".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_multiline_program_tokens() {
        let source = "program p\nvar\ninteger x;\nbegin\nx := 1;\nwrite(x)\nend.\n";
        let tokens = tokenize(source);
        assert!(tokens.len() > 5);
        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TokenKind::Program));
        assert!(kinds.contains(&&TokenKind::Var));
        assert!(kinds.contains(&&TokenKind::Integer));
        assert!(kinds.contains(&&TokenKind::Begin));
        assert!(kinds.contains(&&TokenKind::End));
        assert!(kinds.contains(&&TokenKind::Write));
    }

    #[test]
    fn test_token_line_col_tracking() {
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize("program\n  p\nbegin");
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].col, 1);
        assert_eq!(tokens[1].line, 2);
        assert_eq!(tokens[1].col, 3);
        assert_eq!(tokens[2].line, 3);
        assert_eq!(tokens[2].col, 1);
    }

    #[test]
    fn test_colon_alone_is_error() {
        let mut lexer = Lexer::new();
        let (tokens, errors) = lexer.tokenize("x :");
        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TokenKind::Ident("x".into())));
        assert!(errors.iter().any(|e| e.msg.contains("Unexpected ':'")));
    }

    #[test]
    fn test_adjacent_operators() {
        let kinds = token_kinds("+-*/();");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Times,
                TokenKind::Divide,
                TokenKind::LParent,
                TokenKind::RParent,
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_large_integer() {
        let kinds = token_kinds("999999 0 100 42");
        assert_eq!(
            kinds,
            vec![
                TokenKind::IntConst(999999),
                TokenKind::IntConst(0),
                TokenKind::IntConst(100),
                TokenKind::IntConst(42),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_integer_overflow_is_error() {
        let mut lexer = Lexer::new();
        let (tokens, errors) = lexer.tokenize("9223372036854775808;");
        assert!(errors.iter().any(|e| e.msg.contains("out of range")));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Semicolon));
        assert!(!tokens.iter().any(|t| matches!(t.kind, TokenKind::IntConst(_))));
    }

    #[test]
    fn test_integer_overflow_at_eof_is_error() {
        let mut lexer = Lexer::new();
        let (tokens, errors) = lexer.tokenize("9223372036854775808");
        assert!(errors.iter().any(|e| e.msg.contains("out of range")));
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
    }

    #[test]
    fn test_invalid_character_is_error() {
        let mut lexer = Lexer::new();
        let (tokens, errors) = lexer.tokenize("write(@42)");
        let error = errors
            .iter()
            .find(|e| e.msg.contains("Invalid character '@'"))
            .expect("invalid character error");
        assert_eq!((error.line, error.col), (1, 7));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::IntConst(42)));
    }

    #[test]
    fn test_missing_closing_quote_before_delimiter_is_error() {
        let mut lexer = Lexer::new();
        let (tokens, errors) = lexer.tokenize("write('a)");
        assert!(errors
            .iter()
            .any(|e| e.msg.contains("Unterminated character literal")));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::RParent));
        assert!(!tokens.iter().any(|t| matches!(t.kind, TokenKind::CharConst(_))));
    }

    #[test]
    fn test_program_with_read_write_return() {
        let source = "program p procedure f(integer n); begin return(n) end begin read(x); f(5); write(x) end.";
        let tokens = tokenize(source);
        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TokenKind::Read));
        assert!(kinds.contains(&&TokenKind::Return));
        assert!(kinds.contains(&&TokenKind::Write));
        assert!(kinds.contains(&&TokenKind::Procedure));
    }
}
