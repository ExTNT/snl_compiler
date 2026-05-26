//! 抽象语法树节点类型定义。
//!
//! ## 结构层次
//! - **顶层**: Program → DeclarePart → TypeDec/VarDec/ProcDec
//! - **语句**: StmList → Stm（赋值、条件、循环、读写、返回、调用）
//! - **表达式**: Exp → Binary / IntConst / CharConst / Variable
//! - **变量访问**: VarAccess → 基础变量 + 选择器链（数组下标、记录字段）
//!
//! 所有节点携带 `Loc` 信息用于错误定位。

/// 源码位置。
///
/// 行号和列号均从 1 开始计数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Loc {
    pub line: usize,
    pub col: usize,
}

// ===== 顶层结构 =====

/// 完整程序。
#[derive(Debug, Clone)]
pub struct Program {
    /// 程序名称（program 关键字后的标识符）
    pub name: String,
    /// 声明部分（类型、变量、过程）
    pub decl: DeclarePart,
    /// 程序主体语句序列
    pub body: StmList,
    /// 程序起始位置
    pub loc: Loc,
}

/// 声明部分，包含类型、变量和过程的声明。
#[derive(Debug, Clone)]
pub struct DeclarePart {
    pub types: TypeDec,
    pub vars: VarDec,
    pub procs: ProcDec,
}

// ===== 类型声明 =====

/// 类型声明：可为空或包含一组类型定义。
#[derive(Debug, Clone)]
pub enum TypeDec {
    Empty,
    Defined(Vec<TypeDef>),
}

/// 单个类型定义。
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: String,
    pub body: TypeBody,
    pub loc: Loc,
}

/// 类型体：基础类型、数组、记录或类型别名引用。
#[derive(Debug, Clone)]
pub enum TypeBody {
    Base(BaseType),
    Array(ArrayTypeDef),
    Record(RecordTypeDef),
    /// 引用已声明的类型别名
    Named(String),
}

/// 基础类型（整数或字符）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseType {
    Integer,
    Char,
}

/// 数组类型定义。
#[derive(Debug, Clone)]
pub struct ArrayTypeDef {
    /// 下标下界
    pub low: i64,
    /// 下标上界
    pub high: i64,
    /// 元素类型
    pub elem_type: BaseType,
    pub loc: Loc,
}

/// 记录类型定义。
#[derive(Debug, Clone)]
pub struct RecordTypeDef {
    /// 记录字段列表
    pub fields: Vec<FieldDef>,
    pub loc: Loc,
}

/// 记录字段定义（一个字段类型可关联多个字段名）。
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub typ: FieldTypeDef,
    pub names: Vec<String>,
    pub loc: Loc,
}

/// 记录字段类型（基础类型或数组，不能是记录或别名）。
#[derive(Debug, Clone)]
pub enum FieldTypeDef {
    Base(BaseType),
    Array(ArrayTypeDef),
}

// ===== 变量声明 =====

/// 变量声明。
#[derive(Debug, Clone)]
pub enum VarDec {
    Empty,
    Defined(Vec<VarDef>),
}

/// 单个变量定义。
#[derive(Debug, Clone)]
pub struct VarDef {
    pub type_name: TypeDesig,
    pub names: Vec<String>,
    pub loc: Loc,
}

/// 变量类型描述符。
#[derive(Debug, Clone)]
pub enum TypeDesig {
    Base(BaseType),
    Array(ArrayTypeDef),
    Record(RecordTypeDef),
    Named(String),
}

// ===== 过程声明 =====

/// 过程声明。
#[derive(Debug, Clone)]
pub enum ProcDec {
    Empty,
    Defined(Vec<ProcDef>),
}

/// 单个过程定义。
#[derive(Debug, Clone)]
pub struct ProcDef {
    /// 过程名称
    pub name: String,
    /// 形参列表
    pub params: Vec<ParamDef>,
    /// 过程内部的局部声明
    pub decl: DeclarePart,
    /// 过程体
    pub body: StmList,
    pub loc: Loc,
}

/// 形参定义。
#[derive(Debug, Clone)]
pub struct ParamDef {
    /// 是否为 var 参数（引用传递）
    pub is_var: bool,
    /// 参数类型
    pub type_name: TypeDesig,
    /// 参数名列表（同一类型的多个参数）
    pub names: Vec<String>,
    pub loc: Loc,
}

// ===== 语句 =====

/// 语句列表（一个复合语句）。
#[derive(Debug, Clone)]
pub struct StmList {
    pub stmts: Vec<Stm>,
    pub loc: Loc,
}

/// 语句。
#[derive(Debug, Clone)]
pub enum Stm {
    /// 赋值语句
    Assign {
        lhs: VarAccess,
        rhs: Exp,
        loc: Loc,
    },
    /// 条件语句（If-Then-Else-Fi）
    If {
        cond: Exp,
        then_branch: StmList,
        else_branch: StmList,
        loc: Loc,
    },
    /// 循环语句（While-Do-EndWh）
    While {
        cond: Exp,
        body: StmList,
        loc: Loc,
    },
    /// 读取语句（Read）
    Read {
        var: String,
        loc: Loc,
    },
    /// 写入语句（Write）
    Write {
        exp: Exp,
        loc: Loc,
    },
    /// 返回语句（Return）
    Return {
        exp: Exp,
        loc: Loc,
    },
    /// 过程调用
    Call {
        name: String,
        args: Vec<Exp>,
        loc: Loc,
    },
}

// ===== 表达式 =====

/// 表达式。
#[derive(Debug, Clone)]
pub enum Exp {
    /// 二元运算
    Binary {
        op: BinOp,
        left: Box<Exp>,
        right: Box<Exp>,
        loc: Loc,
    },
    /// 整数常量
    IntConst(i64, Loc),
    /// 字符常量
    CharConst(char, Loc),
    /// 变量引用
    Variable(VarAccess, Loc),
}

/// 二元运算符。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    /// `<` 小于比较
    Lt,
    /// `=` 相等比较（SNL 使用 `=` 而非 `==`）
    Eq,
}

// ===== 变量访问 =====

/// 变量访问表达式。
///
/// 由基础变量名和一系列选择器组成，
/// 选择器可以嵌套组合（如 `a[1].b[2]`）。
#[derive(Debug, Clone)]
pub struct VarAccess {
    /// 基础变量名称
    pub base: String,
    /// 选择器链（数组下标、记录字段）
    pub selector: Vec<Selector>,
    pub loc: Loc,
}

/// 变量选择器。
#[derive(Debug, Clone)]
pub enum Selector {
    /// 数组下标访问（`[exp]`）
    ArraySubscript(Box<Exp>),
    /// 记录字段访问（`.name`）
    Field(String),
    /// 记录字段 + 数组下标（`.name[exp]`）
    FieldSubscript(String, Box<Exp>),
}
