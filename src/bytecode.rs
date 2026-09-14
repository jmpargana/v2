use std::collections::HashMap;
use std::fmt::{self, Write};
use crate::ast::{Expr, Stmt, Op};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ByteCode {
    Push = 0,
    Add = 1,
    Mul = 2,
    LdaSmi = 3,
    Ldar = 4,
    Star = 5,
    Call = 6,
    Return = 7,
}

impl From<u8> for ByteCode {
    fn from(val: u8) -> Self {
        match val {
            0 => ByteCode::Push,
            1 => ByteCode::Add,
            2 => ByteCode::Mul,
            3 => ByteCode::LdaSmi,
            4 => ByteCode::Ldar,
            5 => ByteCode::Star,
            6 => ByteCode::Call,
            7 => ByteCode::Return,
            _ => panic!("Unknown({})", val),
        }
    }
}

impl fmt::Display for ByteCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ByteCode::Push => write!(f, "Push"),
            ByteCode::Add => write!(f, "Add"),
            ByteCode::Mul => write!(f, "Mul"),
            ByteCode::LdaSmi => write!(f, "LdaSmi"),
            ByteCode::Ldar => write!(f, "Ldar"),
            ByteCode::Star => write!(f, "Star"),
            ByteCode::Call => write!(f, "Call"),
            ByteCode::Return => write!(f, "Return"),
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub code: Vec<u8>,
    pub cons: Vec<i64>,
    pub funcs: Vec<Program>,
    pub func_map: HashMap<String, usize>,
    pub param_count: usize,
    pub next_reg: usize,
    pub symbols: HashMap<String, usize>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            code: Vec::new(),
            cons: Vec::new(),
            funcs: Vec::new(),
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        }
    }

    fn alloc_reg(&mut self) -> usize {
        let val = self.next_reg;
        self.next_reg = (self.next_reg + 1) % 256;
        val
    }

    pub fn compile(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            match stmt {
                Stmt::VarDecl { name, value, .. } => {
                    self.compile_expr(value);
                    let reg = self.alloc_reg();
                    self.symbols.insert(name.clone(), reg);
                    self.code.push(ByteCode::Star as u8);
                    self.code.push(reg as u8);
                }
                Stmt::FuncDecl { name, params, body } => {
                    let idx = self.funcs.len();
                    self.funcs.push(Program::new());
                    self.func_map.insert(name.clone(), idx);

                    let mut child = Program::new();
                    child.param_count = params.len();
                    child.func_map = self.func_map.clone();

                    for param in params {
                        let reg = child.alloc_reg();
                        child.symbols.insert(param.clone(), reg);
                    }

                    child.compile(body);
                    self.funcs[idx] = child;
                }
                Stmt::Return(value) => {
                    self.compile_expr(value);
                    self.code.push(ByteCode::Return as u8);
                }
                Stmt::ExprStmt(expr) => {
                    self.compile_expr(expr);
                }
            }
        }
    }

    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::NumberLit(value) => {
                let idx = self.cons.len();
                self.cons.push(*value);
                self.code.push(ByteCode::LdaSmi as u8);
                self.code.push(idx as u8);
            }
            Expr::Ident(name) => {
                let reg = self.symbols[name];
                self.code.push(ByteCode::Ldar as u8);
                self.code.push(reg as u8);
            }
            Expr::Binary { op, left, right } => {
                self.compile_expr(left);
                let reg = self.alloc_reg();
                self.code.push(ByteCode::Star as u8);
                self.code.push(reg as u8);
                self.compile_expr(right);
                match op {
                    Op::Add => {
                        self.code.push(ByteCode::Add as u8);
                        self.code.push(reg as u8);
                    }
                    Op::Mul => {
                        self.code.push(ByteCode::Mul as u8);
                        self.code.push(reg as u8);
                    }
                }
            }
            Expr::Call { func, args } => {
                for arg in args {
                    self.compile_expr(arg);
                    let reg = self.alloc_reg();
                    self.code.push(ByteCode::Star as u8);
                    self.code.push(reg as u8);
                }
                self.code.push(ByteCode::Call as u8);
                self.code.push(self.func_map[func] as u8);
            }
        }
    }

    pub fn equals(&self, other: &Program) -> bool {
        self.code == other.code && self.cons == other.cons
    }

    fn string_indent(&self, indent: &str) -> String {
        let mut b = String::new();
        writeln!(b, "{}Constants: {:?}", indent, self.cons).unwrap();
        writeln!(b, "{}Registers: {}, Params: {}", indent, self.next_reg, self.param_count).unwrap();
        if !self.func_map.is_empty() {
            writeln!(b, "{}FuncMap: {:?}", indent, self.func_map).unwrap();
        }
        writeln!(b, "{}Bytecode:", indent).unwrap();

        let mut i = 0;
        while i < self.code.len() {
            let op = ByteCode::from(self.code[i]);
            match op {
                ByteCode::Return => {
                    writeln!(b, "{}  {:04}  {}", indent, i, op).unwrap();
                    i += 1;
                }
                _ => {
                    if i + 1 < self.code.len() {
                        let operand = self.code[i + 1] as usize;
                        match op {
                            ByteCode::LdaSmi | ByteCode::Push => {
                                if operand < self.cons.len() {
                                    writeln!(b, "{}  {:04}  {:<8} [{}] ({})", indent, i, op, operand, self.cons[operand]).unwrap();
                                } else {
                                    writeln!(b, "{}  {:04}  {:<8} [{}]", indent, i, op, operand).unwrap();
                                }
                            }
                            ByteCode::Star | ByteCode::Ldar | ByteCode::Add | ByteCode::Mul => {
                                writeln!(b, "{}  {:04}  {:<8} r{}", indent, i, op, operand).unwrap();
                            }
                            ByteCode::Call => {
                                let name = self.func_map.iter()
                                    .find(|&(_, &v)| v == operand)
                                    .map(|(k, _)| k.as_str());
                                if let Some(name) = name {
                                    writeln!(b, "{}  {:04}  {:<8} [{}] ({})", indent, i, op, operand, name).unwrap();
                                } else {
                                    writeln!(b, "{}  {:04}  {:<8} [{}]", indent, i, op, operand).unwrap();
                                }
                            }
                            _ => {
                                writeln!(b, "{}  {:04}  {:<8} {}", indent, i, op, operand).unwrap();
                            }
                        }
                        i += 2;
                    } else {
                        writeln!(b, "{}  {:04}  {}", indent, i, op).unwrap();
                        i += 1;
                    }
                }
            }
        }

        for (idx, func) in self.funcs.iter().enumerate() {
            let name = self.func_map.iter()
                .find(|&(_, &v)| v == idx)
                .map(|(k, _)| k.as_str());
            if let Some(name) = name {
                writeln!(b, "{}Func [{}] {:?}:", indent, idx, name).unwrap();
            } else {
                writeln!(b, "{}Func [{}]:", indent, idx).unwrap();
            }
            b.push_str(&func.string_indent(&format!("{}  ", indent)));
        }

        b
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string_indent(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::SymbolKind;
    use crate::ast::{Expr, Stmt, Op};

    #[test]
    fn example_from_lesson() {
        let stmts = vec![
            Stmt::VarDecl {
                kind: SymbolKind::Let,
                name: "x".to_string(),
                value: Expr::NumberLit(10),
            },
            Stmt::VarDecl {
                kind: SymbolKind::Let,
                name: "y".to_string(),
                value: Expr::Binary {
                    op: Op::Add,
                    left: Box::new(Expr::NumberLit(20)),
                    right: Box::new(Expr::Ident("x".to_string())),
                },
            },
            Stmt::ExprStmt(Expr::Binary {
                op: Op::Mul,
                left: Box::new(Expr::Ident("x".to_string())),
                right: Box::new(Expr::Ident("y".to_string())),
            }),
        ];

        let mut got = Program::new();
        got.compile(&stmts);

        let mut want = Program::new();
        want.cons = vec![10, 20];
        want.code = vec![
            ByteCode::LdaSmi as u8, 0,
            ByteCode::Star as u8, 0,
            ByteCode::LdaSmi as u8, 1,
            ByteCode::Star as u8, 1,
            ByteCode::Ldar as u8, 0,
            ByteCode::Add as u8, 1,
            ByteCode::Star as u8, 2,
            ByteCode::Ldar as u8, 0,
            ByteCode::Star as u8, 3,
            ByteCode::Ldar as u8, 2,
            ByteCode::Mul as u8, 3,
        ];

        assert!(got.equals(&want), "got:\n{}\nwant:\n{}", got, want);
    }

    #[test]
    fn add_caller_and_callee() {
        let stmts = vec![
            Stmt::FuncDecl {
                name: "add".to_string(),
                params: vec!["a".to_string(), "b".to_string()],
                body: vec![
                    Stmt::Return(Expr::Binary {
                        op: Op::Add,
                        left: Box::new(Expr::Ident("a".to_string())),
                        right: Box::new(Expr::Ident("b".to_string())),
                    }),
                ],
            },
            Stmt::ExprStmt(Expr::Call {
                func: "add".to_string(),
                args: vec![Expr::NumberLit(10), Expr::NumberLit(20)],
            }),
        ];

        let mut got = Program::new();
        got.compile(&stmts);

        let mut want = Program::new();
        want.cons = vec![10, 20];
        want.code = vec![
            ByteCode::LdaSmi as u8, 0,
            ByteCode::Star as u8, 0,
            ByteCode::LdaSmi as u8, 1,
            ByteCode::Star as u8, 1,
            ByteCode::Call as u8, 0,
        ];

        assert!(got.equals(&want), "got:\n{}\nwant:\n{}", got, want);
    }
}
