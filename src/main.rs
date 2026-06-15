//! SNL 编译器入口点。
//!
//! 四阶段编译管线：
//! 1. 词法分析
//! 2. 语法分析 → 递归下降构建 AST + LL(1) 验证
//! 3. 语义分析 → 生成 `*_report.html`
//! 4. 代码生成 → 输出 MIPS 汇编 `.asm`
//!
//! 用法：`snl_compiler <input.snl> [-o <output.asm>]`

use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::fs;
use std::process;

use snl_compiler::codegen::mips;
use snl_compiler::lexer::{Lexer, LexerError, Token};
use snl_compiler::parser::ll1::Ll1Parser;
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

/// 生成 HTML 报告，将 Token 序列、语法树与符号表整合为自包含的交互式页面。
///
/// 包含嵌入式 CSS 与 JS，替换三个 Markdown 诊断文件。
fn format_report_html(
    input_path: &str,
    tokens: &[snl_compiler::lexer::Token],
    lex_errors: &[snl_compiler::lexer::LexerError],
    prog: Option<&snl_compiler::ast::nodes::Program>,
    syntax_errors: &[snl_compiler::error::CompileError],
    scope_snapshots: Option<&[(usize, HashMap<String, SymbolEntry>)]>,
    semantic_errors: &[snl_compiler::error::CompileError],
) -> String {
    use snl_compiler::lexer::TokenKind;

    let mut h = String::with_capacity(32_768);

    // ===== HTML 头部 =====
    h.push_str("<!DOCTYPE html>\n");
    h.push_str("<html lang=\"zh-CN\">\n<head>\n");
    h.push_str("<meta charset=\"UTF-8\">\n");
    h.push_str(&format!("<title>SNL 编译报告 — {}</title>\n", escape_html(input_path)));

    // ===== CSS =====
    h.push_str("<style>\n");
    h.push_str("body{font-family:-apple-system,sans-serif;max-width:1100px;margin:auto;padding:10px}\n");
    h.push_str(".tab-bar{display:flex;border-bottom:2px solid #ccc;margin-bottom:12px}\n");
    h.push_str(".tab-btn{padding:8px 18px;border:none;background:#eee;cursor:pointer;font-size:15px;margin-right:2px;border-radius:4px 4px 0 0}\n");
    h.push_str(".tab-btn.active{background:#fff;border:2px solid #ccc;border-bottom:2px solid #fff;margin-bottom:-2px}\n");
    h.push_str(".tab-content{display:none}\n");
    h.push_str(".tab-content:first-of-type{display:block}\n");
    h.push_str("table{border-collapse:collapse;width:100%;margin:10px 0;font-size:14px}\n");
    h.push_str("th{background:#f5f5f5;position:sticky;top:0;text-align:left;padding:6px 8px;border:1px solid #ddd}\n");
    h.push_str("td{padding:4px 8px;border:1px solid #ddd}\n");
    h.push_str("tr:nth-child(even){background:#fafafa}\n");
    h.push_str("tr.error-row{background:#fff0f0}\n");
    h.push_str("tr.error-row td{border-color:#fcc}\n");
    h.push_str(".scope-label{font-weight:bold;margin-top:16px;font-size:15px}\n");
    h.push_str(".scope-desc{margin:8px 0;color:#555}\n");
    h.push_str(".tree-node div{padding-left:20px}\n");
    h.push_str("details.tree-node>summary{cursor:pointer;font-family:monospace;white-space:pre;padding:1px 0}\n");
    h.push_str("details.tree-node{margin:0}\n");
    h.push_str(".tree-text{font-family:monospace;white-space:pre;padding:1px 0}\n");
    h.push_str(".err-section{margin:16px 0}\n");
    h.push_str(".err-section h3{color:#c00}\n");
    h.push_str(".no-errors{color:#888;margin:8px 0}\n");
    h.push_str("h2{margin-top:20px;border-bottom:1px solid #eee;padding-bottom:4px}\n");
    h.push_str("@media print{.tab-bar{display:none}}\n");
    h.push_str("</style>\n");

    // ===== 头部脚本（用于页面加载时初始化） =====
    h.push_str("<script>\n");
    h.push_str("function showTab(tabId,btn){\n");
    h.push_str("document.querySelectorAll('.tab-content').forEach(function(t){t.style.display='none'});\n");
    h.push_str("document.querySelectorAll('.tab-btn').forEach(function(b){b.classList.remove('active')});\n");
    h.push_str("document.getElementById(tabId).style.display='block';\n");
    h.push_str("if(btn)btn.classList.add('active');\n");
    h.push_str("}\n");
    h.push_str("function sortTable(tableId,colIdx){\n");
    h.push_str("var table=document.getElementById(tableId);\n");
    h.push_str("var tbody=table.querySelector('tbody');\n");
    h.push_str("var rows=Array.from(tbody.querySelectorAll('tr'));\n");
    h.push_str("var asc=table.dataset.sortAsc!=='false';\n");
    h.push_str("rows.sort(function(a,b){\n");
    h.push_str("var ca=a.cells[colIdx]?a.cells[colIdx].textContent.trim():'';\n");
    h.push_str("var cb=b.cells[colIdx]?b.cells[colIdx].textContent.trim():'';\n");
    h.push_str("var na=parseFloat(ca),nb=parseFloat(cb);\n");
    h.push_str("if(!isNaN(na)&&!isNaN(nb)){ca=na;cb=nb;}\n");
    h.push_str("if(ca<cb)return asc?-1:1;\n");
    h.push_str("if(ca>cb)return asc?1:-1;\n");
    h.push_str("return 0;\n");
    h.push_str("});\n");
    h.push_str("rows.forEach(function(r){tbody.appendChild(r)});\n");
    h.push_str("table.dataset.sortAsc=asc?'false':'true';\n");
    h.push_str("}\n");
    h.push_str("function filterTables(query){\n");
    h.push_str("var q=query.toLowerCase();\n");
    h.push_str("document.querySelectorAll('.tab-content[style*=\"block\"] table tbody tr').forEach(function(r){\n");
    h.push_str("if(!q){r.style.display='';return;}\n");
    h.push_str("r.style.display=r.textContent.toLowerCase().indexOf(q)>=0?'':'none';\n");
    h.push_str("});\n");
    h.push_str("}\n");
    h.push_str("</script>\n");

    h.push_str("</head>\n<body>\n");

    // ===== 标题 =====
    h.push_str(&format!("<h1>SNL 编译报告 — {}</h1>\n", escape_html(input_path)));

    // ===== 标签栏 =====
    h.push_str("<div class=\"tab-bar\">\n");
    h.push_str("<button class=\"tab-btn active\" onclick=\"showTab('tab-token',this)\">Token 序列</button>\n");
    h.push_str("<button class=\"tab-btn\" onclick=\"showTab('tab-tree',this)\">语法树</button>\n");
    h.push_str("<button class=\"tab-btn\" onclick=\"showTab('tab-table',this)\">符号表</button>\n");
    h.push_str("</div>\n");

    // ===== 搜索栏 =====
    h.push_str("<div style=\"margin-bottom:10px\">\n");
    h.push_str("<input type=\"text\" placeholder=\"搜索...\" oninput=\"filterTables(this.value)\"");
    h.push_str(" style=\"width:260px;padding:4px 8px;font-size:14px\">\n");
    h.push_str("</div>\n");

    // ====================================================================
    // 标签页 1: Token 序列
    // ====================================================================
    h.push_str("<div id=\"tab-token\" class=\"tab-content\">\n");

    // 词法错误
    h.push_str("<div class=\"err-section\">\n");
    h.push_str("<h3>词法错误</h3>\n");
    if !lex_errors.is_empty() {
        h.push_str("<table><thead><tr><th>行:列</th><th>信息</th></tr></thead><tbody>\n");
        for err in lex_errors {
            h.push_str("<tr class=\"error-row\"><td>");
            h.push_str(&format!("{}:{}", err.line, err.col));
            h.push_str("</td><td>");
            h.push_str(&escape_html(&err.msg));
            h.push_str("</td></tr>\n");
        }
        h.push_str("</tbody></table>\n");
    } else {
        h.push_str("<p class=\"no-errors\">无。</p>\n");
    }
    h.push_str("</div>\n");

    // Token 表格
    h.push_str("<h2>Token 列表</h2>\n");
    h.push_str("<table id=\"tokentable\"><thead><tr>");
    h.push_str("<th onclick=\"sortTable('tokentable',0)\">序号</th>");
    h.push_str("<th onclick=\"sortTable('tokentable',1)\">Token 类型</th>");
    h.push_str("<th onclick=\"sortTable('tokentable',2)\">值</th>");
    h.push_str("<th onclick=\"sortTable('tokentable',3)\">行:列</th>");
    h.push_str("</tr></thead><tbody>\n");

    for (i, tok) in tokens.iter().enumerate() {
        let kind_str = format!("{:?}", tok.kind);
        let value = match &tok.kind {
            TokenKind::Ident(s) => s.clone(),
            TokenKind::IntConst(n) => n.to_string(),
            TokenKind::CharConst(c) => format!("'{}'", c),
            _ => String::new(),
        };
        h.push_str("<tr><td>");
        h.push_str(&(i + 1).to_string());
        h.push_str("</td><td>");
        h.push_str(&escape_html(&kind_str));
        h.push_str("</td><td>");
        h.push_str(&escape_html(&value));
        h.push_str("</td><td>");
        h.push_str(&format!("{}:{}", tok.line, tok.col));
        h.push_str("</td></tr>\n");
    }
    h.push_str("</tbody></table>\n");
    h.push_str("</div>\n");

    // ====================================================================
    // 标签页 2: 语法树
    // ====================================================================
    h.push_str("<div id=\"tab-tree\" class=\"tab-content\">\n");

    match prog {
        None => {
            h.push_str("<p>（语法分析未完成）</p>\n");
        }
        Some(p) => {
            h.push_str("<h2>抽象语法树</h2>\n");

            // 将 AST 渲染为树形文本，然后解析为 HTML
            let tree_text = format!("{}", p);

            // 辅助函数：计算行首 "│   " 块的个数（每个块 6 字节 UTF-8）
            fn count_depth(line: &str) -> (usize, &str) {
                let marker = "│   ";
                let mut n = 0usize;
                let mut rest = line;
                while rest.starts_with(marker) {
                    n += 1;
                    rest = &rest[marker.len()..];
                }
                (n, rest)
            }

            for line in tree_text.lines() {
                if line.is_empty() {
                    continue;
                }
                let (depth, rest) = count_depth(line);

                // 检测当前行是否为节点行（以 ├── 或 └── 开头）
                let (is_last, content_opt): (bool, Option<&str>) =
                    if let Some(c) = rest.strip_prefix("├── ") {
                        (false, Some(c))
                    } else if let Some(c) = rest.strip_prefix("└── ") {
                        (true, Some(c))
                    } else {
                        (false, None)
                    };

                if let Some(content) = content_opt {
                    let content = content.trim();
                    // 哨兵行 └── .
                    if content == "." {
                        h.push_str(&format!(
                            "<div class=\"tree-text\" style=\"padding-left:{}px\">.</div>\n",
                            depth * 20
                        ));
                        continue;
                    }
                    // 二元表达式子节点：ExpK 开头且内容为 Op 或 Const —— 纯文本不折叠
                    let is_binary_child = content.starts_with("ExpK")
                        && (content.contains("Op  ") || content.contains("Const  "));
                    if is_binary_child {
                        h.push_str(&format!(
                            "<div class=\"tree-text\" style=\"padding-left:{}px\">{}</div>\n",
                            depth * 20,
                            escape_html(content)
                        ));
                    } else {
                        let _ = is_last; // suppress unused warning
                        h.push_str(&format!(
                            "<details class=\"tree-node\" open style=\"padding-left:{}px\"><summary>{}</summary>\n",
                            depth * 20,
                            escape_html(content)
                        ));
                    }
                } else {
                    // 普通行：按缩进深度定位
                    h.push_str(&format!(
                        "<div class=\"tree-text\" style=\"padding-left:{}px\">{}</div>\n",
                        depth * 20,
                        escape_html(rest)
                    ));
                }
            }

            // 关闭所有未闭合的 <details>
            let details_count = tree_text
                .lines()
                .filter(|l| {
                    let (_, rest) = count_depth(l);
                    let content_opt = rest
                        .strip_prefix("├── ")
                        .or_else(|| rest.strip_prefix("└── "))
                        .map(|c| c.trim());
                    match content_opt {
                        None => false,
                        Some(".") => false,
                        Some(c) => {
                            !(c.starts_with("ExpK")
                                && (c.contains("Op  ") || c.contains("Const  ")))
                        }
                    }
                })
                .count();
            for _ in 0..details_count {
                h.push_str("</details>\n");
            }

            h.push_str("<br>\n");

            // 语法错误
            h.push_str("<div class=\"err-section\">\n");
            h.push_str("<h3>语法错误</h3>\n");
            if syntax_errors.is_empty() {
                h.push_str("<p class=\"no-errors\">无。</p>\n");
            } else {
                h.push_str("<table><thead><tr><th>行:列</th><th>信息</th></tr></thead><tbody>\n");
                for err in syntax_errors {
                    h.push_str("<tr class=\"error-row\"><td>");
                    h.push_str(&format!("{}:{}", err.loc.line, err.loc.col));
                    h.push_str("</td><td>");
                    h.push_str(&escape_html(&err.msg));
                    h.push_str("</td></tr>\n");
                }
                h.push_str("</tbody></table>\n");
            }
            h.push_str("</div>\n");
        }
    }
    h.push_str("</div>\n");

    // ====================================================================
    // 标签页 3: 符号表
    // ====================================================================
    h.push_str("<div id=\"tab-table\" class=\"tab-content\">\n");

    match scope_snapshots {
        None => {
            h.push_str("<p>（语义分析未完成）</p>\n");
        }
        Some(snapshots) => {
            h.push_str("<h2>符号表</h2>\n");

            h.push_str("<div class=\"scope-desc\">\n");
            h.push_str("<p>SNL 为过程声明使用嵌套作用域。");
            h.push_str("符号表组织为哈希映射栈，每个作用域级别一个。");
            h.push_str("查找时从最内层作用域向外层遍历。</p>\n");
            h.push_str(&format!("<p>总作用域数: {}</p>\n", snapshots.len()));
            h.push_str("</div>\n");

            h.push_str("<table id=\"symtable\"><thead><tr>");
            h.push_str("<th onclick=\"sortTable('symtable',0)\">作用域</th>");
            h.push_str("<th onclick=\"sortTable('symtable',1)\">级别</th>");
            h.push_str("<th onclick=\"sortTable('symtable',2)\">名称</th>");
            h.push_str("<th onclick=\"sortTable('symtable',3)\">种类</th>");
            h.push_str("<th onclick=\"sortTable('symtable',4)\">类型</th>");
            h.push_str("<th onclick=\"sortTable('symtable',5)\">参数</th>");
            h.push_str("<th onclick=\"sortTable('symtable',6)\">行号</th>");
            h.push_str("</tr></thead><tbody>\n");

            for (level, scope) in snapshots.iter() {
                let scope_label = if *level == 0 {
                    "全局".to_string()
                } else {
                    format!("过程级别 {}", level)
                };
                let mut entries: Vec<&SymbolEntry> = scope.values().collect();
                entries.sort_by(|a, b| a.name.cmp(&b.name));
                for entry in entries {
                    h.push_str("<tr><td>");
                    h.push_str(&scope_label);
                    h.push_str("</td><td>");
                    h.push_str(&entry.level.to_string());
                    h.push_str("</td><td>");
                    h.push_str(&escape_html(&entry.name));
                    h.push_str("</td><td>");
                    h.push_str(&format!("{:?}", entry.kind));
                    h.push_str("</td><td>");
                    if let Some(ty) = &entry.typ {
                        h.push_str(&escape_html(&format!("{}", ty)));
                    }
                    h.push_str("</td><td>");
                    if !entry.params.is_empty() {
                        h.push_str("(");
                        for (j, p) in entry.params.iter().enumerate() {
                            if j > 0 {
                                h.push_str(", ");
                            }
                            if p.is_var {
                                h.push_str("var ");
                            }
                            h.push_str(&escape_html(&p.name));
                            h.push_str(": ");
                            h.push_str(&escape_html(&format!("{}", p.typ)));
                        }
                        h.push_str(")");
                    }
                    h.push_str("</td><td>");
                    h.push_str(&entry.loc.line.to_string());
                    h.push_str("</td></tr>\n");
                }
            }
            h.push_str("</tbody></table>\n");

            // 语义错误
            h.push_str("<div class=\"err-section\">\n");
            h.push_str("<h3>语义错误</h3>\n");
            if semantic_errors.is_empty() {
                h.push_str("<p class=\"no-errors\">无。</p>\n");
            } else {
                h.push_str("<table><thead><tr><th>行:列</th><th>代码</th><th>信息</th></tr></thead><tbody>\n");
                for err in semantic_errors {
                    h.push_str("<tr class=\"error-row\"><td>");
                    h.push_str(&format!("{}:{}", err.loc.line, err.loc.col));
                    h.push_str("</td><td>");
                    h.push_str(&escape_html(&format!("{:?}", err.kind)));
                    h.push_str("</td><td>");
                    h.push_str(&escape_html(&err.msg));
                    h.push_str("</td></tr>\n");
                }
                h.push_str("</tbody></table>\n");
            }
            h.push_str("</div>\n");
        }
    }
    h.push_str("</div>\n");

    // ===== 页面尾部脚本（初始标签页显示） =====
    h.push_str("<script>\n");
    h.push_str("document.addEventListener('DOMContentLoaded',function(){\n");
    h.push_str("var tabs=document.querySelectorAll('.tab-content');\n");
    h.push_str("for(var i=1;i<tabs.length;i++)tabs[i].style.display='none';\n");
    h.push_str("tabs[0].style.display='block';\n");
    h.push_str("});\n");
    h.push_str("</script>\n");

    h.push_str("</body>\n</html>\n");

    h
}

/// 对 HTML 进行转义。
/// 将 `<`、`>`、`&`、`"` 分别替换为 `&lt;`、`&gt;`、`&amp;`、`&quot;`。
fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

/// 对 JavaScript 字符串中的 `</script>` 进行转义。
/// 将 `</script>`（不区分大小写）替换为 `<\/script>`，保留原始大小写。
#[allow(dead_code)]
fn escape_script(s: &str) -> String {
    let lower = s.to_ascii_lowercase();
    let pattern = "</script>";
    let mut out = String::with_capacity(s.len());
    let mut start = 0;
    while let Some(pos) = lower[start..].find(pattern) {
        let abs_pos = start + pos;
        out.push_str(&s[start..abs_pos]);
        let matched = &s[abs_pos..abs_pos + pattern.len()];
        out.push_str(&matched[0..1]);
        out.push('\\');
        out.push_str(&matched[1..]);
        start = abs_pos + pattern.len();
    }
    out.push_str(&s[start..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html_plain_text() {
        assert_eq!(escape_html("hello world"), "hello world");
    }

    #[test]
    fn test_escape_html_lt_gt() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_escape_html_amp() {
        assert_eq!(escape_html("a & b"), "a &amp; b");
    }

    #[test]
    fn test_escape_html_quot() {
        assert_eq!(escape_html("say \"hi\""), "say &quot;hi&quot;");
    }

    #[test]
    fn test_escape_html_all_entities() {
        assert_eq!(
            escape_html("<a href=\"test&go\">"),
            "&lt;a href=&quot;test&amp;go&quot;&gt;"
        );
    }

    #[test]
    fn test_escape_html_empty() {
        assert_eq!(escape_html(""), "");
    }

    #[test]
    fn test_escape_script_basic() {
        assert_eq!(escape_script("x</script>y"), "x<\\/script>y");
    }

    #[test]
    fn test_escape_script_uppercase() {
        assert_eq!(escape_script("</SCRIPT>"), "<\\/SCRIPT>");
    }

    #[test]
    fn test_escape_script_mixed_case() {
        assert_eq!(escape_script("</Script>"), "<\\/Script>");
    }

    #[test]
    fn test_escape_script_multiple() {
        assert_eq!(
            escape_script("a</script>b</script>c"),
            "a<\\/script>b<\\/script>c"
        );
    }

    #[test]
    fn test_escape_script_empty() {
        assert_eq!(escape_script(""), "");
    }

    #[test]
    fn test_escape_script_no_match() {
        assert_eq!(escape_script("hello world"), "hello world");
    }

    // ---- HTML report tests (RED phase — stub returns empty) ----

    use snl_compiler::lexer::{LexerError, Token, TokenKind};

    fn sample_tokens() -> Vec<Token> {
        vec![
            Token {
                kind: TokenKind::Ident("x".into()),
                line: 1,
                col: 1,
            },
            Token {
                kind: TokenKind::Assign,
                line: 1,
                col: 3,
            },
            Token {
                kind: TokenKind::IntConst(42),
                line: 1,
                col: 6,
            },
        ]
    }

    fn sample_errors() -> Vec<LexerError> {
        vec![LexerError {
            msg: "Unterminated comment".into(),
            line: 3,
            col: 1,
        }]
    }

    #[test]
    fn test_html_has_doctype_and_charset() {
        let tokens = sample_tokens();
        let errors = sample_errors();
        let html = format_report_html("test.snl", &tokens, &errors, None, &[], None, &[]);
        assert!(
            html.starts_with("<!DOCTYPE html>"),
            "HTML should start with <!DOCTYPE html>"
        );
        assert!(
            html.contains("<meta charset=\"UTF-8\">"),
            "HTML should contain <meta charset=\"UTF-8\">"
        );
    }

    #[test]
    fn test_html_tab_structure() {
        let tokens = sample_tokens();
        let html = format_report_html("test.snl", &tokens, &[], None, &[], None, &[]);
        assert!(html.contains("id=\"tab-token\""), "Should have tab-token");
        assert!(html.contains("id=\"tab-tree\""), "Should have tab-tree");
        assert!(html.contains("id=\"tab-table\""), "Should have tab-table");
    }

    #[test]
    fn test_html_tab_switching_js() {
        let tokens = sample_tokens();
        let html = format_report_html("test.snl", &tokens, &[], None, &[], None, &[]);
        assert!(
            html.contains("showTab"),
            "JavaScript should define showTab function"
        );
    }

    #[test]
    fn test_html_token_table_content() {
        let tokens = sample_tokens();
        let html = format_report_html("test.snl", &tokens, &[], None, &[], None, &[]);
        assert!(html.contains("<tr>"), "Should contain table rows");
        assert!(html.contains("Ident"), "Should contain token kind Ident");
        assert!(
            html.contains("IntConst"),
            "Should contain token kind IntConst"
        );
    }

    #[test]
    fn test_html_entity_escaping() {
        let tokens = vec![Token {
            kind: TokenKind::Ident("<test>".into()),
            line: 1,
            col: 1,
        }];
        let html = format_report_html("test.snl", &tokens, &[], None, &[], None, &[]);
        assert!(
            html.contains("&lt;test&gt;"),
            "HTML entities should escape < > to &lt; &gt;"
        );
        assert!(
            !html.contains("<test>"),
            "Raw angle brackets should not appear in HTML"
        );
    }

    #[test]
    fn test_html_error_section() {
        let errors = sample_errors();
        let html_with = format_report_html("test.snl", &[], &errors, None, &[], None, &[]);
        assert!(
            html_with.contains("<td>"),
            "Error table should have cells when errors exist"
        );

        let html_without = format_report_html("test.snl", &[], &[], None, &[], None, &[]);
        assert!(
            html_without.contains("无"),
            "Should show 无 when no errors"
        );
    }
}
