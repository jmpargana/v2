use crate::{
    bytecode::{ByteCode, Compiler},
    heap::{Heap, HeapObject},
    lex::Lexer,
    parser::Parser,
    value::Value,
};
use std::fmt;

/// A flat array of Value slots subdivided into Frames.
/// Frames are windows into this array — pushing a call extends the slots,
/// popping truncates back. More cache-friendly than per-frame register arrays.
///
/// This is the runtime equivalent of the "call stack" in V8's execution model.
/// The slots hold register values for all active frames contiguously.
struct CallStack {
    slots: Vec<Value>,
    frames: Vec<Frame>,
}

/// A lightweight descriptor pointing into the CallStack. Not a heap object.
/// Knows where its registers start, which instruction to execute next,
/// and which Closure provides the bytecode and captured environment.
///
/// Caches raw pointers to the current function's code and constant pool
/// to avoid re-dereferencing closure → function on every bytecode dispatch.
struct Frame {
    base_offset: usize,
    pc: usize,
    closure: Value,
    code: *const u8,
    code_len: usize,
    constants: *const Value,
    constants_len: usize,
}

impl Frame {
    /// Build a frame with cached pointers into the function's bytecode and constant pool.
    ///
    /// SAFETY of later derefs: the pointers target heap buffers owned by Vec<u8> and
    /// Vec<Value> inside a BytecodeFunction. These remain stable because:
    /// 1. Moving a Vec struct (e.g. when heap.objects reallocates) does not move its buffer.
    /// 2. Nothing appends to code or constant_pool during execution.
    /// 3. The function is reachable via this frame's closure (a GC root), so GC won't free it.
    fn new(heap: &Heap, closure: Value, base_offset: usize) -> Self {
        let c = heap.read_closure(closure);
        let f = heap.read_function(c.function);
        Frame {
            base_offset,
            pc: 0,
            closure,
            code: f.code.as_ptr(),
            code_len: f.code.len(),
            constants: f.constant_pool.as_ptr(),
            constants_len: f.constant_pool.len(),
        }
    }

    #[inline(always)]
    fn read_byte(&self) -> u8 {
        assert!(self.pc < self.code_len, "bytecode PC out of bounds");
        // SAFETY: pc < code_len verified above, pointer valid per Frame::new
        unsafe { *self.code.add(self.pc) }
    }

    #[inline(always)]
    fn read_constant(&self, idx: usize) -> Value {
        assert!(idx < self.constants_len, "constant pool index out of bounds");
        // SAFETY: idx < constants_len verified above, pointer valid per Frame::new
        unsafe { *self.constants.add(idx) }
    }
}

impl CallStack {
    fn new(heap: &Heap, closure: Value, register_count: usize) -> Self {
        CallStack {
            slots: vec![Value::default(); register_count],
            frames: vec![Frame::new(heap, closure, 0)],
        }
    }

    fn reg(&self, fi: usize, idx: usize) -> Value {
        self.slots[self.frames[fi].base_offset + idx]
    }

    fn set_reg(&mut self, fi: usize, idx: usize, val: Value) {
        self.slots[self.frames[fi].base_offset + idx] = val;
    }

    fn pop_frame(&mut self) {
        if let Some(frame) = self.frames.pop() {
            self.slots.truncate(frame.base_offset);
        }
    }
}

/// The Isolate is a self-contained execution context — V8's core abstraction.
/// Each Isolate has its own heap, GC, and execution state. No shared mutable
/// state with any other Isolate: two Isolates in the same process see
/// completely separate realities.
///
/// Source code goes in via eval(), a result comes out. The caller never
/// touches the heap or bytecode directly.
pub struct Isolate {
    acc: Value,
    heap: Heap,
    gc_threshold: usize,
    // globals: GlobalScope,  // top-level variable bindings (or a JSObject acting as the global object)
}

impl fmt::Display for Isolate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Isolate{{ acc: {}, heap_size: {} }}",
            self.format_value(self.acc),
            self.heap.len()
        )
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
            match self.heap.get(val) {
                Some(HeapObject::String(s)) => s.data.clone(),
                Some(HeapObject::Closure(_)) => "<closure>".to_string(),
                Some(HeapObject::Function(_)) => "<function>".to_string(),
                None => "<freed>".to_string(),
            }
        }
    }

    pub(crate) fn run(&mut self, closure: Value) -> Value {
        let register_count = {
            let c = self.heap.read_closure(closure);
            self.heap.read_function(c.function).register_count
        };
        let mut call_stack = CallStack::new(&self.heap, closure, register_count);

        while !call_stack.frames.is_empty() {
            let fi = call_stack.frames.len() - 1;
            if call_stack.frames[fi].pc >= call_stack.frames[fi].code_len {
                call_stack.pop_frame();
                continue;
            }

            let opcode = ByteCode::from(call_stack.frames[fi].read_byte());
            call_stack.frames[fi].pc += 1;

            match opcode {
                ByteCode::LdaSmi => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    self.acc = call_stack.frames[fi].read_constant(idx);
                }
                ByteCode::Star => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    call_stack.set_reg(fi, idx, self.acc);
                }
                ByteCode::Ldar => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    self.acc = call_stack.reg(fi, idx);
                }
                ByteCode::Add => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    if self.acc.is_heap_object() && call_stack.reg(fi, idx).is_heap_object() {
                        let a = self.heap.read_string(call_stack.reg(fi, idx)).to_string();
                        let b = self.heap.read_string(self.acc).to_string();
                        let result = a + &b;
                        self.acc = self.heap.alloc_string(&result);
                        if self.heap.is_over_threshold(self.gc_threshold) {
                            self.collect_garbage(&call_stack);
                        }
                    } else {
                        self.acc =
                            Value::from_smi(self.acc.as_smi() + call_stack.reg(fi, idx).as_smi());
                    }
                }
                ByteCode::Sub => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    self.acc =
                        Value::from_smi(call_stack.reg(fi, idx).as_smi() - self.acc.as_smi());
                }
                ByteCode::Mul => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    self.acc =
                        Value::from_smi(self.acc.as_smi() * call_stack.reg(fi, idx).as_smi());
                }
                ByteCode::Div => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    self.acc =
                        Value::from_smi(call_stack.reg(fi, idx).as_smi() / self.acc.as_smi());
                }
                ByteCode::Call => {
                    let func_reg = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    let arg_end = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;

                    let closure_val = call_stack.reg(fi, func_reg);
                    let (param_count, callee_reg_count) = {
                        let c = self.heap.read_closure(closure_val);
                        let f = self.heap.read_function(c.function);
                        (f.param_count, f.register_count)
                    };

                    let callee_base = call_stack.slots.len();
                    call_stack
                        .slots
                        .resize(callee_base + callee_reg_count, Value::default());
                    for i in 0..param_count {
                        call_stack.slots[callee_base + i] =
                            call_stack.reg(fi, arg_end - param_count + i);
                    }
                    call_stack
                        .frames
                        .push(Frame::new(&self.heap, closure_val, callee_base));
                }
                ByteCode::Return => {
                    call_stack.pop_frame();
                }
                ByteCode::Push => {
                    call_stack.frames[fi].pc += 1;
                }
                ByteCode::TestEqual => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    if self.acc.is_smi() {
                        self.acc =
                            if self.acc.as_smi() == call_stack.reg(fi, idx).as_smi() {
                                Value::from_smi(1)
                            } else {
                                Value::from_smi(0)
                            };
                    } else {
                        let a = self.heap.read_string(self.acc);
                        let b = self.heap.read_string(call_stack.reg(fi, idx));
                        self.acc = if a == b {
                            Value::from_smi(1)
                        } else {
                            Value::from_smi(0)
                        };
                    }
                }
                ByteCode::TestLess => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    self.acc = if self.acc.as_smi() > call_stack.reg(fi, idx).as_smi() {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::TestGreater => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    self.acc = if self.acc.as_smi() < call_stack.reg(fi, idx).as_smi() {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::TestLessEqual => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    self.acc = if self.acc.as_smi() >= call_stack.reg(fi, idx).as_smi() {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::TestGreaterEqual => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    self.acc = if self.acc.as_smi() <= call_stack.reg(fi, idx).as_smi() {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::TestNotEqual => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    self.acc = if self.acc != call_stack.reg(fi, idx) {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::LogicalAnd => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    let zero = Value::from_smi(0);
                    self.acc = if self.acc != zero && call_stack.reg(fi, idx) != zero {
                        Value::from_smi(1)
                    } else {
                        Value::from_smi(0)
                    };
                }
                ByteCode::LogicalOr => {
                    let idx = call_stack.frames[fi].read_byte() as usize;
                    call_stack.frames[fi].pc += 1;
                    let zero = Value::from_smi(0);
                    self.acc = if self.acc != zero || call_stack.reg(fi, idx) != zero {
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
                    let offset = call_stack.frames[fi].read_byte();
                    call_stack.frames[fi].pc += offset as usize + 1;
                }
                ByteCode::JumpIfFalse => {
                    let offset = call_stack.frames[fi].read_byte();
                    call_stack.frames[fi].pc += 1;
                    if self.acc == Value::from_smi(0) {
                        call_stack.frames[fi].pc += offset as usize;
                    }
                }
            }
        }

        self.acc
    }

    fn collect_garbage(&mut self, call_stack: &CallStack) {
        let mut roots = Vec::new();
        roots.push(self.acc);
        for frame in &call_stack.frames {
            roots.push(frame.closure);
        }
        for &val in &call_stack.slots {
            if val.is_heap_object() {
                roots.push(val);
            }
        }
        self.heap.collect(&roots);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::ByteCode;
    use crate::heap::{BytecodeFunction, Closure};

    fn make_closure(
        heap: &mut Heap,
        code: Vec<u8>,
        constant_pool: Vec<Value>,
        param_count: usize,
    ) -> Value {
        let func = heap.alloc(HeapObject::Function(BytecodeFunction {
            code,
            constant_pool,
            param_count,
            register_count: 16,
        }));
        heap.alloc(HeapObject::Closure(Closure {
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

    #[test]
    fn eval_recursive_factorial() {
        let mut iso = Isolate::new();
        let result = iso.eval(
            "function factorial(n) { if (n < 1) { return 1; } return n * factorial(n - 1); } factorial(10);",
        );
        assert_eq!(iso.format_value(result), "3628800");
    }

    #[test]
    fn eval_recursive_fibonacci() {
        let mut iso = Isolate::new();
        let result = iso.eval(
            "function fib(n) { if (n < 2) { return n; } return fib(n - 1) + fib(n - 2); } fib(10);",
        );
        assert_eq!(iso.format_value(result), "55");
    }

    #[test]
    fn eval_nested_function_calls() {
        let mut iso = Isolate::new();
        let result = iso.eval(
            "function square(x) { return x * x; } \
             function sum_of_squares(a, b) { return square(a) + square(b); } \
             sum_of_squares(3, 4);",
        );
        assert_eq!(iso.format_value(result), "25");
    }

    #[test]
    fn eval_three_level_nested_calls() {
        let mut iso = Isolate::new();
        let result = iso.eval(
            "function square(x) { return x * x; } \
             function sum_of_squares(a, b) { return square(a) + square(b); } \
             function hypotenuse_squared(a, b) { return sum_of_squares(a, b); } \
             hypotenuse_squared(3, 4);",
        );
        assert_eq!(iso.format_value(result), "25");
    }

    #[test]
    fn eval_conditional_chain() {
        let mut iso = Isolate::new();
        let result = iso.eval(
            "function classify(n) { \
               if (n > 100) { return 3; } \
               if (n > 10) { return 2; } \
               if (n > 0) { return 1; } \
               return 0; \
             } \
             let a = classify(200); \
             let b = classify(50); \
             let c = classify(5); \
             let d = classify(0); \
             a * 1000 + b * 100 + c * 10 + d;",
        );
        assert_eq!(iso.format_value(result), "3210");
    }

    #[test]
    fn eval_max_clamp() {
        let mut iso = Isolate::new();
        let result = iso.eval(
            "function max(a, b) { if (a > b) { return a; } else { return b; } } \
             function clamp(val, lo, hi) { \
               if (val < lo) { return lo; } \
               if (val > hi) { return hi; } \
               return val; \
             } \
             max(clamp(500, 0, 100), clamp(30, 0, 100));",
        );
        assert_eq!(iso.format_value(result), "100");
    }

    #[test]
    fn format_value_closure() {
        let mut heap = Heap::new();
        let closure = make_closure(&mut heap, vec![], vec![], 0);
        let iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.format_value(closure), "<closure>");
    }

    #[test]
    fn format_value_function() {
        let mut heap = Heap::new();
        let func = heap.alloc(HeapObject::Function(BytecodeFunction {
            code: vec![],
            constant_pool: vec![],
            param_count: 0,
            register_count: 0,
        }));
        let iso = Isolate::with_heap(heap, 1024);
        assert_eq!(iso.format_value(func), "<function>");
    }

    #[test]
    fn display_isolate() {
        let mut iso = Isolate::new();
        iso.eval("let x = 42;");
        let display = format!("{}", iso);
        assert!(display.contains("42"));
        assert!(display.contains("heap_size"));
    }

    #[test]
    fn gc_preserves_closures_during_execution() {
        let mut iso = Isolate::new();
        iso.gc_threshold = 8;
        let result = iso.eval(
            "function add(a, b) { return a + b; } \
             let x = add(1, 2); \
             let y = add(3, 4); \
             let z = add(5, 6); \
             x + y + z;",
        );
        assert_eq!(iso.format_value(result), "21");
    }

    #[test]
    fn closure_shares_function() {
        let mut heap = Heap::new();
        let func = heap.alloc(HeapObject::Function(BytecodeFunction {
            code: vec![ByteCode::Return as u8],
            constant_pool: vec![],
            param_count: 0,
            register_count: 0,
        }));
        let closure_a = heap.alloc(HeapObject::Closure(Closure {
            function: func,
            upvalues: vec![],
        }));
        let closure_b = heap.alloc(HeapObject::Closure(Closure {
            function: func,
            upvalues: vec![],
        }));
        let func_a = heap.read_closure(closure_a).function;
        let func_b = heap.read_closure(closure_b).function;
        assert_eq!(func_a, func_b);
    }
}
