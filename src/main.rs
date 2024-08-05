mod common;
mod token_types;
mod value;

use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use clap::Parser;
use common::{disassemble, Chunk, VM_Errors, VM};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    debug: bool,

    file: Option<PathBuf>,
}

fn runFile(path: PathBuf, vm: &mut VM) {
    let mut contents = String::new();
    let mut file = File::open(path).expect("could not open file");
    file.read_to_string(&mut contents).expect("cannto read file");
    // by default rust strings are not null terminated
    // we add one to make sure the Eof Check works
    contents.push('\0');

    match vm.interpret(contents) {
        Err(VM_Errors::CompileError) => {}
        Err(VM_Errors::RuntimeError) => {}
        _ => {}
    }
}

fn repl(vm: &mut VM) {
    loop {
        print!(">");

        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        if line.len() == 0 {
            println!();
            break;
        }
        // by default rust strings are not null terminated
        // we add one to make sure the Eof Check works
        line.push('\0');

        vm.interpret(line);
    }
}

fn main() {
    let cli = Cli::parse();
    let mut vm = VM::new(cli.debug);

    match cli.file {
        Some(path) => {
            println!("running on file {:?}", path);
            runFile(path, &mut vm);
        }
        None => {
            println!("starting as repl!");
            repl(&mut vm);
        }
    }

    // let mut c: Chunk = Chunk::new();
    // c.push_constant(1.2);
    // c.push_op(common::OpCode::OpReturn);
    // if cli.debug {
    //     disassemble(&c, "main");
    //     println!("");
    //     println!("====execution====");
    // }

    // vm.interpret(c).unwrap();
}
