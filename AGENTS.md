# AGENTS.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

---

# PROJECT KNOWLEDGE BASE

**Generated:** 2026-06-15 | **Commit:** 8ed3892 | **Branch:** main

## OVERVIEW

SNL (Small Nested Language) → MIPS I assembly compiler. Rust, edition 2024, zero dependencies. 4-phase pipeline: lexer → parser (recursive descent + LL(1) verify) → semantic → codegen. 139 tests, 17 verified samples.

## STRUCTURE

```
src/
├── main.rs      # CLI, 4-phase pipeline, Markdown output (*_token.md, *_tree.md, *_table.md)
├── lib.rs       # Module re-exports: ast, codegen, error, lexer, parser, semantic
├── error.rs     # CompileError + ErrorKind + SemanticErrCode — unified across all phases
├── report.rs    # Self-contained HTML diagnostic report (binary-only, not in lib.rs)
├── lexer/       # DFA tokenization (private: dfa, keyword; public: token)
├── ast/         # Shared AST nodes (Program, Stm, Exp...) + tree-format Display
├── parser/      # Recursive descent (primary, builds AST) + LL(1) table-driven (verification)
├── semantic/    # Two-pass: symbol table → type check (12 error types)
└── codegen/     # MIPS I emission, single mips.rs (1141 lines)
samples/         # 17 SNL programs + generated .asm / _token.md / _tree.md / _table.md
doc/             # SNL spec, audit report, technical docs
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add token type | `src/lexer/token.rs` | `TokenKind` enum |
| Add keyword | `src/lexer/keyword.rs` | Sorted array, binary search |
| Extend grammar (RD) | `src/parser/rd.rs` | One `parse_*` per nonterminal |
| Extend grammar (LL(1)) | `src/parser/grammar.rs` | Production list at line ~98 |
| Add AST node | `src/ast/nodes.rs` | Struct + parent enum variant |
| Add AST display | `src/ast/display.rs` | impl Display or fmt_node arm |
| Add semantic check | `src/semantic/analyzer.rs` | Pass in check_* or new error code |
| Add error code | `src/error.rs` | SemanticErrCode or ErrorKind variant |
| Add MIPS instruction | `src/codegen/mips.rs` | compile_stm / compile_exp / helpers |
| Change error exit policy | `src/main.rs` | Per-stage fatal/continue logic |
| Add HTML report feature | `src/report.rs` | format_report_html(), self-contained CSS/JS |

## CODE MAP

| Symbol | Type | File | Role |
|--------|------|------|------|
| `Lexer::tokenize()` | method | `lexer/mod.rs:55` | Source → Token[], collects LexerError |
| `TokenKind` | enum | `lexer/token.rs:10` | 34 variants, bridge lexer↔parser |
| `RdParser::parse()` | method | `parser/rd.rs:47` | Tokens → Option\<Program>, collects syntax errors |
| `Ll1Parser::parse()` | method | `parser/ll1.rs:53` | LL(1) verify → bool, mismatch as errors |
| `SemanticAnalyzer::analyze()` | method | `semantic/analyzer.rs:64` | Two-pass on Program, collects semantic errors |
| `compile()` | function | `codegen/mips.rs:369` | Program → Result\<String, Vec\<CompileError>> |
| `CompileError` | struct | `error.rs:55` | Unified error, used by all phases |
| `SemanticErrCode` | enum | `error.rs:11` | 12 typed semantic error variants |
| `SymbolTable` | struct | `semantic/symbol.rs:82` | Nested-scope HashMap stack |
| `Loc` | struct | `ast/nodes.rs:17` | 1-indexed {line, col}, shared by everything |
| `format_report_html()` | function | `report.rs:8` | Self-contained HTML diagnostic report (pub(crate)) |

## CONVENTIONS

- **Tests**: Inline `#[cfg(test)] mod tests` at bottom of source files. No `tests/` dir. 139 total (lexer=25, parser=43, semantic=22, codegen=35, report=14).
- **Error accumulation**: All phases collect errors in internal `Vec`, never abort mid-phase. `main.rs` decides exit policy per stage.
- **Lexer is isolated**: Uses local `LexerError`, not `CompileError`. Only module with private child modules.
- **Two parsers**: RD builds AST (authoritative). LL(1) verifies grammar (warning-only on parse mismatch).
- **Codegen owns its types**: `CodegenType` duplicates `TypeInfo` — intentional decoupling from semantic analysis.
- **Symbol table**: Nested scopes via `Vec<HashMap<...>>`. Lookup traverses `iter().rev()` inside-out.
- **Doc language**: Chinese. `//!` for module docs, `///` for item docs.
- **No rustfmt/clippy config**: Default Rust formatting. `cargo clippy` must produce 0 warnings.
- **Register convention**: `$v0` for results, `$t0` for addresses, `$fp`/`$ra` saved on stack at proc entry.
- **Panic policy**: `panic!()` and bare `.unwrap()` prohibited. Use `.expect("why")` or return `CompileError`.

## ANTI-PATTERNS

- **`let _ = self.symbols.insert(...)`** — silently discards Result. Duplicate-definition errors vanish. Use `is_err()`.
- **`to_lowercase()` on hot path** — allocates. Use `to_ascii_lowercase()`.
- **`.clone()` without need** — borrow instead. Especially `TypeInfo` and token lists.
- **Tail recursion without TCO** — Rust TCO is not guaranteed. Use `while` loops.
- **Gating LL(1) behind `#[cfg(test)]`** — it's production-required. Do not re-gate.
- **`CompileError::codegen()` with zero location** — codegen errors carry no source position.
- **`resolve_type()` without cycle detection** — circular type aliases (e.g., `type A = B; type B = A`) cause stack overflow. Use `visited: &mut Vec<String>` parameter pattern from `codegen/mips.rs:101`.
- **`parse_intc()` returns 0 on failure** — silently allows wrong array bounds into AST. Return `Option<i64>`, propagate `None` upward.
- **Silent defaults in codegen** — `emit_var_address` falls back to `(0, 0)` for unknown variables. Check `ctx.errors` growth after fallback.
- **`unwrap()` in production code** — 6 remain: `symbol.rs:119,140`, `analyzer.rs:61`, `first_follow.rs:39,47,78`. Use `.expect("why")`.
- **`let _ = compile_exp(...)`** — discards CodegenType return in `codegen/mips.rs` (6 sites: lines 676, 688, 723, 728, 751, 754). Acceptable today but masks future type-dependent codegen.
- **Zero-location `Loc { line: 0, col: 0 }` fallbacks in semantic** — 5 sites in `analyzer.rs`. Produces confusing error output. Use actual source position.
- **`args[3].clone()` without explicit bounds check** — `main.rs:34` relies on prior `args.len() >= 4` check. Fragile to index-logic changes. Use `args.get(3)` or destructure.

## COMMANDS

```bash
cargo build              # debug
cargo build --release    # release
cargo test               # all 139 tests
cargo test lexer         # 25 tests
cargo test parser        # 43 tests (rd + ll1 + first_follow)
cargo test semantic      # 22 tests
cargo test codegen       # 35 tests
cargo run -- samples/hello.snl                # → hello.asm
cargo run -- samples/hello.snl -o out.asm     # custom output
```

## NOTES

- Single crate, zero external dependencies. Rust edition 2024.
- CI (`RustMulti-PlatformBuild.yml`): manual trigger, build-only (no tests). 6 targets.
- Audit report at `doc/audit.md`: 120+ issues found, 22 fixed. Canonical quality reference.
- Diagnostic output: `{basename}_report.html` — self-contained interactive HTML with tabs, collapsible AST tree, sortable tables, and search filtering. Generated via `format_report_html()` in `main.rs`.
- SNL language spec at `doc/snl.md`.
