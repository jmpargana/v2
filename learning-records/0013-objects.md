# Objects — Property Maps on the Heap

This lesson adds JavaScript objects to the VM. Objects are the first **user-visible compound data type** — everything before this was either a number, a string, or a function. An object holds named properties, each mapping a string key to a Value. This requires a new heap type, three new bytecodes, new syntax for literals and property access, and an update to the GC.

The key architectural insight: an object is a `HashMap<String, Value>` on the heap, pointed to by a tagged Value like everything else. No special treatment from the VM — the same alloc/read/GC-trace pattern used for strings and closures extends to objects.

---

## The new heap type: JSObject

```rust
use std::collections::HashMap;

pub struct JSObject {
    pub properties: HashMap<String, Value>,
}

pub enum HeapObject {
    String(HeapString),
    Function(BytecodeFunction),
    Closure(Closure),
    Object(JSObject),        // NEW
}
```

A JSObject is just a bag of named properties. Each property name is a `String` key, each value is a `Value` (which can be a Smi, a heap string, another object, a closure — anything).

### Why HashMap and not a Vec?

A `Vec<(String, Value)>` would also work and is simpler. The tradeoff:

| Approach | Lookup | Insert | Memory |
|----------|--------|--------|--------|
| `Vec<(String, Value)>` | O(n) linear scan | O(1) amortized push | Compact |
| `HashMap<String, Value>` | O(1) average | O(1) amortized | ~2x overhead |

For a teaching VM, HashMap is the right choice: it matches the mental model ("objects are dictionaries"), the code is clean, and performance doesn't matter yet. V8 uses neither — it uses **hidden classes** (Maps) for O(1) access without hashing, which we'll discuss later.

---

## Three new bytecodes

```rust
#[repr(u8)]
pub enum ByteCode {
    // ... existing opcodes (0–21) ...
    CreateObject = 22,   // allocate empty JSObject, result in acc
    SetProperty  = 23,   // obj[constants[name_idx]] = acc; operands: name_idx, obj_reg
    GetProperty  = 24,   // acc = obj[constants[name_idx]]; operands: name_idx, obj_reg
}
```

These three instructions are enough for object literals, property reads, and property writes.

### VM handlers

```rust
ByteCode::CreateObject => {
    self.acc = self.heap.alloc(HeapObject::Object(JSObject {
        properties: HashMap::new(),
    }));
}

ByteCode::SetProperty => {
    let name_idx = call_stack.frames[fi].read_byte() as usize;
    call_stack.frames[fi].pc += 1;
    let obj_reg = call_stack.frames[fi].read_byte() as usize;
    call_stack.frames[fi].pc += 1;

    let name_val = call_stack.frames[fi].read_constant(name_idx);
    let name = self.heap.read_string(name_val).to_string();
    let obj_val = call_stack.reg(fi, obj_reg);
    self.heap.set_property(obj_val, &name, self.acc);
}

ByteCode::GetProperty => {
    let name_idx = call_stack.frames[fi].read_byte() as usize;
    call_stack.frames[fi].pc += 1;
    let obj_reg = call_stack.frames[fi].read_byte() as usize;
    call_stack.frames[fi].pc += 1;

    let name_val = call_stack.frames[fi].read_constant(name_idx);
    let name = self.heap.read_string(name_val).to_string();
    let obj_val = call_stack.reg(fi, obj_reg);
    self.acc = self.heap.get_property(obj_val, &name);
}
```

Note: property names are stored as heap strings in the constant pool. The bytecode operand `name_idx` is a constant pool index pointing to the string key, not the string itself. This means the same property name used multiple times shares one constant pool entry.

### Heap helpers

```rust
impl Heap {
    pub fn read_object(&self, val: Value) -> &JSObject {
        match &self.objects[val.heap_offset()] {
            Some(HeapObject::Object(o)) => o,
            other => panic!("expected Object, got {:?}", other),
        }
    }

    pub fn set_property(&mut self, obj_val: Value, name: &str, value: Value) {
        match &mut self.objects[obj_val.heap_offset()] {
            Some(HeapObject::Object(o)) => {
                o.properties.insert(name.to_string(), value);
            }
            other => panic!("expected Object, got {:?}", other),
        }
    }

    pub fn get_property(&self, obj_val: Value, name: &str) -> Value {
        match &self.objects[obj_val.heap_offset()] {
            Some(HeapObject::Object(o)) => {
                *o.properties.get(name).expect(&format!("undefined property: {}", name))
            }
            other => panic!("expected Object, got {:?}", other),
        }
    }
}
```

---

## New tokens

Two new tokens for object syntax:

```rust
pub enum SymbolKind {
    // ... existing variants ...
    Dot,     // . (property access)
    Colon,   // : (object literal key-value separator)
}
```

Lexer additions:

```rust
'.' => {
    res.push(Symbol { kind: SymbolKind::Dot, int_val: None, str_val: String::new() });
    i += 1;
}
':' => {
    res.push(Symbol { kind: SymbolKind::Colon, int_val: None, str_val: String::new() });
    i += 1;
}
```

---

## New AST nodes

### Object literal (expression)

```rust
pub enum Expr {
    // ... existing variants ...
    ObjectLit {
        properties: Vec<(String, Expr)>,
    },
}
```

A list of key-value pairs. Keys are always string identifiers (no computed property names yet).

### Property access (expression)

```rust
pub enum Expr {
    // ... existing variants ...
    PropertyAccess {
        object: Box<Expr>,
        property: String,
    },
}
```

The `object` is any expression (usually an `Ident`), and `property` is the dot-accessed name.

### Property assignment (statement)

```rust
pub enum Stmt {
    // ... existing variants ...
    PropertyAssign {
        object: String,
        property: String,
        value: Expr,
    },
}
```

For simplicity, the object is identified by name (not an arbitrary expression). This handles `obj.x = 5;` but not `getObj().x = 5;`.

---

## Parser

### Object literals

Object literals start with `{` — but so do block bodies (if/while/for/function). The parser needs to distinguish them. In statement position, `{` starts a block. In expression position (inside `parse_factor`), `{` starts an object literal:

```rust
// In parse_factor:
SymbolKind::LBrace => {
    self.pop();  // consume {
    let mut properties = Vec::new();
    if self.peek() != SymbolKind::RBrace {
        loop {
            let key = self.expect(SymbolKind::Ident).str_val;
            self.expect(SymbolKind::Colon);
            let value = self.parse_expr();
            properties.push((key, value));
            if self.peek() == SymbolKind::Comma {
                self.pop();
            } else {
                break;
            }
        }
    }
    self.expect(SymbolKind::RBrace);
    Expr::ObjectLit { properties }
}
```

### Property access

Property access is a **postfix** operation: `obj.x` is the expression `obj` followed by `.x`. This is parsed after the primary expression in `parse_factor`, using a loop to handle chained access like `a.b.c`:

```rust
fn parse_factor(&mut self) -> Expr {
    let mut expr = match self.peek() {
        // ... existing arms (Ident, Smi, LBrace, etc.) ...
    };

    // Postfix: dot access
    while self.peek() == SymbolKind::Dot {
        self.pop();  // consume .
        let property = self.expect(SymbolKind::Ident).str_val;
        expr = Expr::PropertyAccess {
            object: Box::new(expr),
            property,
        };
    }

    expr
}
```

### Property assignment

In `parse_expr_stmt`, extend the lookahead to detect `ident.ident = expr;`:

```rust
fn parse_expr_stmt(&mut self) -> Stmt {
    if self.peek() == SymbolKind::Ident {
        // Check for property assignment: obj.prop = expr;
        if self.pos + 2 < self.syms.len()
            && self.syms[self.pos + 1].kind == SymbolKind::Dot
            && self.syms[self.pos + 2].kind == SymbolKind::Ident
        {
            // Peek further to see if it's assignment
            if self.pos + 3 < self.syms.len()
                && self.syms[self.pos + 3].kind == SymbolKind::Assign
            {
                let object = self.pop().str_val;
                self.expect(SymbolKind::Dot);
                let property = self.expect(SymbolKind::Ident).str_val;
                self.expect(SymbolKind::Assign);
                let value = self.parse_expr();
                self.expect(SymbolKind::Semi);
                return Stmt::PropertyAssign { object, property, value };
            }
        }

        // Check for variable assignment: x = expr;
        if self.pos + 1 < self.syms.len()
            && self.syms[self.pos + 1].kind == SymbolKind::Assign
        {
            // ... existing assignment logic ...
        }
    }
    // ... fall through to expression statement ...
}
```

---

## Compiler

### Object literals

```rust
Expr::ObjectLit { properties } => {
    // 1. Create empty object → acc
    self.code.push(ByteCode::CreateObject as u8);

    // 2. Store object reference in a temp register
    let obj_reg = self.alloc_reg();
    self.code.push(ByteCode::Star as u8);
    self.code.push(obj_reg as u8);

    // 3. For each property: compile value, then SetProperty
    for (key, value_expr) in properties {
        self.compile_expr(value_expr, heap);

        // Property name goes into constant pool as a heap string
        let name_idx = self.constant_pool.len();
        self.constant_pool.push(heap.alloc_string(key));

        self.code.push(ByteCode::SetProperty as u8);
        self.code.push(name_idx as u8);
        self.code.push(obj_reg as u8);
    }

    // 4. Reload object into acc (SetProperty may have clobbered it)
    self.code.push(ByteCode::Ldar as u8);
    self.code.push(obj_reg as u8);
}
```

After compilation, the object is in the accumulator. The caller (e.g., `VarDecl`) will `Star` it into a register.

### Property access

```rust
Expr::PropertyAccess { object, property } => {
    self.compile_expr(object, heap);
    let obj_reg = self.alloc_reg();
    self.code.push(ByteCode::Star as u8);
    self.code.push(obj_reg as u8);

    let name_idx = self.constant_pool.len();
    self.constant_pool.push(heap.alloc_string(property));

    self.code.push(ByteCode::GetProperty as u8);
    self.code.push(name_idx as u8);
    self.code.push(obj_reg as u8);
}
```

### Property assignment

```rust
Stmt::PropertyAssign { object, property, value } => {
    self.compile_expr(value, heap);

    let obj_reg = self.symbols[object];
    let name_idx = self.constant_pool.len();
    self.constant_pool.push(heap.alloc_string(property));

    self.code.push(ByteCode::SetProperty as u8);
    self.code.push(name_idx as u8);
    self.code.push(obj_reg as u8);
}
```

---

## Bytecode trace

Source:

```javascript
let point = { x: 10, y: 20 };
let sum = point.x + point.y;
```

Compiled bytecode:

```
addr  instruction              effect
────  ───────────────────────  ──────────────────────────────
0000  CreateObject              acc = {} (empty JSObject on heap)
0002  Star r0                   r0 = {}

0004  LdaSmi [0] (10)          acc = 10
0006  SetProperty [1] r0        r0.x = 10    (constants[1] = "x")
0009  LdaSmi [2] (20)          acc = 20
0011  SetProperty [3] r0        r0.y = 20    (constants[3] = "y")

0014  Ldar r0                   acc = point (reload object)
0016  Star r1                   r1 = point   (let point = ...)

0018  Ldar r1                   acc = point
0020  Star r2                   r2 = point (stash for GetProperty)
0022  GetProperty [1] r2        acc = point.x = 10
0025  Star r3                   r3 = 10 (stash for Add)

0027  Ldar r1                   acc = point
0029  Star r4                   r4 = point (stash for GetProperty)
0031  GetProperty [3] r4        acc = point.y = 20
0034  Add r3                    acc = 10 + 20 = 30
0036  Star r5                   r5 = 30  (let sum = 30)
```

Constants: `[Smi(10), HeapString("x"), Smi(20), HeapString("y")]`

Note how property names `"x"` and `"y"` sit in the constant pool alongside numeric literals. The same constant pool index is reused when the property is both written (during object creation) and read (during `point.x`).

---

## GC update

JSObject properties contain Values that may point to other heap objects. The GC must trace through them:

```rust
fn mark(&self, marked: &mut Vec<bool>, val: Value) {
    if !val.is_heap_object() { return; }
    let idx = val.heap_offset();
    if idx >= self.objects.len() || marked[idx] { return; }
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
        Some(HeapObject::Object(o)) => {            // NEW
            for &v in o.properties.values() {
                self.mark(marked, v);
            }
        }
        None => {}
    }
}
```

This handles nested objects naturally: if `obj.inner` points to another JSObject, the GC traces through it recursively. The same pattern that traces closures → functions → constant pools now traces objects → property values → nested objects.

---

## How V8 does it: hidden classes

Our HashMap approach is simple but slow for real-world code. V8 uses a radically different design: **hidden classes** (called "Maps" internally).

### The problem with HashMap

Every property lookup hashes the key string, probes the table, and compares strings. For `point.x` in a hot loop, this happens every iteration. In JavaScript, objects with the same shape (same properties added in the same order) are extremely common:

```javascript
let p1 = { x: 1, y: 2 };
let p2 = { x: 3, y: 4 };
let p3 = { x: 5, y: 6 };
// All three have the same shape: { x, y }
```

### V8's solution: hidden classes (Maps)

Instead of a HashMap per object, V8 gives each object a pointer to a **hidden class** that describes its shape:

```
HiddenClass "point"
  x → offset 0
  y → offset 1

Object p1: [map: @HC_point, slot0: 1, slot1: 2]
Object p2: [map: @HC_point, slot0: 3, slot1: 4]
Object p3: [map: @HC_point, slot0: 5, slot1: 6]
```

Property access becomes a fixed-offset load: `p1.x` reads `slot[0]`. No hashing, no string comparison. All objects with the same shape share one hidden class.

### Inline caches (ICs)

Hidden classes alone aren't enough — the VM still needs to look up the offset each time. V8 adds **inline caches**: the first time `point.x` executes, the VM records "for hidden class @HC_point, x is at offset 0." On subsequent executions, it checks the hidden class and does a direct slot load. This turns property access into a comparison + memory load — as fast as a struct field access in C.

### Transition chains

When you add a property, V8 creates a **transition** from one hidden class to another:

```
{}  ──(add x)──►  {x}  ──(add y)──►  {x, y}
```

Objects that add properties in the same order follow the same transition chain and share hidden classes. This is why V8 docs recommend consistent object shapes — adding properties in different orders creates different hidden classes.

### Comparison

| Aspect | Our HashMap | V8 Hidden Classes + IC |
|--------|-------------|----------------------|
| Property lookup | Hash + probe + string compare | Map check + offset load |
| Memory per object | HashMap overhead (~40 bytes + entries) | Map pointer + dense slots (~8 bytes per property) |
| Objects with same shape | Each has its own HashMap | Share one hidden class |
| Adding a property | HashMap insert | Transition to new hidden class |
| GC tracing | Iterate HashMap values | Iterate dense slots |
| Implementation complexity | ~20 lines | ~5,000 lines in V8 |

For a teaching VM, HashMap is the right choice. It teaches the concept (objects are key-value maps) without the optimization complexity. Hidden classes are a natural follow-up lesson when you want to explore why V8 is fast.

---

## What this unlocks

With objects, the VM can express:

- **Structured data:** `let point = { x: 10, y: 20 };`
- **Nested objects:** `let rect = { origin: { x: 0, y: 0 }, size: { w: 100, h: 50 } };`
- **Property mutation:** `point.x = 30;`
- **Object-returning functions:** `function makePoint(x, y) { return { x: x, y: y }; }`
- **Data aggregation:** passing structured data between functions

Still missing (future lessons):
- **Prototypes:** the mechanism for inheritance — `obj.__proto__` links to a parent object, and property lookup walks the chain
- **`this`:** inside a method, `this` refers to the receiver object — requires call-site binding
- **Methods:** functions stored as object properties and called with `obj.method()`
- **Computed property names:** `obj[expr]` requires evaluating an expression as a key
- **`delete`:** removing properties from objects
- **`in` operator:** testing whether a property exists
- **`for...in`:** iterating over an object's enumerable properties
