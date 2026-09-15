use crate::bytecode::{ByteCode, Program};
use std::fmt;

struct Frame<'a> {
    ip: usize,
    reg: [i64; 256],
    program: &'a Program,
}

pub struct VM {
    acc: i64,
}

impl fmt::Display for VM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VM{{acc: {}}}", self.acc)
    }
}

impl VM {
    pub fn new() -> Self {
        VM { acc: 0 }
    }

    pub fn fde(&mut self, program: &Program) -> i64 {
        let mut stack: Vec<Frame> = vec![Frame {
            ip: 0,
            reg: [0; 256],
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
                    self.acc += stack[fi].reg[idx];
                }
                ByteCode::Sub => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc = stack[fi].reg[idx] - self.acc;
                }
                ByteCode::Mul => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc *= stack[fi].reg[idx];
                }
                ByteCode::Div => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc = stack[fi].reg[idx] / self.acc;
                }
                ByteCode::Call => {
                    let func_idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    let function = &program.funcs[func_idx];
                    let next_reg = stack[fi].program.next_reg;
                    let param_count = function.param_count;
                    let mut regs = [0i64; 256];
                    for i in 0..param_count {
                        regs[i] = stack[fi].reg[next_reg - param_count + i];
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
                    self.acc = if self.acc == stack[fi].reg[idx] { 1 } else { 0 };
                }
                ByteCode::TestLess => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc = if self.acc > stack[fi].reg[idx] { 1 } else { 0 };
                }
                ByteCode::TestGreater => {
                    let idx = stack[fi].program.code[stack[fi].ip] as usize;
                    stack[fi].ip += 1;
                    self.acc = if self.acc < stack[fi].reg[idx] { 1 } else { 0 };
                }
                ByteCode::Jump => {
                    let offset = stack[fi].program.code[stack[fi].ip];
                    stack[fi].ip += offset as usize + 1;
                }
                ByteCode::JumpIfFalse => {
                    let offset = stack[fi].program.code[stack[fi].ip];
                    stack[fi].ip += 1;
                    if self.acc == 0 {
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
            cons: vec![30, 20, 40],
            funcs: vec![],
            func_map: HashMap::new(),
            param_count: 0,
            next_reg: 0,
            symbols: HashMap::new(),
        };

        let mut vm = VM::new();
        assert_eq!(vm.fde(&program), 830);
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
            ],
            cons: vec![10, 20],
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

        let mut vm = VM::new();
        assert_eq!(vm.fde(&program), 30);
    }
}
