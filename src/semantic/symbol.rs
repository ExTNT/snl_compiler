use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

use crate::ast::nodes::Loc;

#[derive(Debug, Clone, PartialEq)]
pub enum IdKind {
    TypeId,
    VarId,
    ProcId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    Integer,
    Char,
    Array(Box<TypeInfo>, i64, i64), // element type, low, high
    Record(Vec<FieldInfo>),
    Named(String), // unresolved reference — resolved during analysis
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldInfo {
    pub name: String,
    pub typ: TypeInfo,
}

#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    pub is_var: bool,
    pub typ: TypeInfo,
}

#[derive(Debug, Clone)]
pub struct SymbolEntry {
    pub name: String,
    pub kind: IdKind,
    pub typ: Option<TypeInfo>,
    pub params: Vec<ParamInfo>,
    pub level: usize,
    pub loc: Loc,
}

pub struct SymbolTable {
    scopes: Vec<HashMap<String, SymbolEntry>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn current_level(&self) -> usize {
        self.scopes.len() - 1
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn insert(&mut self, entry: SymbolEntry) -> Result<(), String> {
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(&entry.name) {
            Err(format!("Duplicate identifier '{}'", entry.name))
        } else {
            scope.insert(entry.name.clone(), entry);
            Ok(())
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        for scope in self.scopes.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return Some(entry);
            }
        }
        None
    }

    pub fn lookup_current(&self, name: &str) -> Option<&SymbolEntry> {
        self.scopes.last().unwrap().get(name)
    }

    pub fn scopes(&self) -> &[HashMap<String, SymbolEntry>] {
        &self.scopes
    }
}

// ===== Display implementations =====

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
