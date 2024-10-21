use crate::ast::{Program, Statement, Neuron, StreamDirection, StreamStatement};

pub struct CodeGenerator;

impl CodeGenerator {
    pub fn generate(program: &Program) -> Vec<u8> {
        let mut bytecode = Vec::new();
        for statement in &program.statements {
            match statement {
                Statement::NeuronDeclaration(neuron) => {
                    Self::generate_neuron(&neuron, &mut bytecode);
                }
                Statement::LocalStream(stream) => {
                    Self::generate_stream(stream, &mut bytecode);
                }
            }
        }
        bytecode
    }

    fn generate_neuron(neuron: &Neuron, bytecode: &mut Vec<u8>) {
        bytecode.push(0x01); // Example: Opcode for Neuron Declaration
        bytecode.push(neuron.cells as u8); // Serialize the number of cells

        // Serialize shellcode (as an example)
        if !neuron.shellcode.is_empty() {
            bytecode.push(0x02); // Example: Opcode for shellcode config
        }

        // Serialize encryption config (if any)
        if !neuron.encryption.is_empty() {
            bytecode.push(0x03); // Example: Opcode for encryption
        }

        // Add more serialization logic as per the needs of your language
    }

    fn generate_stream(stream: &StreamStatement, bytecode: &mut Vec<u8>) {
        match stream.direction {
            StreamDirection::Send => {
                bytecode.push(0x02); // Opcode for send stream
            }
            StreamDirection::Receive => {
                bytecode.push(0x03); // Opcode for receive stream
            }
        }

        // Serialize data
        for byte in stream.data.as_bytes() {
            bytecode.push(*byte);
        }

        // Serialize callback (for now, just the body string)
        for byte in stream.callback.body.as_bytes() {
            bytecode.push(*byte);
        }
    }

}
