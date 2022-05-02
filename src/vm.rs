use std::error::Error;
use std::fmt::Display;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::compiler::Compiler;
use crate::scanner::Scanner;
use crate::value::{print_value, Value};

#[derive(Debug, Default)]
pub struct Vm {
    chunk: Chunk, // reference?
    ip: usize,
    stack: Vec<Value>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            ..Default::default()
        }
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let scanner = Scanner::new(source);
        let chunk = Chunk::new();
        let compiler = Compiler::new(scanner, chunk);

        let chunk = match compiler.compile() {
            Ok(chunk) => chunk,
            Err(_) => return Err(VmError::CompileError),
        };

        self.chunk = chunk;
        self.ip = 0;
        self.run()
    }

    pub fn run(&mut self) -> InterpretResult {
        loop {
            #[cfg(debug_assertions)]
            {
                self.chunk.disassemble_instruction(self.ip);
                self.debug_trace_execution();
            }

            let instruction: OpCode = self.read_byte().into();

            match instruction {
                OpCode::Return => {
                    let value = self.stack.pop().unwrap();
                    print_value(value);
                    println!();
                    return Ok(());
                }
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.stack.push(constant);
                }
                OpCode::Negate => {
                    let top = self.stack.last_mut().unwrap();
                    *top = -*top;
                }
                OpCode::Add => self.binary_op(Add::add),
                OpCode::Subtract => self.binary_op(Sub::sub),
                OpCode::Multiply => self.binary_op(Mul::mul),
                OpCode::Divide => self.binary_op(Div::div),
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte();
        self.chunk.constants[byte as usize]
    }

    fn reset_stack(&mut self) {
        self.stack.clear()
    }

    fn debug_trace_execution(&self) {
        print!("          ");

        for slot in &self.stack {
            print!("[ {slot} ]");
        }

        println!();
    }

    fn binary_op(&mut self, op: impl Fn(f64, f64) -> f64) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(op(a, b));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmError {
    CompileError,
    RuntimeError,
}

impl Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for VmError {}

pub type InterpretResult = Result<(), VmError>;
