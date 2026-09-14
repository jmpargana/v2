use std::fmt;
use crate::lex::SymbolKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Add,
    Mul,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Mul => write!(f, "*"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    NumberLit(i64),
    Ident(String),
    Binary { op: Op, left: Box<Expr>, right: Box<Expr> },
    Call { func: String, args: Vec<Expr> },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::NumberLit(v) => write!(f, "{}", v),
            Expr::Ident(name) => write!(f, "{}", name),
            Expr::Binary { op, left, right } => write!(f, "({} {} {})", left, op, right),
            Expr::Call { func, args } => {
                let args_str: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                write!(f, "{}({})", func, args_str.join(", "))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    VarDecl { kind: SymbolKind, name: String, value: Expr },
    FuncDecl { name: String, params: Vec<String>, body: Vec<Stmt> },
    Return(Expr),
    ExprStmt(Expr),
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::VarDecl { kind, name, value } => write!(f, "{} {} = {};", kind, name, value),
            Stmt::FuncDecl { name, params, body } => {
                write!(f, "function {}({}) {{", name, params.join(", "))?;
                for s in body {
                    write!(f, " {}", s)?;
                }
                write!(f, " }}")
            }
            Stmt::Return(value) => write!(f, "return {};", value),
            Stmt::ExprStmt(expr) => write!(f, "{};", expr),
        }
    }
}
