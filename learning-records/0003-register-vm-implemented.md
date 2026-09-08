# User implemented the full accumulator-based register VM

The user converted the stack machine to an accumulator + register model: compiler emits LdaSmi/Star/Add/Mul with register operands, VM uses acc + [256]int register file. Also built a disassembler (String() on Program and VM) unprompted, and chose a ring-buffer register allocator. Kept the constant pool (LdaSmi loads by pool index, not inline).

## Evidence
- bytecode.go: Compile with alloReg(), disassembler, ring-buffer allocator
- vm.go: accumulator-based FDE loop, VM state printer
- Tests green for register-based compilation and execution

## Implications
- Ready for variables — the user asked about Ldar and understands it's for reading named values from registers
- The ring-buffer allocator is a creative choice but won't correctly handle deep nesting that exceeds 256 — fine for now
- Built the disassembler without being asked — self-directed, prefers tooling for debugging
- All instructions are now 2 bytes (opcode + operand), which simplifies the disassembler but diverges from V8 where some opcodes have no operand
