//! 语义分析器。
//!
//! 对 AST 进行两遍遍历，完成符号表构建和类型检查。
//!
//! - [`symbol`] — 符号表数据结构（嵌套作用域哈希映射栈）
//! - [`analyzer`] — 语义分析器（声明收集 + 语句检查）

pub mod analyzer;
pub mod symbol;
