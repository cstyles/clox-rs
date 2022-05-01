use std::io::{BufRead, Write};

use vm::{InterpretResult, VmError};

use vm::Vm;

mod chunk;
mod compiler;
mod debug;
mod scanner;
mod value;
mod vm;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        repl();
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        eprintln!("Usage: clox [path]");
        std::process::exit(64);
    }
}

fn repl() {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = String::with_capacity(1024);
    print_prompt();

    while stdin.read_line(&mut buffer).is_ok() {
        interpret(buffer.trim());
        buffer.clear();
        print_prompt();
    }
}

fn run_file(path: &str) {
    let source = std::fs::read_to_string(path).expect("error reading file");
    match interpret(&source) {
        Ok(_) => todo!(),
        Err(VmError::CompileError) => std::process::exit(65),
        Err(VmError::RuntimeError) => std::process::exit(70),
    }
}

fn interpret(source: &str) -> InterpretResult {
    compiler::compile(source);
    Ok(())
}

fn print_prompt() {
    print!("> ");
    std::io::stdout().flush().expect("error flushing stdout");
}
