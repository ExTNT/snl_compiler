use std::collections::{HashMap, HashSet};

use crate::lexer::token::TokenKind;

use super::grammar::{Grammar, GrammarSymbol, NonTerm, Production};

pub struct FirstFollow {
    pub first: HashMap<NonTerm, HashSet<TokenKind>>,
    pub follow: HashMap<NonTerm, HashSet<TokenKind>>,
}

impl FirstFollow {
    pub fn compute(grammar: &Grammar) -> Self {
        let nonterms = all_nonterms();
        let mut first: HashMap<NonTerm, HashSet<TokenKind>> =
            nonterms.iter().map(|nt| (*nt, HashSet::new())).collect();
        let mut follow: HashMap<NonTerm, HashSet<TokenKind>> =
            nonterms.iter().map(|nt| (*nt, HashSet::new())).collect();

        // Add EOF to FOLLOW(Program)
        follow
            .get_mut(&grammar.start)
            .unwrap()
            .insert(TokenKind::Eof);

        // Fixpoint iteration for FIRST
        loop {
            let mut changed = false;
            for prod in &grammar.productions {
                let new_first = first_of_string(&prod.rhs, &first);
                let existing = first.get_mut(&prod.lhs).unwrap();
                for tk in new_first {
                    if existing.insert(tk) {
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        // Fixpoint iteration for FOLLOW
        loop {
            let mut changed = false;
            for prod in &grammar.productions {
                for i in 0..prod.rhs.len() {
                    if let GrammarSymbol::N(nt) = &prod.rhs[i] {
                        let beta = &prod.rhs[i + 1..];
                        let beta_first = first_of_string(beta, &first);

                        // Collect lhs_follow before mutable borrow of follow[nt]
                        let need_lhs_follow = beta_first.contains(&TokenKind::Eof);
                        let lhs_follow: Vec<TokenKind> = if need_lhs_follow {
                            follow[&prod.lhs].iter().cloned().collect()
                        } else {
                            Vec::new()
                        };

                        let existing = follow.get_mut(nt).unwrap();
                        for tk in &beta_first {
                            if *tk != TokenKind::Eof {
                                if existing.insert(tk.clone()) {
                                    changed = true;
                                }
                            }
                        }
                        if need_lhs_follow {
                            for tk in lhs_follow {
                                if existing.insert(tk) {
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }

        FirstFollow { first, follow }
    }

    pub fn predict_set(&self, prod: &Production) -> HashSet<TokenKind> {
        let first_rhs = first_of_string(&prod.rhs, &self.first);
        if first_rhs.contains(&TokenKind::Eof) {
            // If RHS can derive ε, include FOLLOW(LHS)
            let mut result: HashSet<TokenKind> = first_rhs
                .into_iter()
                .filter(|t| *t != TokenKind::Eof)
                .collect();
            if let Some(follow_set) = self.follow.get(&prod.lhs) {
                result.extend(follow_set.iter().cloned());
            }
            result
        } else {
            first_rhs
        }
    }
}

fn first_of_string(
    symbols: &[GrammarSymbol],
    first: &HashMap<NonTerm, HashSet<TokenKind>>,
) -> HashSet<TokenKind> {
    let mut result = HashSet::new();
    let mut all_derive_epsilon = true;

    for sym in symbols {
        match sym {
            GrammarSymbol::T(tk) => {
                result.insert(tk.clone());
                all_derive_epsilon = false;
                break;
            }
            GrammarSymbol::N(nt) => {
                if let Some(nt_first) = first.get(nt) {
                    for tk in nt_first {
                        if *tk != TokenKind::Eof {
                            result.insert(tk.clone());
                        }
                    }
                    if !nt_first.contains(&TokenKind::Eof) {
                        all_derive_epsilon = false;
                        break;
                    }
                }
            }
        }
    }

    if all_derive_epsilon {
        result.insert(TokenKind::Eof);
    }

    result
}

fn all_nonterms() -> Vec<NonTerm> {
    use NonTerm::*;
    vec![
        NProgram,
        DeclarePart,
        TypeDec,
        TypeDecList,
        TypeDecMore,
        TypeName,
        BaseType,
        FieldDecList,
        FieldDecMore,
        IdList,
        IdMore,
        VarDec,
        VarDecList,
        VarDecMore,
        VarIdList,
        VarIdMore,
        ProcDec,
        ProcDecMore,
        ParamList,
        ParamDecList,
        ParamMore,
        Param,
        FormList,
        FidMore,
        ProgramBody,
        StmList,
        StmMore,
        Stm,
        AssignmentRest,
        ActParamList,
        ActParamMore,
        RelExp,
        OtherRelE,
        Exp,
        OtherTerm,
        Term,
        OtherFactor,
        Factor,
        Variable,
        VariMore,
        FieldVar,
        FieldVarMore,
        AssCall,
        CallStmRest,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::grammar::encode_grammar;

    #[test]
    fn test_first_sets_non_empty() {
        let grammar = encode_grammar();
        let ff = FirstFollow::compute(&grammar);
        // Every non-terminal should have at least one FIRST token
        for (nt, first_set) in &ff.first {
            assert!(!first_set.is_empty(), "FIRST({:?}) should not be empty", nt);
        }
    }

    #[test]
    fn test_follow_sets_non_empty() {
        let grammar = encode_grammar();
        let ff = FirstFollow::compute(&grammar);
        // Every non-terminal should have at least one FOLLOW token
        for (nt, follow_set) in &ff.follow {
            assert!(
                !follow_set.is_empty(),
                "FOLLOW({:?}) should not be empty",
                nt
            );
        }
    }

    #[test]
    fn test_eof_in_follow_program() {
        let grammar = encode_grammar();
        let ff = FirstFollow::compute(&grammar);
        assert!(ff.follow[&NonTerm::NProgram].contains(&TokenKind::Eof));
    }

    #[test]
    fn test_first_of_program() {
        let grammar = encode_grammar();
        let ff = FirstFollow::compute(&grammar);
        assert!(ff.first[&NonTerm::NProgram].contains(&TokenKind::Program));
    }

    #[test]
    fn test_first_of_exp() {
        let grammar = encode_grammar();
        let ff = FirstFollow::compute(&grammar);
        let exp_first = &ff.first[&NonTerm::Exp];
        // Exp should have IntConst, CharConst, Ident in its FIRST set
        let has_int = exp_first
            .iter()
            .any(|t| matches!(t, TokenKind::IntConst(_)));
        let has_char = exp_first
            .iter()
            .any(|t| matches!(t, TokenKind::CharConst(_)));
        let has_ident = exp_first.iter().any(|t| matches!(t, TokenKind::Ident(_)));
        assert!(
            has_int || has_char || has_ident,
            "Exp FIRST should contain terminals"
        );
    }

    #[test]
    fn test_predict_set_for_each_production() {
        let grammar = encode_grammar();
        let ff = FirstFollow::compute(&grammar);
        for prod in &grammar.productions {
            let predict = ff.predict_set(prod);
            assert!(
                !predict.is_empty(),
                "Predict set for {:?} -> ... should not be empty",
                prod.lhs
            );
        }
    }

    #[test]
    fn test_no_eof_in_first_of_non_nullable() {
        let grammar = encode_grammar();
        let ff = FirstFollow::compute(&grammar);
        // For non-nullable non-terminals like IntConst, Eof should not be in their FIRST
        let exp_first = &ff.first[&NonTerm::Exp];
        assert!(
            !exp_first.contains(&TokenKind::Eof),
            "Exp is not nullable, no Eof in FIRST"
        );
    }
}
