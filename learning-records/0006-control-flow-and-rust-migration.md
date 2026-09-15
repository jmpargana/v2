# User implemented control flow and migrated to Rust

Full control flow support: TestEqual, TestLess, TestGreater comparison opcodes, JumpIfFalse and Jump for branching, backpatching for forward jumps, optional else blocks. Bool expression type in AST compiled to comparison opcodes. Parser handles `if (cond) { body }` and `if (cond) { body } else { alt }` with boolean expressions wired into parse_expr at correct precedence level (below arithmetic).

Migrated entire project from Go to Rust — lexer, parser, AST, compiler, VM, CLI, and all tests. Motivation: wants full control over memory layout to learn V8 internals (segregated heaps, GC, isolate memory separation). Rust's ownership model and lack of GC make memory management explicit, which is exactly what the user wants to learn.

## Evidence
- TestEqual/TestLess/TestGreater opcodes with register operand
- JumpIfFalse/Jump opcodes with offset operand
- Backpatching via emit_jump/patch_jump pattern
- Bool expr variant in AST, compiled like Binary but with comparison opcodes
- Parser properly chains: parse_expr → comparison → parse_additive → parse_term → parse_factor
- 19 Rust tests all passing (lex: 7, parser: 14, bytecode: 2, vm: 2)
- CLI reads .js files end-to-end

## Implications
- All bytecode fundamentals complete: arithmetic, variables, functions, control flow
- Ready for V8-style runtime concepts: tagged values, heap, isolate structure
- Rust migration enables direct memory management needed for heap/GC implementation
- Parser has no remaining TODOs — boolean expressions fully integrated
