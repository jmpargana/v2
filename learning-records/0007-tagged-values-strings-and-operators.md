# User implemented tagged values, heap strings, and expanded operators

Massive independent progress after Lesson 7. Implemented the full tagged Value system (Smi/HeapObject discrimination via low bit), Heap struct with alloc_string/read_string, and migrated VM entirely from i64 to Value. Then went far beyond: added string literals through the whole pipeline (lexer → parser → AST → compiler → VM), added Sub/Div arithmetic, all comparison operators (<=, >=, !=), logical operators (&&, ||, !), and proper operator precedence in the parser.

## Evidence
- Value newtype over u64 with from_smi/as_smi/from_heap/heap_offset, Display, Default
- Heap with alloc_string (type byte + u32 LE length + UTF-8 data) and read_string
- VM takes Heap in constructor, all opcodes unwrap/wrap with Value::from_smi
- TestEqual dispatches to heap.read_string for heap object comparison
- 21 opcodes total: Push, Add, Sub, Mul, Div, LdaSmi, Ldar, Star, Call, Return, TestEqual, TestLess, TestGreater, TestLessEqual, TestGreaterEqual, TestNotEqual, LogicalAnd, LogicalOr, LogicalNot, Jump, JumpIfFalse
- Parser has full precedence: logical_or → logical_and → comparison → additive → term → factor
- Lexer handles string literals ("..."), <=, >=, !=, &&, ||, !
- Compiler allocates strings on heap at compile time, stores heap pointer in cons pool
- Call opcode now takes third byte (arg_end) for parameter copying
- 50 tests all passing

## Implications
- Ready for garbage collection — heap only grows, no way to reclaim dead objects
- Heap walking requires object_size calculation based on type byte — self-describing objects
- String constants in cons pool are heap roots that must survive GC
- The Rust borrow checker will force explicit ownership decisions during GC (heap + roots both need mut access)
- Next: mark-compact GC, then isolate struct bundling VM + Heap
