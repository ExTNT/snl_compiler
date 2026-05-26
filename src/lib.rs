//! SNL 语言编译器。
//!
//! 该编译器将 SNL（一种类 Pascal 的教学编程语言）源代码编译为 MIPS 汇编。
//! 编译管线分为四个阶段：词法分析 → 语法分析 → 语义分析 → 代码生成。
//!
//! 各阶段对应的模块：
//! - [`lexer`] — 基于 DFA 的词法分析，将源码分割为 Token 序列
//! - [`parser`] — 递归下降语法分析器，构建 AST，辅以 LL(1) 验证
//! - [`semantic`] — 语义分析：符号表构建、类型检查
//! - [`codegen`] — 生成 MIPS 汇编代码
//! - [`ast`] — 各阶段共享的抽象语法树节点定义
//! - [`error`] — 统一的编译错误类型

pub mod ast;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod semantic;
