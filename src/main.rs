



mod lexer;
mod parser;
mod ast;
mod codegen;
mod vm;
mod neuron;
mod stream;
mod types;
mod express;
mod schemas;
mod mathista;


use schemas::Neuron;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use actix::prelude::*;
use lexer::Lexer;
use parser::Parser;
use codegen::CodeGenerator;
use vm::VirtualMachine;
use serde_json::json;
use tokio::task;



/* 
    Onion lang pipeline
    1 => read source code
    2 => use lexer to generate tokens or token stream from the source code strings and chars
    3 => use parser to generate ast statements from token stream
    4 => build a program from ast and extract bytecodes or shellcode from that 
    5 => execute bytecode/shellcode on vm using opcodes
*/


#[actix_web::main] // use actix_web main context since we have actors
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{


    stemlib::upAndRun().await;

    // Read the main.onion file
    let source_code = std::fs::read_to_string("app.onion").expect("Failed to read file");

    // Print the source code for debugging
    println!("Source Code: \n{}", source_code);

    // Lexing and Parsing
    let lexer = Lexer::new(source_code);
    let mut parser = Parser::new(lexer);

    let program = parser.parse_program();
    
    // Print the parsed program (AST) for debugging
    println!("Parsed Program: {:#?}", program);

    // Code Generation
    let bytecode = CodeGenerator::generate(&program);
    
    // Print generated bytecode for debugging
    println!("Generated Bytecode: {:?}", bytecode);

    // Create a Virtual Machine
    let mut vm = VirtualMachine::new(bytecode);

    // Run the VM which will process opcodes for neuron creation and streaming
    println!("Running the virtual machine...");
    vm.run().await;

    Ok(())

}