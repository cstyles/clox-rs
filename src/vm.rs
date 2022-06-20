use std::cmp::Ordering;
use std::error::Error;
use std::fmt::Display;
use std::ops::{Div, Mul, Not, Sub};
use std::rc::Rc;

use fnv::FnvHashMap;

use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::object::Object;
use crate::string::LoxString;
use crate::value::{print_value, Value};

#[derive(Debug, Default)]
pub struct Vm {
    chunk: Chunk, // reference?
    ip: usize,
    stack: Vec<Value>,
    strings: FnvHashMap<Rc<String>, LoxString>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            ..Default::default()
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
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
                    print_value(&value);
                    println!();
                    return Ok(());
                }
                OpCode::Constant => {
                    let constant = self.read_constant().clone();
                    self.stack.push(constant);
                }
                OpCode::Negate => match self.stack.last_mut().unwrap() {
                    Value::Number(value) => *value = -*value,
                    _ => {
                        self.runtime_error("Operand must be a number.");
                        return Err(VmError::RuntimeError);
                    }
                },
                OpCode::Add => match (self.stack.pop().unwrap(), self.stack.pop().unwrap()) {
                    (Value::Number(b), Value::Number(a)) => self.stack.push(Value::Number(a + b)),
                    (Value::Obj(b), Value::Obj(a)) => {
                        match (b.as_ref(), a.as_ref()) {
                            (Object::Str(b), Object::Str(a)) => self.concatenate(a, b),
                            _ => {
                                // Two objects but at least one wasn't a string
                                self.runtime_error("Operands must be two numbers or two strings.");
                                return Err(VmError::RuntimeError);
                            }
                        };
                    }
                    _ => {
                        // At least one value wasn't a number nor a string
                        self.runtime_error("Operands must be two numbers or two strings.");
                        return Err(VmError::RuntimeError);
                    }
                },
                OpCode::Subtract => self.numeric_binary_op(Sub::sub)?,
                OpCode::Multiply => self.numeric_binary_op(Mul::mul)?,
                OpCode::Divide => self.numeric_binary_op(Div::div)?,
                OpCode::Nil => self.stack.push(Value::Nil),
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::Not => {
                    let top = self.stack.last_mut().unwrap();
                    *top = top.not();
                }
                OpCode::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(a == b));
                }
                OpCode::Greater => self.comparison_binary_op(Ordering::Greater)?,
                OpCode::Less => self.comparison_binary_op(Ordering::Less)?,
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    fn read_constant(&mut self) -> &Value {
        let byte = self.read_byte();
        &self.chunk.constants[byte as usize]
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

    fn numeric_binary_op(&mut self, op: impl Fn(f64, f64) -> f64) -> Result<(), VmError> {
        match (self.stack.pop().unwrap(), self.stack.pop().unwrap()) {
            (Value::Number(b), Value::Number(a)) => {
                self.stack.push(Value::Number(op(a, b)));
                Ok(())
            }
            _ => {
                self.runtime_error("Operands must be numbers.");
                Err(VmError::RuntimeError)
            }
        }
    }

    fn comparison_binary_op(&mut self, ordering: Ordering) -> Result<(), VmError> {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();

        match a.partial_cmp(&b) {
            Some(o) => {
                self.stack.push(Value::Bool(o == ordering));
                Ok(())
            }
            None => {
                self.runtime_error("Operands must be numbers.");
                Err(VmError::RuntimeError)
            }
        }
    }

    fn peek(&self, distance: usize) -> &Value {
        self.stack.get(self.stack.len() - 1 - distance).unwrap()
    }

    fn concatenate(&mut self, a: &LoxString, b: &LoxString) {
        let new_object = Object::Str(LoxString::add(self, a, b));
        let new_value = Value::Obj(Box::new(new_object));
        self.stack.push(new_value);
    }

    pub fn intern_string(&mut self, string: String) -> LoxString {
        let string = Rc::new(string);

        self.strings
            .entry(string.clone())
            .or_insert_with(|| LoxString::from(string))
            .clone()
    }

    fn runtime_error(&self, message: impl AsRef<str>) {
        eprintln!("{}", message.as_ref());

        let line = self.chunk.lines[self.ip - 1];
        eprintln!("[line {line}] in script");
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
