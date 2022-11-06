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

        let instruction: &OpCode = &self.code[offset].try_into().unwrap();
        match instruction {
            Constant | DefineGlobal | GetGlobal | SetGlobal => {
                self.constant_instruction(instruction.name(), offset)
            }
            Return | Less | Greater | Equal | Not | False | True | Nil | Divide | Multiply
            | Subtract | Add | Negate | Print | Pop => {
                self.simple_instruction(instruction.name(), offset)
            }
            GetLocal | SetLocal => self.byte_instruction(instruction.name(), offset),
            Jump => self.jump_instruction(instruction.name(), 1, offset),
            JumpIfFalse => self.jump_instruction(instruction.name(), 1, offset),
            Loop => self.jump_instruction(instruction.name(), -1, offset),
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

    fn byte_instruction(&self, name: &str, offset: usize) -> usize {
        let slot = self.code[offset + 1];
        println!("{:-16} {:4} ", name, slot);
        offset + 2
    }

    fn jump_instruction(&self, name: &str, sign: i16, offset: usize) -> usize {
        let top = self.code[offset + 1] as u16;
        let bottom = self.code[offset + 2] as u16;
        let jump = (top << 8) | bottom;

        let destination = (3 + sign * jump as i16) as isize;
        println!(
            "{:-16} {:4} -> {}",
            name,
            offset,
            offset as isize + destination
        );

        offset + 3
    }
}
