use crate::{
    bytecode::{ByteCode, Compiler},
    heap::{Heap, HeapObject},
    lex::Lexer,
    parser::Parser,
    value::Value,
};
use std::fmt;

struct Frame {
    ip: usize,
    reg: [Value; 256],
    closure: Value,
}

pub struct Isolate {
    acc: Value,
    heap: Heap,
    gc_threshold: usize,
}

impl fmt::Display for Isolate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Isolate{{acc: {}}}", self.acc)
    }
}

impl Isolate {
    pub fn new() -> Self {
        Isolate {
            acc: Default::default(),
            heap: Heap::new(),
            gc_threshold: 1024,
        }
    }

    pub fn with_heap(heap: Heap, gc_threshold: usize) -> Self {
        Isolate {
            acc: Default::default(),
            heap,
            gc_threshold,
        }
    }

    pub fn eval(&mut self, source: &str) -> Value {
        let tokens = Lexer::lex(source);
        let stmts = Parser::new(tokens).parse();
        let mut compiler = Compiler::init();
        compiler.compile_stmts(&stmts, &mut self.heap);
        let closure = compiler.finalize(&mut self.heap);
        self.run(closure)
    }

    pub fn format_value(&self, val: Value) -> String {
        if val.is_smi() {
            val.as_smi().to_string()
        } else {
            self.heap.read_string(val).to_string()
        }
    }

    fn read_byte(&self, stack: &[Frame], fi: usize) -> u8 {
        let closure = self.heap.read_closure(stack[fi].closure);
        let func = self.heap.read_function(closure.function);
        func.code[stack[fi].ip]
    }

    fn code_len(&self, stack: &[Frame], fi: usize) -> usize {
        let closure = self.heap.read_closure(stack[fi].closure);
        let func = self.heap.read_function(closure.function);
        func.code.len()
    }

    fn read_cons(&self, stack: &[Frame], fi: usize, idx: usize) -> Value {
        let closure = self.heap.read_closure(stack[fi].closure);
        let func = self.heap.read_function(closure.function);
        func.cons[idx]
    }

    pub(crate) fn run(&mut self, closure: Value) -> Value {
        let mut stack: Vec<Frame> = vec![Frame {
            ip: 0,
            reg: [Default::default(); 256],
            closure,
        }];

        while !stack.is_empty() {
            let fi = stack.len() - 1;
            if stack[fi].ip >= self.code_len(&stack, fi) {
                stack.pop();
                continue;
            }

            let opcode = ByteCode::from(self.read_byte(&stack, fi));
            stack[fi].ip += 1;

            match opcode {
                ByteCode::LdaSmi => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    self.acc = self.read_cons(&stack, fi, idx);
                }
                ByteCode::Star => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    stack[fi].reg[idx] = self.acc;
                }
                ByteCode::Ldar => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    self.acc = stack[fi].reg[idx];
                }
                ByteCode::Add => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    if self.acc.is_heap_object() && stack[fi].reg[idx].is_heap_object() {
                        let a = self.heap.read_string(stack[fi].reg[idx]).to_string();
                        let b = self.heap.read_string(self.acc).to_string();
                        let result = a + &b;
                        self.acc = self.heap.alloc_string(&result);
                        if self.heap.is_over_threshold(self.gc_threshold) {
                            self.collect_garbage(&stack);
                        }
                    } else {
                        self.acc =
                            Value::from_smi(self.acc.as_smi() + stack[fi].reg[idx].as_smi());
                    }
                }
                ByteCode::Sub => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    self.acc = Value::from_smi(stack[fi].reg[idx].as_smi() - self.acc.as_smi());
                }
                ByteCode::Mul => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    self.acc = Value::from_smi(self.acc.as_smi() * stack[fi].reg[idx].as_smi());
                }
                ByteCode::Div => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    self.acc = Value::from_smi(stack[fi].reg[idx].as_smi() / self.acc.as_smi());
                }
                ByteCode::Call => {
                    let func_reg = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    let arg_end = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;

                    let closure_val = stack[fi].reg[func_reg];
                    let param_count = {
                        let c = self.heap.read_closure(closure_val);
                        self.heap.read_function(c.function).param_count
                    };

                    let mut regs = [Value::default(); 256];
                    for i in 0..param_count {
                        regs[i] = stack[fi].reg[arg_end - param_count + i];
                    }
                    stack.push(Frame {
                        ip: 0,
                        reg: regs,
                        closure: closure_val,
                    });
                }
                ByteCode::Return => {
                    stack.pop();
                }
                ByteCode::Push => {
                    stack[fi].ip += 1;
                }
                ByteCode::TestEqual => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    if self.acc.is_smi() {
                        self.acc = if self.acc.as_smi() == stack[fi].reg[idx].as_smi() {
                            Value::from_smi(1)
                        } else {
                            Value::from_smi(0)
                        };
                    } else {
                        let a = self.heap.read_string(self.acc);
                        let b = self.heap.read_string(stack[fi].reg[idx]);
                        self.acc = if a == b {
                            Value::from_smi(1)
                        } else {
                            Value::from_smi(0)
                        };
                    }
                }
                ByteCode::TestLess => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    self.acc = if self.acc.as_smi() > stack[fi].reg[idx].as_smi() {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::TestGreater => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    self.acc = if self.acc.as_smi() < stack[fi].reg[idx].as_smi() {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::TestLessEqual => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    self.acc = if self.acc.as_smi() >= stack[fi].reg[idx].as_smi() {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::TestGreaterEqual => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    self.acc = if self.acc.as_smi() <= stack[fi].reg[idx].as_smi() {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::TestNotEqual => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    self.acc = if self.acc != stack[fi].reg[idx] {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::LogicalAnd => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    let zero = Value::from_smi(0);
                    self.acc = if self.acc != zero && stack[fi].reg[idx] != zero {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::LogicalOr => {
                    let idx = self.read_byte(&stack, fi) as usize;
                    stack[fi].ip += 1;
                    let zero = Value::from_smi(0);
                    self.acc = if self.acc != zero || stack[fi].reg[idx] != zero {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::LogicalNot => {
                    self.acc = if self.acc == Value::from_smi(0) {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::Jump => {
                    let offset = self.read_byte(&stack, fi);
                    stack[fi].ip += offset as usize + 1;
                }
                ByteCode::JumpIfFalse => {
                    let offset = self.read_byte(&stack, fi);
                    stack[fi].ip += 1;
                    if self.acc == Value::from_smi(0) {
                        stack[fi].ip += offset as usize;
                    }
                }
            }
        }

        self.acc
    }

    fn collect_garbage(&mut self, stack: &[Frame]) {
        let mut roots = Vec::new();
        roots.push(self.acc);
        for frame in stack.iter() {
            roots.push(frame.closure);
            for &val in frame.reg.iter() {
                if val.is_heap_object() {
                    roots.push(val);
                }
            }
        }
        self.heap.collect(&roots);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::ByteCode;
    use crate::heap::{BytecodeFunction, HeapClosure};

    fn make_closure(heap: &mut Heap, code: Vec<u8>, cons: Vec<Value>, param_count: usize) -> Value {
        let func = heap.alloc(HeapObject::Function(BytecodeFunction {
            code,
            cons,
            param_count,
            reg_count: 0,
        }));
        heap.alloc(HeapObject::Closure(HeapClosure {
            function: func,
            upvalues: vec![],
        }))
    }

    #[test]
    fn first_example() {
        let mut heap = Heap::new();
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::Star as u8, 1,
                ByteCode::LdaSmi as u8, 2,
                ByteCode::Mul as u8, 1,
                ByteCode::Add as u8, 0,
            ],
            vec![Value::from_smi(30), Value::from_smi(20), Value::from_smi(40)],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 830);
    }

    #[test]
    fn with_function_call() {
        let mut heap = Heap::new();

        let child = make_closure(
            &mut heap,
            vec![
                ByteCode::Ldar as u8, 0,
                ByteCode::Star as u8, 2,
                ByteCode::Ldar as u8, 1,
                ByteCode::Add as u8, 2,
                ByteCode::Return as u8,
            ],
            vec![],
            2,
        );

        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::Star as u8, 1,
                ByteCode::LdaSmi as u8, 2,
                ByteCode::Star as u8, 2,
                ByteCode::Call as u8, 0, 3,
            ],
            vec![child, Value::from_smi(10), Value::from_smi(20)],
            0,
        );

        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 30);
    }

    #[test]
    fn test_equal_true() {
        let mut heap = Heap::new();
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 0,
                ByteCode::TestEqual as u8, 0,
            ],
            vec![Value::from_smi(42)],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 1);
    }

    #[test]
    fn test_equal_false() {
        let mut heap = Heap::new();
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::TestEqual as u8, 0,
            ],
            vec![Value::from_smi(1), Value::from_smi(2)],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 0);
    }

    #[test]
    fn test_less_than() {
        let mut heap = Heap::new();
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::TestLess as u8, 0,
            ],
            vec![Value::from_smi(5), Value::from_smi(10)],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 1);
    }

    #[test]
    fn test_greater_than() {
        let mut heap = Heap::new();
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::TestGreater as u8, 0,
            ],
            vec![Value::from_smi(10), Value::from_smi(5)],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 1);
    }

    #[test]
    fn test_logical_not() {
        let mut heap = Heap::new();
        let closure = make_closure(
            &mut heap,
            vec![ByteCode::LdaSmi as u8, 0, ByteCode::LogicalNot as u8],
            vec![Value::from_smi(0)],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 1);
    }

    #[test]
    fn test_logical_and() {
        let mut heap = Heap::new();
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 0,
                ByteCode::LogicalAnd as u8, 0,
            ],
            vec![Value::from_smi(1)],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 1);
    }

    #[test]
    fn test_logical_or() {
        let mut heap = Heap::new();
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::LogicalOr as u8, 0,
            ],
            vec![Value::from_smi(1), Value::from_smi(0)],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 1);
    }

    #[test]
    fn test_jump_if_false() {
        let mut heap = Heap::new();
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::JumpIfFalse as u8, 2,
                ByteCode::LdaSmi as u8, 1,
            ],
            vec![Value::from_smi(0), Value::from_smi(99)],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 0);
    }

    #[test]
    fn test_jump_if_false_not_taken() {
        let mut heap = Heap::new();
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::JumpIfFalse as u8, 2,
                ByteCode::LdaSmi as u8, 1,
            ],
            vec![Value::from_smi(1), Value::from_smi(99)],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 99);
    }

    #[test]
    fn string_equality_same() {
        let mut heap = Heap::new();
        let a = heap.alloc_string("hello");
        let b = heap.alloc_string("hello");
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::TestEqual as u8, 0,
            ],
            vec![a, b],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 1);
    }

    #[test]
    fn string_equality_different() {
        let mut heap = Heap::new();
        let a = heap.alloc_string("hello");
        let b = heap.alloc_string("world");
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::TestEqual as u8, 0,
            ],
            vec![a, b],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 0);
    }

    #[test]
    fn subtraction() {
        let mut heap = Heap::new();
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::Sub as u8, 0,
            ],
            vec![Value::from_smi(10), Value::from_smi(3)],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 7);
    }

    #[test]
    fn division() {
        let mut heap = Heap::new();
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::Div as u8, 0,
            ],
            vec![Value::from_smi(20), Value::from_smi(4)],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.run(closure).as_smi(), 5);
    }

    #[test]
    fn string_concatenation() {
        let mut heap = Heap::new();
        let a = heap.alloc_string("hello");
        let b = heap.alloc_string(" world");
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::Add as u8, 0,
            ],
            vec![a, b],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 1024);
        let result = iso.run(closure);
        assert!(result.is_heap_object());
        assert_eq!(iso.format_value(result), "hello world");
    }

    #[test]
    fn garbage_collection() {
        let mut heap = Heap::new();
        let a = heap.alloc_string("a");
        let b = heap.alloc_string("b");
        let closure = make_closure(
            &mut heap,
            vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::Add as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::Add as u8, 0,
            ],
            vec![a, b],
            0,
        );
        let mut iso = Isolate::with_heap(heap, 6);
        let result = iso.run(closure);
        assert!(result.is_heap_object());
        assert_eq!(iso.format_value(result), "ab");
    }

    #[test]
    fn eval_arithmetic() {
        let mut iso = Isolate::new();
        let result = iso.eval("let x = 10 + 20;");
        assert_eq!(iso.format_value(result), "30");
    }

    #[test]
    fn eval_function_call() {
        let mut iso = Isolate::new();
        let result = iso.eval("function add(a, b) { return a + b; } add(10, 20);");
        assert_eq!(iso.format_value(result), "30");
    }

    #[test]
    fn eval_string() {
        let mut iso = Isolate::new();
        let result = iso.eval("let s = \"hello\" + \" world\";");
        assert_eq!(iso.format_value(result), "hello world");
    }

    #[test]
    fn two_isolates_are_independent() {
        let mut iso_a = Isolate::new();
        let mut iso_b = Isolate::new();
        iso_a.eval("let x = 10 + 20;");
        iso_b.eval("let x = 99;");
        assert_eq!(iso_a.format_value(iso_a.acc), "30");
        assert_eq!(iso_b.format_value(iso_b.acc), "99");
    }
}
