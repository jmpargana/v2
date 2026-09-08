# User extended g8 to a JavaScript subset with variables

Major rewrite: proper AST with Node/Expr/Stmt interfaces, recursive descent parser with operator precedence (parseTerm/parseFactor), extended lexer with keywords (let/const/var/function/return), identifiers, semicolons, braces, commas. Compiler correctly handles VarDecl (symbol table mapping names to registers), Ident (Ldar from symbol table), and BinaryExpr (allocReg for temps). Added Init() constructor for Program.

## Evidence
- AST: NumberLit, Ident, BinaryExpr, CallExpr, VarDecl, FuncDecl, ReturnStmt, ExprStmt
- Parser: full grammar with precedence, function declarations, call expressions, parameters
- Compiler: correct register allocation pattern (CompileExpr with no reg parameter, symbols map for variables)
- Lexer: keywords map, identifiers, all JS punctuation tokens
- Tests for lexer (6 cases), parser (7 cases), compiler (1 case — has nil map bug in test setup)

## Implications
- Ready for functions — FuncDecl and CallExpr are already parsed, just need compilation
- The user understands the accumulator invariant (comment: "result is in accumulator after all operations anyway")
- Register allocation for variables vs temporaries is solid
- Will need call frames, a call stack, and separate bytecode per function
