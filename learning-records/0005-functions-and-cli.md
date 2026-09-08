# User implemented functions, call frames, and a CLI

Full function support: FuncDecl compiles to separate Program, CallExpr compiles args into consecutive registers and emits OpCall, ReturnStmt emits OpReturn. VM has a frame stack ([]* Frame), shared accumulator. Arg copying uses convention: last N allocated registers before the call. Also built a main.go CLI that reads .js files and runs them end-to-end.

## Evidence
- OpCall/OpReturn opcodes, FuncDecl/CallExpr/ReturnStmt compilation
- VM frame stack with OpCall pushing frames and OpReturn popping
- Recursive disassembler (stringIndent) showing nested function bytecode
- main.go: file reader → lex → parse → compile → run → print result
- 17 tests all passing

## Implications
- Ready for control flow (if/else, comparisons, jumps) — the last major missing concept
- The arg copying convention (nextReg - paramCount) works for current tests but will break if there are statements after a call that allocate more registers — may need to revisit
- Has not tested recursion yet — would be a good validation that the call stack works correctly
- Building toward V8 isolate — control flow is the last fundamental before that's reachable
