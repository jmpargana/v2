use crate::value::Value;

/// A heap-allocated object. All objects on the heap are wrapped in this enum.
///
/// Current variants: String, Function (BytecodeFunction), Closure
/// Future variants will include Upvalue, JSObject, Array, HiddenClass
#[derive(Debug)]
pub enum HeapObject {
    String(HeapString),
    Function(BytecodeFunction),
    Closure(Closure),
    // Upvalue(Upvalue),          // indirection for captured variables (open → stack slot, closed → owned value)
    // Object(JSObject),          // user-visible object with HiddenClass and property slots
    // Array(JSArray),            // dense array
    // HiddenClass(HiddenClass),  // property layout descriptor, shared across same-shape objects
}

/// A heap-allocated string.
///
/// Future: interned strings, rope representation for concatenation,
/// or external strings backed by source code.
#[derive(Debug)]
pub struct HeapString {
    pub data: String,
}

/// A BytecodeFunction + captured environment. The runtime instance of a function.
/// Multiple Closures can share one BytecodeFunction (same code, different captures).
///
/// When the compiler processes `function f() { ... }`, it creates a Closure pointing
/// to the compiled BytecodeFunction and captures the relevant upvalues.
///
/// Target: upvalues will become Vec<UpvalueRef> when variable capture is implemented.
#[derive(Debug)]
pub struct Closure {
    pub function: Value,
    pub upvalues: Vec<Value>,
}

/// A compiled function template. Lives on the heap. Shared across Closures.
/// Contains the bytecode instruction stream, constant pool, and frame metadata.
/// Does not close over anything — that's the Closure's job.
///
/// The constant pool holds Values referenced by bytecode (LdaSmi [idx]):
/// literal numbers, heap-allocated strings, and nested Closure references.
/// Because it lives on the heap, the GC traces through it naturally.
#[derive(Debug)]
pub struct BytecodeFunction {
    pub code: Vec<u8>,
    pub constant_pool: Vec<Value>,
    pub param_count: usize,
    pub register_count: usize,
    // pub upvalue_descriptors: Vec<UpvalueDescriptor>,  // which parent slots to capture, open vs closed
}

/// Manages all dynamically allocated objects. The GC lives here.
///
/// Exposes alloc() as the only way to create heap objects. During collection,
/// the caller provides roots (call stack slots, globals); the GC traces the
/// object graph recursively and sweeps unreachable objects.
///
/// Currently non-moving (indexed slots with Option). Future: semi-space or
/// compacting collector, generational collection.
pub struct Heap {
    objects: Vec<Option<HeapObject>>,
    // gc_state: GCState,  // mark bits, generation info, semi-space pointers
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

    pub fn read_closure(&self, val: Value) -> &Closure {
        match &self.objects[val.heap_offset()] {
            Some(HeapObject::Closure(c)) => c,
            other => panic!("expected Closure, got {:?}", other),
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
                for &v in &f.constant_pool {
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

    pub fn is_over_threshold(&self, threshold: usize) -> bool {
        self.objects.len() >= threshold
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }
}
