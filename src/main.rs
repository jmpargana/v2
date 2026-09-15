mod ast;
mod bytecode;
mod heap;
mod lex;
mod parser;
mod value;
mod vm;

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: g8 <file.js>");
        process::exit(1);
    }

    let src = fs::read_to_string(&args[1]).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });

    let tokens = lex::Lexer::lex(&src);
    let stmts = parser::Parser::new(tokens).parse();
    let mut heap = heap::Heap::new();
    let mut program = bytecode::Program::new();
    program.compile(&stmts, &mut heap);
    let result = vm::VM::new(heap).fde(&program);

    println!("{}", result);
}
