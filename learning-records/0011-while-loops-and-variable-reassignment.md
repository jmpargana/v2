# While Loops, Variable Reassignment, and Backward Jumps

This lesson adds two features that unlock iterative programming: `while` loops and variable reassignment (`x = x + 1`). Together they make loops useful — without reassignment, a loop body can't update its condition and runs forever or not at all.

The key bytecode insight: the existing `Jump` and `JumpIfFalse` only jump *forward* (they add to `pc`). Loops need to jump *backward* to re-test the condition. This requires a new `JumpLoop` bytecode that *subtracts* from `pc`.

---

## Why forward jumps aren't enough

The current `Jump` handler:

```rust
ByteCode::Jump => {
    let offset = self.read_byte(&call_stack, fi);
    call_stack.frames[fi].pc += offset as usize + 1;  // always moves forward
}
```

An `if` statement only needs forward jumps — it skips over the body. But a `while` loop needs to go *back* to the condition after executing the body:

```
     ┌──────────────────────────┐
     │    condition test         │ ◄─── loop back edge
     │    JumpIfFalse ──────────┼──► exit
     │    body...                │
     │    JumpLoop ─────────────┘
     │
     ▼ (exit)
     next statement
```

V8 calls this the **back edge** — the jump from the end of the loop body back to the condition. In V8's Ignition interpreter, this is the `JumpLoop` bytecode, and it's special: it's the *only* bytecode that moves `pc` backward. This matters for optimization — back edges are where V8 checks whether to tier up to TurboFan (the optimizing compiler). For us, it's just a subtraction instead of an addition.

---

## The new bytecode: JumpLoop

```rust
#[repr(u8)]
pub enum ByteCode {
    // ... existing opcodes ...
    JumpLoop = 21,   // backward jump: pc -= offset
}
```

The VM handler:

```rust
ByteCode::JumpLoop => {
    let offset = self.read_byte(&call_stack, fi);
    call_stack.frames[fi].pc += 1;
    call_stack.frames[fi].pc -= offset as usize;
}
```

Note: we read the offset byte (advancing `pc` by 1), then subtract the offset. The offset is measured from the *current* `pc` position (after reading the operand), pointing back to the condition's first instruction.

### Why not reuse Jump with signed offsets?

We *could* make `Jump` take a signed `i8` offset, supporting both forward and backward jumps. V8 doesn't do this for two reasons:

1. **Back edges are semantically different.** They're where the runtime checks interrupt requests (stack overflow, GC safe points, debugger breakpoints, optimization triggers). A single `Jump` opcode would need a runtime check "am I going backward?" on every jump.
2. **Range.** A `u8` offset gives 0–255 range in one direction. A signed `i8` gives only -128 to +127 in each direction. Separate opcodes double the effective range.

For our VM, reason #1 is the important one. When we add GC safe points or stack overflow checks later, we'll want them on every `JumpLoop` but not every forward `Jump`.

---

## Variable reassignment

Currently, `let x = 10;` allocates a register and stores 10. But there's no way to write `x = 10;` (assignment without `let`/`const`/`var`). The parser sees `x` as an identifier expression, then `=` and doesn't know what to do.

### AST change

```rust
pub enum Stmt {
    // ... existing variants ...
    Assign {
        name: String,
        value: Expr,
    },
}
```

### Parser change

When we see an `Ident` followed by `=` (not `==`), it's an assignment:

```rust
fn parse_expr_stmt(&mut self) -> Stmt {
    if self.peek() == SymbolKind::Ident {
        // Look ahead: is this `ident = expr;` (assignment) or `ident ...` (expression)?
        if self.pos + 1 < self.syms.len() && self.syms[self.pos + 1].kind == SymbolKind::Assign {
            let name = self.pop().str_val;
            self.expect(SymbolKind::Assign);
            let value = self.parse_expr();
            self.expect(SymbolKind::Semi);
            return Stmt::Assign { name, value };
        }
    }
    let expr = self.parse_expr();
    self.expect(SymbolKind::Semi);
    Stmt::ExprStmt(expr)
}
```

### Compiler: assignment reuses the existing register

```rust
Stmt::Assign { name, value } => {
    self.compile_expr(value, heap);
    let reg = self.symbols[name];   // look up the register from the declaration
    self.code.push(ByteCode::Star as u8);
    self.code.push(reg as u8);
}
```

This is the key: `let x = 10;` allocates register `r0` for `x` and maps `"x" -> 0` in the symbol table. Later, `x = x + 1;` compiles the expression `x + 1`, then stores the result back into `r0` using the same mapping. No new register needed — it's just `Star r0` again.

---

## Compiling while loops

### AST

```rust
pub enum Stmt {
    // ... existing variants ...
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
}
```

### Lexer

Add `While` to `SymbolKind` and recognize the keyword `"while"`.

### Parser

```rust
SymbolKind::While => self.parse_while(),

fn parse_while(&mut self) -> Stmt {
    self.expect(SymbolKind::While);
    self.expect(SymbolKind::LPar);
    let condition = self.parse_expr();
    self.expect(SymbolKind::RPar);
    self.expect(SymbolKind::LBrace);
    let mut body = Vec::new();
    while self.peek() != SymbolKind::RBrace {
        body.push(self.parse_stmt());
    }
    self.expect(SymbolKind::RBrace);
    Stmt::While { condition, body }
}
```

### Compiler

The compilation strategy follows a standard pattern:

```rust
Stmt::While { condition, body } => {
    // 1. Record the loop header position (where JumpLoop will target)
    let loop_start = self.code.len();

    // 2. Compile the condition
    self.compile_expr(condition, heap);

    // 3. Emit JumpIfFalse with placeholder offset (to exit the loop)
    let exit_jump = self.emit_jump(ByteCode::JumpIfFalse);

    // 4. Compile the body
    self.compile_stmts(body, heap);

    // 5. Emit JumpLoop back to loop_start
    let loop_offset = self.code.len() - loop_start + 2;  // +2 for the JumpLoop opcode + operand
    self.code.push(ByteCode::JumpLoop as u8);
    self.code.push(loop_offset as u8);

    // 6. Patch the exit jump to land here (after the JumpLoop)
    self.patch_jump(exit_jump);
}
```

The offset calculation for `JumpLoop`:
- `self.code.len()` is where we're about to emit `JumpLoop`
- `loop_start` is the first byte of the condition
- We need `pc` after reading the JumpLoop operand to land on `loop_start`
- After reading the operand, `pc = emit_pos + 2`
- We want `pc - offset = loop_start`, so `offset = emit_pos + 2 - loop_start = (self.code.len() - loop_start) + 2`

---

## Bytecode output: `while (x > 0) { x = x - 1; }`

Assuming `x` is in register `r0`:

```
0000  Ldar     r0          ; load x into acc
0002  Star      r1          ; save x to temp register
0004  LdaSmi   [0] (0)     ; load constant 0
0006  TestGreater r1        ; acc = (x > 0) ? 1 : 0
0008  JumpIfFalse  10       ; if false, jump to exit (offset 10 → addr 0020)

; --- loop body: x = x - 1 ---
0010  Ldar     r0          ; load x
0012  Star      r2          ; save to temp
0014  LdaSmi   [1] (1)     ; load 1
0016  Sub      r2          ; acc = x - 1
0018  Star      r0          ; x = result (reassignment!)

0020  JumpLoop  20          ; jump back to 0000 (pc = 0022 - 20 = 0002... wait)
```

Actually let's be more precise. The compiler emits:

```
addr  bytes          meaning
────  ─────          ───────
0000  Ldar r0        ; load x → acc              ◄── loop_start
0002  Star r1        ; acc → r1 (temp for comparison)
0004  LdaSmi [0]     ; load literal 0 → acc
0006  TestGreater r1 ; acc = (r1 > acc) → (x > 0)
0008  JumpIfFalse 10 ; if acc==0, skip 10 bytes → lands at 0020

0010  Ldar r0        ; load x → acc
0012  Star r2        ; acc → r2 (temp for subtraction)
0014  LdaSmi [1]     ; load literal 1 → acc
0016  Sub r2         ; acc = r2 - acc → x - 1
0018  Star r0        ; acc → r0 (x = x - 1)

0020  JumpLoop 22    ; pc after read = 0022, 0022 - 22 = 0000 ✓
```

After `JumpLoop`, the VM is back at address `0000`, re-evaluating the condition. When `x` reaches 0, `TestGreater` produces 0, `JumpIfFalse` fires, and execution continues at the first instruction past the loop.

---

## Side-by-side: recursion vs. iteration

Before this change, computing a sum required recursion:

```javascript
// BEFORE: recursion (only option)
function sum(n) {
    if (n < 1) { return 0; }
    return n + sum(n - 1);
}
sum(100);
```

This pushes 100 frames onto the CallStack, each with its own register window. For `sum(10000)`, that's 10,000 frames.

After adding `while` and reassignment:

```javascript
// AFTER: iteration
let total = 0;
let i = 100;
while (i > 0) {
    total = total + i;
    i = i - 1;
}
total;
```

This uses a single frame with a few registers. No call stack growth, no GC pressure from frame allocation. The bytecode loops in place.

### Performance characteristics

| Aspect | Recursive sum(n) | Iterative while loop |
|--------|-------------------|---------------------|
| Frames used | n + 1 | 1 |
| CallStack slots | ~3n (params + temps) | ~4 (fixed) |
| Function lookups per iteration | 1 (resolve closure) | 0 |
| Bytecodes per iteration | ~12 (call + body + return) | ~10 (condition + body + JumpLoop) |
| Stack overflow risk | Yes, at ~10K depth | No |

---

## How V8 compiles while loops

V8's Ignition bytecode for `while (x > 0) { x = x - 1; }` looks like:

```
[generating bytecode for function: ]
   0: Ldar a0
   2: Star r0
   4: LdaZero
   5: TestGreaterThan r0, [0]
   7: JumpIfFalse [16]
   9: Ldar a0
  11: Sub [1], [1]
  13: Star a0
  15: JumpLoop [0], [15]
  17: ...
```

Notable differences from our version:
- V8's `JumpLoop` takes two operands: a feedback slot (for optimization profiling) and the offset. Ours takes just the offset.
- V8 has `LdaZero` as a specialized opcode for loading 0. We use `LdaSmi [idx]` for everything.
- V8's `Sub` takes an immediate operand directly. Ours operates on a register.
- V8 uses `a0` for function arguments vs `r0` for locals. We don't distinguish.

The structure is identical: condition → JumpIfFalse → body → JumpLoop. This is the universal compilation strategy for `while` loops across virtually all bytecode VMs (JVM, CPython, Lua, SpiderMonkey).

---

## The JumpLoop offset calculation, carefully

Getting the offset right is the trickiest part. Here's the calculation step by step:

```
loop_start = code.len()          // e.g., 0

... compile condition ...          // emits bytes at 0..7
exit_jump = emit_jump(JumpIfFalse) // emits at 8,9 (opcode + placeholder)

... compile body ...               // emits bytes at 10..19

// Now emit JumpLoop:
// code.len() is 20 (where JumpLoop opcode goes)
// After reading: JumpLoop at 20, offset at 21, pc is now 22
// We want: pc - offset == loop_start
// So: 22 - offset == 0, offset == 22
// Generalized: offset = (code.len() + 2) - loop_start

let offset = self.code.len() + 2 - loop_start;
self.code.push(ByteCode::JumpLoop as u8);    // at position 20
self.code.push(offset as u8);                 // at position 21
// pc after reading operand: 22
// 22 - 22 = 0 = loop_start ✓
```

The `+2` accounts for the two bytes of the `JumpLoop` instruction itself (opcode + operand). If we forget this, the loop jumps to byte 2 instead of byte 0 — it would skip the first instruction of the condition on every iteration.

---

## What this unlocks

With `while` loops and variable reassignment, the VM can now express:

- **Iterative algorithms**: sum, factorial, GCD, any accumulator pattern
- **String building**: `while (i < 10) { s = s + "x"; i = i + 1; }`
- **Bounded iteration**: repeat N times without recursion
- **State machines**: loop with condition-driven transitions

Still missing (future lessons):
- **`for` loops**: syntactic sugar over `while` — `for (init; cond; update) { body }` desugars to `init; while (cond) { body; update; }`
- **`break`/`continue`**: require tracking the loop's exit and header addresses during compilation
- **Upvalue capture**: closures that read/write variables from enclosing scopes
