//! SNL 文法编码。
//!
//! 将 SNL 语言的 EBNF 文法表示为产生式集合。
//! 该文法被设计为 LL(1)，可直接用于构建预测分析表。
//!
//! ## 设计要点
//! - 原始文法中的 EBNF 重复结构（`{...}`）已通过引入辅助非终结符
//!   （如 `TypeDecMore`、`VarDecMore`）展开为右递归
//! - 原始文法中的可选结构（`[...]`）已通过 ε 产生式处理
//! - Token 字面量变体使用占位值（`String::new()`、`0`、`'\0'`），
//!   因为解析表匹配时只关心 Token 种类而不关心具体值

use crate::lexer::token::TokenKind;

/// 非终结符枚举。
///
/// 命名与 SNL 标准文法保持一致。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NonTerm {
    NProgram,
    DeclarePart,
    TypeDec,
    TypeDecList,
    TypeDecMore,
    TypeName,
    BaseType,
    FieldDecList,
    FieldDecMore,
    IdList,
    IdMore,
    VarDec,
    VarDecList,
    VarDecMore,
    VarIdList,
    VarIdMore,
    ProcDec,
    ProcDecMore,
    ParamList,
    ParamDecList,
    ParamMore,
    Param,
    FormList,
    FidMore,
    ProgramBody,
    StmList,
    StmMore,
    Stm,
    AssignmentRest,
    ActParamList,
    ActParamMore,
    RelExp,
    OtherRelE,
    Exp,
    OtherTerm,
    Term,
    OtherFactor,
    Factor,
    Variable,
    VariMore,
    FieldVar,
    FieldVarMore,
    AssCall,
    CallStmRest,
}

/// 文法符号：终结符或非终结符。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GrammarSymbol {
    T(TokenKind),
    N(NonTerm),
}

/// 一条产生式：左部 + 右部符号序列。
pub struct Production {
    pub lhs: NonTerm,
    pub rhs: Vec<GrammarSymbol>,
}

/// 上下文无关文法：产生式集合 + 起始符号。
pub struct Grammar {
    pub productions: Vec<Production>,
    pub start: NonTerm,
}

/// 构建 SNL 编码文法。
///
/// 返回的 `Grammar` 可用于 FIRST/FOLLOW 计算和 LL(1) 表构建。
pub fn encode_grammar() -> Grammar {
    use GrammarSymbol::*;
    use NonTerm::*;
    use TokenKind as TK;

    // 生成占位 Token（具体值在 LL(1) 解析时通过 normalize 忽略）
    let ident = TK::Ident(String::new());
    let intc = TK::IntConst(0);
    let charc = TK::CharConst('\0');

    #[rustfmt::skip]
    let prods = vec![
        // Program ::= PROGRAM ID DeclarePart ProgramBody .
        prod(NProgram, vec![T(TK::Program), T(ident.clone()), N(DeclarePart), N(ProgramBody), T(TK::Dot)]),

        // DeclarePart ::= TypeDec VarDec ProcDec
        prod(DeclarePart, vec![N(TypeDec), N(VarDec), N(ProcDec)]),

        // ---- TypeDec ----
        prod(TypeDec, vec![]),                                          // ε
        prod(TypeDec, vec![T(TK::Type), N(TypeDecList)]),              // TYPE TypeDecList

        // ---- TypeDecList ----
        prod(TypeDecList, vec![T(ident.clone()), T(TK::Equal), N(TypeName), T(TK::Semicolon), N(TypeDecMore)]),

        // ---- TypeDecMore ----
        prod(TypeDecMore, vec![]),                                      // ε
        prod(TypeDecMore, vec![T(ident.clone()), T(TK::Equal), N(TypeName), T(TK::Semicolon), N(TypeDecMore)]),

        // ---- TypeName ----
        prod(TypeName, vec![N(BaseType)]),
        prod(TypeName, vec![T(TK::Array), T(TK::LBracket), T(intc.clone()), T(TK::Range), T(intc.clone()), T(TK::RBracket), T(TK::Of), N(BaseType)]),
        prod(TypeName, vec![T(TK::Record), N(FieldDecList), T(TK::End)]),
        prod(TypeName, vec![T(ident.clone())]),

        // ---- BaseType ----
        prod(BaseType, vec![T(TK::Integer)]),
        prod(BaseType, vec![T(TK::Char)]),

        // ---- FieldDecList ----
        prod(FieldDecList, vec![N(BaseType), N(IdList), T(TK::Semicolon), N(FieldDecMore)]),
        prod(FieldDecList, vec![T(TK::Array), T(TK::LBracket), T(intc.clone()), T(TK::Range), T(intc.clone()), T(TK::RBracket), T(TK::Of), N(BaseType), N(IdList), T(TK::Semicolon), N(FieldDecMore)]),

        // ---- FieldDecMore ----
        prod(FieldDecMore, vec![]),                                    // ε
        prod(FieldDecMore, vec![N(BaseType), N(IdList), T(TK::Semicolon), N(FieldDecMore)]),
        prod(FieldDecMore, vec![T(TK::Array), T(TK::LBracket), T(intc.clone()), T(TK::Range), T(intc.clone()), T(TK::RBracket), T(TK::Of), N(BaseType), N(IdList), T(TK::Semicolon), N(FieldDecMore)]),

        // ---- IdList ----
        prod(IdList, vec![T(ident.clone()), N(IdMore)]),

        // ---- IdMore ----
        prod(IdMore, vec![]),                                          // ε
        prod(IdMore, vec![T(TK::Comma), T(ident.clone()), N(IdMore)]),

        // ---- VarDec ----
        prod(VarDec, vec![]),                                          // ε
        prod(VarDec, vec![T(TK::Var), N(VarDecList)]),

        // ---- VarDecList ----
        prod(VarDecList, vec![N(TypeName), N(VarIdList), T(TK::Semicolon), N(VarDecMore)]),

        // ---- VarDecMore ----
        prod(VarDecMore, vec![]),                                      // ε
        prod(VarDecMore, vec![N(TypeName), N(VarIdList), T(TK::Semicolon), N(VarDecMore)]),

        // ---- VarIdList ----
        prod(VarIdList, vec![T(ident.clone()), N(VarIdMore)]),

        // ---- VarIdMore ----
        prod(VarIdMore, vec![]),                                       // ε
        prod(VarIdMore, vec![T(TK::Comma), T(ident.clone()), N(VarIdMore)]),

        // ---- ProcDec ----
        prod(ProcDec, vec![]),                                         // ε
        prod(ProcDec, vec![T(TK::Procedure), T(ident.clone()), T(TK::LParent), N(ParamList), T(TK::RParent), T(TK::Semicolon), N(DeclarePart), N(ProgramBody), N(ProcDecMore)]),

        // ---- ProcDecMore ----
        prod(ProcDecMore, vec![]),                                     // ε
        prod(ProcDecMore, vec![T(TK::Procedure), T(ident.clone()), T(TK::LParent), N(ParamList), T(TK::RParent), T(TK::Semicolon), N(DeclarePart), N(ProgramBody), N(ProcDecMore)]),

        // ---- ParamList ----
        prod(ParamList, vec![]),                                       // ε
        prod(ParamList, vec![N(ParamDecList)]),

        // ---- ParamDecList ----
        prod(ParamDecList, vec![N(Param), N(ParamMore)]),

        // ---- ParamMore ----
        prod(ParamMore, vec![]),                                       // ε
        prod(ParamMore, vec![T(TK::Semicolon), N(ParamDecList)]),

        // ---- Param ----
        prod(Param, vec![N(TypeName), N(FormList)]),
        prod(Param, vec![T(TK::Var), N(TypeName), N(FormList)]),

        // ---- FormList ----
        prod(FormList, vec![T(ident.clone()), N(FidMore)]),

        // ---- FidMore ----
        prod(FidMore, vec![]),                                         // ε
        prod(FidMore, vec![T(TK::Comma), T(ident.clone()), N(FidMore)]),

        // ---- ProgramBody ----
        prod(ProgramBody, vec![T(TK::Begin), N(StmList), T(TK::End)]),

        // ---- StmList ----
        prod(StmList, vec![N(Stm), N(StmMore)]),

        // ---- StmMore ----
        prod(StmMore, vec![]),                                         // ε
        prod(StmMore, vec![T(TK::Semicolon), N(Stm), N(StmMore)]),

        // ---- Stm ----
        // IF RelExp THEN StmList ELSE StmList FI
        prod(Stm, vec![T(TK::If), N(RelExp), T(TK::Then), N(StmList), T(TK::Else), N(StmList), T(TK::Fi)]),
        // WHILE RelExp DO StmList ENDWH
        prod(Stm, vec![T(TK::While), N(RelExp), T(TK::Do), N(StmList), T(TK::EndWh)]),
        // READ ( ID )
        prod(Stm, vec![T(TK::Read), T(TK::LParent), T(ident.clone()), T(TK::RParent)]),
        // WRITE ( Exp )
        prod(Stm, vec![T(TK::Write), T(TK::LParent), N(Exp), T(TK::RParent)]),
        // RETURN ( Exp )
        prod(Stm, vec![T(TK::Return), T(TK::LParent), N(Exp), T(TK::RParent)]),
        // ID AssCall
        prod(Stm, vec![T(ident.clone()), N(AssCall)]),

        // ---- AssCall ::= AssignmentRest | CallStmRest ----
        prod(AssCall, vec![N(AssignmentRest)]),
        prod(AssCall, vec![N(CallStmRest)]),

        // ---- AssignmentRest ::= VariMore := Exp ----
        prod(AssignmentRest, vec![N(VariMore), T(TK::Assign), N(Exp)]),

        // ---- CallStmRest ::= ( ActParamList ) ----
        prod(CallStmRest, vec![T(TK::LParent), N(ActParamList), T(TK::RParent)]),

        // ---- ActParamList ----
        prod(ActParamList, vec![]),                                    // ε
        prod(ActParamList, vec![N(Exp), N(ActParamMore)]),

        // ---- ActParamMore ----
        prod(ActParamMore, vec![]),                                    // ε
        prod(ActParamMore, vec![T(TK::Comma), N(Exp), N(ActParamMore)]),

        // ---- RelExp ----
        prod(RelExp, vec![N(Exp), N(OtherRelE)]),

        // ---- OtherRelE ----
        prod(OtherRelE, vec![]),                                       // ε
        prod(OtherRelE, vec![T(TK::Less), N(Exp)]),
        prod(OtherRelE, vec![T(TK::Equal), N(Exp)]),

        // ---- Exp ----
        prod(Exp, vec![N(Term), N(OtherTerm)]),

        // ---- OtherTerm ----
        prod(OtherTerm, vec![]),                                       // ε
        prod(OtherTerm, vec![T(TK::Plus), N(Exp)]),
        prod(OtherTerm, vec![T(TK::Minus), N(Exp)]),

        // ---- Term ----
        prod(Term, vec![N(Factor), N(OtherFactor)]),

        // ---- OtherFactor ----
        prod(OtherFactor, vec![]),                                     // ε
        prod(OtherFactor, vec![T(TK::Times), N(Term)]),
        prod(OtherFactor, vec![T(TK::Divide), N(Term)]),

        // ---- Factor ----
        prod(Factor, vec![T(TK::LParent), N(Exp), T(TK::RParent)]),
        prod(Factor, vec![T(intc.clone())]),
        prod(Factor, vec![T(charc.clone())]),
        prod(Factor, vec![N(Variable)]),

        // ---- Variable ----
        prod(Variable, vec![T(ident.clone()), N(VariMore)]),

        // ---- VariMore ----
        prod(VariMore, vec![]),                                        // ε
        prod(VariMore, vec![T(TK::LBracket), N(Exp), T(TK::RBracket)]),
        prod(VariMore, vec![T(TK::Dot), N(FieldVar)]),

        // ---- FieldVar ----
        prod(FieldVar, vec![T(ident.clone()), N(FieldVarMore)]),

        // ---- FieldVarMore ----
        prod(FieldVarMore, vec![]),                                    // ε
        prod(FieldVarMore, vec![T(TK::LBracket), N(Exp), T(TK::RBracket)]),
    ];

    Grammar {
        productions: prods,
        start: NProgram,
    }
}

/// 辅助函数：构造一条产生式。
fn prod(lhs: NonTerm, rhs: Vec<GrammarSymbol>) -> Production {
    Production { lhs, rhs }
}
