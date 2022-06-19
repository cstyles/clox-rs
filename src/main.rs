use std::io::{BufRead, Write};

use vm::{Vm, VmError};

mod chunk;
mod compiler;
mod debug;
mod scanner;
mod token;
mod value;
mod vm;

fn main() {
    let vm = Vm::new();
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        repl(vm);
    } else if args.len() == 2 {
        run_file(vm, &args[1]);
    } else {
        eprintln!("Usage: clox [path]");
        std::process::exit(64);
    }
}

fn repl(mut vm: Vm) {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = String::with_capacity(1024);
    print_prompt();

    while stdin.read_line(&mut buffer).is_ok() {
        vm.interpret(buffer.trim());
        buffer.clear();
        print_prompt();
    }
}

fn run_file(mut vm: Vm, path: &str) {
    let source = std::fs::read_to_string(path).expect("error reading file");
    match vm.interpret(&source) {
        Ok(_) => {}
        Err(VmError::CompileError) => std::process::exit(65),
        Err(VmError::RuntimeError) => std::process::exit(70),
    }
}

fn print_prompt() {
    print!("> ");
    std::io::stdout().flush().expect("error flushing stdout");
}
