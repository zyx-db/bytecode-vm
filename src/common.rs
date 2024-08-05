use crate::token_types::TokenType;
use crate::value::{Value, ValueArr};
use std::hint::unreachable_unchecked;

const STACK_SIZE: usize = 150;

pub enum OpCode {
    OpReturn,
    OpConstant,
    ConstantIdx(usize),
}

struct Token {
    variant: TokenType,
    start: usize,
    length: usize,
    line: usize,
}

impl Token {
    pub fn error(msg: String, line: usize) -> Self {
        Token {
            length: msg.len(),
            variant: TokenType::Error(msg),
            start: 0,
            line
        }
    }

    pub fn create(t: TokenType, start: usize, end: usize, line: usize) -> Self {
        Token {
            variant: t,
            start,
            length: end - start,
            line
        }
    }
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

    // scanner
    source: String,
    start: usize,
    current: usize,
    line: usize,
}

impl VM {
    pub fn new(debug: bool) -> Self {
        return VM {
            chunk: None,
            ip: 0,
            debug,
            stack: Stack::new(),
            // scanner:
            source: "".to_string(),
            start: 0,
            current: 0,
            line: 1,
        };
    }

    pub fn interpret(&mut self, source: String) -> Result<(), VM_Errors> {
        match self.compile(source) {
            Ok(_) => {
                self.ip = 0;
                self.run()
            }
            Err(e) => {Err(e)}
        }
        //self.chunk = Some(c);
        //self.ip = 0;
        //return self.run();
    }

    // TODO: chap 17
    fn compile(&mut self, source: String) -> Result<(), VM_Errors> {
        let mut line = 0;
        self.source = source;
        loop {
            let token: Token = self.scanToken();
            if token.line != line {
                line = token.line;
                print!("{} ", line);
            } else {
                print!(" |");
            }
            println!("{:?} {} {}", token.variant, token.length, token.start);

            match token.variant {
                TokenType::Eof => {
                    println!("WE SHOULD BREAK HERE");
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    fn scanToken(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        let c: Option<char> = self.advance();

        use TokenType::*;
        match c {
            // at the end
            Some('\0') => {Token::create(Eof, self.start, self.current, self.line)}
            Some('(') => {Token::create(LeftParen, self.start, self.current, self.line)}
            Some(')') => {Token::create(RightParen, self.start, self.current, self.line)}
            Some('{') => {Token::create(LeftBrace, self.start, self.current, self.line)}
            Some('}') => {Token::create(RightBrace, self.start, self.current, self.line)}
            Some(';') => {Token::create(Semicolon, self.start, self.current, self.line)}
            Some(',') => {Token::create(Comma, self.start, self.current, self.line)}
            Some('.') => {Token::create(Dot, self.start, self.current, self.line)}
            Some('-') => {Token::create(Minus, self.start, self.current, self.line)}
            Some('+') => {Token::create(Plus, self.start, self.current, self.line)}
            Some('/') => {Token::create(Slash, self.start, self.current, self.line)}
            Some('*') => {Token::create(Star, self.start, self.current, self.line)}
            Some('!') => {
                let t = if self.check('=') {BangEqual} else {Bang};
                Token::create(t, self.start, self.current, self.line)
            }
            Some('=') => {
                let t = if self.check('=') {EqualEqual} else {Equal};
                Token::create(t, self.start, self.current, self.line)
            }
            Some('<') => {
                let t = if self.check('=') {LessEqual} else {Less};
                Token::create(t, self.start, self.current, self.line)
            }
            Some('>') => {
                let t = if self.check('=') {GreaterEqual} else {Greater};
                Token::create(t, self.start, self.current, self.line)
            }
            Some('"') => {
                self.string()
            }
            Some('0'..='9') => {
                self.number()
            }
            Some('a'..='z') | Some('A'..='Z') | Some('_') => {
                self.identifier()
            }

            _ => {Token::error("Unexpected character.".to_string(), self.line)}
        }
    }

    fn identifier(&mut self) -> Token {
        loop {
            let c = self.peek();
            match c {
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {self.advance();}
                _ => {break;}
            }
        }
        Token::create(self.identifier_type(), self.start, self.current, self.line)
    }

    fn identifier_type(&mut self) -> TokenType {
        let start_c = self.source.chars().nth(self.start).expect("non utf8?");
        use TokenType::*;
        match start_c {
            'a' => {self.check_keyword(1, 2, "nd".into(), And)}
            'c' => {self.check_keyword(1, 4, "lass".into(), Class)}
            'e' => {self.check_keyword(1, 3, "lse".into(), Else)}
            'f' => {
                 if self.current - self.start > 1 {
                    match self.source.chars().nth(self.start + 1).expect("non utf8?"){
                        'a' => {self.check_keyword(2, 3, "lse".into(), False)}
                        'o' => {self.check_keyword(2, 1, "r".into(), For)}
                        'u' => {self.check_keyword(2, 1, "n".into(), Fun)}
                        _ => {Identifier}
                    }
                } else {
                    Identifier
                }

            }
            'i' => {self.check_keyword(1, 1, "f".into(), If)}
            'n' => {self.check_keyword(1, 2, "il".into(), Nil)}
            'o' => {self.check_keyword(1, 1, "r".into(), Or)}
            'p' => {self.check_keyword(1, 4, "rint".into(), Print)}
            'r' => {self.check_keyword(1, 5, "eturn".into(), Return)}
            's' => {self.check_keyword(1, 4, "uper".into(), Super)}
            't' => {
                if self.current - self.start > 1 {
                    match self.source.chars().nth(self.start + 1).expect("non utf8?"){
                        'h' => {self.check_keyword(2, 2, "is".into(), This)}
                        'r' => {self.check_keyword(2, 2, "ue".into(), True)}
                        _ => {Identifier}
                    }
                } else {
                    Identifier
                }
            }
            'v' => {self.check_keyword(1, 2, "ar".into(), Var)}
            'w' => {self.check_keyword(1, 4, "hile".into(), While)}

            _ => {Identifier}
        }
    }

    fn check_keyword(&mut self, start: usize, length: usize, remaining: String, t: TokenType) -> TokenType {
        if (self.current - self.start) == (start + length){
            let mut i = self.start + start;
            let mut offset = 0;
            let mut valid: bool = true;
            while i <= self.current {
                let left = self.source.chars().nth(i).expect("non utf8?");
                let right = remaining.chars().nth(offset).unwrap();
                if left != right {
                    valid = false;
                    break;
                }
                i += 1;
                offset += 1;
            }

            match valid {
                true => {t}
                false => {TokenType::Identifier}
            }
        } else {
            TokenType::Identifier
        }
    }

    fn number(&mut self) -> Token {
        let c = self.peek();
        loop {
            match c {
                '0'..='9' => {self.advance();}
                _ => {break;}
            }
        }

        let c = self.peek();
        let cn = self.peek_next();
        if c == '.' && cn >= '0' && cn <= '9' {
            // "eat" the '.'
            self.advance();

            loop {
                match c {
                    '0'..='9' => {self.advance();}
                    _ => {break;}
                }
            }
        }

        Token::create(TokenType::Number, self.start, self.current, self.line)
    }

    fn string(&mut self) -> Token {
        while self.peek() != '"' && self.peek() != '\0' {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.peek() == '\0' {
            return Token::error("Unterminated String".to_string(), self.line);
        }

        self.advance();
        Token::create(TokenType::Str, self.start, self.current, self.line)
    }

    fn skip_whitespace(&mut self){
        loop {
            let c = self.peek();
            match c {
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && self.peek() != '\0' {
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                _ => {break}
            }
        }
    }

    fn peek(&self) -> char {
        self.source.chars().nth(self.current).expect("peek non utf?")
    }

    fn peek_next(&self) -> char {
        if self.source.chars().nth(self.current).expect("non utf?") == '\0' {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).expect("non utf?")
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.current += 1;
        self.source.chars().nth(self.current - 1)
    }

    fn check(&mut self, expected: char) -> bool {
        let cur = self.source.chars().nth(self.current).expect("non-utf?");
        if cur != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn run(&mut self) -> Result<(), VM_Errors> {
        loop {
            let instruction = &(self.chunk.as_ref().expect("could not access chunk?")).code[self.ip];
            if self.debug {
                let data = self.chunk.as_ref().expect("could not access chunk?");
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
