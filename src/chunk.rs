use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::value::{Value, ValueArray};

#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum OpCode {
    Return = 0,
    Constant = 1,
    Negate = 2,
    Add = 3,
    Subtract = 4,
    Multiply = 5,
    Divide = 6,
    Nil = 7,
    True = 8,
    False = 9,
    Not = 10,
    Equal = 11,
    Greater = 12,
    Less = 13,
    Print = 14,
    Pop = 15,
    DefineGlobal = 16,
    GetGlobal = 17,
    SetGlobal = 18,
    GetLocal = 19,
    SetLocal = 20,
    JumpIfFalse = 21,
    Jump = 22,
}

impl OpCode {
    pub fn name(&self) -> &'static str {
        match self {
            OpCode::Return => "OP_RETURN",
            OpCode::Constant => "OP_CONSTANT",
            OpCode::Negate => "OP_NEGATE",
            OpCode::Add => "OP_ADD",
            OpCode::Subtract => "OP_SUBTRACT",
            OpCode::Multiply => "OP_MULTIPLY",
            OpCode::Divide => "OP_DIVIDE",
            OpCode::Nil => "OP_NIL",
            OpCode::True => "OP_TRUE",
            OpCode::False => "OP_FALSE",
            OpCode::Not => "OP_NOT",
            OpCode::Equal => "OP_EQUAL",
            OpCode::Greater => "OP_GREATER",
            OpCode::Less => "OP_LESS",
            OpCode::Print => "OP_PRINT",
            OpCode::Pop => "OP_POP",
            OpCode::DefineGlobal => "OP_DEFINE_GLOBAL",
            OpCode::GetGlobal => "OP_GET_GLOBAL",
            OpCode::SetGlobal => "OP_SET_GLOBAL",
            OpCode::GetLocal => "OP_GET_LOCAL",
            OpCode::SetLocal => "OP_SET_LOCAL",
            OpCode::JumpIfFalse => "OP_JUMP_IF_FALSE",
            OpCode::Jump => "OP_JUMP",
        }
    }
}

#[derive(Debug, Default)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: ValueArray,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn write_byte(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn write_opcode(&mut self, chunk: OpCode, line: usize) {
        self.write_byte(chunk.into(), line);
    }

    pub fn add_constant(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    pub fn count(&self) -> usize {
        self.code.len()
    }
}

mod tests {
    #[allow(unused)]
    use super::*;

    #[test]
    fn correct_size() {
        assert_eq!(1, std::mem::size_of::<OpCode>())
    }
}
