use crate::ast::{Expr, Op, Stmt};
use crate::heap::{BytecodeFunction, Heap, Closure, HeapObject, HeapString};
use crate::value::Value;
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

/// Compiles AST statements into bytecode for a single function scope.
/// Each FuncDecl creates a child Compiler; the root Compiler produces the
/// top-level program. Call compile_stmts() to emit bytecode, then finalize()
/// to allocate the BytecodeFunction + Closure on the heap.
///
/// The closures map tracks function declarations in the current scope so that
/// child compilers can inherit them (enabling recursive and cross-scope calls).
#[derive(Debug)]
pub struct Compiler {
    pub code: Vec<u8>,
    pub constant_pool: Vec<Value>,
    pub param_count: usize,
    pub next_reg: usize,
    pub symbols: HashMap<String, usize>,
    pub closures: HashMap<String, Value>,
}

impl Compiler {
    pub fn init() -> Self {
        Compiler {
            code: Vec::new(),
            constant_pool: Vec::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
            closures: HashMap::new(),
        }
    }

    fn alloc_reg(&mut self) -> usize {
        let val = self.next_reg;
        self.next_reg = (self.next_reg + 1) % 256;
        val
    }

    pub fn compile_stmts(&mut self, stmts: &[Stmt], heap: &mut Heap) {
        for stmt in stmts {
            self.compile_stmt(stmt, heap);
        }
    }

    pub fn finalize(mut self, heap: &mut Heap) -> Value {
        let func = heap.alloc(HeapObject::Function(BytecodeFunction {
            code: std::mem::take(&mut self.code),
            constant_pool: std::mem::take(&mut self.constant_pool),
            param_count: self.param_count,
            register_count: self.next_reg,
        }));
        heap.alloc(HeapObject::Closure(Closure {
            function: func,
            upvalues: vec![],
        }))
    }

    fn compile_stmt(&mut self, stmt: &Stmt, heap: &mut Heap) {
        match stmt {
            Stmt::VarDecl { name, value, .. } => {
                self.compile_expr(value, heap);
                let reg = self.alloc_reg();
                self.symbols.insert(name.clone(), reg);
                self.code.push(ByteCode::Star as u8);
                self.code.push(reg as u8);
            }
            Stmt::FuncDecl { name, params, body } => {
                // Two-phase allocation: create placeholder so recursive/sibling calls work
                let placeholder_func = heap.alloc(HeapObject::Function(BytecodeFunction {
                    code: vec![],
                    constant_pool: vec![],
                    param_count: params.len(),
                    register_count: 0,
                }));
                let closure_val = heap.alloc(HeapObject::Closure(Closure {
                    function: placeholder_func,
                    upvalues: vec![],
                }));

                let cons_idx = self.constant_pool.len();
                self.constant_pool.push(closure_val);
                self.code.push(ByteCode::LdaSmi as u8);
                self.code.push(cons_idx as u8);
                let reg = self.alloc_reg();
                self.symbols.insert(name.clone(), reg);
                self.code.push(ByteCode::Star as u8);
                self.code.push(reg as u8);
                self.closures.insert(name.clone(), closure_val);

                let mut child = Compiler::init();
                child.param_count = params.len();
                for param in params {
                    let r = child.alloc_reg();
                    child.symbols.insert(param.clone(), r);
                }

                // Inject parent-scope closures for recursive and cross-scope calls
                child.closures = self.closures.clone();
                let mut parent_closures: Vec<(String, Value)> = self
                    .closures
                    .iter()
                    .map(|(n, v)| (n.clone(), *v))
                    .collect();
                parent_closures.sort_by(|a, b| a.0.cmp(&b.0));
                for (fn_name, fn_val) in &parent_closures {
                    let cidx = child.constant_pool.len();
                    child.constant_pool.push(*fn_val);
                    child.code.push(ByteCode::LdaSmi as u8);
                    child.code.push(cidx as u8);
                    let creg = child.alloc_reg();
                    child.symbols.insert(fn_name.clone(), creg);
                    child.code.push(ByteCode::Star as u8);
                    child.code.push(creg as u8);
                }

                child.compile_stmts(body, heap);

                let real_func = heap.alloc(HeapObject::Function(BytecodeFunction {
                    code: child.code,
                    constant_pool: child.constant_pool,
                    param_count: child.param_count,
                    register_count: child.next_reg,
                }));
                heap.patch_closure(closure_val, real_func);
            }
            Stmt::Return(value) => {
                self.compile_expr(value, heap);
                self.code.push(ByteCode::Return as u8);
            }
            Stmt::ExprStmt(expr) => {
                self.compile_expr(expr, heap);
            }
            Stmt::Cond {
                condition,
                body,
                alternate,
            } => {
                self.compile_expr(condition, heap);
                let jump_false = self.emit_jump(ByteCode::JumpIfFalse);
                self.compile_stmts(body, heap);
                if let Some(alt) = alternate {
                    let jump = self.emit_jump(ByteCode::Jump);
                    self.patch_jump(jump_false);
                    self.compile_stmts(alt, heap);
                    self.patch_jump(jump);
                } else {
                    self.patch_jump(jump_false);
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

    fn compile_expr(&mut self, expr: &Expr, heap: &mut Heap) {
        match expr {
            Expr::StringLit(val) => {
                let idx = self.constant_pool.len();
                self.constant_pool.push(heap.alloc(HeapObject::String(HeapString {
                    data: val.to_string(),
                })));
                self.code.push(ByteCode::LdaSmi as u8);
                self.code.push(idx as u8);
            }
            Expr::NumberLit(value) => {
                let idx = self.constant_pool.len();
                self.constant_pool.push(Value::from_smi(*value));
                self.code.push(ByteCode::LdaSmi as u8);
                self.code.push(idx as u8);
            }
            Expr::Ident(name) => {
                let reg = self.symbols[name];
                self.code.push(ByteCode::Ldar as u8);
                self.code.push(reg as u8);
            }
            Expr::Binary { op, left, right } => {
                self.compile_expr(left, heap);
                let reg = self.alloc_reg();
                self.code.push(ByteCode::Star as u8);
                self.code.push(reg as u8);
                self.compile_expr(right, heap);
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
                    self.compile_expr(arg, heap);
                    let reg = self.alloc_reg();
                    self.code.push(ByteCode::Star as u8);
                    self.code.push(reg as u8);
                }
                let func_reg = self.symbols[func];
                self.code.push(ByteCode::Call as u8);
                self.code.push(func_reg as u8);
                self.code.push(self.next_reg as u8);
            }
            Expr::Not(operand) => {
                self.compile_expr(operand, heap);
                self.code.push(ByteCode::LogicalNot as u8);
            }
            Expr::Bool { op, left, right } => {
                self.compile_expr(left, heap);
                let reg = self.alloc_reg();
                self.code.push(ByteCode::Star as u8);
                self.code.push(reg as u8);
                self.compile_expr(right, heap);
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

    fn disassemble(&self, indent: &str) -> String {
        let mut b = String::new();
        writeln!(b, "{}Constants: {:?}", indent, self.constant_pool).unwrap();
        writeln!(
            b,
            "{}Registers: {}, Params: {}",
            indent, self.next_reg, self.param_count
        )
        .unwrap();
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
                                if operand < self.constant_pool.len() {
                                    writeln!(
                                        b,
                                        "{}  {:04}  {:<8} [{}] ({:?})",
                                        indent, i, op, operand, self.constant_pool[operand]
                                    )
                                    .unwrap();
                                } else {
                                    writeln!(b, "{}  {:04}  {:<8} [{}]", indent, i, op, operand)
                                        .unwrap();
                                }
                            }
                            ByteCode::Star
                            | ByteCode::Ldar
                            | ByteCode::Add
                            | ByteCode::Sub
                            | ByteCode::Mul
                            | ByteCode::Div
                            | ByteCode::TestEqual
                            | ByteCode::TestLess
                            | ByteCode::TestGreater
                            | ByteCode::TestLessEqual
                            | ByteCode::TestGreaterEqual
                            | ByteCode::TestNotEqual
                            | ByteCode::LogicalAnd
                            | ByteCode::LogicalOr => {
                                writeln!(b, "{}  {:04}  {:<8} r{}", indent, i, op, operand)
                                    .unwrap();
                            }
                            ByteCode::Call => {
                                let arg_end = if i + 2 < self.code.len() {
                                    self.code[i + 2] as usize
                                } else {
                                    0
                                };
                                writeln!(
                                    b,
                                    "{}  {:04}  {:<8} r{} args@r{}",
                                    indent, i, op, operand, arg_end
                                )
                                .unwrap();
                                i += 1;
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

        b
    }
}

impl fmt::Display for Compiler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.disassemble(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Op, Stmt};
    use crate::heap::Heap;
    use crate::lex::SymbolKind;

    fn compile_and_read(stmts: &[Stmt], heap: &mut Heap) -> (Value, Value) {
        let mut compiler = Compiler::init();
        compiler.compile_stmts(stmts, heap);
        let closure_val = compiler.finalize(heap);
        let func_val = heap.read_closure(closure_val).function;
        (closure_val, func_val)
    }

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

        let mut heap = Heap::new();
        let (_, func_val) = compile_and_read(&stmts, &mut heap);
        let func = heap.read_function(func_val);

        assert_eq!(
            func.code,
            vec![
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
            ]
        );
        assert_eq!(func.constant_pool, vec![Value::from_smi(10), Value::from_smi(20)]);
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

        let mut heap = Heap::new();
        let (_, func_val) = compile_and_read(&stmts, &mut heap);
        let func = heap.read_function(func_val);

        assert_eq!(
            func.code,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::Star as u8, 1,
                ByteCode::LdaSmi as u8, 2,
                ByteCode::Star as u8, 2,
                ByteCode::Call as u8, 0, 3,
            ]
        );
        assert!(func.constant_pool[0].is_heap_object());
        assert_eq!(func.constant_pool[1], Value::from_smi(10));
        assert_eq!(func.constant_pool[2], Value::from_smi(20));

        let child_closure = heap.read_closure(func.constant_pool[0]);
        let child_func = heap.read_function(child_closure.function);
        assert_eq!(child_func.param_count, 2);
        assert_eq!(
            child_func.code,
            vec![
                // preamble: inject parent closure "add" into r2
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 2,
                // body: return a + b
                ByteCode::Ldar as u8, 0,
                ByteCode::Star as u8, 3,
                ByteCode::Ldar as u8, 1,
                ByteCode::Add as u8, 3,
                ByteCode::Return as u8,
            ]
        );
        assert_eq!(child_func.constant_pool.len(), 1);
        assert!(child_func.constant_pool[0].is_heap_object());
    }

    #[test]
    fn string_var_decl() {
        let stmts = vec![Stmt::VarDecl {
            kind: SymbolKind::Let,
            name: "s".to_string(),
            value: Expr::StringLit("hello".to_string()),
        }];

        let mut heap = Heap::new();
        let (_, func_val) = compile_and_read(&stmts, &mut heap);
        let func = heap.read_function(func_val);

        assert_eq!(
            func.code,
            vec![ByteCode::LdaSmi as u8, 0, ByteCode::Star as u8, 0]
        );
        assert!(func.constant_pool[0].is_heap_object());
        assert_eq!(heap.read_string(func.constant_pool[0]), "hello");
    }

    #[test]
    fn string_equality_bytecode() {
        let stmts = vec![Stmt::ExprStmt(Expr::Bool {
            op: Op::Equal,
            left: Box::new(Expr::StringLit("a".to_string())),
            right: Box::new(Expr::StringLit("b".to_string())),
        })];

        let mut heap = Heap::new();
        let (_, func_val) = compile_and_read(&stmts, &mut heap);
        let func = heap.read_function(func_val);

        assert_eq!(
            func.code,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::TestEqual as u8, 0,
            ]
        );
        assert_eq!(heap.read_string(func.constant_pool[0]), "a");
        assert_eq!(heap.read_string(func.constant_pool[1]), "b");
    }

    #[test]
    fn conditional_bytecode() {
        let stmts = vec![Stmt::Cond {
            condition: Expr::Bool {
                op: Op::Gt,
                left: Box::new(Expr::Ident("x".to_string())),
                right: Box::new(Expr::NumberLit(0)),
            },
            body: vec![Stmt::VarDecl {
                kind: SymbolKind::Let,
                name: "y".to_string(),
                value: Expr::NumberLit(1),
            }],
            alternate: None,
        }];

        let mut compiler = Compiler::init();
        compiler.symbols.insert("x".to_string(), 0);
        compiler.next_reg = 1;
        let mut heap = Heap::new();
        compiler.compile_stmts(&stmts, &mut heap);
        let closure_val = compiler.finalize(&mut heap);
        let func_val = heap.read_closure(closure_val).function;
        let func = heap.read_function(func_val);

        assert_eq!(
            func.code,
            vec![
                ByteCode::Ldar as u8, 0,
                ByteCode::Star as u8, 1,
                ByteCode::LdaSmi as u8, 0,
                ByteCode::TestGreater as u8, 1,
                ByteCode::JumpIfFalse as u8, 4,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::Star as u8, 2,
            ]
        );
    }

    #[test]
    fn logical_not_bytecode() {
        let stmts = vec![Stmt::ExprStmt(Expr::Not(Box::new(Expr::Ident(
            "x".to_string(),
        ))))];

        let mut compiler = Compiler::init();
        compiler.symbols.insert("x".to_string(), 0);
        compiler.next_reg = 1;
        let mut heap = Heap::new();
        compiler.compile_stmts(&stmts, &mut heap);
        let closure_val = compiler.finalize(&mut heap);
        let func_val = heap.read_closure(closure_val).function;
        let func = heap.read_function(func_val);

        assert_eq!(
            func.code,
            vec![ByteCode::Ldar as u8, 0, ByteCode::LogicalNot as u8]
        );
    }
}
