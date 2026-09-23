# User has working Isolate and GC, ready for heap-allocated functions

The Isolate (VM struct) owns heap, acc, and gc_threshold. GC is mark-compact with remap. String concatenation is the first runtime heap allocation. User identified a fundamental architecture issue: Program lives outside the heap, so constant pools can't be GC-traced or updated. Cloning cons into frames creates stale pointers after compaction.

User wants to move to the V8 object graph model: BytecodeFunction and Closure as heap objects, GC traces the full object graph recursively, constant pools are inside functions on the heap.

## Evidence
- Detailed architecture paste showing the full V8-style object graph (BytecodeFunction, Closure, Upvalue, CallStack, Frame)
- Correctly identified the stale-cons-on-re-call problem
- Understands why functions on the heap solve the constant pool update issue
- Asked about Closure definition, upvalue open/closed mechanism

## Implications
- Next step is heap restructure: Vec<u8> → Vec<Option<HeapObject>> for complex object types
- Frame needs to hold a Closure Value instead of &Program
- GC changes from mark-compact with remap to recursive mark-sweep (objects don't move)
- Compiler must allocate BytecodeFunction and Closure on heap, return Value
- Upvalues needed for variable capture in nested functions — open/closed mechanism
