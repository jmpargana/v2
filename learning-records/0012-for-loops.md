# For Loops — Syntactic Sugar over While

This lesson adds `for` loops to the language. The key insight: `for` loops require zero new bytecodes. They are pure syntactic sugar — the parser reads `for` syntax and the compiler emits the exact same bytecodes as a `while` loop. The entire change is in the frontend (lexer, parser, AST) and a small compiler match arm.

---

## The desugaring

Every `for` loop maps directly onto a `while` loop:

```javascript
// Source:
for (let i = 0; i < 10; i = i + 1) {
    total = total + i;
}

// Equivalent:
let i = 0;
while (i < 10) {
    total = total + i;
    i = i + 1;
}
```

The three parts of the `for` header — **init**, **condition**, **update** — rearrange into:

```
for (init; condition; update) { body }
    ↓
init;
while (condition) { body; update; }
```

This is the universal compilation strategy for C-style `for` loops across virtually all bytecode VMs (V8, JVM, CPython, Lua, SpiderMonkey).

---

## Implementation

### Lexer

Add `For` to `SymbolKind` and recognize `"for"` as a keyword:

```rust
SymbolKind::For,

"for" => res.push(Symbol { kind: SymbolKind::For, ... }),
```

### AST

Add a new statement variant:

```rust
pub enum Stmt {
    // ... existing variants ...
    ForLoop {
        init: Box<Stmt>,
        condition: Expr,
        update: Box<Stmt>,
        body: Vec<Stmt>,
    },
}
```

`init` is typically a `VarDecl`, `update` is typically an `Assign`. Both are `Box<Stmt>` because `Stmt` is recursive.

### Parser

Two design choices:

1. **Desugar in the parser** — emit existing AST nodes (no `ForLoop` variant needed)
2. **Add a ForLoop AST node** — parser creates it, compiler handles it

We use option 2 to preserve source structure in the AST.

```rust
fn parse_stmt(&mut self) -> Stmt {
    match self.peek() {
        // ... existing arms ...
        SymbolKind::For => self.parse_for_loop(),
        _ => self.parse_expr_stmt(),
    }
}

fn parse_for_loop(&mut self) -> Stmt {
    self.expect(SymbolKind::For);
    self.expect(SymbolKind::LPar);

    let init = self.parse_stmt();        // "let i = 0;" — includes semicolon

    let condition = self.parse_expr();   // "i < 10"
    self.expect(SymbolKind::Semi);       // separator semicolon

    let update = self.parse_for_update(); // "i = i + 1" — NO semicolon
    self.expect(SymbolKind::RPar);

    self.expect(SymbolKind::LBrace);
    let mut body = Vec::new();
    while self.peek() != SymbolKind::RBrace {
        body.push(self.parse_stmt());
    }
    self.expect(SymbolKind::RBrace);

    Stmt::ForLoop {
        init: Box::new(init),
        condition,
        update: Box::new(update),
        body,
    }
}
```

**The update clause is the tricky part.** `i = i + 1` looks like an assignment but is followed by `)`, not `;`. It needs its own parser:

```rust
fn parse_for_update(&mut self) -> Stmt {
    // Same logic as parse_expr_stmt but without the trailing semicolon
    if self.peek() == SymbolKind::Ident
        && self.pos + 1 < self.syms.len()
        && self.syms[self.pos + 1].kind == SymbolKind::Assign
    {
        let name = self.pop().str_val;
        self.expect(SymbolKind::Assign);
        let value = self.parse_expr();
        return Stmt::Assign { name, value };
    }
    let expr = self.parse_expr();
    Stmt::ExprStmt(expr)
}
```

### Compiler

The compiler desugars at compile time. This is the payoff — all the hard work was done in Lesson 11 with `while` loops:

```rust
Stmt::ForLoop { init, condition, update, body } => {
    // 1. Compile init (runs once, before the loop)
    self.compile_stmt(init, heap);

    // 2. Record loop start (same as while)
    let loop_start = self.code.len();

    // 3. Compile condition + exit jump (same as while)
    self.compile_expr(condition, heap);
    let exit_jump = self.emit_jump(ByteCode::JumpIfFalse);

    // 4. Compile body (same as while)
    self.compile_stmts(body, heap);

    // 5. Compile update (runs after each iteration)
    self.compile_stmt(update, heap);

    // 6. JumpLoop back to condition (same as while)
    self.code.push(ByteCode::JumpLoop as u8);
    self.code.push((self.code.len() + 1 - loop_start) as u8);

    // 7. Patch exit jump (same as while)
    self.patch_jump(exit_jump);
}
```

Steps 2, 3, 6, 7 are identical to `WhileLoop`. Steps 1 and 5 are the only additions.

**Critical detail:** The init is compiled *before* `loop_start`. This ensures `JumpLoop` doesn't re-execute the initializer on each iteration.

---

## Bytecode output

Source:
```javascript
var total = 0;
for (var i = 1; i <= 5; i = i + 1) {
    total = total + i;
}
total;
```

Compiled bytecode:
```
addr  instruction        effect
────  ─────────────────  ──────────────────────
0000  LdaSmi [0] (0)     acc = 0
0002  Star r0             r0 = 0  (var total = 0)
0004  LdaSmi [1] (1)     acc = 1
0006  Star r1             r1 = 1  (var i = 1 — init)

                          ◄── loop_start (addr 0008)
0008  Ldar r1             acc = i
0010  Star r2             r2 = i  (stash for comparison)
0012  LdaSmi [2] (5)     acc = 5
0014  TestLessEqual r2    acc = (i <= 5) ? 1 : 0
0016  JumpIfFalse 20      if acc==0, jump to exit

0018  Ldar r0             acc = total
0020  Star r3             r3 = total  (stash for add)
0022  Ldar r1             acc = i
0024  Add r3              acc = total + i
0026  Star r0             r0 = acc  (total = total + i)

0028  Ldar r1             acc = i       ← update starts
0030  Star r4             r4 = i
0032  LdaSmi [3] (1)     acc = 1
0034  Add r4              acc = i + 1
0036  Star r1             r1 = acc  (i = i + 1)

0038  JumpLoop 32         pc = 0040 - 32 = 0008  ← back edge

0040  Ldar r0             acc = total  (final expression)
```

This is byte-for-byte identical to the `while` loop version of the same program.

---

## Why syntactic sugar matters

"Syntactic sugar" sounds dismissive, but it's one of the most important concepts in language design:

1. **Zero runtime cost.** The bytecodes are identical. The VM cannot tell whether the source was `for` or `while`.
2. **Locality of intent.** `for (let i = 0; i < n; i = i + 1)` puts the loop variable, bound, and increment on one line. With `while`, these are scattered across three locations.
3. **Compiler simplicity.** No new bytecodes, no new VM handlers, no new optimization paths.

---

## How V8 compiles for loops

V8 handles `for` in `VisitForStatement` (bytecode-generator.cc). The structure is identical:

```
VisitForStatement:
  1. Visit(init)
  2. LoopHeader()
  3. Visit(cond) + BreakIfFalse()
  4. Visit(body)
  5. Visit(next) + JumpToHeader()
```

V8 uses a `LoopBuilder` helper to manage offset bookkeeping. The emitted bytecodes match our pattern exactly: condition → JumpIfFalse → body → update → JumpLoop.

---

## What this unlocks

- **Counted iteration:** `for (let i = 0; i < n; i = i + 1)`
- **Summation patterns:** `for (let i = 1; i <= n; i = i + 1) { total = total + i; }`
- **Nested loops:** `for (...) { for (...) { ... } }`

Still missing (future lessons):
- **`break`/`continue`:** require tracking the loop's exit and header addresses during compilation, and a loop-context stack in the compiler
- **`do-while`:** condition check at the bottom — body always executes at least once
- **`for...in`/`for...of`:** require iterators and object/array support
