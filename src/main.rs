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
    h.push_str("body{font:15px/1.6 -apple-system,system-ui,sans-serif;max-width:1100px;margin:auto;padding:24px 16px;background:#f8f9fa;color:#1a1a2e}\n");
    h.push_str("h1{font-size:24px;color:#0f172a;margin:0 0 20px}\n");
    h.push_str("h2{font-size:16px;color:#334155;border-bottom:1px solid #e5e7eb;padding-bottom:6px;margin:20px 0 12px}\n");
    h.push_str("h3{font-size:14px;color:#475569;margin:16px 0 8px}\n");
    h.push_str(".tab-bar{display:flex;gap:8px;margin-bottom:20px}\n");
    h.push_str(".tab-btn{padding:8px 20px;border:none;background:transparent;color:#64748b;font-size:14px;font-weight:500;border-radius:6px;cursor:pointer;transition:all .15s}\n");
    h.push_str(".tab-btn:hover{background:#e5e7eb;color:#334155}\n");
    h.push_str(".tab-btn.active{background:#2563eb;color:#fff}\n");
    h.push_str(".tab-content{background:#fff;border-radius:10px;box-shadow:0 1px 3px rgba(0,0,0,.06);padding:24px;margin-bottom:16px;display:none}\n");
    h.push_str(".tab-content:first-of-type{display:block}\n");
    h.push_str(".search-bar{margin-bottom:16px}\n");
    h.push_str(".search-bar input{width:260px;padding:8px 14px;border:1px solid #d1d5db;border-radius:8px;font-size:14px;outline:none;transition:border-color .15s}\n");
    h.push_str(".search-bar input:focus{border-color:#2563eb;box-shadow:0 0 0 3px rgba(37,99,235,.1)}\n");
    h.push_str("table{border-collapse:collapse;width:100%;font-size:13px}\n");
    h.push_str("th{background:#f1f5f9;color:#475569;font-weight:600;text-align:left;padding:10px 12px;border-bottom:2px solid #e5e7eb;position:sticky;top:0;cursor:pointer;white-space:nowrap}\n");
    h.push_str("th:hover{background:#e2e8f0}\n");
    h.push_str("td{padding:8px 12px;border-bottom:1px solid #f1f5f9}\n");
    h.push_str("tr:hover td{background:#f8fafc}\n");
    h.push_str(".scope-label{font-weight:600;color:#0f172a;margin-top:24px;font-size:15px}\n");
    h.push_str(".scope-desc{margin:8px 0;color:#64748b;line-height:1.7}\n");
    h.push_str(".tree-toolbar{display:flex;align-items:center;gap:8px;flex-wrap:wrap;margin:0 0 12px}\n");
    h.push_str(".tree-toolbar input[type=text]{width:260px;padding:6px 10px;border:1px solid #d1d5db;border-radius:6px;font-size:13px;outline:none}\n");
    h.push_str(".tree-toolbar input[type=text]:focus{border-color:#2563eb;box-shadow:0 0 0 3px rgba(37,99,235,.1)}\n");
    h.push_str(".tree-toolbar button{padding:6px 10px;border:1px solid #d1d5db;background:#fff;color:#334155;border-radius:6px;font-size:13px;cursor:pointer}\n");
    h.push_str(".tree-toolbar button:hover{background:#f1f5f9}\n");
    h.push_str(".tree-toolbar label{display:flex;align-items:center;gap:4px;color:#64748b;font-size:13px}\n");
    h.push_str(".tree-scroll{max-width:100%;overflow-x:auto;overflow-y:hidden;padding-bottom:8px}\n");
    h.push_str(".tree-node{margin:0;border-left:3px solid transparent;transition:border-color .15s}\n");
    h.push_str(".tree-node[open]{margin-bottom:2px}\n");
    h.push_str(".tree-node summary{cursor:pointer;font-family:'SF Mono',SFMono-Regular,Consolas,monospace;font-size:13px;white-space:pre;padding:3px 8px;border-radius:4px;transition:background .1s;overflow:hidden;text-overflow:ellipsis}\n");
    h.push_str(".tree-node summary:hover{background:#f1f5f9}\n");
    h.push_str(".tree-node div{padding-left:20px}\n");
    h.push_str(".tree-text{font-family:'SF Mono',SFMono-Regular,Consolas,monospace;font-size:13px;white-space:pre;padding:2px 8px;color:#64748b;overflow:hidden;text-overflow:ellipsis}\n");
    h.push_str(".tree-node.tree-match>summary,.tree-text.tree-match{background:#fef3c7;color:#92400e}\n");
    h.push_str("/* Syntax tree color coding */\n");
    h.push_str(".tn-decl{border-left-color:#3b82f6!important}.tn-decl summary{color:#1e40af}\n");
    h.push_str(".tn-stmt{border-left-color:#10b981!important}.tn-stmt summary{color:#065f46}\n");
    h.push_str(".tn-expr{border-left-color:#f59e0b!important}.tn-expr summary{color:#92400e}\n");
    h.push_str("/* Tree indent guides */\n");
    h.push_str(".tree-guide{border-left:1px dashed #e5e7eb;margin-left:8px;padding-left:12px}\n");
    h.push_str(".err-section{margin:16px 0;padding:12px 16px;background:#fef2f2;border-left:3px solid #ef4444;border-radius:0 8px 8px 0}\n");
    h.push_str(".err-section h3{color:#dc2626;margin-top:0}\n");
    h.push_str(".no-errors{color:#9ca3af;font-style:italic;margin:8px 0}\n");
    h.push_str("@media print{body{background:#fff;font-size:12px}.tab-bar,.search-bar{display:none}.tab-content{box-shadow:none;border:1px solid #ddd;break-inside:avoid}}\n");
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
    h.push_str("filterTree(query);\n");
    h.push_str("var q=query.toLowerCase();\n");
    h.push_str("document.querySelectorAll('.tab-content[style*=\"block\"] table tbody tr').forEach(function(r){\n");
    h.push_str("if(!q){r.style.display='';return;}\n");
    h.push_str("r.style.display=r.textContent.toLowerCase().indexOf(q)>=0?'':'none';\n");
    h.push_str("});\n");
    h.push_str("}\n");
    h.push_str("function setTreeOpen(open){\n");
    h.push_str("document.querySelectorAll('#tree-browser details.tree-node').forEach(function(d){d.open=open});\n");
    h.push_str("}\n");
    h.push_str("function filterTree(query){\n");
    h.push_str("var root=document.getElementById('tree-browser');\n");
    h.push_str("if(!root)return;\n");
    h.push_str("var q=(query||'').toLowerCase();\n");
    h.push_str("var only=document.getElementById('tree-only-matches');\n");
    h.push_str("var onlyMatches=only&&only.checked;\n");
    h.push_str("var items=root.querySelectorAll('.tree-node,.tree-text');\n");
    h.push_str("items.forEach(function(item){item.classList.remove('tree-match');item.style.display=''});\n");
    h.push_str("if(!q)return;\n");
    h.push_str("items.forEach(function(item){\n");
    h.push_str("var text=(item.dataset.treeText||item.textContent||'').toLowerCase();\n");
    h.push_str("if(text.indexOf(q)>=0){\n");
    h.push_str("item.classList.add('tree-match');\n");
    h.push_str("var parent=item.closest('details.tree-node');\n");
    h.push_str("while(parent){parent.open=true;parent=parent.parentElement.closest('details.tree-node')}\n");
    h.push_str("}\n");
    h.push_str("});\n");
    h.push_str("if(!onlyMatches)return;\n");
    h.push_str("root.querySelectorAll('.tree-text').forEach(function(t){if(!t.classList.contains('tree-match'))t.style.display='none'});\n");
    h.push_str("root.querySelectorAll('details.tree-node').forEach(function(d){\n");
    h.push_str("var keep=d.classList.contains('tree-match')||d.querySelector('.tree-match');\n");
    h.push_str("d.style.display=keep?'':'none';\n");
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
            h.push_str("<div class=\"tree-toolbar\">\n");
            h.push_str("<input id=\"tree-search\" type=\"text\" placeholder=\"搜索语法树...\" oninput=\"filterTree(this.value)\">\n");
            h.push_str("<button type=\"button\" onclick=\"setTreeOpen(true)\">全部展开</button>\n");
            h.push_str("<button type=\"button\" onclick=\"setTreeOpen(false)\">全部折叠</button>\n");
            h.push_str("<label><input id=\"tree-only-matches\" type=\"checkbox\" onchange=\"filterTree(document.getElementById('tree-search').value)\">只显示匹配项</label>\n");
            h.push_str("</div>\n");

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

            h.push_str("<div class=\"tree-scroll\">\n");
            h.push_str("<div id=\"tree-browser\">\n");
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
                        let escaped_content = escape_html(content);
                        h.push_str(&format!(
                            "<div class=\"tree-text\" data-tree-text=\"{}\" title=\"{}\" style=\"padding-left:{}px\">{}</div>\n",
                            escaped_content,
                            escaped_content,
                            depth * 20,
                            escaped_content
                        ));
                        continue;
                    }
                    // 二元表达式子节点：ExpK 开头且内容为 Op 或 Const —— 纯文本不折叠
                    let is_binary_child = content.starts_with("ExpK")
                        && (content.contains("Op  ") || content.contains("Const  "));
                    if is_binary_child {
                        let escaped_content = escape_html(content);
                        h.push_str(&format!(
                            "<div class=\"tree-text\" data-tree-text=\"{}\" title=\"{}\" style=\"padding-left:{}px\">{}</div>\n",
                            escaped_content,
                            escaped_content,
                            depth * 20,
                            escaped_content
                        ));
                    } else {
                        let _ = is_last; // suppress unused warning
                        let type_class = node_type_class(content);
                        let escaped_content = escape_html(content);
                        h.push_str(&format!(
                            "<div class=\"tree-guide\" style=\"padding-left:{}px\"><details class=\"tree-node {}\" data-tree-text=\"{}\" open><summary title=\"{}\">{}</summary>\n",
                            depth * 20,
                            type_class,
                            escaped_content,
                            escaped_content,
                            escaped_content
                        ));
                    }
                } else {
                    // 普通行：按缩进深度定位
                    let escaped_rest = escape_html(rest);
                    h.push_str(&format!(
                        "<div class=\"tree-text\" data-tree-text=\"{}\" title=\"{}\" style=\"padding-left:{}px\">{}</div>\n",
                        escaped_rest,
                        escaped_rest,
                        depth * 20,
                        escaped_rest
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
                h.push_str("</details></div>\n");
            }
            h.push_str("</div>\n");
            h.push_str("</div>\n");

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
                        h.push('(');
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
                        h.push(')');
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

/// 根据 AST 节点内容返回类型特定的 CSS 类名。
/// - 声明类（ProK/PheadK/VarK/TypeK/ProcDecK/DecK）→ `"tn-decl"`
/// - 语句类（StmLk/StmtK）→ `"tn-stmt"`
/// - 表达式类（ExpK）→ `"tn-expr"`
/// - 其他 → `""`（无类型类）
fn node_type_class(content: &str) -> &str {
    if content.starts_with("ProK")
        || content.starts_with("PheadK")
        || content.starts_with("VarK")
        || content.starts_with("TypeK")
        || content.starts_with("ProcDecK")
        || content.starts_with("DecK")
    {
        "tn-decl"
    } else if content.starts_with("StmLk") || content.starts_with("StmtK") {
        "tn-stmt"
    } else if content.starts_with("ExpK") {
        "tn-expr"
    } else {
        ""
    }
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

    // ---- HTML report tests (RED phase — stub returns empty) ----

    use snl_compiler::ast::nodes::{DeclarePart, Loc, ProcDec, Program, StmList, TypeDec, VarDec};
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

    fn sample_program() -> Program {
        let loc = Loc { line: 1, col: 1 };
        Program {
            name: "very_long_program_name_for_tree_width_regression".into(),
            decl: DeclarePart {
                types: TypeDec::Empty,
                vars: VarDec::Empty,
                procs: ProcDec::Empty,
            },
            body: StmList {
                stmts: Vec::new(),
                loc,
            },
            loc,
        }
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
    fn test_html_tree_has_horizontal_scroll_container() {
        let program = sample_program();
        let html = format_report_html("test.snl", &[], &[], Some(&program), &[], None, &[]);

        assert!(
            html.contains(".tree-scroll{max-width:100%;overflow-x:auto;"),
            "Tree CSS should constrain width and enable horizontal scrolling"
        );
        assert!(
            html.contains("<div class=\"tree-scroll\">"),
            "Rendered AST should be wrapped in the scroll container"
        );
    }

    #[test]
    fn test_html_tree_browser_controls_and_search_hooks() {
        let program = sample_program();
        let html = format_report_html("test.snl", &[], &[], Some(&program), &[], None, &[]);

        assert!(
            html.contains("id=\"tree-search\""),
            "Tree browser should include a tree search input"
        );
        assert!(
            html.contains("id=\"tree-only-matches\""),
            "Tree browser should include a match-only toggle"
        );
        assert!(
            html.contains("function setTreeOpen"),
            "Tree browser should include expand/collapse controls"
        );
        assert!(
            html.contains("function filterTree"),
            "Tree browser should include tree filtering"
        );
        assert!(
            html.contains(".tree-node.tree-match>summary,.tree-text.tree-match"),
            "Tree browser should highlight matching nodes"
        );
        assert!(
            html.contains("data-tree-text=\"ProK\""),
            "Rendered tree nodes should carry searchable text"
        );
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
