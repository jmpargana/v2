mod lex;
mod ast;
mod parser;
mod bytecode;
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
    let mut program = bytecode::Program::new();
    program.compile(&stmts);
    let result = vm::VM::new().fde(&program);

    println!("{}", result);
}
