# User has a working lexer, parser, and AST

The user built a lexer, recursive-descent parser, and AST for a simple arithmetic expression language (integers, +, *, parenthesized grouping) in Go. This demonstrates solid understanding of tokenization, recursive descent parsing, and tree data structures. The AST uses a union-style node: `Op` + `Left`/`Right` for binary ops, `Value *int` for leaf integers.

## Evidence
- Working tests for both lexer and parser
- Handles nested expressions like `30 + (20 * 40)`

## Implications
- No need to teach parsing fundamentals — jump straight to bytecode emission
- The AST structure (binary tree with leaf values) maps cleanly to a post-order traversal for bytecode generation
- The `Op` type (OADD, OMUL) will need a parallel set of bytecode opcodes
