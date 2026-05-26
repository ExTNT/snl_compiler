//! 语法分析器（Parser）。
//!
//! 提供两种语法分析实现：
//! - **rd**: 递归下降分析器（主解析器），直接构建 AST
//! - **ll1**: LL(1) 表驱动分析器，用于验证文法的 LL(1) 性质
//!
//! 辅助模块：
//! - [`grammar`] — SNL 文法编码（EBNF 产生式）
//! - [`first_follow`] — FIRST/FOLLOW 集合计算
//! - [`parse_table`] — LL(1) 预测分析表构建

pub mod first_follow;
pub mod grammar;
pub mod ll1;
pub mod parse_table;
pub mod rd;
