#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Loc {
    pub line: usize,
    pub col: usize,
}

// ===== TOP LEVEL =====

#[derive(Debug, Clone)]
pub struct Program {
    pub name: String,
    pub decl: DeclarePart,
    pub body: StmList,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub struct DeclarePart {
    pub types: TypeDec,
    pub vars: VarDec,
    pub procs: ProcDec,
}

// ===== TYPE DECLARATIONS =====

#[derive(Debug, Clone)]
pub enum TypeDec {
    Empty,
    Defined(Vec<TypeDef>),
}

#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: String,
    pub body: TypeBody,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub enum TypeBody {
    Base(BaseType),
    Array(ArrayTypeDef),
    Record(RecordTypeDef),
    Named(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseType {
    Integer,
    Char,
}

#[derive(Debug, Clone)]
pub struct ArrayTypeDef {
    pub low: i64,
    pub high: i64,
    pub elem_type: BaseType,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub struct RecordTypeDef {
    pub fields: Vec<FieldDef>,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub typ: FieldTypeDef,
    pub names: Vec<String>,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub enum FieldTypeDef {
    Base(BaseType),
    Array(ArrayTypeDef),
}

// ===== VARIABLE DECLARATIONS =====

#[derive(Debug, Clone)]
pub enum VarDec {
    Empty,
    Defined(Vec<VarDef>),
}

#[derive(Debug, Clone)]
pub struct VarDef {
    pub type_name: TypeDesig,
    pub names: Vec<String>,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub enum TypeDesig {
    Base(BaseType),
    Array(ArrayTypeDef),
    Record(RecordTypeDef),
    Named(String),
}

// ===== PROCEDURE DECLARATIONS =====

#[derive(Debug, Clone)]
pub enum ProcDec {
    Empty,
    Defined(Vec<ProcDef>),
}

#[derive(Debug, Clone)]
pub struct ProcDef {
    pub name: String,
    pub params: Vec<ParamDef>,
    pub decl: DeclarePart,
    pub body: StmList,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub struct ParamDef {
    pub is_var: bool,
    pub type_name: TypeDesig,
    pub names: Vec<String>,
    pub loc: Loc,
}

// ===== STATEMENTS =====

#[derive(Debug, Clone)]
pub struct StmList {
    pub stmts: Vec<Stm>,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub enum Stm {
    Assign {
        lhs: VarAccess,
        rhs: Exp,
        loc: Loc,
    },
    If {
        cond: Exp,
        then_branch: StmList,
        else_branch: StmList,
        loc: Loc,
    },
    While {
        cond: Exp,
        body: StmList,
        loc: Loc,
    },
    Read {
        var: String,
        loc: Loc,
    },
    Write {
        exp: Exp,
        loc: Loc,
    },
    Return {
        exp: Exp,
        loc: Loc,
    },
    Call {
        name: String,
        args: Vec<Exp>,
        loc: Loc,
    },
}

// ===== EXPRESSIONS =====

#[derive(Debug, Clone)]
pub enum Exp {
    Binary {
        op: BinOp,
        left: Box<Exp>,
        right: Box<Exp>,
        loc: Loc,
    },
    IntConst(i64, Loc),
    CharConst(char, Loc),
    Variable(VarAccess, Loc),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Eq,
}

// ===== VARIABLE ACCESS =====

#[derive(Debug, Clone)]
pub struct VarAccess {
    pub base: String,
    pub selector: Vec<Selector>,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub enum Selector {
    ArraySubscript(Box<Exp>),
    Field(String),
    FieldSubscript(String, Box<Exp>),
}
