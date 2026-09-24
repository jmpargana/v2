# Cached Frame Pointers: Eliminating Per-Instruction Heap Lookups

This lesson explains an optimization to the bytecode dispatch loop: caching raw pointers to the current function's code and constant pool inside each `Frame`, so the interpreter avoids re-dereferencing `closure -> function` on every single bytecode instruction.

---

## The problem: redundant indirection in the hot loop

Before this change, reading a single byte of bytecode required two heap lookups:

```rust
// OLD — called on EVERY instruction dispatch
fn read_byte(&self, call_stack: &CallStack, fi: usize) -> u8 {
    let closure = self.heap.read_closure(call_stack.frames[fi].closure);  // lookup 1
    let func = self.heap.read_function(closure.function);                 // lookup 2
    func.code[call_stack.frames[fi].pc]
}
```

Each `read_closure` and `read_function` indexes into `heap.objects: Vec<Option<HeapObject>>`, pattern-matches the enum variant, and returns a reference. Per instruction, the loop called this pattern 2-3 times:

1. Once to check `code_len` (end-of-function guard)
2. Once for the opcode byte
3. Once (or more) for operand bytes and constant pool lookups

That's **4-6 heap indirections per bytecode instruction**, all to reach data that doesn't change within a frame — the closure and function are fixed from `Call` to `Return`.

---

## The fix: cache raw pointers at frame entry

The `Frame` struct now caches `*const u8` and `*const Value` pointing directly into the function's `code` and `constant_pool` buffers:

```rust
struct Frame {
    base_offset: usize,
    pc: usize,
    closure: Value,
    code: *const u8,        // points into BytecodeFunction.code's heap buffer
    code_len: usize,
    constants: *const Value, // points into BytecodeFunction.constant_pool's heap buffer
}
```

These are computed once when the frame is created (`Frame::new`), and all bytecode reads go through them:

```rust
impl Frame {
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
        }
    }

    #[inline(always)]
    fn read_byte(&self) -> u8 {
        unsafe { *self.code.add(self.pc) }
    }

    #[inline(always)]
    fn read_constant(&self, idx: usize) -> Value {
        unsafe { *self.constants.add(idx) }
    }
}
```

The dispatch loop now reads:

```rust
// NEW — zero heap lookups per instruction
let opcode = ByteCode::from(call_stack.frames[fi].read_byte());
```

Instead of:

```rust
// OLD — two heap lookups per call
let opcode = ByteCode::from(self.read_byte(&call_stack, fi));
```

---

## Why this is safe (the `unsafe` contract)

The raw pointers dereference memory owned by `Vec<u8>` and `Vec<Value>` inside a `BytecodeFunction` on the heap. Three properties keep the pointers valid:

**1. Moving a `Vec` struct doesn't move its buffer.**
`Vec<u8>` is a `(pointer, length, capacity)` triple. The actual data lives in a separate heap allocation. When the `heap.objects` Vec reallocates (on `alloc`), it moves the `Option<HeapObject>` structs — including the `BytecodeFunction` and its `Vec<u8>` field — but the `Vec<u8>`'s *buffer* stays at the same address. Our raw pointer targets the buffer, not the struct.

```
heap.objects Vec:
  ┌──────────────────────┐         heap buffer (stable):
  │ Some(Function {      │         ┌──────────────────┐
  │   code: Vec {        │────────>│ LdaSmi, 0, Star, │ <── code ptr
  │     ptr ─────────────│         │ 0, LdaSmi, 1 ... │
  │     len: 14          │         └──────────────────┘
  │     cap: 16          │
  │   }                  │
  │ })                   │
  └──────────────────────┘
         ↑
  This part moves on realloc.
  The buffer (right) does not.
```

**2. Nothing mutates the code or constant_pool during execution.**
The bytecode is compiled before `run()` is called. No instruction handler appends to, shrinks, or reallocates `code` or `constant_pool`. The Vecs are effectively frozen.

**3. GC won't collect the function.**
Each `Frame` stores its `closure` as a `Value`. The GC roots include all frame closures. The closure references the function. Therefore the function is always reachable, and `collect()` will never set it to `None` (which would drop the Vec and free the buffer).

---

## What changed in the dispatch loop

The three old helper methods on `Isolate` were deleted:

```rust
// DELETED
fn read_byte(&self, call_stack: &CallStack, fi: usize) -> u8 { ... }
fn code_len(&self, call_stack: &CallStack, fi: usize) -> usize { ... }
fn read_constant(&self, call_stack: &CallStack, fi: usize, idx: usize) -> Value { ... }
```

Their functionality moved to `Frame::read_byte()`, `Frame.code_len`, and `Frame::read_constant()`.

Every call site in the `run()` match arms was updated mechanically:

| Before | After |
|--------|-------|
| `self.read_byte(&call_stack, fi)` | `call_stack.frames[fi].read_byte()` |
| `self.code_len(&call_stack, fi)` | `call_stack.frames[fi].code_len` |
| `self.read_constant(&call_stack, fi, idx)` | `call_stack.frames[fi].read_constant(idx)` |

The `Call` handler now creates frames via `Frame::new(&self.heap, ...)` instead of a struct literal, which is where the pointer caching happens.

---

## Cost model: before vs. after

| Operation | Before | After |
|-----------|--------|-------|
| Read one opcode byte | 2 heap lookups + array index | 1 pointer deref |
| Read one operand byte | 2 heap lookups + array index | 1 pointer deref |
| Read one constant | 2 heap lookups + array index | 1 pointer deref |
| End-of-code check | 2 heap lookups + `.len()` | field comparison |
| Function call (frame push) | struct literal | 2 heap lookups (once) + cache pointers |
| Frame pop | unchanged | unchanged |

For a typical instruction like `LdaSmi 0` (opcode + operand + constant read), the old code did **6 heap lookups**. The new code does **3 pointer derefs** plus a field read — and those pointer derefs inline to single `mov` instructions.

The cost is paid once per `Call` instead of per instruction. For `factorial(10)` executing ~60 instructions across 11 frames, that's ~360 heap lookups eliminated, replaced by 11 `Frame::new` calls (22 lookups total).

---

## Why not just use references?

The natural Rust approach would be to cache `&[u8]` and `&[Value]` slices:

```rust
struct Frame<'a> {
    code: &'a [u8],
    constants: &'a [Value],
    // ...
}
```

This doesn't work because the `Add` (string concatenation) handler calls `self.heap.alloc_string()`, which borrows `self.heap` mutably. If `Frame` holds an immutable borrow into the heap, you get:

```
error[E0502]: cannot borrow `self.heap` as mutable because it is also borrowed as immutable
```

The borrow checker can't see that the slice points into a *different* allocation than the one `alloc_string` might reallocate. Raw pointers sidestep this — they're not borrows, so they don't participate in the borrow checker's analysis.

This is the standard Rust pattern for self-referential performance caches: use raw pointers when you can prove stability but the borrow checker can't.

---

## Alternatives considered

**Re-borrow per iteration:** Decode the full instruction (opcode + operands) in one block holding the immutable borrow, then drop it before executing. Halves the lookups with no `unsafe`, but still does 2 lookups per instruction instead of 0.

**Move bytecode out of the GC heap:** Store `BytecodeFunction` in a separate `Vec<BytecodeFunction>` arena on the `Isolate`, indexed by `FunctionId(usize)`. Since it's a separate field from `heap`, the borrow checker allows `&self.functions[id]` and `&mut self.heap` simultaneously. Architecturally cleanest — no `unsafe` needed — but a larger refactor that changes how the compiler, heap, and GC interact.
