use crate::chunk::{Chunk, OpCode};
use crate::value;

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);

        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        let instruction: &OpCode = &self.code[offset].into();
        match instruction {
            OpCode::Return => self.simple_instruction(instruction.name(), offset),
            OpCode::Constant => self.constant_instruction(instruction.name(), offset),
            _ => {
                println!("Unknown opcode: {:?}", instruction);
                offset + 1
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
        value::print_value(self.constants[constant as usize]);
        println!();

        offset + 2
    }
}
