use crate::chunk::{Chunk, OpCode};

mod chunk;
mod debug;
mod value;

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write_opcode(OpCode::Constant, 123);
    chunk.write_byte(constant as u8, 123);
    chunk.write_opcode(OpCode::Return, 123);

    chunk.disassemble("test chunk");
}
