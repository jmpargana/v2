use crate::value::Value;

#[derive(Debug)]
pub enum HeapObject {
    String(HeapString),
    Function(BytecodeFunction),
    Closure(HeapClosure),
}

#[derive(Debug)]
pub struct HeapString {
    pub data: String,
}

#[derive(Debug)]
pub struct HeapClosure {
    pub function: Value,
    pub upvalues: Vec<Value>,
}

#[derive(Debug)]
pub struct BytecodeFunction {
    pub code: Vec<u8>,
    pub cons: Vec<Value>,
    pub param_count: usize,
    pub reg_count: usize,
}

pub struct Heap {
    objects: Vec<Option<HeapObject>>,
}

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

    pub fn alloc_string(&mut self, s: &str) -> Value {
        self.alloc(HeapObject::String(HeapString {
            data: s.to_string(),
        }))
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

    pub fn collect(&mut self, roots: &[Value]) {
        let mut marked = vec![false; self.objects.len()];
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
            Some(HeapObject::String(_)) => {}
            Some(HeapObject::Function(f)) => {
                for &v in &f.cons {
                    self.mark(marked, v);
                }
            }
            Some(HeapObject::Closure(c)) => {
                self.mark(marked, c.function);
                for &v in &c.upvalues {
                    self.mark(marked, v);
                }
            }
            None => {}
        }
    }

    pub fn get(&self, val: Value) -> Option<&HeapObject> {
        self.objects.get(val.heap_offset())?.as_ref()
    }

    pub fn patch_closure(&mut self, closure_val: Value, new_func: Value) {
        match &mut self.objects[closure_val.heap_offset()] {
            Some(HeapObject::Closure(c)) => c.function = new_func,
            other => panic!("expected Closure to patch, got {:?}", other),
        }
    }

    pub fn is_over_threshold(&self, threshold: usize) -> bool {
        self.objects.len() >= threshold
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }
}
