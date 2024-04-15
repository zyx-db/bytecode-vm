use crate::value::{Value, ValueArr};
use std::hint::unreachable_unchecked;

const STACK_SIZE: usize = 150;

pub enum OpCode {
    OpReturn,
    OpConstant,
    ConstantIdx(usize),
}

pub struct Chunk {
    code: Vec<OpCode>,
    constants: ValueArr,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn push_constant(&mut self, v: Value) {
        self.code.push(OpCode::OpConstant);
        self.add_constant(v);
    }

    pub fn push_op(&mut self, op: OpCode) {
        self.code.push(op);
    }

    pub fn add_constant(&mut self, v: Value) -> usize {
        let idx = self.constants.len();
        self.constants.push(v);
        self.code.push(OpCode::ConstantIdx(idx));
        idx
    }

    fn get_constant(&self, offset: u32) -> Value {
        let idx = {
            match self.code[(offset + 1) as usize] {
                OpCode::ConstantIdx(x) => x,
                _ => unsafe { unreachable_unchecked() },
            }
        };
        self.constants[idx]
    }
}

pub fn disassemble(chunk: &Chunk, name: &str) {
    println!("===={}====", name);
    let mut offset = 0;
    for _ in 0..chunk.code.len() - 1 {
        disassemble_instruction(chunk, &mut offset)
    }
}

fn disassemble_instruction(chunk: &Chunk, offset: &mut u32) {
    use OpCode::*;
    let instruction = &chunk.code[*offset as usize];
    match instruction {
        OpReturn => *offset += print_simple_instruction("OP_RETURN", *offset),
        OpConstant => *offset += print_constant_instruction("OP_CONSTANT", *offset, chunk),
        _ => {
            println!("how tf am i matching a idx");
            unsafe { unreachable_unchecked() }
        }
    }
}

fn print_simple_instruction(name: &str, offset: u32) -> u32 {
    println!("{:04} {}", offset, name);
    1
}

fn print_constant_instruction(name: &str, offset: u32, c: &Chunk) -> u32 {
    let value = c.get_constant(offset);
    println!("{:04} {:10} {:16}", offset, name, value);
    2
}

#[derive(Debug)]
pub enum VM_Errors {
    CompileError,
    RuntimeError,
}

struct Stack {
    values: [Value; STACK_SIZE],
    top: usize,
}

impl Stack {
    fn new() -> Self {
        return Stack {
            values: [Value::default(); STACK_SIZE],
            top: 0,
        };
    }

    fn push(&mut self, value: Value) {
        self.values[self.top] = value;
        self.top += 1;
    }

    fn pop(&mut self) -> Value {
        self.top -= 1;
        return self.values[self.top];
    }
}

pub struct VM {
    chunk: Option<Chunk>,
    ip: usize,
    stack: Stack,
    debug: bool,
}

impl VM {
    pub fn new(debug: bool) -> Self {
        return VM {
            chunk: None,
            ip: 0,
            debug,
            stack: Stack::new(),
        };
    }

    pub fn interpret(&mut self, c: Chunk) -> Result<(), VM_Errors> {
        self.chunk = Some(c);
        self.ip = 0;
        return self.run();
    }

    fn run(&mut self) -> Result<(), VM_Errors> {
        loop {
            let instruction = &(self.chunk.as_ref().unwrap()).code[self.ip];
            if self.debug {
                let data = self.chunk.as_ref().unwrap();
                for i in 0..self.stack.top {
                    print!("[ {} ]", self.stack.values[i]);
                }
                if self.stack.top > 0 {
                    print!("\n");
                }
                disassemble_instruction(data, &mut (self.ip as u32))
            }
            use OpCode::*;
            match instruction {
                OpReturn => {
                    println!("value {}", self.stack.pop());
                    return Ok(());
                }
                OpConstant => {
                    let data = self.chunk.as_ref().unwrap();
                    let value = data.get_constant(self.ip as u32);
                    self.stack.push(value);
                    // move ip again for constant
                    self.ip += 1;
                }
                _ => {}
            }
            self.ip += 1;
        }
    }
}
