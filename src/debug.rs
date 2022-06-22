#[allow(unused)]
use crate::chunk::{Chunk, OpCode};
use crate::value;
use OpCode::*;

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);

        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        let instruction: &OpCode = &self.code[offset].into();
        match instruction {
            Constant | DefineGlobal | GetGlobal => {
                self.constant_instruction(instruction.name(), offset)
            }
            Return | Less | Greater | Equal | Not | False | True | Nil | Divide | Multiply
            | Subtract | Add | Negate | Print | Pop => {
                self.simple_instruction(instruction.name(), offset)
            }
        }
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{name}");
        offset + 1
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant = self.code[offset + 1];

        print!("{:-16} {:4} ", name, constant);
        value::print_value(&self.constants[constant as usize]);
        println!();

        offset + 2
    }
}
