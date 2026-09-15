# Mission: Build a V8-Style Isolate Runtime in Rust

## Why
Understand how V8 works from the inside out by building a minimal isolate runtime from scratch. Started with the full compilation pipeline (lexing, parsing, bytecode emission, VM execution), now moving toward the runtime infrastructure that makes V8 an isolate-based engine: tagged values, heap-allocated objects, segregated heaps, garbage collection, and memory isolation between contexts.

## Success looks like
- ~~Can explain what bytecode is and why it exists~~ ✓
- ~~Can compile an AST into a flat bytecode instruction sequence~~ ✓
- ~~Can execute bytecode on a register-based VM~~ ✓
- ~~Understands V8's accumulator model and how it differs from a stack machine~~ ✓
- ~~Can implement functions with call frames and parameter passing~~ ✓
- ~~Can implement control flow with comparison opcodes, jumps, and backpatching~~ ✓
- Can implement tagged value representation (Smi vs HeapObject)
- Can implement a heap with typed object allocation
- Can build an isolate struct that owns its VM and heap
- Can implement mark-sweep garbage collection
- Can run multiple isolates independently in one process

## Constraints
- Working in Rust — chosen for explicit memory management, no runtime GC
- Building incrementally — each concept implemented and tested before the next
- Learns by building, not just reading

## Out of scope (for now)
- JIT compilation
- Full JavaScript semantics
- Pointer compression (will revisit after basic tagging works)
