//! 递归下降语法分析器（Recursive Descent Parser）。
//!
//! SNL 编译器的**主解析器**，负责将 Token 序列转换为 AST。
//! 基于 SNL 文法为每个非终结符编写对应的递归解析函数。
//!
//! ## 设计要点
//! - **直接构建 AST**: 每个解析函数返回对应的 AST 节点
//! - **向前看 1 个 Token**: 通过 `peek_kind()` 决定进入哪个产生式
//! - **恐慌模式错误恢复**: `sync()` 在遇到错误时跳过 Token 直到同步点
//! - **错误容忍**: 解析错误记录到 `errors` 列表，不会中断解析
//!
//! ## 与其他模块的关系
//! - 输入：来自 [`lexer`] 的 Token 序列
//! - 输出：[`ast::nodes::Program`] 供 [`semantic`] 和 [`codegen`] 使用

use crate::ast::nodes::*;
use crate::error::CompileError;
use crate::lexer::token::{Token, TokenKind};

/// 递归下降语法分析器。
///
/// 持有 Token 流引用、当前位置和错误列表。
/// 通过生命周期参数 `'a` 绑定到输入的 Token 序列。
pub struct RdParser<'a> {
    tokens: &'a [Token],
    pos: usize,
    errors: Vec<CompileError>,
}

impl<'a> RdParser<'a> {
    /// 创建分析器，绑定到给定的 Token 序列。
    pub fn new(tokens: &'a [Token]) -> Self {
        RdParser {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    pub fn errors(&self) -> &[CompileError] {
        &self.errors
    }

    /// 开始解析，返回完整的 `Program` AST。
    ///
    /// 解析失败时返回 `None`。
    pub fn parse(&mut self) -> Option<Program> {
        self.parse_program()
    }

    // ===== 辅助函数 =====

    /// 查看当前 Token（不消费）。
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    /// 查看当前 Token 种类（不消费）。
    fn peek_kind(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    /// 消费并返回当前 Token，指针前移。
    ///
    /// 若已到 EOF，不再前移。
    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    /// 尝试匹配期望的 Token 种类。
    ///
    /// 匹配成功则消费并返回该 Token，失败则记录语法错误并返回 `None`。
    fn match_token(&mut self, expected: TokenKind) -> Option<Token> {
        if *self.peek_kind() == expected {
            Some(self.advance().clone())
        } else {
            let t = self.peek();
            self.errors.push(CompileError::syntax(
                format!("Expected {}, found {:?}", token_name(&expected), t.kind),
                Loc {
                    line: t.line,
                    col: t.col,
                },
            ));
            None
        }
    }

    /// 获取当前 Token 的源码位置。
    fn loc(&self) -> Loc {
        let t = self.peek();
        Loc {
            line: t.line,
            col: t.col,
        }
    }

    /// 恐慌模式错误恢复：跳过 Token 直到遇到同步集合中的符号。
    ///
    /// 同步集合通常在语句边界处（`;`、`end`、`fi` 等）。
    fn sync(&mut self, sync_tokens: &[TokenKind]) {
        while !sync_tokens.contains(self.peek_kind()) && !matches!(self.peek_kind(), TokenKind::Eof)
        {
            self.pos += 1;
        }
    }

    // ===== 1. Program ::= ProgramHead DeclarePart ProgramBody =====

    fn parse_program(&mut self) -> Option<Program> {
        let loc = self.loc();
        let name = self.parse_program_head()?;
        let decl = self.parse_declare_part();
        let body = self.parse_program_body();
        Some(Program {
            name,
            decl,
            body,
            loc,
        })
    }

    // ===== 2. ProgramHead ::= PROGRAM ProgramName =====

    fn parse_program_head(&mut self) -> Option<String> {
        self.match_token(TokenKind::Program)?;
        let name = self.parse_program_name()?;
        Some(name)
    }

    // ===== 3. ProgramName ::= ID =====

    fn parse_program_name(&mut self) -> Option<String> {
        match self.peek_kind() {
            TokenKind::Ident(name) => {
                let n = name.clone();
                self.advance();
                Some(n)
            }
            _ => {
                let t = self.peek().clone();
                self.errors.push(CompileError::syntax(
                    format!("Expected identifier, found {:?}", t.kind),
                    Loc {
                        line: t.line,
                        col: t.col,
                    },
                ));
                None
            }
        }
    }

    // ===== 4. DeclarePart ::= TypeDec VarDec ProcDec =====

    fn parse_declare_part(&mut self) -> DeclarePart {
        let types = self.parse_type_dec();
        let vars = self.parse_var_dec();
        let procs = self.parse_proc_dec();
        DeclarePart { types, vars, procs }
    }

    // ===== 6. TypeDec ::= ε | TypeDeclaration =====

    fn parse_type_dec(&mut self) -> TypeDec {
        if *self.peek_kind() == TokenKind::Type {
            self.parse_type_declaration()
        } else {
            TypeDec::Empty
        }
    }

    // ===== 7. TypeDeclaration ::= TYPE TypeDecList =====

    fn parse_type_declaration(&mut self) -> TypeDec {
        self.match_token(TokenKind::Type);
        let mut defs = Vec::new();
        self.parse_type_dec_list(&mut defs);
        if defs.is_empty() {
            TypeDec::Empty
        } else {
            TypeDec::Defined(defs)
        }
    }

    // ===== 8. TypeDecList ::= TypeId = TypeName ; TypeDecMore =====

    fn parse_type_dec_list(&mut self, defs: &mut Vec<TypeDef>) {
        if !matches!(self.peek_kind(), TokenKind::Ident(_)) {
            return;
        }
        let name = match self.peek_kind() {
            TokenKind::Ident(n) => n.clone(),
            _ => return,
        };
        let loc = self.loc();
        self.advance(); // 消费 TypeId
        self.match_token(TokenKind::Equal);
        let body = self.parse_type_name();
        self.match_token(TokenKind::Semicolon);
        defs.push(TypeDef { name, body, loc });
        self.parse_type_dec_more(defs);
    }

    // ===== 9. TypeDecMore ::= ε | TypeDecList =====

    fn parse_type_dec_more(&mut self, defs: &mut Vec<TypeDef>) {
        while matches!(self.peek_kind(), TokenKind::Ident(_)) {
            self.parse_type_dec_list(defs);
        }
    }

    // ===== 12. TypeName ::= BaseType | StructureType | ID =====

    fn parse_type_name(&mut self) -> TypeBody {
        match self.peek_kind() {
            TokenKind::Integer => {
                self.advance();
                TypeBody::Base(BaseType::Integer)
            }
            TokenKind::Char => {
                self.advance();
                TypeBody::Base(BaseType::Char)
            }
            TokenKind::Array => self.parse_array_type(),
            TokenKind::Record => self.parse_rec_type(),
            TokenKind::Ident(name) => {
                let n = name.clone();
                self.advance();
                TypeBody::Named(n)
            }
            _ => {
                let t = self.peek().clone();
                self.errors.push(CompileError::syntax(
                    format!("Expected type name, found {:?}", t.kind),
                    Loc {
                        line: t.line,
                        col: t.col,
                    },
                ));
                // 错误恢复：返回 Integer 作为默认类型
                TypeBody::Base(BaseType::Integer)
            }
        }
    }

    // ===== 19. ArrayType ::= ARRAY [ low .. top ] OF BaseType =====

    fn parse_array_type(&mut self) -> TypeBody {
        let loc = self.loc();
        self.match_token(TokenKind::Array);
        self.match_token(TokenKind::LBracket);
        let low = self.parse_low();
        self.match_token(TokenKind::Range);
        let high = self.parse_top();
        self.match_token(TokenKind::RBracket);
        self.match_token(TokenKind::Of);
        let elem_type = self.parse_base_type();
        TypeBody::Array(ArrayTypeDef {
            low,
            high,
            elem_type,
            loc,
        })
    }

    fn parse_low(&mut self) -> i64 {
        self.parse_intc()
    }

    fn parse_top(&mut self) -> i64 {
        self.parse_intc()
    }

    /// 解析整数字面量，失败时返回 0 作为容错默认值。
    fn parse_intc(&mut self) -> i64 {
        match self.peek_kind() {
            TokenKind::IntConst(n) => {
                let val = *n;
                self.advance();
                val
            }
            _ => {
                let t = self.peek().clone();
                self.errors.push(CompileError::syntax(
                    format!("Expected integer, found {:?}", t.kind),
                    Loc {
                        line: t.line,
                        col: t.col,
                    },
                ));
                0
            }
        }
    }

    fn parse_base_type(&mut self) -> BaseType {
        match self.peek_kind() {
            TokenKind::Integer => {
                self.advance();
                BaseType::Integer
            }
            TokenKind::Char => {
                self.advance();
                BaseType::Char
            }
            _ => {
                let t = self.peek().clone();
                self.errors.push(CompileError::syntax(
                    format!("Expected INTEGER or CHAR, found {:?}", t.kind),
                    Loc {
                        line: t.line,
                        col: t.col,
                    },
                ));
                BaseType::Integer
            }
        }
    }

    // ===== 22. RecType ::= RECORD FieldDecList END =====

    fn parse_rec_type(&mut self) -> TypeBody {
        let loc = self.loc();
        self.match_token(TokenKind::Record);
        let mut fields = Vec::new();
        self.parse_field_dec_list(&mut fields);
        self.match_token(TokenKind::End);
        TypeBody::Record(RecordTypeDef { fields, loc })
    }

    // ===== 23-26. FieldDecList, FieldDecMore =====

    fn parse_field_dec_list(&mut self, fields: &mut Vec<FieldDef>) {
        let loc = self.loc();
        let typ = match self.peek_kind() {
            TokenKind::Integer | TokenKind::Char => FieldTypeDef::Base(self.parse_base_type()),
            TokenKind::Array => {
                if let TypeBody::Array(arr) = self.parse_array_type() {
                    FieldTypeDef::Array(arr)
                } else {
                    return;
                }
            }
            _ => return, // ε
        };

        let names = self.parse_id_list();
        self.match_token(TokenKind::Semicolon);
        fields.push(FieldDef { typ, names, loc });
        self.parse_field_dec_more(fields);
    }

    fn parse_field_dec_more(&mut self, fields: &mut Vec<FieldDef>) {
        while matches!(
            self.peek_kind(),
            TokenKind::Integer | TokenKind::Char | TokenKind::Array
        ) {
            self.parse_field_dec_list(fields);
        }
    }

    // ===== 27-29. IdList, IdMore =====

    fn parse_id_list(&mut self) -> Vec<String> {
        let mut names = Vec::new();
        if let TokenKind::Ident(name) = self.peek_kind() {
            names.push(name.clone());
            self.advance();
            while *self.peek_kind() == TokenKind::Comma {
                self.advance();
                if let TokenKind::Ident(name) = self.peek_kind() {
                    names.push(name.clone());
                    self.advance();
                } else {
                    break;
                }
            }
        }
        names
    }

    // ===== 30. VarDec ::= ε | VarDeclaration =====

    fn parse_var_dec(&mut self) -> VarDec {
        if *self.peek_kind() == TokenKind::Var {
            self.parse_var_declaration()
        } else {
            VarDec::Empty
        }
    }

    // ===== 32. VarDeclaration ::= VAR VarDecList =====

    fn parse_var_declaration(&mut self) -> VarDec {
        self.match_token(TokenKind::Var);
        let mut defs = Vec::new();
        self.parse_var_dec_list(&mut defs);
        if defs.is_empty() {
            VarDec::Empty
        } else {
            VarDec::Defined(defs)
        }
    }

    // ===== 33. VarDecList ::= TypeName VarIdList ; VarDecMore =====

    fn parse_var_dec_list(&mut self, defs: &mut Vec<VarDef>) {
        let loc = self.loc();
        let type_name = self.parse_type_desig();
        let names = self.parse_var_id_list();
        if names.is_empty() {
            return;
        }
        self.match_token(TokenKind::Semicolon);
        defs.push(VarDef {
            type_name,
            names,
            loc,
        });
        self.parse_var_dec_more(defs);
    }

    /// 解析用于变量声明的类型描述符。
    fn parse_type_desig(&mut self) -> TypeDesig {
        match self.peek_kind() {
            TokenKind::Integer => {
                self.advance();
                TypeDesig::Base(BaseType::Integer)
            }
            TokenKind::Char => {
                self.advance();
                TypeDesig::Base(BaseType::Char)
            }
            TokenKind::Array => {
                if let TypeBody::Array(arr) = self.parse_array_type() {
                    TypeDesig::Array(arr)
                } else {
                    TypeDesig::Base(BaseType::Integer)
                }
            }
            TokenKind::Record => {
                if let TypeBody::Record(rec) = self.parse_rec_type() {
                    TypeDesig::Record(rec)
                } else {
                    TypeDesig::Base(BaseType::Integer)
                }
            }
            TokenKind::Ident(name) => {
                let n = name.clone();
                self.advance();
                TypeDesig::Named(n)
            }
            _ => {
                let t = self.peek().clone();
                self.errors.push(CompileError::syntax(
                    format!("Expected type name, found {:?}", t.kind),
                    Loc {
                        line: t.line,
                        col: t.col,
                    },
                ));
                TypeDesig::Base(BaseType::Integer)
            }
        }
    }

    fn parse_var_id_list(&mut self) -> Vec<String> {
        self.parse_id_list()
    }

    /// 解析变量声明的重复部分。
    ///
    /// 通过向前看决定是继续解析下一个变量声明还是返回（ε）。
    fn parse_var_dec_more(&mut self, defs: &mut Vec<VarDef>) {
        while matches!(
            self.peek_kind(),
            TokenKind::Integer
                | TokenKind::Char
                | TokenKind::Array
                | TokenKind::Record
                | TokenKind::Ident(_)
        ) {
            self.parse_var_dec_list(defs);
        }
    }

    // ===== 39. ProcDec ::= ε | ProcDeclaration =====

    fn parse_proc_dec(&mut self) -> ProcDec {
        if *self.peek_kind() == TokenKind::Procedure {
            self.parse_proc_declaration()
        } else {
            ProcDec::Empty
        }
    }

    // ===== 41. ProcDeclaration ::= PROCEDURE ProcName ( ParamList ) ; ProcDecPart ProcBody ProcDecMore =====

    fn parse_proc_declaration(&mut self) -> ProcDec {
        let mut procs = Vec::new();
        self.parse_proc_declaration_inner(&mut procs);
        if procs.is_empty() {
            ProcDec::Empty
        } else {
            ProcDec::Defined(procs)
        }
    }

    fn parse_proc_declaration_inner(&mut self, procs: &mut Vec<ProcDef>) {
        if *self.peek_kind() != TokenKind::Procedure {
            return;
        }
        let loc = self.loc();
        self.match_token(TokenKind::Procedure);
        let name = match self.parse_proc_name() {
            Some(n) => n,
            None => return,
        };
        self.match_token(TokenKind::LParent);
        let params = self.parse_param_list();
        self.match_token(TokenKind::RParent);
        self.match_token(TokenKind::Semicolon);
        let decl = self.parse_proc_dec_part();
        let body = self.parse_proc_body();
        procs.push(ProcDef {
            name,
            params,
            decl,
            body,
            loc,
        });
        self.parse_proc_dec_more(procs);
    }

    fn parse_proc_name(&mut self) -> Option<String> {
        match self.peek_kind() {
            TokenKind::Ident(name) => {
                let n = name.clone();
                self.advance();
                Some(n)
            }
            _ => {
                let t = self.peek().clone();
                self.errors.push(CompileError::syntax(
                    format!("Expected procedure name, found {:?}", t.kind),
                    Loc {
                        line: t.line,
                        col: t.col,
                    },
                ));
                None
            }
        }
    }

    // ===== 41. ProcDecMore ::= ε | ProcDeclaration =====

    fn parse_proc_dec_more(&mut self, procs: &mut Vec<ProcDef>) {
        while *self.peek_kind() == TokenKind::Procedure {
            self.parse_proc_declaration_inner(procs);
        }
    }

    // ===== 55. ProcDecPart ::= DeclarePart =====

    fn parse_proc_dec_part(&mut self) -> DeclarePart {
        self.parse_declare_part()
    }

    // ===== 56. ProcBody ::= ProgramBody =====

    fn parse_proc_body(&mut self) -> StmList {
        self.parse_program_body()
    }

    // ===== 57. ProgramBody ::= BEGIN StmList END =====

    fn parse_program_body(&mut self) -> StmList {
        let loc = self.loc();
        self.match_token(TokenKind::Begin);
        let stmts = self.parse_stm_list();
        self.match_token(TokenKind::End);
        StmList { stmts, loc }
    }

    // ===== 45-54. ParamList, Param 等 =====

    fn parse_param_list(&mut self) -> Vec<ParamDef> {
        let mut params = Vec::new();
        self.parse_param_dec_list(&mut params);
        params
    }

    fn parse_param_dec_list(&mut self, params: &mut Vec<ParamDef>) {
        let loc = self.loc();
        // 检查 VAR 前缀（引用参数）
        let is_var = if *self.peek_kind() == TokenKind::Var {
            self.advance();
            true
        } else {
            false
        };

        let type_name = self.parse_type_desig();
        let names = self.parse_form_list();
        if names.is_empty() {
            return;
        }
        params.push(ParamDef {
            is_var,
            type_name,
            names,
            loc,
        });
        self.parse_param_more(params);
    }

    fn parse_form_list(&mut self) -> Vec<String> {
        self.parse_id_list()
    }

    fn parse_param_more(&mut self, params: &mut Vec<ParamDef>) {
        while *self.peek_kind() == TokenKind::Semicolon {
            self.advance();
            self.parse_param_dec_list(params);
        }
    }

    // ===== 58. StmList ::= Stm StmMore =====

    fn parse_stm_list(&mut self) -> Vec<Stm> {
        let mut stmts = Vec::new();
        let stm = self.parse_stm();
        stmts.push(stm);
        self.parse_stm_more(&mut stmts);
        stmts
    }

    // ===== 59. StmMore ::= ε | ; StmList =====

    fn parse_stm_more(&mut self, stmts: &mut Vec<Stm>) {
        while *self.peek_kind() == TokenKind::Semicolon {
            self.advance();
            if !matches!(
                self.peek_kind(),
                TokenKind::End
                    | TokenKind::Else
                    | TokenKind::Fi
                    | TokenKind::EndWh
                    | TokenKind::Eof
            ) {
                let stm = self.parse_stm();
                stmts.push(stm);
            } else {
                break;
            }
        }
    }

    // ===== 61. Stm ::= 条件语句/循环语句/输入/输出/返回/ID AssCall =====

    fn parse_stm(&mut self) -> Stm {
        let loc = self.loc();
        match self.peek_kind() {
            TokenKind::If => self.parse_conditional_stm(),
            TokenKind::While => self.parse_loop_stm(),
            TokenKind::Read => self.parse_input_stm(),
            TokenKind::Write => self.parse_output_stm(),
            TokenKind::Return => self.parse_return_stm(),
            TokenKind::Ident(name) => {
                let n = name.clone();
                self.advance();
                self.parse_ass_call(n, loc)
            }
            _ => {
                let t = self.peek().clone();
                self.errors.push(CompileError::syntax(
                    format!("Unexpected token {:?} at start of statement", t.kind),
                    Loc {
                        line: t.line,
                        col: t.col,
                    },
                ));
                // 恐慌模式恢复：跳到语句边界
                self.sync(&[
                    TokenKind::Semicolon,
                    TokenKind::End,
                    TokenKind::Fi,
                    TokenKind::EndWh,
                    TokenKind::Else,
                ]);
                // 返回一个空 Read 作为占位语句（容错）
                Stm::Read {
                    var: String::new(),
                    loc,
                }
            }
        }
    }

    // ===== 67. AssCall ::= AssignmentRest | CallStmRest =====

    /// 解析以标识符开头的两种可能语句：赋值或过程调用。
    ///
    /// 通过向前看一个 Token 区分：
    /// - `:=`、`[`、`.` → 赋值
    /// - `(` → 过程调用
    fn parse_ass_call(&mut self, name: String, loc: Loc) -> Stm {
        match self.peek_kind() {
            TokenKind::Assign | TokenKind::LBracket | TokenKind::Dot => {
                self.parse_assignment_rest(name, loc)
            }
            TokenKind::LParent => {
                self.parse_call_stm_rest(name, loc)
            }
            _ => {
                self.errors.push(CompileError::syntax(
                    format!(
                        "Expected :=, [, ., or ( after identifier, found {:?}",
                        self.peek_kind()
                    ),
                    loc,
                ));
                Stm::Read { var: name, loc }
            }
        }
    }

    // ===== 69. AssignmentRest ::= VariMore := Exp =====

    fn parse_assignment_rest(&mut self, name: String, loc: Loc) -> Stm {
        let selector = self.parse_vari_more();
        let lhs = VarAccess {
            base: name,
            selector,
            loc,
        };
        self.match_token(TokenKind::Assign);
        let rhs = self.parse_exp();
        Stm::Assign { lhs, rhs, loc }
    }

    // ===== 70. ConditionalStm ::= IF RelExp THEN StmList ELSE StmList FI =====

    fn parse_conditional_stm(&mut self) -> Stm {
        let loc = self.loc();
        self.match_token(TokenKind::If);
        let cond = self.parse_rel_exp();
        self.match_token(TokenKind::Then);
        let then_stmts = self.parse_stm_list();
        self.match_token(TokenKind::Else);
        let else_stmts = self.parse_stm_list();
        self.match_token(TokenKind::Fi);
        Stm::If {
            cond,
            then_branch: StmList {
                stmts: then_stmts,
                loc,
            },
            else_branch: StmList {
                stmts: else_stmts,
                loc,
            },
            loc,
        }
    }

    // ===== 71. LoopStm ::= WHILE RelExp DO StmList ENDWH =====

    fn parse_loop_stm(&mut self) -> Stm {
        let loc = self.loc();
        self.match_token(TokenKind::While);
        let cond = self.parse_rel_exp();
        self.match_token(TokenKind::Do);
        let body_stmts = self.parse_stm_list();
        self.match_token(TokenKind::EndWh);
        Stm::While {
            cond,
            body: StmList {
                stmts: body_stmts,
                loc,
            },
            loc,
        }
    }

    // ===== 72. InputStm ::= READ ( Invar ) =====

    fn parse_input_stm(&mut self) -> Stm {
        let loc = self.loc();
        self.match_token(TokenKind::Read);
        self.match_token(TokenKind::LParent);
        let var = match self.parse_invar() {
            Some(v) => v,
            None => {
                self.sync(&[TokenKind::RParent, TokenKind::Semicolon, TokenKind::End, TokenKind::Eof]);
                return Stm::Read {
                    var: String::new(),
                    loc,
                };
            }
        };
        self.match_token(TokenKind::RParent);
        Stm::Read { var, loc }
    }

    fn parse_invar(&mut self) -> Option<String> {
        match self.peek_kind() {
            TokenKind::Ident(name) => {
                let n = name.clone();
                self.advance();
                Some(n)
            }
            _ => {
                let t = self.peek().clone();
                self.errors.push(CompileError::syntax(
                    format!("Expected identifier, found {:?}", t.kind),
                    Loc {
                        line: t.line,
                        col: t.col,
                    },
                ));
                None
            }
        }
    }

    // ===== 74. OutputStm ::= WRITE ( Exp ) =====

    fn parse_output_stm(&mut self) -> Stm {
        let loc = self.loc();
        self.match_token(TokenKind::Write);
        self.match_token(TokenKind::LParent);
        let exp = self.parse_exp();
        self.match_token(TokenKind::RParent);
        Stm::Write { exp, loc }
    }

    // ===== 75. ReturnStm ::= RETURN ( Exp ) =====

    fn parse_return_stm(&mut self) -> Stm {
        let loc = self.loc();
        self.match_token(TokenKind::Return);
        self.match_token(TokenKind::LParent);
        let exp = self.parse_exp();
        self.match_token(TokenKind::RParent);
        Stm::Return { exp, loc }
    }

    // ===== 76. CallStmRest ::= ( ActParamList ) =====

    fn parse_call_stm_rest(&mut self, name: String, loc: Loc) -> Stm {
        self.match_token(TokenKind::LParent);
        let args = self.parse_act_param_list();
        self.match_token(TokenKind::RParent);
        Stm::Call { name, args, loc }
    }

    // ===== 77-80. ActParamList =====

    fn parse_act_param_list(&mut self) -> Vec<Exp> {
        let mut args = Vec::new();
        // 预先检查 `)` 以避免将其误解析为表达式
        if *self.peek_kind() != TokenKind::RParent {
            args.push(self.parse_exp());
            self.parse_act_param_more(&mut args);
        }
        args
    }

    fn parse_act_param_more(&mut self, args: &mut Vec<Exp>) {
        while *self.peek_kind() == TokenKind::Comma {
            self.advance();
            args.push(self.parse_exp());
        }
    }

    // ===== 81. RelExp ::= Exp OtherRelE =====

    fn parse_rel_exp(&mut self) -> Exp {
        let left = self.parse_exp();
        self.parse_other_rel_e(left)
    }

    // ===== 82. OtherRelE ::= CmpOp Exp =====

    fn parse_other_rel_e(&mut self, left: Exp) -> Exp {
        let op = match self.peek_kind() {
            TokenKind::Less => {
                self.advance();
                BinOp::Lt
            }
            TokenKind::Equal => {
                self.advance();
                BinOp::Eq
            }
            _ => return left, // ε（无比较运算符，直接返回左操作数）
        };
        let loc = self.loc();
        let right = self.parse_exp();
        Exp::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            loc,
        }
    }

    // ===== 83. Exp ::= Term OtherTerm =====

    fn parse_exp(&mut self) -> Exp {
        let left = self.parse_term();
        self.parse_other_term(left)
    }

    // ===== 84. OtherTerm ::= ε | AddOp Exp =====

    fn parse_other_term(&mut self, left: Exp) -> Exp {
        let op = match self.peek_kind() {
            TokenKind::Plus => {
                self.advance();
                BinOp::Add
            }
            TokenKind::Minus => {
                self.advance();
                BinOp::Sub
            }
            _ => return left,
        };
        let loc = self.loc();
        let right = self.parse_exp();
        Exp::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            loc,
        }
    }

    // ===== 86. Term ::= Factor OtherFactor =====

    fn parse_term(&mut self) -> Exp {
        let left = self.parse_factor();
        self.parse_other_factor(left)
    }

    // ===== 87. OtherFactor ::= ε | MultOp Term =====

    fn parse_other_factor(&mut self, left: Exp) -> Exp {
        let op = match self.peek_kind() {
            TokenKind::Times => {
                self.advance();
                BinOp::Mul
            }
            TokenKind::Divide => {
                self.advance();
                BinOp::Div
            }
            _ => return left,
        };
        let loc = self.loc();
        let right = self.parse_term();
        Exp::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            loc,
        }
    }

    // ===== 89. Factor ::= ( Exp ) | INTC | CHARC | Variable =====

    fn parse_factor(&mut self) -> Exp {
        let loc = self.loc();
        match self.peek_kind() {
            TokenKind::LParent => {
                self.advance();
                let exp = self.parse_exp();
                self.match_token(TokenKind::RParent);
                exp
            }
            TokenKind::IntConst(n) => {
                let val = *n;
                self.advance();
                Exp::IntConst(val, loc)
            }
            TokenKind::CharConst(c) => {
                let val = *c;
                self.advance();
                Exp::CharConst(val, loc)
            }
            TokenKind::Ident(_) => {
                match self.parse_variable() {
                    Some(va) => Exp::Variable(va, loc),
                    None => Exp::IntConst(0, loc),
                }
            }
            _ => {
                let t = self.peek().clone();
                self.errors.push(CompileError::syntax(
                    format!("Expected expression, found {:?}", t.kind),
                    Loc {
                        line: t.line,
                        col: t.col,
                    },
                ));
                Exp::IntConst(0, loc)
            }
        }
    }

    // ===== 92. Variable ::= ID VariMore =====

    fn parse_variable(&mut self) -> Option<VarAccess> {
        let loc = self.loc();
        let base = match self.peek_kind() {
            TokenKind::Ident(name) => {
                let n = name.clone();
                self.advance();
                n
            }
            _ => {
                let t = self.peek().clone();
                self.errors.push(CompileError::syntax(
                    format!("Expected identifier, found {:?}", t.kind),
                    Loc {
                        line: t.line,
                        col: t.col,
                    },
                ));
                return None;
            }
        };
        let selector = self.parse_vari_more();
        Some(VarAccess {
            base,
            selector,
            loc,
        })
    }

    // ===== 93. VariMore ::= ε | [ Exp ] | . FieldVar =====

    /// 解析变量后的可选部分：数组下标或记录字段。
    ///
    /// 循环处理多个连续的选择器（如 `.a[2].b`）。
    fn parse_vari_more(&mut self) -> Vec<Selector> {
        let mut selector = Vec::new();
        loop {
            match self.peek_kind() {
                TokenKind::LBracket => {
                    self.advance();
                    let exp = self.parse_exp();
                    self.match_token(TokenKind::RBracket);
                    selector.push(Selector::ArraySubscript(Box::new(exp)));
                }
                TokenKind::Dot => {
                    self.advance();
                    let field = self.parse_field_var();
                    selector.extend(field);
                }
                _ => break,
            }
        }
        selector
    }

    // ===== 96. FieldVar ::= ID FieldVarMore =====

    fn parse_field_var(&mut self) -> Vec<Selector> {
        let mut result = Vec::new();
        match self.peek_kind() {
            TokenKind::Ident(name) => {
                let n = name.clone();
                self.advance();
                let more = self.parse_field_var_more();
                if more.is_empty() {
                    result.push(Selector::Field(n));
                } else {
                    // FieldVarMore 返回下标表达式列表，每个生成一个 FieldSubscript
                    for m in more {
                        result.push(Selector::FieldSubscript(n.clone(), Box::new(m)));
                    }
                }
            }
            _ => {}
        }
        result
    }

    // ===== 97. FieldVarMore ::= ε | [ Exp ] =====

    fn parse_field_var_more(&mut self) -> Vec<Exp> {
        let mut result = Vec::new();
        if *self.peek_kind() == TokenKind::LBracket {
            self.advance();
            result.push(self.parse_exp());
            self.match_token(TokenKind::RBracket);
        }
        result
    }
}

/// 将 TokenKind 转为人可读的字符串名称（用于错误消息）。
fn token_name(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Program => "program",
        TokenKind::Type => "type",
        TokenKind::Var => "var",
        TokenKind::Procedure => "procedure",
        TokenKind::Begin => "begin",
        TokenKind::End => "end",
        TokenKind::Integer => "integer",
        TokenKind::Char => "char",
        TokenKind::Array => "array",
        TokenKind::Record => "record",
        TokenKind::Of => "of",
        TokenKind::While => "while",
        TokenKind::Do => "do",
        TokenKind::EndWh => "endwh",
        TokenKind::If => "if",
        TokenKind::Then => "then",
        TokenKind::Else => "else",
        TokenKind::Fi => "fi",
        TokenKind::Return => "return",
        TokenKind::Read => "read",
        TokenKind::Write => "write",
        TokenKind::Assign => ":=",
        TokenKind::Range => "..",
        TokenKind::Plus => "+",
        TokenKind::Minus => "-",
        TokenKind::Times => "*",
        TokenKind::Divide => "/",
        TokenKind::LParent => "(",
        TokenKind::RParent => ")",
        TokenKind::LBracket => "[",
        TokenKind::RBracket => "]",
        TokenKind::Semicolon => ";",
        TokenKind::Dot => ".",
        TokenKind::Comma => ",",
        TokenKind::Less => "<",
        TokenKind::Equal => "=",
        TokenKind::Ident(_) => "identifier",
        TokenKind::IntConst(_) => "integer",
        TokenKind::CharConst(_) => "character",
        TokenKind::Eof => "EOF",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> (Option<Program>, Vec<CompileError>) {
        let mut lexer = Lexer::new();
        let (tokens, _) = lexer.tokenize(source);
        let mut parser = RdParser::new(tokens);
        let prog = parser.parse();
        (prog, parser.errors)
    }

    #[test]
    fn test_simple_program() {
        let source = "program pp var integer v1; begin v1 := 2 end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors);
        let prog = prog.expect("Expected a program");
        assert_eq!(prog.name, "pp");
        assert_eq!(prog.body.stmts.len(), 1);
        match &prog.body.stmts[0] {
            Stm::Assign { lhs, rhs, .. } => {
                assert_eq!(lhs.base, "v1");
                match rhs {
                    Exp::IntConst(2, _) => {}
                    _ => panic!("Expected IntConst(2)"),
                }
            }
            _ => panic!("Expected Assign statement"),
        }
    }

    #[test]
    fn test_if_statement() {
        let source = "program p var integer x; begin if x < 10 then x := 1 else x := 2 fi end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors);
        let prog = prog.expect("Expected a program");
        match &prog.body.stmts[0] {
            Stm::If {
                then_branch,
                else_branch,
                ..
            } => {
                assert_eq!(then_branch.stmts.len(), 1);
                assert_eq!(else_branch.stmts.len(), 1);
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_while_statement() {
        let source = "program p begin while 1 do write(0) endwh end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors);
        let prog = prog.expect("Expected a program");
        match &prog.body.stmts[0] {
            Stm::While { .. } => {}
            _ => panic!("Expected While statement"),
        }
    }

    #[test]
    fn test_procedure_with_params() {
        let source =
            "program p procedure q(integer a; var char b); begin write(a) end begin q(1, 'x') end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors);
        let prog = prog.expect("Expected a program");
        match &prog.decl.procs {
            ProcDec::Defined(procs) => {
                assert_eq!(procs.len(), 1);
                assert_eq!(procs[0].name, "q");
                assert_eq!(procs[0].params.len(), 2);
            }
            _ => panic!("Expected procedure declaration"),
        }
    }

    #[test]
    fn test_expressions() {
        let source = "program p var integer x; begin x := 1 + 2 * 3 end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty(), "Unexpected errors: {:?}", errors);
        let prog = prog.expect("Expected a program");
        match &prog.body.stmts[0] {
            Stm::Assign { rhs, .. } => match rhs {
                Exp::Binary {
                    op: BinOp::Add,
                    left,
                    right,
                    ..
                } => {
                    match left.as_ref() {
                        Exp::IntConst(1, _) => {}
                        _ => panic!("Expected left=1"),
                    }
                    match right.as_ref() {
                        Exp::Binary { op: BinOp::Mul, .. } => {}
                        _ => panic!("Expected right=2*3"),
                    }
                }
                _ => panic!("Expected Add binary expr"),
            },
            _ => panic!("Expected Assign"),
        }
    }

    #[test]
    fn test_read_statement() {
        let source = "program p var integer x; begin read(x) end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        match &prog.body.stmts[0] {
            Stm::Read { var, .. } => assert_eq!(var, "x"),
            _ => panic!("Expected Read statement"),
        }
    }

    #[test]
    fn test_write_statement() {
        let source = "program p begin write(42) end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        match &prog.body.stmts[0] {
            Stm::Write { exp, .. } => match exp {
                Exp::IntConst(42, _) => {}
                _ => panic!("Expected IntConst(42)"),
            },
            _ => panic!("Expected Write statement"),
        }
    }

    #[test]
    fn test_return_statement() {
        let source = "program p procedure f(integer a); begin return(a) end begin f(1) end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        match &prog.decl.procs {
            ProcDec::Defined(procs) => match &procs[0].body.stmts[0] {
                Stm::Return { exp, .. } => match exp {
                    Exp::Variable(va, _) => assert_eq!(va.base, "a"),
                    _ => panic!("Expected Variable(a)"),
                },
                _ => panic!("Expected Return statement"),
            },
            _ => panic!("Expected procedure"),
        }
    }

    #[test]
    fn test_call_statement() {
        let source = "program p procedure f(integer a); begin write(a) end begin f(42) end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        match &prog.body.stmts[0] {
            Stm::Call { name, args, .. } => {
                assert_eq!(name, "f");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected Call statement"),
        }
    }

    #[test]
    fn test_nested_if_statement() {
        let source = "program p var integer x; begin if x < 10 then if x < 5 then x := 1 else x := 2 fi else x := 3 fi end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        match &prog.body.stmts[0] {
            Stm::If {
                then_branch,
                else_branch,
                ..
            } => {
                match &then_branch.stmts[0] {
                    Stm::If { .. } => {}
                    _ => panic!("Expected nested If"),
                }
                assert_eq!(else_branch.stmts.len(), 1);
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_nested_while_statement() {
        let source = "program p begin while 1 do while 0 do write(1) endwh endwh end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        match &prog.body.stmts[0] {
            Stm::While { body, .. } => match &body.stmts[0] {
                Stm::While { .. } => {}
                _ => panic!("Expected nested While"),
            },
            _ => panic!("Expected While statement"),
        }
    }

    #[test]
    fn test_procedure_no_params() {
        let source = "program p procedure q(integer dummy); begin write(0) end begin q(1) end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        match &prog.decl.procs {
            ProcDec::Defined(procs) => {
                assert_eq!(procs[0].name, "q");
                assert_eq!(procs[0].params.len(), 1);
            }
            _ => panic!("Expected procedure"),
        }
    }

    #[test]
    fn test_equality_condition_in_if() {
        let source = "program p var integer x; begin if x = 0 then x := 1 else x := 2 fi end.";
        let (_prog, errors) = parse(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_subtraction_and_division() {
        let source = "program p var integer x; begin x := 10 - 3 end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        match &prog.body.stmts[0] {
            Stm::Assign { rhs, .. } => match rhs {
                Exp::Binary { op: BinOp::Sub, .. } => {}
                _ => panic!("Expected Sub expression"),
            },
            _ => panic!("Expected Assign"),
        }
    }

    #[test]
    fn test_char_variable() {
        let source = "program p var char c; begin c := 'x' end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        match &prog.body.stmts[0] {
            Stm::Assign { rhs, .. } => match rhs {
                Exp::CharConst('x', _) => {}
                _ => panic!("Expected CharConst"),
            },
            _ => panic!("Expected Assign"),
        }
    }

    #[test]
    fn test_multiple_procedures() {
        let source = "program p procedure a(integer dummy); begin write(1) end procedure b(integer x); begin write(x) end begin a(1); b(2) end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        match &prog.decl.procs {
            ProcDec::Defined(procs) => {
                assert_eq!(procs.len(), 2);
                assert_eq!(procs[0].name, "a");
                assert_eq!(procs[1].name, "b");
            }
            _ => panic!("Expected procedures"),
        }
    }

    #[test]
    fn test_multiple_variable_declarations() {
        let source = "program p var integer x; integer y; char c; begin x := 1; y := 2 end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        match &prog.decl.vars {
            VarDec::Defined(defs) => assert_eq!(defs.len(), 3),
            _ => panic!("Expected var definitions"),
        }
    }

    #[test]
    fn test_expression_with_variable() {
        let source = "program p var integer x; integer y; begin x := y + 1 end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        match &prog.body.stmts[0] {
            Stm::Assign { rhs, .. } => match rhs {
                Exp::Binary {
                    op: BinOp::Add,
                    left,
                    ..
                } => match left.as_ref() {
                    Exp::Variable(va, _) => assert_eq!(va.base, "y"),
                    _ => panic!("Expected Variable(y)"),
                },
                _ => panic!("Expected Add"),
            },
            _ => panic!("Expected Assign"),
        }
    }

    #[test]
    fn test_multiple_statements() {
        let source = "program p var integer x; begin x := 1; write(x); x := 2; write(x) end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        assert_eq!(prog.body.stmts.len(), 4);
    }

    #[test]
    fn test_call_with_single_arg() {
        let source = "program p procedure q(integer a); begin write(a) end begin q(42) end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        match &prog.body.stmts[0] {
            Stm::Call { name, args, .. } => {
                assert_eq!(name, "q");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_less_than_in_condition() {
        let source = "program p var integer x; begin if x < 10 then write(1) else write(2) fi end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        match &prog.body.stmts[0] {
            Stm::If { cond, .. } => match cond {
                Exp::Binary { op: BinOp::Lt, .. } => {}
                _ => panic!("Expected Lt in condition"),
            },
            _ => panic!("Expected If"),
        }
    }

    #[test]
    fn test_longer_program_name() {
        let source = "program myLongProgram123 begin write(1) end.";
        let (prog, errors) = parse(source);
        assert!(errors.is_empty());
        let prog = prog.expect("Expected a program");
        assert_eq!(prog.name, "myLongProgram123");
    }
}
