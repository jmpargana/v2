use crate::{
    bytecode::{ByteCode, Program},
    heap::Heap,
    value::Value,
};
use std::fmt;

struct Frame<'a> {
    ip: usize,
    reg: [Value; 256],
    program: &'a Program,
}

pub struct VM {
    acc: Value,
    heap: Heap,
}

impl fmt::Display for VM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VM{{acc: {}}}", self.acc)
    }
}

impl VM {
    pub fn new(heap: Heap) -> Self {
        VM {
            acc: Default::default(),
            heap,
        }
    }

    pub fn format_value(&self, val: Value) -> String {
        if val.is_smi() {
            val.as_smi().to_string()
        } else {
            self.heap.read_string(val).to_string()
        }
    }

    pub fn fde(&mut self, program: &Program) -> Value {
        let mut stack: Vec<Frame> = vec![Frame {
            ip: 0,
            reg: [Default::default(); 256],
            program,
        }];

        while !stack.is_empty() {
            let fi = stack.len() - 1;
            if stack[fi].ip >= stack[fi].program.code.len() {
                stack.pop();
                continue;
            }

            let opcode = ByteCode::from(stack[fi].program.code[stack[fi].ip]);
            stack[fi].ip += 1;

            match opcode {
                ByteCode::LdaSmi => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc = stack[fi].program.cons[idx];
                }
                ByteCode::Star => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    stack[fi].reg[idx] = self.acc;
                }
                ByteCode::Ldar => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc = stack[fi].reg[idx];
                }
                ByteCode::Add => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    if self.acc.is_heap_object() && stack[fi].reg[idx].is_heap_object() {
                        let a = self.heap.read_string(stack[fi].reg[idx]).to_string();
                        let b = self.heap.read_string(self.acc).to_string();
                        let result = a + &b;
                        self.acc = self.heap.alloc_string(&result);
                    } else {
                        self.acc =
                            Value::from_smi(self.acc.as_smi() + stack[fi].reg[idx].as_smi())
                    }
                }
                ByteCode::Sub => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc = Value::from_smi(stack[fi].reg[idx].as_smi() - self.acc.as_smi())
                }
                ByteCode::Mul => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc = Value::from_smi(self.acc.as_smi() * stack[fi].reg[idx].as_smi())
                }
                ByteCode::Div => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc = Value::from_smi(stack[fi].reg[idx].as_smi() / self.acc.as_smi())
                }
                ByteCode::Call => {
                    let func_idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    let arg_end = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    let function = &program.funcs[func_idx];
                    let param_count = function.param_count;
                    let mut regs = [Value::default(); 256];
                    for i in 0..param_count {
                        regs[i] = stack[fi].reg[arg_end - param_count + i];
                    }
                    stack.push(Frame {
                        ip: 0,
                        reg: regs,
                        program: function,
                    });
                }
                ByteCode::Return => {
                    stack.pop();
                }
                ByteCode::Push => {
                    stack[fi].ip += 1;
                }
                ByteCode::TestEqual => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
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
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc = if self.acc.as_smi() > stack[fi].reg[idx].as_smi() {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::TestGreater => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc = if self.acc.as_smi() < stack[fi].reg[idx].as_smi() {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::TestLessEqual => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc = if self.acc.as_smi() >= stack[fi].reg[idx].as_smi() {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::TestGreaterEqual => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc = if self.acc.as_smi() <= stack[fi].reg[idx].as_smi() {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::TestNotEqual => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc = if self.acc != stack[fi].reg[idx] {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::LogicalAnd => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    let zero = Value::from_smi(0);
                    self.acc = if self.acc != zero && stack[fi].reg[idx] != zero {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::LogicalOr => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
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
                    let offset = stack[fi].program.code[stack[fi].ip];
                    stack[fi].ip += offset as usize + 1;
                }
                ByteCode::JumpIfFalse => {
                    let offset = stack[fi].program.code[stack[fi].ip];
                    stack[fi].ip += 1;
                    if self.acc == Value::from_smi(0) {
                        stack[fi].ip += offset as usize;
                    }
                }
            }
        }

        self.acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{ByteCode, Program};
    use std::collections::HashMap;

    #[test]
    fn first_example() {
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8,
                0,
                ByteCode::Star as u8,
                0,
                ByteCode::LdaSmi as u8,
                1,
                ByteCode::Star as u8,
                1,
                ByteCode::LdaSmi as u8,
                2,
                ByteCode::Mul as u8,
                1,
                ByteCode::Add as u8,
                0,
            ],
            cons: vec![Value::from_smi(30), Value::from_smi(20), Value::from_smi(40)],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };

        let mut vm = VM::new(Heap::new());
        assert_eq!(vm.fde(&program).as_smi(),830);
    }

    #[test]
    fn with_function_call() {
        let program = Program {
            code: vec![
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
                2,
            ],
            cons: vec![Value::from_smi(10), Value::from_smi(20)],
            funcs: vec![Program {
                code: vec![
                    ByteCode::Ldar as u8,
                    0,
                    ByteCode::Star as u8,
                    2,
                    ByteCode::Ldar as u8,
                    1,
                    ByteCode::Add as u8,
                    2,
                    ByteCode::Return as u8,
                ],
                cons: vec![],
                param_count: 2,
                next_reg: 3,
                symbols: HashMap::from([("a".to_string(), 0), ("b".to_string(), 1)]),
                funcs: vec![],
                func_map: HashMap::new(),
            }],
            func_map: HashMap::from([("add".to_string(), 0)]),
            param_count: 0,
            next_reg: 2,
            symbols: HashMap::new(),
        };

        let mut vm = VM::new(Heap::new());
        assert_eq!(vm.fde(&program).as_smi(),30);
    }

    #[test]
    fn test_equal_true() {
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 0,
                ByteCode::TestEqual as u8, 0,
            ],
            cons: vec![Value::from_smi(42)],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };
        let mut vm = VM::new(Heap::new());
        assert_eq!(vm.fde(&program).as_smi(),1);
    }

    #[test]
    fn test_equal_false() {
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::TestEqual as u8, 0,
            ],
            cons: vec![Value::from_smi(1), Value::from_smi(2)],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };
        let mut vm = VM::new(Heap::new());
        assert_eq!(vm.fde(&program).as_smi(),0);
    }

    #[test]
    fn test_less_than() {
        // reg[0] = 5, acc = 10 → acc > reg[0] → TestLess yields 1
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::TestLess as u8, 0,
            ],
            cons: vec![Value::from_smi(5), Value::from_smi(10)],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };
        let mut vm = VM::new(Heap::new());
        assert_eq!(vm.fde(&program).as_smi(),1);
    }

    #[test]
    fn test_greater_than() {
        // reg[0] = 10, acc = 5 → acc < reg[0] → TestGreater yields 1
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::TestGreater as u8, 0,
            ],
            cons: vec![Value::from_smi(10), Value::from_smi(5)],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };
        let mut vm = VM::new(Heap::new());
        assert_eq!(vm.fde(&program).as_smi(),1);
    }

    #[test]
    fn test_logical_not() {
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::LogicalNot as u8,
            ],
            cons: vec![Value::from_smi(0)],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };
        let mut vm = VM::new(Heap::new());
        assert_eq!(vm.fde(&program).as_smi(),1);
    }

    #[test]
    fn test_logical_and() {
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 0,
                ByteCode::LogicalAnd as u8, 0,
            ],
            cons: vec![Value::from_smi(1)],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };
        let mut vm = VM::new(Heap::new());
        assert_eq!(vm.fde(&program).as_smi(),1);
    }

    #[test]
    fn test_logical_or() {
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::LogicalOr as u8, 0,
            ],
            cons: vec![Value::from_smi(1), Value::from_smi(0)],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };
        let mut vm = VM::new(Heap::new());
        assert_eq!(vm.fde(&program).as_smi(),1);
    }

    #[test]
    fn test_jump_if_false() {
        // acc = 0 → JumpIfFalse skips over LdaSmi(99) → acc stays 0
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::JumpIfFalse as u8, 2,
                ByteCode::LdaSmi as u8, 1,
            ],
            cons: vec![Value::from_smi(0), Value::from_smi(99)],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };
        let mut vm = VM::new(Heap::new());
        assert_eq!(vm.fde(&program).as_smi(),0);
    }

    #[test]
    fn test_jump_if_false_not_taken() {
        // acc = 1 → JumpIfFalse not taken → LdaSmi loads 99
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::JumpIfFalse as u8, 2,
                ByteCode::LdaSmi as u8, 1,
            ],
            cons: vec![Value::from_smi(1), Value::from_smi(99)],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };
        let mut vm = VM::new(Heap::new());
        assert_eq!(vm.fde(&program).as_smi(),99);
    }

    #[test]
    fn string_equality_same() {
        let mut heap = Heap::new();
        let a = heap.alloc_string("hello");
        let b = heap.alloc_string("hello");
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::TestEqual as u8, 0,
            ],
            cons: vec![a, b],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };
        let mut vm = VM::new(heap);
        assert_eq!(vm.fde(&program).as_smi(),1);
    }

    #[test]
    fn string_equality_different() {
        let mut heap = Heap::new();
        let a = heap.alloc_string("hello");
        let b = heap.alloc_string("world");
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::TestEqual as u8, 0,
            ],
            cons: vec![a, b],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };
        let mut vm = VM::new(heap);
        assert_eq!(vm.fde(&program).as_smi(),0);
    }

    #[test]
    fn subtraction() {
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::Sub as u8, 0,
            ],
            cons: vec![Value::from_smi(10), Value::from_smi(3)],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };
        let mut vm = VM::new(Heap::new());
        assert_eq!(vm.fde(&program).as_smi(),7);
    }

    #[test]
    fn division() {
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::Div as u8, 0,
            ],
            cons: vec![Value::from_smi(20), Value::from_smi(4)],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };
        let mut vm = VM::new(Heap::new());
        assert_eq!(vm.fde(&program).as_smi(),5);
    }

    #[test]
    fn string_concatenation() {
        let mut heap = Heap::new();
        let a = heap.alloc_string("hello");
        let b = heap.alloc_string(" world");
        let program = Program {
            code: vec![
                ByteCode::LdaSmi as u8, 0,
                ByteCode::Star as u8, 0,
                ByteCode::LdaSmi as u8, 1,
                ByteCode::Add as u8, 0,
            ],
            cons: vec![a, b],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };
        let mut vm = VM::new(heap);
        let result = vm.fde(&program);
        assert!(result.is_heap_object());
        assert_eq!(vm.format_value(result), "hello world");
    }
}
