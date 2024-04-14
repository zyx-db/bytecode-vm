mod common;
mod value;

use clap::Parser;
use common::{disassemble, Chunk, VM};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut vm = VM::new(cli.debug);
    let mut c: Chunk = Chunk::new();
    c.push_constant(1.2);
    c.push_op(common::OpCode::OpReturn);
    if cli.debug {
        disassemble(&c, "main");
        println!("");
        println!("====execution====");
    }

    vm.interpret(c).unwrap();
}
