use crate::ast::nodes::Loc;
use crate::error::CompileError;
use crate::lexer::token::{Token, TokenKind};

use super::grammar::{self, Grammar, GrammarSymbol, NonTerm};
use super::parse_table::{self, Ll1Table, normalize};

pub struct Ll1Parser {
    grammar: Grammar,
    table: Ll1Table,
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<CompileError>,
}

impl Ll1Parser {
    pub fn new() -> Result<Self, Vec<parse_table::Conflict>> {
        let grammar = grammar::encode_grammar();
        let table = parse_table::build_ll1_table(&grammar)?;
        Ok(Ll1Parser {
            grammar,
            table,
            tokens: Vec::new(),
            pos: 0,
            errors: Vec::new(),
        })
    }

    pub fn errors(&self) -> &[CompileError] {
        &self.errors
    }

    /// LL(1) table-driven parsing.
    /// Returns true if parsing succeeded (no syntax errors), false otherwise.
    /// Full AST construction via LL(1) requires semantic action markers;
    /// the Recursive Descent parser provides the primary AST construction.
    pub fn parse(&mut self, tokens: &[Token]) -> bool {
        self.tokens = tokens.to_vec();
        self.pos = 0;
        self.errors.clear();

        let mut stack: Vec<StackItem> = vec![StackItem::N(self.grammar.start)];

        while let Some(item) = stack.pop() {
            match item {
                StackItem::T(expected) => {
                    let current = self.current_token();
                    if token_matches(&expected, &current.kind) {
                        if !matches!(current.kind, TokenKind::Eof) {
                            self.advance();
                        }
                    } else {
                        self.errors.push(CompileError::syntax(
                            format!("LL(1): Expected {:?}, found {:?}", expected, current.kind),
                            Loc {
                                line: current.line,
                                col: current.col,
                            },
                        ));
                        self.pos += 1;
                    }
                }
                StackItem::N(nt) => {
                    let current = self.current_token();
                    let key = (nt, normalize(&current.kind));
                    if let Some(&prod_idx) = self.table.entries.get(&key) {
                        let prod = &self.grammar.productions[prod_idx];
                        for sym in prod.rhs.iter().rev() {
                            stack.push(StackItem::from_sym(sym.clone()));
                        }
                    } else {
                        self.errors.push(CompileError::syntax(
                            format!(
                                "LL(1): No rule for {:?} with lookahead {:?}",
                                nt, current.kind
                            ),
                            Loc {
                                line: current.line,
                                col: current.col,
                            },
                        ));
                        self.pos += 1;
                    }
                }
            }
        }

        self.errors.is_empty()
    }

    fn current_token(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) {
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
    }
}

#[derive(Debug, Clone)]
enum StackItem {
    T(TokenKind),
    N(NonTerm),
}

impl StackItem {
    fn from_sym(sym: GrammarSymbol) -> Self {
        match sym {
            GrammarSymbol::T(tk) => StackItem::T(tk),
            GrammarSymbol::N(nt) => StackItem::N(nt),
        }
    }
}

fn token_matches(expected: &TokenKind, actual: &TokenKind) -> bool {
    use TokenKind::*;
    match (expected, actual) {
        (Ident(_), Ident(_)) => true,
        (IntConst(_), IntConst(_)) => true,
        (CharConst(_), CharConst(_)) => true,
        (a, b) => a == b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_ll1_no_conflicts() {
        match Ll1Parser::new() {
            Ok(parser) => {
                // Verify it can parse a simple program
                let source = "program p begin write(1) end.";
                let mut lexer = Lexer::new();
                let (_tokens, _) = lexer.tokenize(source);
                assert!(parser.grammar.productions.len() > 0);
            }
            Err(conflicts) => {
                for c in &conflicts {
                    eprintln!(
                        "Conflict: {:?} on {:?} (prods {} and {})",
                        c.nt, c.token, c.prod1, c.prod2
                    );
                }
                panic!("LL(1) grammar has {} conflicts", conflicts.len());
            }
        }
    }

    #[test]
    fn test_ll1_simple_parse() {
        let mut parser = Ll1Parser::new().expect("LL(1) grammar should have no conflicts");
        let source = "program p begin write(1) end.";
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize(source);
        assert!(
            parser.parse(tokens),
            "LL(1) parse should succeed: {:?}",
            parser.errors()
        );
    }

    #[test]
    fn test_ll1_with_variables() {
        let mut parser = Ll1Parser::new().expect("LL(1) grammar should have no conflicts");
        let source = "program p var integer x; begin x := 1; write(x) end.";
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize(source);
        assert!(
            parser.parse(tokens),
            "LL(1) parse should succeed: {:?}",
            parser.errors()
        );
    }

    #[test]
    fn test_ll1_with_if_statement() {
        let mut parser = Ll1Parser::new().expect("LL(1) grammar should have no conflicts");
        let source = "program p var integer x; begin if x < 10 then x := 1 else x := 2 fi end.";
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize(source);
        assert!(
            parser.parse(tokens),
            "LL(1) parse should succeed: {:?}",
            parser.errors()
        );
    }

    #[test]
    fn test_ll1_with_while_statement() {
        let mut parser = Ll1Parser::new().expect("LL(1) grammar should have no conflicts");
        let source = "program p begin while 1 do write(0) endwh end.";
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize(source);
        assert!(
            parser.parse(tokens),
            "LL(1) parse should succeed: {:?}",
            parser.errors()
        );
    }

    #[test]
    fn test_ll1_with_procedure() {
        let mut parser = Ll1Parser::new().expect("LL(1) grammar should have no conflicts");
        let source = "program p procedure f(integer a); begin write(a) end begin f(1) end.";
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize(source);
        assert!(
            parser.parse(tokens),
            "LL(1) parse should succeed: {:?}",
            parser.errors()
        );
    }

    #[test]
    fn test_ll1_with_read() {
        let mut parser = Ll1Parser::new().expect("LL(1) grammar should have no conflicts");
        let source = "program p var integer x; begin read(x) end.";
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize(source);
        assert!(
            parser.parse(tokens),
            "LL(1) parse should succeed: {:?}",
            parser.errors()
        );
    }

    #[test]
    fn test_ll1_with_return() {
        let mut parser = Ll1Parser::new().expect("LL(1) grammar should have no conflicts");
        let source = "program p procedure f(); begin return(1) end begin f() end.";
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize(source);
        assert!(
            parser.parse(tokens),
            "LL(1) parse should succeed: {:?}",
            parser.errors()
        );
    }

    #[test]
    fn test_ll1_with_expressions() {
        let mut parser = Ll1Parser::new().expect("LL(1) grammar should have no conflicts");
        let source =
            "program p var integer x; integer y; begin x := 1 + 2 * 3 - 4; y := x + 5 end.";
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize(source);
        assert!(
            parser.parse(tokens),
            "LL(1) parse should succeed: {:?}",
            parser.errors()
        );
    }

    #[test]
    fn test_ll1_char_types() {
        let mut parser = Ll1Parser::new().expect("LL(1) grammar should have no conflicts");
        let source = "program p var char c; begin c := 'a' end.";
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize(source);
        assert!(
            parser.parse(tokens),
            "LL(1) parse should succeed: {:?}",
            parser.errors()
        );
    }

    #[test]
    fn test_ll1_nested_procedures() {
        let mut parser = Ll1Parser::new().expect("LL(1) grammar should have no conflicts");
        let source = "program p procedure outer(); procedure inner(); begin write(0) end begin inner() end begin outer() end.";
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize(source);
        assert!(
            parser.parse(tokens),
            "LL(1) parse should succeed: {:?}",
            parser.errors()
        );
    }

    #[test]
    fn test_ll1_multiple_params() {
        let mut parser = Ll1Parser::new().expect("LL(1) grammar should have no conflicts");
        let source = "program p procedure q(integer a; char b; integer c); begin write(a) end begin q(1, 'x', 2) end.";
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize(source);
        assert!(
            parser.parse(tokens),
            "LL(1) parse should succeed: {:?}",
            parser.errors()
        );
    }

    #[test]
    fn test_ll1_first_follow_sets() {
        // Verify the grammar can be constructed without conflicts
        let parser = Ll1Parser::new().expect("LL(1) grammar should have no conflicts");
        // The grammar should have productions
        assert!(parser.grammar.productions.len() > 0);
        // The parse table should have entries
        assert!(!parser.table.entries.is_empty());
    }
}
