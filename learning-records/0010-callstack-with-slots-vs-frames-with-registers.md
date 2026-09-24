# From Frames-with-Registers to a CallStack-with-Slots

This lesson explains the architectural shift in commit `7bc9a78` ("refactor: architecture suggested"), where the VM moved from each frame owning a fixed-size register array to a single flat `CallStack` whose `Vec<Value>` slots are shared across all frames.

---

## The old architecture: Frame owns its registers

```rust
// BEFORE (commit 0a52fb5)
struct Frame<'a> {
    ip: usize,
    reg: [Value; 256],      // <-- fixed-size array, lives inside the frame
    closure: &'a Value,      // <-- borrowed pointer into... somewhere
}
```

Every time the VM called a function, it pushed a new `Frame` onto `Vec<Frame>`. Each frame carried its own `[Value; 256]` — a full 256-slot register file, inline.

### How a function call worked

```
Stack before call:              Stack after call:
┌─────────────────────┐         ┌─────────────────────┐
│ Frame 0             │         │ Frame 0             │
│  ip: 11             │         │  ip: 11             │
│  reg: [10, 20, ...] │  ──►    │  reg: [10, 20, ...] │
│  closure: &val      │         │  closure: &val      │
└─────────────────────┘         ├─────────────────────┤
                                │ Frame 1             │
                                │  ip: 0              │
                                │  reg: [10, 20, ...] │  ← args copied in
                                │  closure: &val      │
                                └─────────────────────┘
```

The `Call` handler would:
1. Create a fresh `[Value::default(); 256]`.
2. Copy arguments from the caller's registers into the new array.
3. Push the entire `Frame` (with its 256-slot array) onto the stack.

### Problems with this design

**1. Every frame burns 2 KB regardless of need.**
`[Value; 256]` = 256 x 8 bytes = 2048 bytes. A function using 3 registers still allocates 256 slots. For recursive calls like `factorial(10)`, that's 10 frames x 2 KB = 20 KB of mostly-zeroed registers.

**2. Terrible cache behavior.**
Each frame's register array is a separate chunk of 2 KB. When the VM accesses `stack[fi].reg[idx]`, the CPU must hop between these disjoint memory regions. In a tight recursive loop, the working set scatters across memory — every frame push is a cache miss for the new register file.

**3. Copying the whole array on function calls.**
Even though Rust initializes `[Value::default(); 256]` and then only writes `param_count` slots, the entire 256-element array must be zero-initialized on the stack before the copy. This is pure waste.

**4. Lifetime headaches.**
`Frame<'a>` borrows its closure as `&'a Value`. This ties the frame's lifetime to wherever that Value lives. The compiler enforces that frames can't outlive the closure — which sounds safe but means you can't freely mutate the heap (which owns closures) while frames exist. The old code worked around this by having closures live in the `Program` struct outside the heap, but that created the stale-pointer problem documented in lesson 0009.

---

## The new architecture: CallStack with flat slots

```rust
// AFTER (commit 7bc9a78)
struct CallStack {
    slots: Vec<Value>,       // <-- one flat array for ALL frames
    frames: Vec<Frame>,      // <-- lightweight descriptors
}

struct Frame {
    base_offset: usize,      // where my registers start in `slots`
    pc: usize,               // program counter (was `ip`)
    closure: Value,           // owned Value, not a borrow
}
```

The key insight: **registers are just values at known offsets**. There's no reason each frame needs its own array. Instead, a single `Vec<Value>` holds every frame's registers contiguously, and each `Frame` just remembers where its window starts (`base_offset`).

### How a function call works now

```
slots: [  frame0_regs...  |  frame1_regs...  |  frame2_regs...  ]
        ^                  ^                   ^
        base=0             base=5              base=8

frames: [ Frame{base:0, pc:11, closure:...},
          Frame{base:5, pc:0,  closure:...},
          Frame{base:8, pc:3,  closure:...} ]
```

The `Call` handler now:
1. Records `callee_base = slots.len()` — the current end of the slots vector.
2. Extends `slots` by exactly `register_count` (not 256 — only what the function actually needs).
3. Copies `param_count` arguments from the caller's window into the callee's window.
4. Pushes a lightweight `Frame { base_offset: callee_base, pc: 0, closure: closure_val }`.

```rust
// The actual code (isolate.rs, lines 206-232)
ByteCode::Call => {
    let func_reg = self.read_byte(&call_stack, fi) as usize;
    call_stack.frames[fi].pc += 1;
    let arg_end = self.read_byte(&call_stack, fi) as usize;
    call_stack.frames[fi].pc += 1;

    let closure_val = call_stack.reg(fi, func_reg);
    let (param_count, callee_reg_count) = {
        let c = self.heap.read_closure(closure_val);
        let f = self.heap.read_function(c.function);
        (f.param_count, f.register_count)
    };

    let callee_base = call_stack.slots.len();
    call_stack.slots.resize(callee_base + callee_reg_count, Value::default());
    for i in 0..param_count {
        call_stack.slots[callee_base + i] = call_stack.reg(fi, arg_end - param_count + i);
    }
    call_stack.frames.push(Frame {
        base_offset: callee_base,
        pc: 0,
        closure: closure_val,
    });
}
```

### Popping a frame is just truncation

```rust
fn pop_frame(&mut self) {
    if let Some(frame) = self.frames.pop() {
        self.slots.truncate(frame.base_offset);
    }
}
```

No deallocation, no destructor. `truncate` adjusts the length of the Vec without freeing the backing memory — the capacity stays, ready for the next call. This is essentially free.

---

## Register access: the indirection

In the old code, accessing register `idx` of frame `fi` was direct:
```rust
// OLD: direct array index
stack[fi].reg[idx]
```

In the new code, it's `base_offset + idx`:
```rust
// NEW: offset into flat array
fn reg(&self, fi: usize, idx: usize) -> Value {
    self.slots[self.frames[fi].base_offset + idx]
}
```

This is one extra addition per register access. In practice, the `base_offset` for the current frame is read once and reused across all register operations in that bytecode dispatch — the CPU keeps it in a register. The cost is negligible compared to the cache wins.

---

## Side-by-side: what changed in each part

| Aspect | Old (Frame owns registers) | New (CallStack with slots) |
|--------|---------------------------|---------------------------|
| Register storage | `[Value; 256]` per frame | One `Vec<Value>` for all frames |
| Memory per frame | 2 KB fixed | Only `register_count` slots (typically 4-16 values) |
| Frame struct size | ~2072 bytes (regs + ip + pointer) | 24 bytes (base + pc + closure) |
| Function call cost | Zero-init 256 slots + copy args | Extend Vec by N + copy args |
| Frame pop cost | Drop 2 KB array | `truncate()` (adjust length field) |
| Register access | `stack[fi].reg[idx]` | `slots[frame.base_offset + idx]` |
| Closure reference | `&'a Value` (borrowed) | `Value` (owned, Copy) |
| GC root scanning | Scan 256 slots per frame | Scan only live `slots` range |
| Cache behavior | Each frame's regs disjoint in memory | All registers contiguous |

---

## Why this matters for GC

The old garbage collector had to scan every frame's full `[Value; 256]`:

```rust
// OLD GC root scanning
for frame in stack.iter() {
    for val in frame.reg.iter() {      // all 256, even if only 3 used
        if val.is_heap_object() {
            live.insert(val.heap_offset());
        }
    }
}
```

The new GC scans only the live portion of the slots vector plus the closure values:

```rust
// NEW GC root scanning
fn collect_garbage(&mut self, call_stack: &CallStack) {
    let mut roots = Vec::new();
    roots.push(self.acc);
    for frame in &call_stack.frames {
        roots.push(frame.closure);       // each closure is a root
    }
    for &val in &call_stack.slots {      // only allocated slots
        if val.is_heap_object() {
            roots.push(val);
        }
    }
    self.heap.collect(&roots);
}
```

With 10 active frames using 5 registers each, the old code scanned 2560 values. The new code scans 50 slots + 10 closures = 60 values. That's a 40x reduction in GC root scanning work.

---

## The Closure owns its bytecode now

The other half of this refactor moved functions and closures onto the heap (documented in lesson 0009). In the old code, `Frame` held a borrowed reference to a `Program` struct that lived outside the heap. Now:

- `BytecodeFunction` lives on the heap as a `HeapObject::Function`
- `Closure` lives on the heap as a `HeapObject::Closure`, pointing to its `BytecodeFunction`
- `Frame.closure` is a `Value` (a tagged heap pointer) — no lifetimes needed
- The GC traces through `Closure → BytecodeFunction → constant_pool` naturally

This means the VM can freely allocate and collect during execution without worrying about dangling references from frames into a separate program structure.

---

## What V8 actually does

V8's Ignition interpreter uses exactly this model. The V8 `RegisterList` is a contiguous range in the interpreter's register file, addressed by `base + index`. Each `InterpretedFrame` on the native stack holds the base pointer, the bytecode offset, and the closure — the same three fields as our `Frame`.

The key difference: V8's register file lives on the native C++ stack (using `alloca`-style allocation), not a `Vec`. This gives it true zero-cost allocation and deallocation — pushing a frame is just bumping the stack pointer. Our `Vec<Value>` approach is the portable equivalent: the Vec's backing buffer serves as our "interpreter stack."

---

## Walkthrough: `factorial(3)`

To make this concrete, here's what happens inside the CallStack when running `factorial(3)`:

```
═══ Initial state ═══
slots: [ closure_val ]     (1 slot for the top-level closure register)
frames: [ Frame{base:0, pc:0} ]

═══ After "let x = factorial(3)" calls factorial ═══
slots: [ closure_val | 3, _, _, _ ]    (factorial needs 4 registers: n, temp, fact_closure, scratch)
frames: [ Frame{base:0, pc:8},         (caller paused after Call instruction)
          Frame{base:1, pc:0} ]        (factorial starts)

═══ factorial(3) calls factorial(2) ═══
slots: [ closure_val | 3, _, _, _ | 2, _, _, _ ]
frames: [ Frame{base:0, pc:8},
          Frame{base:1, pc:?},
          Frame{base:5, pc:0} ]

═══ factorial(2) calls factorial(1) ═══
slots: [ closure_val | 3, _, _, _ | 2, _, _, _ | 1, _, _, _ ]
frames: [ Frame{base:0, pc:8},
          Frame{base:1, pc:?},
          Frame{base:5, pc:?},
          Frame{base:9, pc:0} ]

═══ factorial(1) returns 1 (base case) ═══
slots: [ closure_val | 3, _, _, _ | 2, _, _, _ ]   ← truncated back to base:9
frames: [ Frame{base:0}, Frame{base:1}, Frame{base:5} ]
acc = 1

═══ Unwind continues... ═══
Each return: pop frame, truncate slots, acc holds the result.
Final acc = 6
```

All register values live in one contiguous array. Each frame is just a 24-byte descriptor saying "my registers start at offset N." Pushing is cheap (extend the Vec). Popping is cheaper (adjust the length).
