//! LL(1) 预测分析表构建。
//!
//! 基于 FIRST/FOLLOW 集合为文法的每条产生式计算预测集合，
//! 并填入分析表中。若两个产生式的预测集产生交集，则文法不是 LL(1)，
//! 此时报告冲突。

use std::collections::HashMap;

use crate::lexer::token::TokenKind;

use super::first_follow::FirstFollow;
use super::grammar::{Grammar, NonTerm};

/// 将 Token 种类归一化，剥离字面量值。
///
/// LL(1) 分析表以 Token 种类为键，不需要区分具体的标识符名或数值。
/// 例如 `Ident("x")` 和 `Ident("y")` 均为 `Ident`。
pub fn normalize(kind: &TokenKind) -> TokenKind {
    match kind {
        TokenKind::Ident(_) => TokenKind::Ident(String::new()),
        TokenKind::IntConst(_) => TokenKind::IntConst(0),
        TokenKind::CharConst(_) => TokenKind::CharConst('\0'),
        other => other.clone(),
    }
}

/// LL(1) 分析表冲突记录。
#[derive(Debug)]
pub struct Conflict {
    /// 冲突所在的非终结符
    pub nt: NonTerm,
    /// 冲突的向前看符号
    pub token: TokenKind,
    /// 冲突的产生式索引 1
    pub prod1: usize,
    /// 冲突的产生式索引 2
    pub prod2: usize,
}

/// LL(1) 预测分析表。
///
/// 键为 `(非终结符, 归一化后的向前看 Token)`，
/// 值为对应产生式在 Grammar.productions 中的索引。
pub struct Ll1Table {
    pub entries: HashMap<(NonTerm, TokenKind), usize>,
}

/// 构建 LL(1) 分析表。
///
/// 若文法不是 LL(1)，返回冲突列表。
///
/// # 参数
/// - `grammar`: SNL 编码文法
///
/// # 返回
/// `Ok(Ll1Table)` — 构建成功；`Err(Vec<Conflict>)` — 存在 LL(1) 冲突
pub fn build_ll1_table(grammar: &Grammar) -> Result<Ll1Table, Vec<Conflict>> {
    let ff = FirstFollow::compute(grammar);
    let mut entries: HashMap<(NonTerm, TokenKind), usize> = HashMap::new();
    let mut conflicts: Vec<Conflict> = Vec::new();

    for (i, prod) in grammar.productions.iter().enumerate() {
        let predict = ff.predict_set(prod);
        for tk in predict {
            // Eof 不作为分析表键值（由栈空时接受处理）
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
