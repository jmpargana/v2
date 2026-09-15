use crate::lex::SymbolKind;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Gt,
    Equal,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Sub => write!(f, "-"),
            Op::Mul => write!(f, "*"),
            Op::Div => write!(f, "/"),
            Op::Lt => write!(f, "<"),
            Op::Gt => write!(f, ">"),
            Op::Equal => write!(f, "=="),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    NumberLit(i64),
    Ident(String),
    Bool {
        op: Op,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Binary {
        op: Op,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        func: String,
        args: Vec<Expr>,
    },
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
            Expr::Bool { op, left, right } => write!(f, "({} {} {})", left, op, right),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    VarDecl {
        kind: SymbolKind,
        name: String,
        value: Expr,
    },
    Cond {
        condition: Expr,
        body: Vec<Stmt>,
        alternate: Option<Vec<Stmt>>,
    },
    FuncDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
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
            Stmt::Cond {
                condition,
                body,
                alternate,
            } => {
                write!(f, "if ({}) {{", condition)?;
                for s in body {
                    write!(f, " {}", s)?;
                }
                write!(f, " }}")?;
                if alternate.is_some() {
                    write!(f, "else {{")?;
                    for s in alternate.as_ref().unwrap() {
                        write!(f, " {}", s)?;
                    }
                    write!(f, " }}")?;
                }
                Ok(())
            }
        }
    }
}
