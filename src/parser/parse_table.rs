use std::collections::HashMap;

use crate::lexer::token::TokenKind;

use super::first_follow::FirstFollow;
use super::grammar::{Grammar, NonTerm};

/// Normalize a token kind for table lookup, stripping literal values.
pub fn normalize(kind: &TokenKind) -> TokenKind {
    match kind {
        TokenKind::Ident(_) => TokenKind::Ident(String::new()),
        TokenKind::IntConst(_) => TokenKind::IntConst(0),
        TokenKind::CharConst(_) => TokenKind::CharConst('\0'),
        other => other.clone(),
    }
}

#[derive(Debug)]
pub struct Conflict {
    pub nt: NonTerm,
    pub token: TokenKind,
    pub prod1: usize,
    pub prod2: usize,
}

pub struct Ll1Table {
    pub entries: HashMap<(NonTerm, TokenKind), usize>,
}

pub fn build_ll1_table(grammar: &Grammar) -> Result<Ll1Table, Vec<Conflict>> {
    let ff = FirstFollow::compute(grammar);
    let mut entries: HashMap<(NonTerm, TokenKind), usize> = HashMap::new();
    let mut conflicts: Vec<Conflict> = Vec::new();

    for (i, prod) in grammar.productions.iter().enumerate() {
        let predict = ff.predict_set(prod);
        for tk in predict {
            // Eof doesn't go into the parse table as a lookahead key
            // (it's handled by the parser accepting when stack is empty)
            if tk == TokenKind::Eof {
                continue;
            }
            let key = (prod.lhs, tk);
            if let Some(&existing) = entries.get(&key) {
                conflicts.push(Conflict {
                    nt: prod.lhs,
                    token: key.1.clone(),
                    prod1: existing,
                    prod2: i,
                });
            } else {
                entries.insert(key, i);
            }
        }
    }

    if conflicts.is_empty() {
        Ok(Ll1Table { entries })
    } else {
        Err(conflicts)
    }
}
