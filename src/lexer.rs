

#[derive(PartialEq, Clone, Debug)]
pub enum Token {
    Neuron,
    Identifier(String),
    Number(i64),
    OpenBrace,
    CloseBrace,
    Colon,
    ArrowSend,  // "->"
    ArrowReceive,  // "<-"
    Pipe,  // "|"
    ParenOpen,  // "("
    ParenClose,  // ")"
    Comma,
    Encryption(String),
    EOF,
    Hex(String),
    Invalid,
}

#[derive(Clone, Debug)]
pub struct Lexer {
    input: String,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut lexer = Lexer {
            position: 0,
            current_char: input.chars().next(),
            input,
        };
        lexer
    }

    fn advance(&mut self) {
        self.position += 1;
        if self.position >= self.input.len() {
            self.current_char = None;
        } else {
            self.current_char = self.input.chars().nth(self.position);
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_hex(&mut self) -> Token {
        let mut hex_str = String::new();
        if self.current_char == Some('0') {
            self.advance();
            if self.current_char == Some('x') {
                hex_str.push_str("0x");
                self.advance();

                // Read the hexadecimal value
                while let Some(c) = self.current_char {
                    if c.is_digit(16) {
                        hex_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }

                return Token::Hex(hex_str); // Return hex as a Hex token
            }
        }
        Token::Invalid
    }

    fn read_number(&mut self) -> Token {
        let mut num_str = String::new();
        while let Some(c) = self.current_char {
            if c.is_digit(10) {
                num_str.push(c);
                self.advance(); // move to the next char
            } else {
                break;
            }
        }
        Token::Number(num_str.parse().unwrap())
    }

    fn read_identifier(&mut self) -> Token {
        let mut id_str = String::new();

        // Handle hex values like "0xff24"
        if self.current_char == Some('0') {
            self.advance();
            if self.current_char == Some('x') {
                id_str.push_str("0x");
                self.advance();

                // Parse the hex digits
                while let Some(c) = self.current_char {
                    if c.is_digit(16) {
                        id_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }

                return Token::Identifier(id_str); // Return hex as an identifier for now
            }
        }

        // Normal identifier parsing
        while let Some(c) = self.current_char {
            if c.is_alphabetic() || c == '_' || c.is_digit(10) {
                id_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        match id_str.as_str() {
            "neuron" => Token::Neuron,
            "aes256" => Token::Encryption("aes256".to_string()),
            _ => Token::Identifier(id_str),
        }
    }
    
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if let Some(c) = self.current_char {
            println!("Lexer reading character: '{}'", c); // Log each character

            match c {
                '{' => {
                    self.advance();
                    Token::OpenBrace
                }
                '}' => {
                    self.advance();
                    Token::CloseBrace
                }
                ':' => {
                    self.advance();
                    Token::Colon
                }
                '-' => {
                    self.advance();
                    if self.current_char == Some('>') {
                        self.advance();
                        Token::ArrowSend
                    } else {
                        Token::Invalid
                    }
                }
                '<' => {
                    self.advance();
                    if self.current_char == Some('-') {
                        self.advance();
                        Token::ArrowReceive
                    } else {
                        Token::Invalid
                    }
                }
                '|' => {
                    self.advance();
                    Token::Pipe
                }
                '(' => {
                    self.advance();
                    if self.current_char == Some(')') {
                        self.advance();
                        Token::ParenOpen
                    } else {
                        Token::Invalid
                    }
                }
                ')' => {
                    self.advance();
                    Token::ParenClose
                }
                '=' => {
                    self.advance();
                    if self.current_char == Some('>') {
                        self.advance();
                        Token::ArrowSend // Recognize the arrow (=>)
                    } else {
                        println!("Unknown character encountered: '='");
                        Token::Invalid
                    }
                }
                '0' => self.read_hex(), // Handle hex numbers starting with "0x"
                ',' => {
                    self.advance();
                    Token::Comma
                }
                _ if c.is_digit(10) => self.read_number(),
                _ if c.is_alphabetic() => self.read_identifier(),
                _ => {
                    println!("Unknown character encountered: '{}'", c);
                    self.advance();
                    Token::Invalid
                }
            }
        } else {
            Token::EOF
        }
    }

}