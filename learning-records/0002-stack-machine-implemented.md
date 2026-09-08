# User implemented the full stack-machine pipeline

The user implemented the bytecode compiler (post-order AST traversal emitting OpPush/OpAdd/OpMul) and a working VM with a fetch-decode-execute loop. Tests pass for compilation and execution. The user named the VM's main loop `FDE` (fetch-decode-execute) unprompted, showing they internalized the concept.

## Evidence
- `bytecode.go`: Compile method with post-order traversal, Equals for test comparison
- `vm.go`: VM struct with ip/stack, Push/Pop, FDE loop
- Tests green for `1 * (2 + 3)` compilation and `30 + (20 * 40)` execution

## Implications
- Ready to move to register-based bytecode — the user already noted "2. Register" as the next step in their bytecode.go comments
- Understands the constant pool, instruction pointer, and stack discipline
- Conceptual questions about OS stack/heap vs VM stack/heap show systems-level thinking
