use crate::lexer::{Lexer, Token};
use crate::ast::{Callback, Neuron, Program, Statement, StreamDirection, StreamStatement};

#[derive(Clone, Debug)]
pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        Parser { lexer, current_token }
    }

    fn next_token(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    pub fn parse_program(&mut self) -> Program {
        let mut statements = Vec::new();
        while self.current_token != Token::EOF {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }
        Program { statements }
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        println!("Parsing statement: {:?}", self.current_token);
        
        if self.current_token == Token::Invalid {
            println!("Error: Encountered Invalid token during parsing.");
            return None;
        }

        match &self.current_token {
            Token::Neuron => self.parse_neuron_declaration(),
            Token::Identifier(ident) if ident == "local" => self.parse_stream_statement(),
            _ => {
                println!("Unknown statement: {:?}", self.current_token);
                None
            },
        }
    }

    fn parse_neuron_declaration(&mut self) -> Option<Statement> {
        println!("Parsing neuron declaration...");

        self.next_token(); // Skip 'neuron'

        if let Token::Identifier(name) = &self.clone().current_token {
            println!("Neuron name: {:?}", name);
            self.next_token(); // Skip neuron name
            if self.current_token == Token::OpenBrace {
                self.next_token(); // Skip '{'
                let neuron = self.parse_neuron_body(name.clone());
                println!("Parsed neuron: {:#?}", neuron);
                return Some(Statement::NeuronDeclaration(neuron));
            } else {
                println!("Error: Expected '{{' after neuron name.");
            }
        } else {
            println!("Error: Expected identifier (neuron name) after 'neuron'.");
        }

        None
    }

    fn parse_neuron_body(&mut self, name: String) -> Neuron {
        println!("Parsing neuron body...");

        let mut cells = 0;
        let mut shellcode = String::new();
        let mut encryption = String::new();
        let mut nested_data = Vec::new();

        while self.current_token != Token::CloseBrace {
            println!("Current token in neuron body: {:?}", self.current_token);

            match &self.current_token {
                Token::Identifier(ident) if ident == "cells" => {
                    self.next_token(); // skip 'cells'
                    if self.current_token == Token::Colon {
                        self.next_token(); // skip ':'
                        if let Token::Number(value) = &self.current_token {
                            cells = *value;
                        } else {
                            println!("Error: Expected number after 'cells:'.");
                        }
                        self.next_token(); // move to next token after the value
                    } else {
                        println!("Error: Expected ':' after 'cells'.");
                    }
                }
                Token::Identifier(ident) if ident == "shellcodeConfig" => {
                    self.next_token(); // skip 'shellcodeConfig'
                    if self.current_token == Token::Colon {
                        self.next_token(); // skip ':'
                        if self.current_token == Token::OpenBrace {
                            self.next_token(); // skip '{'
                            self.parse_shellcode_config(&mut shellcode, &mut encryption);
                            if self.current_token == Token::CloseBrace {
                                self.next_token(); // skip closing '}'
                            } else {
                                println!("Error: Expected closing braces for shellcodeConfig.");
                            }
                        }
                    } else {
                        println!("Error: Expected ':' after 'shellcodeConfig'.");
                    }
                }
                Token::Identifier(ident) if ident == "nestedData" => {
                    self.next_token(); // skip 'nestedData'
                    if self.current_token == Token::Colon {
                        self.next_token(); // skip ':'
                        if self.current_token == Token::OpenBrace {
                            self.next_token(); // skip '{'
                            self.parse_nested_data(&mut nested_data);
                            if self.current_token == Token::CloseBrace {
                                self.next_token(); // skip closing '}'
                            } else {
                                println!("Error: Expected closing braces for nestedData.");
                            }
                        }
                    } else {
                        println!("Error: Expected ':' after 'nestedData'.");
                    }
                }
                Token::Comma => {
                    self.next_token(); // skip commas between fields
                }
                _ => {
                    println!("Error: Unexpected token in neuron body: {:?}", self.current_token);
                    self.next_token(); // skip the unexpected token
                }
            }
        }

        Neuron {
            name,
            cells,
            shellcode,
            encryption,
            nested_data,
        }
    }

    fn parse_shellcode_config(&mut self, shellcode: &mut String, encryption: &mut String) {
        while self.current_token != Token::CloseBrace {
            println!("Parsing shellcodeConfig: {:?}", self.current_token);
    
            match &self.current_token {
                Token::Identifier(ident) if ident == "shellcode" => {
                    self.next_token(); // Skip 'shellcode'
                    if self.current_token == Token::Colon {
                        self.next_token(); // Skip ':'
                        if let Token::Identifier(hex_value) = &self.current_token {
                            shellcode.push_str(hex_value); // Store the hex value or placeholder
                        } else {
                            println!("Error: Expected hexadecimal value or '0x...' after 'shellcode:'.");
                        }
                    }
                }
                Token::Identifier(ident) if ident == "encryption" => {
                    self.next_token(); // Skip 'encryption'
                    if self.current_token == Token::Colon {
                        self.next_token(); // Skip ':'
                        if let Token::Encryption(enc) = &self.current_token {
                            encryption.push_str(enc);
                        }
                    }
                }
                _ => {
                    println!("Error: Unexpected token in shellcodeConfig: {:?}", self.current_token);
                }
            }
    
            self.next_token();
        }
    }
    

    fn parse_nested_data(&mut self, nested_data: &mut Vec<(String, String)>) {
        while self.current_token != Token::CloseBrace {
            println!("Parsing nestedData: {:?}", self.current_token);

            if let Token::Identifier(key) = &self.current_token {
                self.clone().next_token(); // Skip key
                if self.current_token == Token::Colon {
                    self.clone().next_token(); // Skip ':'
                    if let Token::Identifier(value) = &self.current_token {
                        nested_data.push((key.clone(), value.clone()));
                    }
                }
            }

            self.next_token();
        }
    }

    fn parse_stream_statement(&mut self) -> Option<Statement> {
        // Expecting: "local stream over neuron_name"
        self.next_token(); // Skip 'local'

        if let Token::Identifier(ident) = &self.current_token {
            if ident == "stream" {
                self.next_token(); // Skip 'stream'

                if let Token::Identifier(neuron_name) = &self.current_token {
                    self.clone().next_token(); // Skip neuron name

                    if self.current_token == Token::Colon {
                        self.clone().next_token(); // Skip ':'
                        
                        // Handle send stream "data -> |"
                        if let Token::ArrowSend = &self.current_token {
                            self.clone().next_token(); // Skip '->'
                            let direction = StreamDirection::Send;
                            return self.parse_stream_callback(neuron_name.clone(), direction);
                        }
                        // Handle receive stream "data <- |"
                        else if let Token::ArrowReceive = &self.current_token {
                            self.clone().next_token(); // Skip '<-'
                            let direction = StreamDirection::Receive;
                            return self.parse_stream_callback(neuron_name.clone(), direction);
                        } else {
                            println!("Error: Expected '->' or '<-' after neuron name.");
                        }
                    } else {
                        println!("Error: Expected ':' after neuron name.");
                    }
                } else {
                    println!("Error: Expected neuron name after 'stream'.");
                }
            } else {
                println!("Error: Expected 'stream' after 'local'.");
            }
        }
        None
    }

    fn parse_stream_callback(&mut self, neuron_name: String, direction: StreamDirection) -> Option<Statement> {
        if let Token::Pipe = &self.current_token {
            self.next_token(); // Skip '|'
            
            if let Token::ParenOpen = &self.current_token {
                self.next_token(); // Skip '('

                if let Token::ParenClose = &self.current_token {
                    self.next_token(); // Skip ')'

                    if let Token::ArrowSend = &self.current_token {
                        self.next_token(); // Skip '=>'

                        if let Token::OpenBrace = &self.current_token {
                            self.next_token(); // Skip '{'
                            
                            // For simplicity, assume the callback is empty
                            if let Token::CloseBrace = &self.current_token {
                                self.next_token(); // Skip '}'

                                let stream_stmt = StreamStatement {
                                    neuron_name,
                                    direction,
                                    data: "data".to_string(),
                                    callback: Callback { body: "callback".to_string() }, // Placeholder
                                };

                                return Some(Statement::LocalStream(stream_stmt));
                            } else {
                                println!("Error: Expected closing braces after callback body.");
                            }
                        } else {
                            println!("Error: Expected braces after '=>'.");
                        }
                    } else {
                        println!("Error: Expected '=>' after '()'.");
                    }
                } else {
                    println!("Error: Expected ')' after '('.");
                }
            } else {
                println!("Error: Expected '()' after '|'.");
            }
        }
        None
    }

}
