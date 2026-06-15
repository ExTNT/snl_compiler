//! 符号表数据结构。
//!
//! SNL 使用嵌套作用域：全局作用域 + 每个过程一个局部作用域。
//! 符号表以**哈希映射栈**的形式组织，查找时从栈顶（最内层作用域）向栈底遍历。
//!
//! ## 设计
//! - 每个作用域是一个 `HashMap<String, SymbolEntry>`
//! - 插入/查询仅作用于当前最内层作用域
//! - `lookup()` 从内向外搜索，实现作用域遮蔽
//! - Display 实现用于生成 `*_table.md` 调试文件

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

use crate::ast::nodes::Loc;

/// 标识符种类。
#[derive(Debug, Clone, PartialEq)]
pub enum IdKind {
    /// 类型名
    TypeId,
    /// 变量名
    VarId,
    /// 过程名
    ProcId,
}

/// 类型信息（用于语义分析阶段的类型检查）。
///
/// `Named` 变体表示尚未解析的类型别名引用，
/// 在后续分析中通过查表解析。
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    Integer,
    Char,
    /// 数组类型：元素类型、下界、上界
    Array(Box<TypeInfo>, i64, i64),
    /// 记录类型：字段列表
    Record(Vec<FieldInfo>),
    /// 未解析的类型别名引用
    Named(String),
}

/// 记录字段信息。
#[derive(Debug, Clone, PartialEq)]
pub struct FieldInfo {
    pub name: String,
    pub typ: TypeInfo,
}

/// 过程形参信息。
#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    /// 是否为 var 参数（引用传递）
    pub is_var: bool,
    pub typ: TypeInfo,
}

/// 符号表条目。
///
/// 每个条目记录一个程序实体的完整信息：
/// 名称、种类、类型、形参（仅过程）、作用域层级和源位置。
#[derive(Debug, Clone)]
pub struct SymbolEntry {
    pub name: String,
    pub kind: IdKind,
    pub typ: Option<TypeInfo>,
    pub params: Vec<ParamInfo>,
    /// 作用域嵌套层级（0 = 全局）
    pub level: usize,
    pub loc: Loc,
}

/// 符号表：嵌套作用域的哈希映射栈。
///
/// ## 使用方式
/// - `enter_scope()` / `exit_scope()` 进入/退出过程作用域
/// - `insert()` 向当前作用域添加条目
/// - `lookup()` 从内向外搜索
/// - `scopes()` 返回所有作用域的引用（用于快照导出）
pub struct SymbolTable {
    scopes: Vec<HashMap<String, SymbolEntry>>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    /// 创建空的符号表（含全局作用域）。
    pub fn new() -> Self {
        SymbolTable {
            scopes: vec![HashMap::new()],
        }
    }

    /// 返回当前嵌套层级（0 = 全局）。
    pub fn current_level(&self) -> usize {
        self.scopes.len() - 1
    }

    /// 进入新的嵌套作用域（例如进入一个过程体）。
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// 退出当前嵌套作用域。
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// 向当前最内层作用域插入符号。
    ///
    /// 若当前作用域已存在同名符号则返回错误。
    pub fn insert(&mut self, entry: SymbolEntry) -> Result<(), String> {
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(&entry.name) {
            Err(format!("Duplicate identifier '{}'", entry.name))
        } else {
            scope.insert(entry.name.clone(), entry);
            Ok(())
        }
    }

    /// 从内向外查找符号。
    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        for scope in self.scopes.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return Some(entry);
            }
        }
        None
    }

    /// 仅在当前最内层作用域中查找（不向外搜索）。
    pub fn lookup_current(&self, name: &str) -> Option<&SymbolEntry> {
        self.scopes.last().unwrap().get(name)
    }

    /// 返回所有作用域的引用切片。
    pub fn scopes(&self) -> &[HashMap<String, SymbolEntry>] {
        &self.scopes
    }
}

// ===== Display 实现 =====

impl Display for IdKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            IdKind::TypeId => write!(f, "TypeId"),
            IdKind::VarId => write!(f, "VarId"),
            IdKind::ProcId => write!(f, "ProcId"),
        }
    }
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TypeInfo::Integer => write!(f, "integer"),
            TypeInfo::Char => write!(f, "char"),
            TypeInfo::Array(elem, low, high) => write!(f, "array[{},{}] of {}", low, high, elem),
            TypeInfo::Record(fields) => {
                write!(f, "record(")?;
                for (i, fi) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", fi.name, fi.typ)?;
                }
                write!(f, ")")
            }
            TypeInfo::Named(name) => write!(f, "{}", name),
        }
    }
}

impl Display for SymbolEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} | {} | level={}", self.name, self.kind, self.level)?;
        if let Some(ty) = &self.typ {
            write!(f, " | {}", ty)?;
        }
        if !self.params.is_empty() {
            write!(f, " | params:(")?;
            for (i, p) in self.params.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                if p.is_var {
                    write!(f, "var ")?;
                }
                write!(f, "{}: {}", p.name, p.typ)?;
            }
            write!(f, ")")?;
        }
        write!(f, " | line {}", self.loc.line)
    }
}

impl Display for SymbolTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "| Scope | Level | Name | Kind | Type | Params | Line |")?;
        writeln!(f, "|-------|-------|------|------|------|--------|------|")?;
        for (i, scope) in self.scopes.iter().enumerate() {
            let scope_label = if i == 0 {
                "Global".to_string()
            } else {
                format!("Proc #{}", i)
            };
            let mut entries: Vec<&SymbolEntry> = scope.values().collect();
            entries.sort_by(|a, b| a.name.cmp(&b.name));
            for entry in entries {
                write!(
                    f,
                    "| {} | {} | {} | {} |",
                    scope_label, entry.level, entry.name, entry.kind
                )?;
                if let Some(ty) = &entry.typ {
                    write!(f, " {}", ty)?;
                }
                write!(f, " |")?;
                if !entry.params.is_empty() {
                    write!(f, " (")?;
                    for (j, p) in entry.params.iter().enumerate() {
                        if j > 0 {
                            write!(f, ", ")?;
                        }
                        if p.is_var {
                            write!(f, "var ")?;
                        }
                        write!(f, "{}: {}", p.name, p.typ)?;
                    }
                    write!(f, ")")?;
                }
                writeln!(f, " | {} |", entry.loc.line)?;
            }
        }
        Ok(())
    }
}
