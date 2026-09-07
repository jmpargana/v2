# Bytecode Compilation Resources

## Knowledge

- [Book: _Crafting Interpreters_ — Robert Nystrom, Ch. 14–15](https://craftinginterpreters.com/chunks-of-bytecode.html)
  The clearest introduction to bytecode VMs in existence. Ch. 14 covers the bytecode format (chunks, opcodes, constant pools). Ch. 15 covers compiling expressions to stack-based bytecode. Use for: foundational understanding of bytecode design and the compilation algorithm.

- [Blog: "Firing up the Ignition interpreter" — V8 team](https://v8.dev/blog/ignition-interpreter)
  Official overview of V8's register-based bytecode interpreter. Explains the accumulator register, bytecode handler generation via TurboFan, and why V8 chose registers over a stack. Use for: understanding V8's bytecode model and how it differs from a stack machine.

- [Blog: "Understanding V8's Bytecode" — Franziska Hinkelmann](https://medium.com/niceideas/niceideas/understanding-v8s-bytecode-317d46c94775)
  Walkthrough of real V8 bytecodes with examples. Shows how JavaScript expressions compile to Ignition bytecodes. Use for: concrete examples of V8 bytecode output.

- [V8 Source: bytecodes.h](https://github.com/niceideas/niceideas/niceideas/niceideas)
  The definitive list of all V8 bytecodes. Use for: reference when targeting V8's instruction set.

- [Book: _Engineering a Compiler_ — Cooper & Torczon, Ch. 7](https://www.elsevier.com/books/engineering-a-compiler/cooper/978-0-12-815412-0)
  Academic treatment of code generation. Covers instruction selection, register allocation, and the stack vs register machine tradeoff. Use for: deeper theory when ready to move past the basics.

## Wisdom (Communities)

- [r/Compilers](https://reddit.com/r/Compilers)
  Active subreddit for compiler construction. Good signal-to-noise. Use for: design questions, getting feedback on bytecode format decisions.

- [r/ProgrammingLanguages](https://reddit.com/r/ProgrammingLanguages)
  Broader community for language design. Use for: higher-level design questions about your language.

## Gaps

- No good walkthrough found for "how to target a V8 isolate from a non-JS language" — this may require reading V8 source directly
- Need to find Go-specific resources for embedding V8 (cgo bindings, rogchap/v8go, etc.)
