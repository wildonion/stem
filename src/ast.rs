
#[derive(Debug)]
pub struct Neuron {
    pub name: String,
    pub cells: i64,
    pub shellcode: String,
    pub encryption: String,
    pub nested_data: Vec<(String, String)>,  // (key, value) pairs
}

#[derive(Debug)]
pub struct Callback {
    pub body: String, // Placeholder for now. You could parse the actual callback body in the future.
}

#[derive(Debug)]
pub enum StreamDirection {
    Send,    // Represents '->'
    Receive, // Represents '<-'
}

#[derive(Debug)]
pub struct StreamStatement {
    pub neuron_name: String,
    pub direction: StreamDirection,
    pub data: String,
    pub callback: Callback,
}

#[derive(Debug)]
pub enum Statement {
    NeuronDeclaration(Neuron),
    LocalStream(StreamStatement), // Represents both send and receive streams
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}
