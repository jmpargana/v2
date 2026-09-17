# User implemented mark-compact GC, string concatenation, and cons-in-frame

Implemented full mark-compact garbage collection: Heap.collect takes live offset set, compacts to new Vec<u8>, returns remap. VM.collect_garbage scans roots (acc, all frame regs, all frame cons), calls collect, applies remap. GC triggered in Add opcode after string concatenation when heap exceeds threshold.

String concatenation via Add opcode: checks if both operands are heap objects, reads both strings, concatenates, allocates result on heap. This is the first runtime heap allocation — compile-time strings are constant-pool values, but concat results are new objects that can become garbage.

Adopted cons-in-frame pattern: each Frame clones program.cons on creation. GC scans and remaps frame.cons. Has the stale-Program.cons-on-re-call issue discussed but not yet triggered by tests.

fde now returns Value instead of i64. Added format_value method for display.

## Evidence
- Heap::object_size, Heap::collect (mark-compact), Heap::is_over_threshold
- VM::collect_garbage scans acc + frame.reg + frame.cons, applies remap
- Add opcode handles string concat with runtime heap allocation
- GC triggered after string concat when heap > threshold
- string_concatenation test: "hello" + " world" = "hello world"
- 51 tests all passing

## Implications
- Ready for Isolate struct — VM already owns heap, acc, gc_threshold
- The cons-in-frame pattern works but has stale pointer risk on repeated calls after GC
- Should consider moving to cons_table (Vec<Vec<Value>>) to eliminate stale-clone issue
- Loops (while) would exercise GC properly — currently only one concat per test
