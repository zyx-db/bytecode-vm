mod common;
mod value;

use common::{Chunk, disassemble};

fn main() {
    let mut c: Chunk = Chunk::new();
    c.push_constant(1.2);
    c.push_op(common::OpCode::OpReturn);
    disassemble(&c, "main");
}
