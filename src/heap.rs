use std::collections::{HashMap, HashSet};

use crate::value::Value;

pub struct Heap {
    objects: Vec<Option<HeapObject>>,
}

pub enum HeapObject {
    String(HeapString),
    Function(BytecodeFunction),
    Closure(HeapClosure),
}

pub struct HeapString {
    pub data: String,
}

pub struct HeapClosure {
    pub function: Value,
    // TODO: implement later the open and closed
    pub upvalues: Vec<Value>,
}

pub struct BytecodeFunction {
    pub code: Vec<u8>,
    pub cons: Vec<Value>,
    pub param_count: usize,
    // Why?
    pub reg_count: usize,
}

#[repr(u8)]
pub enum InstanceType {
    String = 1,
}

// TODO: abstract write and read value without hardcoded function.
impl Heap {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn alloc(&mut self, obj: HeapObject) -> Value {
        if let Some(idx) = self.objects.iter().position(|o| o.is_none()) {
            self.objects[idx] = Some(obj);
            return Value::from_heap(idx);
        }
        let idx = self.objects.len();
        self.objects.push(Some(obj));
        Value::from_heap(idx)
    }

    pub fn read_string(&self, val: Value) -> &str {
        match &self.objects[val.heap_offset()] {
            Some(HeapObject::String(s)) => &s.data,
            other => panic!("expected String, got {:?}", other),
        }
    }

    pub fn read_function(&self, val: Value) -> &BytecodeFunction {
        match &self.objects[val.heap_offset()] {
            Some(HeapObject::Function(f)) => f,
            other => panic!("expected Function, got {:?}", other),
        }
    }

    pub fn read_closure(&self, val: Value) -> &HeapClosure {
        match &self.objects[val.heap_offset()] {
            Some(HeapObject::Closure(c)) => c,
            other => panic!("expected Closure, got {:?}", other),
        }
    }

    // Mark + Compact
    pub fn collect(&mut self, roots: &[Value]) {
        let mut marked = [false; self.objects.len()];

        for &root in roots {
            self.mark(&mut marked, root);
        }

        for i in 0..self.objects.len() {
            if !marked[i] {
                self.objects[i] = None;
            }
        }
    }

    fn mark(&self, marked: &mut Vec<bool>, val: Value) {
        if !val.is_heap_object() {
            return;
        }
        let idx = val.heap_offset();

        if idx >= self.objects.len() || marked[idx] {
            return;
        }

        marked[idx] = true;

        match &self.objects[idx] {
            Some(HeapObject::String(_)) => {
                // nothing to be done
            }
            Some(HeapObject::Function(c)) => {
                for &v in &c.cons {
                    self.mark(marked, v);
                }
            }
            Some(HeapObject::Closure(c)) => {
                self.mark(marked, c.function);
                for &v in &c.upvalues {
                    self.mark(marked, v);
                }
            }
            None => todo!(),
        }
    }
}
