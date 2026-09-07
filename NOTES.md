# Teaching Notes

- User already has a working lexer + parser + AST in Go for arithmetic expressions (add, multiply, parenthesized grouping)
- User prefers to learn by building — they wrote the lexer/parser themselves before asking for help
- Grammar: `Expr := Res Op Res`, `Res := (Expr) | Digit`, `Op := + | *`
- Start with stack-based bytecode (simpler), then bridge to V8's register-based model
