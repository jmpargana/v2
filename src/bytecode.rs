use crate::ast::{Expr, Op, Stmt};
use std::collections::HashMap;
use std::fmt::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ByteCode {
    Push = 0,
    Add = 1,
    Sub = 2,
    Mul = 3,
    Div = 4,
    LdaSmi = 5,
    Ldar = 6,
    Star = 7,
    Call = 8,
    Return = 9,
    TestEqual = 10,
    TestLess = 11,
    TestGreater = 12,
    Jump = 13,
    JumpIfFalse = 14,
    TestLessEqual = 15,
    TestGreaterEqual = 16,
    TestNotEqual = 17,
    LogicalAnd = 18,
    LogicalOr = 19,
    LogicalNot = 20,
}

impl From<u8> for ByteCode {
    fn from(val: u8) -> Self {
        match val {
            0 => ByteCode::Push,
            1 => ByteCode::Add,
            2 => ByteCode::Sub,
            3 => ByteCode::Mul,
            4 => ByteCode::Div,
            5 => ByteCode::LdaSmi,
            6 => ByteCode::Ldar,
            7 => ByteCode::Star,
            8 => ByteCode::Call,
            9 => ByteCode::Return,
            10 => ByteCode::TestEqual,
            11 => ByteCode::TestLess,
            12 => ByteCode::TestGreater,
            13 => ByteCode::Jump,
            14 => ByteCode::JumpIfFalse,
            15 => ByteCode::TestLessEqual,
            16 => ByteCode::TestGreaterEqual,
            17 => ByteCode::TestNotEqual,
            18 => ByteCode::LogicalAnd,
            19 => ByteCode::LogicalOr,
            20 => ByteCode::LogicalNot,
            _ => panic!("Unknown({})", val),
        }
    }
}

impl fmt::Display for ByteCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ByteCode::Push => write!(f, "Push"),
            ByteCode::Add => write!(f, "Add"),
            ByteCode::Sub => write!(f, "Sub"),
            ByteCode::Mul => write!(f, "Mul"),
            ByteCode::Div => write!(f, "Div"),
            ByteCode::LdaSmi => write!(f, "LdaSmi"),
            ByteCode::Ldar => write!(f, "Ldar"),
            ByteCode::Star => write!(f, "Star"),
            ByteCode::Call => write!(f, "Call"),
            ByteCode::Return => write!(f, "Return"),
            ByteCode::TestEqual => write!(f, "TestEqual"),
            ByteCode::TestLess => write!(f, "TestLess"),
            ByteCode::TestGreater => write!(f, "TestGreater"),
            ByteCode::Jump => write!(f, "Jump"),
            ByteCode::JumpIfFalse => write!(f, "JumpIfFalse"),
            ByteCode::TestLessEqual => write!(f, "TestLessEqual"),
            ByteCode::TestGreaterEqual => write!(f, "TestGreaterEqual"),
            ByteCode::TestNotEqual => write!(f, "TestNotEqual"),
            ByteCode::LogicalAnd => write!(f, "LogicalAnd"),
            ByteCode::LogicalOr => write!(f, "LogicalOr"),
            ByteCode::LogicalNot => write!(f, "LogicalNot"),
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
                Stmt::Cond {
                    condition,
                    body,
                    alternate,
                } => {
                    self.compile_expr(condition);
                    let jump_false = self.emit_jump(ByteCode::JumpIfFalse);
                    self.compile(body);

                    if alternate.is_some() {
                        let jump = self.emit_jump(ByteCode::Jump);
                        self.patch_jump(jump_false);
                        self.compile(alternate.as_ref().unwrap());
                        self.patch_jump(jump);
                    } else {
                        self.patch_jump(jump_false);
                    }
                }
            }
        }
    }

    fn patch_jump(&mut self, placeholder: usize) {
        let offset = self.code.len() - placeholder - 1;
        self.code[placeholder] = offset as u8;
    }

    fn emit_jump(&mut self, op: ByteCode) -> usize {
        self.code.push(op as u8);
        self.code.push(0);
        self.code.len() - 1
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
                    Op::Sub => {
                        self.code.push(ByteCode::Sub as u8);
                        self.code.push(reg as u8);
                    }
                    Op::Mul => {
                        self.code.push(ByteCode::Mul as u8);
                        self.code.push(reg as u8);
                    }
                    Op::Div => {
                        self.code.push(ByteCode::Div as u8);
                        self.code.push(reg as u8);
                    }
                    _ => {
                        panic!("unexpected op in Binary expr: {}", op);
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
            Expr::Not(operand) => {
                self.compile_expr(operand);
                self.code.push(ByteCode::LogicalNot as u8);
            }
            Expr::Bool { op, left, right } => {
                self.compile_expr(left);
                let reg = self.alloc_reg();
                self.code.push(ByteCode::Star as u8);
                self.code.push(reg as u8);
                self.compile_expr(right);
                let op_byte_code = match op {
                    Op::Lt => ByteCode::TestLess,
                    Op::Gt => ByteCode::TestGreater,
                    Op::Lte => ByteCode::TestLessEqual,
                    Op::Gte => ByteCode::TestGreaterEqual,
                    Op::Equal => ByteCode::TestEqual,
                    Op::NotEqual => ByteCode::TestNotEqual,
                    Op::And => ByteCode::LogicalAnd,
                    Op::Or => ByteCode::LogicalOr,
                    _ => panic!("unexpected op in Bool expr: {}", op),
                } as u8;
                self.code.push(op_byte_code);
                self.code.push(reg as u8);
            }
        }
    }

    pub fn equals(&self, other: &Program) -> bool {
        self.code == other.code && self.cons == other.cons
    }

    fn string_indent(&self, indent: &str) -> String {
        let mut b = String::new();
        writeln!(b, "{}Constants: {:?}", indent, self.cons).unwrap();
        writeln!(
            b,
            "{}Registers: {}, Params: {}",
            indent, self.next_reg, self.param_count
        )
        .unwrap();
        if !self.func_map.is_empty() {
            writeln!(b, "{}FuncMap: {:?}", indent, self.func_map).unwrap();
        }
        writeln!(b, "{}Bytecode:", indent).unwrap();

        let mut i = 0;
        while i < self.code.len() {
            let op = ByteCode::from(self.code[i]);
            match op {
                ByteCode::Return | ByteCode::LogicalNot => {
                    writeln!(b, "{}  {:04}  {}", indent, i, op).unwrap();
                    i += 1;
                }
                _ => {
                    if i + 1 < self.code.len() {
                        let operand = self.code[i + 1] as usize;
                        match op {
                            ByteCode::LdaSmi | ByteCode::Push => {
                                if operand < self.cons.len() {
                                    writeln!(
                                        b,
                                        "{}  {:04}  {:<8} [{}] ({})",
                                        indent, i, op, operand, self.cons[operand]
                                    )
                                    .unwrap();
                                } else {
                                    writeln!(b, "{}  {:04}  {:<8} [{}]", indent, i, op, operand)
                                        .unwrap();
                                }
                            }
                            ByteCode::Star | ByteCode::Ldar | ByteCode::Add | ByteCode::Sub | ByteCode::Mul | ByteCode::Div
                            | ByteCode::TestEqual | ByteCode::TestLess | ByteCode::TestGreater
                            | ByteCode::TestLessEqual | ByteCode::TestGreaterEqual | ByteCode::TestNotEqual
                            | ByteCode::LogicalAnd | ByteCode::LogicalOr => {
                                writeln!(b, "{}  {:04}  {:<8} r{}", indent, i, op, operand)
                                    .unwrap();
                            }
                            ByteCode::Call => {
                                let name = self
                                    .func_map
                                    .iter()
                                    .find(|&(_, &v)| v == operand)
                                    .map(|(k, _)| k.as_str());
                                if let Some(name) = name {
                                    writeln!(
                                        b,
                                        "{}  {:04}  {:<8} [{}] ({})",
                                        indent, i, op, operand, name
                                    )
                                    .unwrap();
                                } else {
                                    writeln!(b, "{}  {:04}  {:<8} [{}]", indent, i, op, operand)
                                        .unwrap();
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
            let name = self
                .func_map
                .iter()
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
    use crate::ast::{Expr, Op, Stmt};
    use crate::lex::SymbolKind;

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
            ByteCode::LdaSmi as u8,
            0,
            ByteCode::Star as u8,
            0,
            ByteCode::LdaSmi as u8,
            1,
            ByteCode::Star as u8,
            1,
            ByteCode::Ldar as u8,
            0,
            ByteCode::Add as u8,
            1,
            ByteCode::Star as u8,
            2,
            ByteCode::Ldar as u8,
            0,
            ByteCode::Star as u8,
            3,
            ByteCode::Ldar as u8,
            2,
            ByteCode::Mul as u8,
            3,
        ];

        assert!(got.equals(&want), "got:\n{}\nwant:\n{}", got, want);
    }

    #[test]
    fn add_caller_and_callee() {
        let stmts = vec![
            Stmt::FuncDecl {
                name: "add".to_string(),
                params: vec!["a".to_string(), "b".to_string()],
                body: vec![Stmt::Return(Expr::Binary {
                    op: Op::Add,
                    left: Box::new(Expr::Ident("a".to_string())),
                    right: Box::new(Expr::Ident("b".to_string())),
                })],
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
            ByteCode::LdaSmi as u8,
            0,
            ByteCode::Star as u8,
            0,
            ByteCode::LdaSmi as u8,
            1,
            ByteCode::Star as u8,
            1,
            ByteCode::Call as u8,
            0,
        ];

        assert!(got.equals(&want), "got:\n{}\nwant:\n{}", got, want);
    }
}
