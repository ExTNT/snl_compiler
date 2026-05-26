//! AST 节点的格式化显示（Display trait 实现）。
//!
//! 将 AST 以树形文本格式输出，使用 Unicode 框线字符（├── └── │）
//! 直观展示程序的语法结构。主要用于生成 `*_tree.md` 调试文件。

use std::fmt::{self, Display, Formatter};

use super::nodes::*;

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "ProK")?;
        writeln!(f, "├── PheadK  {}", self.name)?;
        self.decl.fmt_node(f, "│   ")?;
        self.body.fmt_node(f, "│   ")?;
        writeln!(f, "└── .")?;
        Ok(())
    }
}

// ===== DeclarePart =====

impl DeclarePart {
    /// 格式化声明部分子树。
    ///
    /// # 参数
    /// - `prefix`: 当前行的缩进前缀（用于树形对齐）
    pub fn fmt_node(&self, f: &mut Formatter<'_>, prefix: &str) -> fmt::Result {
        match &self.types {
            TypeDec::Empty => {}
            TypeDec::Defined(defs) => {
                write!(f, "{}├── TypeK\n", prefix)?;
                let child_prefix = format!("{}│   ", prefix);
                for def in defs {
                    write!(f, "{}├── DecK  ", child_prefix)?;
                    def.body.fmt_type_body(f)?;
                    writeln!(f, "  {}", def.name)?;
                }
            }
        }
        match &self.vars {
            VarDec::Empty => {}
            VarDec::Defined(defs) => {
                write!(f, "{}├── VarK\n", prefix)?;
                let child_prefix = format!("{}│   ", prefix);
                for def in defs {
                    write!(f, "{}├── DecK  ", child_prefix)?;
                    def.type_name.fmt_type_body(f)?;
                    for name in &def.names {
                        write!(f, "  {}", name)?;
                    }
                    writeln!(f)?;
                }
            }
        }
        match &self.procs {
            ProcDec::Empty => {}
            ProcDec::Defined(procs) => {
                for proc in procs {
                    writeln!(f, "{}├── ProcDecK  {}", prefix, proc.name)?;
                    let child_prefix = format!("{}│   ", prefix);
                    proc.fmt_inner(f, &child_prefix)?;
                }
            }
        }
        Ok(())
    }
}

impl ProcDef {
    /// 格式化过程定义的内部内容（形参、局部声明、过程体）。
    fn fmt_inner(&self, f: &mut Formatter<'_>, prefix: &str) -> fmt::Result {
        if !self.params.is_empty() {
            for param in &self.params {
                write!(f, "{}├── DecK  ", prefix)?;
                if param.is_var {
                    write!(f, "var param:  ")?;
                } else {
                    write!(f, "value param:  ")?;
                }
                param.type_name.fmt_type_body(f)?;
                for name in &param.names {
                    write!(f, "  {}", name)?;
                }
                writeln!(f)?;
            }
        }
        self.decl.fmt_node(f, prefix)?;
        self.body.fmt_node(f, prefix)?;
        Ok(())
    }
}

// ===== StmList =====

impl StmList {
    pub fn fmt_node(&self, f: &mut Formatter<'_>, prefix: &str) -> fmt::Result {
        writeln!(f, "{}├── StmLk", prefix)?;
        let child_prefix = format!("{}│   ", prefix);
        for stm in &self.stmts {
            stm.fmt_node(f, &child_prefix)?;
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.stmts.is_empty()
    }
}

// ===== Stm =====

impl Stm {
    pub fn fmt_node(&self, f: &mut Formatter<'_>, prefix: &str) -> fmt::Result {
        match self {
            Stm::Assign { lhs, rhs, loc: _ } => {
                writeln!(f, "{}├── StmtK  Assign", prefix)?;
                let cp = format!("{}│   ", prefix);
                write!(f, "{}├── ExpK  ", cp)?;
                lhs.fmt_lhs(f)?;
                writeln!(f)?;
                write!(f, "{}└── ExpK  ", cp)?;
                rhs.fmt_node(f)?;
            }
            Stm::If {
                cond,
                then_branch,
                else_branch,
                loc: _,
            } => {
                writeln!(f, "{}├── StmtK  If", prefix)?;
                let cp = format!("{}│   ", prefix);
                write!(f, "{}├── ExpK  ", cp)?;
                cond.fmt_node(f)?;
                then_branch.fmt_node(f, &cp)?;
                else_branch.fmt_node(f, &cp)?;
            }
            Stm::While { cond, body, loc: _ } => {
                writeln!(f, "{}├── StmtK  While", prefix)?;
                let cp = format!("{}│   ", prefix);
                write!(f, "{}├── ExpK  ", cp)?;
                cond.fmt_node(f)?;
                body.fmt_node(f, &cp)?;
            }
            Stm::Read { var, loc: _ } => {
                writeln!(f, "{}├── StmtK  Read  {}", prefix, var)?;
            }
            Stm::Write { exp, loc: _ } => {
                writeln!(f, "{}├── StmtK  Write", prefix)?;
                let cp = format!("{}│   ", prefix);
                write!(f, "{}└── ExpK  ", cp)?;
                exp.fmt_node(f)?;
            }
            Stm::Return { exp, loc: _ } => {
                writeln!(f, "{}├── StmtK  Return", prefix)?;
                let cp = format!("{}│   ", prefix);
                write!(f, "{}└── ExpK  ", cp)?;
                exp.fmt_node(f)?;
            }
            Stm::Call { name, args, loc: _ } => {
                writeln!(f, "{}├── StmtK  Call", prefix)?;
                let cp = format!("{}│   ", prefix);
                if args.is_empty() {
                    writeln!(f, "{}└── ExpK  {}  IdV", cp, name)?;
                } else {
                    for arg in args {
                        write!(f, "{}├── ExpK  ", cp)?;
                        arg.fmt_node(f)?;
                    }
                }
            }
        }
        Ok(())
    }
}

// ===== Exp =====

impl Exp {
    pub fn fmt_node(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Exp::Binary {
                op,
                left: _,
                right: _,
                loc: _,
            } => {
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Lt => "<",
                    BinOp::Eq => "=",
                };
                writeln!(f, "Op  {}", op_str)
            }
            Exp::IntConst(val, _) => {
                writeln!(f, "Const  {}", val)
            }
            Exp::CharConst(c, _) => {
                writeln!(f, "Const  '{}'", c)
            }
            Exp::Variable(va, _) => {
                write!(f, "{}  IdV", va.base)?;
                for sel in &va.selector {
                    match sel {
                        Selector::ArraySubscript(exp) => match exp.as_ref() {
                            Exp::IntConst(n, _) => write!(f, "[{}]", n)?,
                            Exp::Variable(v, _) => write!(f, "[{}]", v.base)?,
                            _ => write!(f, "[...]")?,
                        },
                        Selector::Field(name) => write!(f, ".{}", name)?,
                        Selector::FieldSubscript(name, exp) => {
                            write!(f, ".{}", name)?;
                            match exp.as_ref() {
                                Exp::IntConst(n, _) => write!(f, "[{}]", n)?,
                                Exp::Variable(v, _) => write!(f, "[{}]", v.base)?,
                                _ => write!(f, "[...]")?,
                            }
                        }
                    }
                }
                writeln!(f)
            }
        }
    }
}

// ===== TypeBody / TypeDesig 显示 =====

impl TypeBody {
    fn fmt_type_body(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TypeBody::Base(BaseType::Integer) => write!(f, "IntegerK"),
            TypeBody::Base(BaseType::Char) => write!(f, "CharK"),
            TypeBody::Array(arr) => {
                write!(f, "ArrayK  [{},{}]  ", arr.low, arr.high)?;
                match arr.elem_type {
                    BaseType::Integer => write!(f, "IntegerK"),
                    BaseType::Char => write!(f, "CharK"),
                }
            }
            TypeBody::Record(rec) => {
                write!(f, "RecordK")?;
                for field in &rec.fields {
                    write!(f, "  ")?;
                    fmt_field_type_def(&field.typ, f)?;
                    for name in &field.names {
                        write!(f, "  {}", name)?;
                    }
                }
                Ok(())
            }
            TypeBody::Named(name) => write!(f, "{}", name),
        }
    }
}

impl TypeDesig {
    fn fmt_type_body(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TypeDesig::Base(BaseType::Integer) => write!(f, "IntegerK"),
            TypeDesig::Base(BaseType::Char) => write!(f, "CharK"),
            TypeDesig::Array(arr) => {
                write!(f, "ArrayK  [{},{}]  ", arr.low, arr.high)?;
                match arr.elem_type {
                    BaseType::Integer => write!(f, "IntegerK"),
                    BaseType::Char => write!(f, "CharK"),
                }
            }
            TypeDesig::Record(rec) => {
                write!(f, "RecordK")?;
                for field in &rec.fields {
                    write!(f, "  ")?;
                    fmt_field_type_def(&field.typ, f)?;
                    for name in &field.names {
                        write!(f, "  {}", name)?;
                    }
                }
                Ok(())
            }
            TypeDesig::Named(name) => write!(f, "{}", name),
        }
    }
}

fn fmt_field_type_def(ftd: &FieldTypeDef, f: &mut Formatter<'_>) -> fmt::Result {
    match ftd {
        FieldTypeDef::Base(BaseType::Integer) => write!(f, "IntegerK"),
        FieldTypeDef::Base(BaseType::Char) => write!(f, "CharK"),
        FieldTypeDef::Array(arr) => {
            write!(f, "ArrayK  [{},{}]  ", arr.low, arr.high)?;
            match arr.elem_type {
                BaseType::Integer => write!(f, "IntegerK"),
                BaseType::Char => write!(f, "CharK"),
            }
        }
    }
}

// ===== VarAccess 左侧显示 =====

impl VarAccess {
    /// 格式化赋值语句左侧的变量访问（含选择器链）。
    fn fmt_lhs(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}  IdV", self.base)?;
        for sel in &self.selector {
            match sel {
                Selector::ArraySubscript(exp) => match exp.as_ref() {
                    Exp::IntConst(n, _) => write!(f, "[{}]", n)?,
                    Exp::Variable(v, _) => write!(f, "[{}]", v.base)?,
                    _ => write!(f, "[...]")?,
                },
                Selector::Field(name) => write!(f, ".{}", name)?,
                Selector::FieldSubscript(name, exp) => {
                    write!(f, ".{}", name)?;
                    match exp.as_ref() {
                        Exp::IntConst(n, _) => write!(f, "[{}]", n)?,
                        Exp::Variable(v, _) => write!(f, "[{}]", v.base)?,
                        _ => write!(f, "[...]")?,
                    }
                }
            }
        }
        Ok(())
    }
}
