use crate::value::{Value, ValueArray};

#[derive(Debug)]
#[repr(u8)]
pub enum OpCode {
    Return,
    Constant,
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        use OpCode::*;

        match byte {
            0 => Return,
            1 => Constant,
            _ => panic!("Unknown opcode: {byte}"),
        }
    }
}

impl OpCode {
    pub fn name(&self) -> &'static str {
        match self {
            OpCode::Return => "OP_RETURN",
            OpCode::Constant => "OP_CONSTANT",
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
        self.write_byte(chunk as u8, line);
    }

    pub fn add_constant(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
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
