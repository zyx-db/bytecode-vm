use std::hint::unreachable_unchecked;

use crate::value::{Value, ValueArr};

pub enum OpCode {
    OpReturn,
    OpConstant,
    ConstantIdx(usize)
}

pub struct Chunk {
    code: Vec<OpCode>,
    constants: ValueArr,
}

impl Chunk {
    pub fn new() -> Self{
        Chunk{code: Vec::new(), constants: Vec::new()}
    }

    pub fn push_constant(&mut self, v: Value){
        self.code.push(OpCode::OpConstant);
        self.add_constant(v);
    }

    pub fn push_op(&mut self, op: OpCode){
        self.code.push(op);
    }

    pub fn add_constant(&mut self, v: Value) -> usize{
        let idx = self.constants.len();
        self.constants.push(v);
        self.code.push(OpCode::ConstantIdx(idx));
        idx
    }
}

pub fn disassemble(chunk: &Chunk, name: &str){
    println!("===={}====", name);
    let mut offset = 0;
    for instruction in &chunk.code {
        use OpCode::*;
        match instruction{
            OpReturn => {
                offset += print_simple_instruction("OP_RETURN", offset);
            }
            OpConstant => {
                offset += print_constant_instruction("OP_CONSTANT", offset, chunk)
            }
            ConstantIdx(_) => {}
            _ => {
                println!("how tf am i matching a idx");
                unsafe {unreachable_unchecked()}
            }
        }
    }
}

fn print_simple_instruction(name: &str, offset: u32) -> u32{
    println!("{:04} {}", offset, name);
    1
}

fn print_constant_instruction(name: &str, offset: u32, c: &Chunk) -> u32{
    let idx = {
        match c.code[(offset + 1) as usize]{
            OpCode::ConstantIdx(x) => { x }
            _ => {
                println!("wtf am i matching");
                unsafe {unreachable_unchecked()}
            }
        }
    };
    println!("{:04} {:10} {:16}", offset, name, c.constants[idx]);
    2
}
