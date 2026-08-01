//! SNL 编译器入口点。
//!
//! 四阶段编译管线：
//! 1. 词法分析
//! 2. 语法分析 → 递归下降构建 AST + LL(1) 验证
//! 3. 语义分析 → 生成 `*_report.html`
//! 4. 代码生成 → 输出 MIPS 汇编 `.asm`
//!
//! 用法：`snl_compiler <input.snl> [-o <output.asm>]`

use std::env;
use std::fs;
use std::process;

mod report;

use report::format_report_html;
use snl_compiler::codegen::mips;
use snl_compiler::lexer::{Lexer, LexerError, Token};
use snl_compiler::parser::ll1::Ll1Parser;
use snl_compiler::parser::rd::RdParser;
use snl_compiler::semantic::analyzer::SemanticAnalyzer;

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
    let saved_tokens: Vec<Token> = tokens.to_vec();
    let saved_lex_errors: Vec<LexerError> = lex_errors.to_vec();

    if !saved_lex_errors.is_empty() {
        eprintln!("=== Lexical Errors ===");
        for err in &saved_lex_errors {
            eprintln!("  Line {}:{} — {}", err.line, err.col, err.msg);
        }
        let html = format_report_html(
            input_path,
            &saved_tokens,
            &saved_lex_errors,
            None,
            &[],
            None,
            &[],
        );
        fs::write(format!("{}_report.html", base_name), &html).unwrap_or_else(|e| {
            eprintln!("Warning: could not write report: {}", e);
        });
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
            let syntax_errors: Vec<_> = parser.errors().to_vec();
            let html = format_report_html(
                input_path,
                &saved_tokens,
                &saved_lex_errors,
                None,
                &syntax_errors,
                None,
                &[],
            );
            fs::write(format!("{}_report.html", base_name), &html).unwrap_or_else(|e| {
                eprintln!("Warning: could not write report: {}", e);
            });
            process::exit(1);
        }
    };

    let syntax_errors = parser.errors().to_vec();

    if !syntax_errors.is_empty() {
        eprintln!("=== Syntax Errors ===");
        for err in &syntax_errors {
            eprintln!("  Line {}:{} — {}", err.loc.line, err.loc.col, err.msg);
        }
    }

    // ===== 阶段 2.5: LL(1) 文法验证 =====
    match Ll1Parser::new() {
        Ok(mut ll1) => {
            if !ll1.parse(tokens) {
                eprintln!("=== LL(1) Verification Errors ===");
                for err in ll1.errors() {
                    eprintln!("  Line {}:{} — {}", err.loc.line, err.loc.col, err.msg);
                }
                eprintln!("Warning: LL(1) verification failed (RD parse succeeded)");
            }
        }
        Err(conflicts) => {
            eprintln!("=== LL(1) Grammar Conflicts ===");
            for c in &conflicts {
                eprintln!(
                    "  Conflict at ({:?}, {:?}): production {} vs {}",
                    c.nt, c.token, c.prod1, c.prod2
                );
            }
            process::exit(1);
        }
    }

    // ===== 阶段 3: 语义分析 =====
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&prog);

    let semantic_errors = analyzer.errors().to_vec();
    let scope_snapshots = analyzer.scope_snapshots().to_vec();

    // 生成 HTML 报告
    let html = format_report_html(
        input_path,
        &saved_tokens,
        &saved_lex_errors,
        Some(&prog),
        &syntax_errors,
        Some(&scope_snapshots),
        &semantic_errors,
    );
    fs::write(format!("{}_report.html", base_name), &html).unwrap_or_else(|e| {
        eprintln!("Warning: could not write report: {}", e);
    });

    if !semantic_errors.is_empty() {
        eprintln!("=== Semantic Errors ===");
        for err in &semantic_errors {
            eprintln!("  Line {}:{} — {}", err.loc.line, err.loc.col, err.msg);
        }
    }

    if !syntax_errors.is_empty() || !semantic_errors.is_empty() {
        process::exit(1);
    }

    // ===== 阶段 4: MIPS 代码生成 =====
    let asm = match mips::compile(&prog) {
        Ok(asm) => asm,
        Err(errors) => {
            eprintln!("=== Codegen Errors ===");
            for err in &errors {
                eprintln!("  {}", err);
            }
            process::exit(1);
        }
    };

    match fs::write(&output_path, &asm) {
        Ok(_) => println!("Success: MIPS assembly written to '{}'", output_path),
        Err(e) => {
            eprintln!("Error writing '{}': {}", output_path, e);
            process::exit(1);
        }
    }
}
