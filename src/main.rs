//! SNL 编译器入口点。
//!
//! 四阶段编译管线：
//! 1. 词法分析 → 生成 `*_token.md`
//! 2. 语法分析 → 生成 `*_tree.md`
//! 3. 语义分析 → 生成 `*_table.md`
//! 4. 代码生成 → 输出 MIPS 汇编 `.asm`
//!
//! 用法：`snl_compiler <input.snl> [-o <output.asm>]`

use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::fs;
use std::process;

use snl_compiler::codegen::mips;
use snl_compiler::lexer::Lexer;
use snl_compiler::parser::rd::RdParser;
use snl_compiler::semantic::analyzer::SemanticAnalyzer;
use snl_compiler::semantic::symbol::SymbolEntry;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: snl_compiler <input.snl> [-o <output.asm>]");
        process::exit(1);
    }

    let input_path = &args[1];
    let output_path = if args.len() >= 4 && args[2] == "-o" {
        args[3].clone()
    } else {
        input_path.replace(".snl", ".asm")
    };

    let base_name = input_path.strip_suffix(".snl").unwrap_or(input_path);

    let source = match fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", input_path, e);
            process::exit(1);
        }
    };

    // ===== 阶段 1: 词法分析 =====
    let mut lexer = Lexer::new();
    let (tokens, lex_errors) = lexer.tokenize(&source);

    // 生成 token.md
    let token_md = format_token_md(input_path, tokens, lex_errors);
    fs::write(format!("{}_token.md", base_name), &token_md).unwrap_or_else(|e| {
        eprintln!("Warning: could not write token.md: {}", e);
    });

    if !lex_errors.is_empty() {
        eprintln!("=== Lexical Errors ===");
        for err in lex_errors {
            eprintln!("  Line {}:{} — {}", err.line, err.col, err.msg);
        }
        process::exit(1);
    }

    // ===== 阶段 2: 递归下降语法分析 =====
    let mut parser = RdParser::new(tokens);
    let prog = match parser.parse() {
        Some(p) => p,
        None => {
            eprintln!("=== Syntax Errors ===");
            for err in parser.errors() {
                eprintln!("  Line {}:{} — {}", err.loc.line, err.loc.col, err.msg);
            }
            process::exit(1);
        }
    };

    let syntax_errors = parser.errors().to_vec();

    // 生成 tree.md（AST + 语法错误）
    let tree_md = format_tree_md(input_path, &prog, &syntax_errors);
    fs::write(format!("{}_tree.md", base_name), &tree_md).unwrap_or_else(|e| {
        eprintln!("Warning: could not write tree.md: {}", e);
    });

    if !syntax_errors.is_empty() {
        eprintln!("=== Syntax Errors ===");
        for err in &syntax_errors {
            eprintln!("  Line {}:{} — {}", err.loc.line, err.loc.col, err.msg);
        }
    }

    // ===== 阶段 3: 语义分析 =====
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&prog);

    let semantic_errors = analyzer.errors().to_vec();
    let scope_snapshots = analyzer.scope_snapshots().to_vec();

    // 生成 table.md（符号表 + 语义错误）
    let table_md = format_table_md(input_path, &scope_snapshots, &semantic_errors);
    fs::write(format!("{}_table.md", base_name), &table_md).unwrap_or_else(|e| {
        eprintln!("Warning: could not write table.md: {}", e);
    });

    if !semantic_errors.is_empty() {
        eprintln!("=== Semantic Errors ===");
        for err in &semantic_errors {
            eprintln!("  Line {}:{} — {}", err.loc.line, err.loc.col, err.msg);
        }
        process::exit(1);
    }

    // ===== 阶段 4: MIPS 代码生成 =====
    let asm = mips::compile(&prog);

    match fs::write(&output_path, &asm) {
        Ok(_) => println!("Success: MIPS assembly written to '{}'", output_path),
        Err(e) => {
            eprintln!("Error writing '{}': {}", output_path, e);
            process::exit(1);
        }
    }
}

/// 生成 Token 序列的 Markdown 表格。
fn format_token_md(
    input_path: &str,
    tokens: &[snl_compiler::lexer::Token],
    errors: &[snl_compiler::lexer::LexerError],
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# `{}` Token 序列\n\n", input_path));

    if !errors.is_empty() {
        out.push_str("## 词法错误\n\n");
        out.push_str("| 行:列 | 信息 |\n");
        out.push_str("|-------|------|\n");
        for err in errors {
            out.push_str(&format!("| {}:{} | {} |\n", err.line, err.col, err.msg));
        }
        out.push('\n');
    }

    out.push_str("## Token 列表\n\n");
    out.push_str("| 序号 | Token 类型 | 值 | 行:列 |\n");
    out.push_str("|------|----------|----|-------|\n");

    for (i, tok) in tokens.iter().enumerate() {
        let kind_str = format!("{:?}", tok.kind);
        let value = match &tok.kind {
            snl_compiler::lexer::TokenKind::Ident(s) => s.clone(),
            snl_compiler::lexer::TokenKind::IntConst(n) => n.to_string(),
            snl_compiler::lexer::TokenKind::CharConst(c) => format!("'{}'", c),
            _ => String::new(),
        };
        out.push_str(&format!(
            "| {} | {} | {} | {}:{} |\n",
            i + 1,
            kind_str,
            value,
            tok.line,
            tok.col
        ));
    }

    out
}

/// 生成语法树（AST）的 Markdown 文档。
fn format_tree_md(
    input_path: &str,
    prog: &snl_compiler::ast::nodes::Program,
    errors: &[snl_compiler::error::CompileError],
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# `{}` 语法树\n\n", input_path));

    out.push_str("## 抽象语法树\n\n```\n");
    out.push_str(&format!("{}", prog));
    out.push_str("```\n\n");

    out.push_str("## 语法错误\n\n");
    if errors.is_empty() {
        out.push_str("无。\n");
    } else {
        out.push_str("| 行:列 | 信息 |\n");
        out.push_str("|-------|------|\n");
        for err in errors {
            out.push_str(&format!(
                "| {}:{} | {} |\n",
                err.loc.line, err.loc.col, err.msg
            ));
        }
    }

    out
}

/// 生成符号表的 Markdown 文档。
///
/// 按作用域层级展示所有符号（类型、变量、过程），
/// 包含名称、种类、类型、形参列表和声明行号。
fn format_table_md(
    input_path: &str,
    scope_snapshots: &[(usize, HashMap<String, SymbolEntry>)],
    errors: &[snl_compiler::error::CompileError],
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# `{}` 符号表\n\n", input_path));

    out.push_str("## 作用域结构\n\n");
    out.push_str("SNL 为过程声明使用嵌套作用域。");
    out.push_str("符号表组织为哈希映射栈，每个作用域级别一个。");
    out.push_str("查找时从最内层作用域向外层遍历。\n\n");

    out.push_str(&format!("总作用域数: {}\n\n", scope_snapshots.len()));

    // 按顺序展示作用域：全局优先，然后是嵌套作用域
    out.push_str("| 作用域 | 级别 | 名称 | 种类 | 类型 | 参数 | 行号 |\n");
    out.push_str("|--------|------|------|------|------|------|------|\n");

    for (level, scope) in scope_snapshots.iter() {
        let scope_label = if *level == 0 {
            "全局".to_string()
        } else {
            format!("过程级别 {}", level)
        };
        let mut entries: Vec<&SymbolEntry> = scope.values().collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        for entry in entries {
            write!(
                out,
                "| {} | {} | {} | {} |",
                scope_label, entry.level, entry.name, entry.kind
            )
            .unwrap();
            if let Some(ty) = &entry.typ {
                write!(out, " {}", ty).unwrap();
            }
            write!(out, " |").unwrap();
            if !entry.params.is_empty() {
                write!(out, " (").unwrap();
                for (j, p) in entry.params.iter().enumerate() {
                    if j > 0 {
                        write!(out, ", ").unwrap();
                    }
                    if p.is_var {
                        write!(out, "var ").unwrap();
                    }
                    write!(out, "{}: {}", p.name, p.typ).unwrap();
                }
                write!(out, ")").unwrap();
            }
            writeln!(out, " | {} |", entry.loc.line).unwrap();
        }
    }

    out.push('\n');
    out.push_str("## 语义错误\n\n");
    if errors.is_empty() {
        out.push_str("无。\n");
    } else {
        out.push_str("| 行:列 | 代码 | 信息 |\n");
        out.push_str("|-------|------|------|\n");
        for err in errors {
            out.push_str(&format!(
                "| {}:{} | {:?} | {} |\n",
                err.loc.line, err.loc.col, err.kind, err.msg
            ));
        }
    }

    out
}
