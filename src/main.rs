mod ast;
mod bytecode;
mod heap;
mod isolate;
mod lex;
mod parser;
mod value;

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

    let mut vm = isolate::Isolate::new();
    let result = vm.eval(&src);

    println!("{}", vm.format_value(result));
}
