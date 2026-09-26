# v2

A miniature JavaScript engine written in Rust, inspired by V8's architecture. Named after the smallest engine in the V8 family.

## What it can do

v2 implements a complete pipeline from source code to execution:

- **Lexer** — tokenizes a subset of JavaScript (numbers, strings, identifiers, operators, keywords)
- **Parser** — recursive descent parser producing an AST
- **Bytecode compiler** — compiles AST to a register-based bytecode format modeled after V8's Ignition interpreter
- **Virtual machine** — executes bytecode with a call stack, frames, and an accumulator register
- **Tagged values** — Smi (small integer) tagging scheme, same concept as V8's
- **Heap** — managed heap for strings, functions, and closures
- **Garbage collector** — mark-and-sweep GC triggered by heap threshold
- **Isolates** — self-contained execution contexts with independent heaps, like V8's `v8::Isolate`

### Supported language features

```javascript
// Variables
let x = 10;
const y = 20;
var z = x + y;

// Arithmetic
let result = (x + y) * z - 4 / 2;

// Strings and concatenation
let greeting = "hello" + " world";

// Functions with parameters and return values
function add(a, b) { return a + b; }
add(10, 20);

// Recursion
function factorial(n) {
  if (n < 1) { return 1; }
  return n * factorial(n - 1);
}
factorial(10);

// Conditionals with else
function max(a, b) {
  if (a > b) { return a; } else { return b; }
}

// While loops
var i = 10;
var result = 1;
while (i > 0) {
    result = result * i;
    i = i - 1;
}
result;

// Variable reassignment
let x = 10;
x = x + 1;

// Comparison operators: <, >, <=, >=, ==, !=
// Logical operators: &&, ||, !
// Nested function calls
function sum_of_squares(a, b) {
  return square(a) + square(b);
}
```

## What's missing compared to real V8

### Language features
- `while` loops and variable reassignment (**done**)
- Other loops (`for`, `do-while`, `for-in`, `for-of`)
- Objects, prototypes, and property access
- Arrays
- Classes and `this`
- Closures / variable capture (the struct exists but upvalues aren't wired)
- Arrow functions
- `try` / `catch` / `throw`
- `async` / `await` and Promises
- Generators and iterators
- Modules (`import` / `export`)
- Destructuring, spread/rest operators
- Template literals
- Regular expressions
- `switch`, ternary operator
- `typeof`, `instanceof`, `in`
- Assignment operators (`+=`, `-=`, etc.) and `++` / `--`
- Bitwise operators

### Types
- No floating-point numbers (integers only via Smi tagging)
- No `null`, `undefined`, or boolean types (booleans are `0` / `1` Smis)
- No type coercion
- No `Symbol`, `BigInt`, `WeakRef`

### Runtime
- No hidden classes or inline caches
- No JIT compilation (V8's Turbofan / Maglev)
- No deoptimization
- No lazy parsing or pre-parsing
- No event loop
- No built-in functions (`console.log`, `Math`, `JSON`, etc.)
- No `eval` or `Function` constructor

### GC
- Mark-and-sweep only (no generational, incremental, or concurrent collection)
- Non-moving collector (no compaction)

## Usage

```
cargo run -- script.js
```

## Running tests

```
cargo test
```
