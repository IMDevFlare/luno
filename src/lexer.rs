#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Let,
    Be,
    Define,
    Def,
    GiveBack,
    Return,
    If,
    Elif,
    Then,
    Otherwise,
    Else,
    Repeat,
    From,
    To,
    As,
    For,
    Each,
    In,
    While,
    Build,
    Class,
    New,
    Say,
    Print,
    Log,
    Warn,
    Error,
    Try,
    Except,
    With,
    Raise,
    Pass,
    Break,
    Continue,
    Lambda,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    EqualEqual,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    LessLess,
    And,
    Or,
    Not,
    PlusPlus,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,

    // Literals
    Identifier(String),
    String(String),
    Number(f64),
    Bool(bool),
    Nil,

    // Symbols
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Colon,
    SemiColon,

    Eof,
}

pub struct Lexer {
    source: Vec<char>,
    current: usize,
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            current: 0,
            tokens: Vec::new(),
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, String> {
        while !self.is_at_end() {
            self.scan_token()?;
        }
        self.tokens.push(Token::Eof);
        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<(), String> {
        let c = self.advance();
        match c {
            '(' => self.tokens.push(Token::LeftParen),
            ')' => self.tokens.push(Token::RightParen),
            '{' => self.tokens.push(Token::LeftBrace),
            '}' => self.tokens.push(Token::RightBrace),
            '[' => self.tokens.push(Token::LeftBracket),
            ']' => self.tokens.push(Token::RightBracket),
            ',' => self.tokens.push(Token::Comma),
            '.' => self.tokens.push(Token::Dot),
            ':' => self.tokens.push(Token::Colon),
            '+' => {
                if self.match_char('+') {
                    self.tokens.push(Token::PlusPlus);
                } else {
                    self.tokens.push(Token::Plus);
                }
            }
            '-' => {
                if self.match_char('-') {
                    // Comment
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.tokens.push(Token::Minus);
                }
            }
            '*' => self.tokens.push(Token::Star),
            '/' => self.tokens.push(Token::Slash),
            '=' => {
                if self.match_char('=') {
                    self.tokens.push(Token::EqualEqual);
                } else {
                    self.tokens.push(Token::Equal);
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.tokens.push(Token::BangEqual);
                } else {
                    return Err("Unexpected character: !".to_string());
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.tokens.push(Token::GreaterEqual);
                } else {
                    self.tokens.push(Token::Greater);
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.tokens.push(Token::LessEqual);
                } else {
                    self.tokens.push(Token::Less);
                }
            }
            '#' => {
                // Comment
                while self.peek() != '\n' && !self.is_at_end() {
                    self.advance();
                }
            }
            ' ' | '\r' | '\t' | '\n' => {}
            '"' => self.string()?,
            _ => {
                if c.is_ascii_digit() {
                    self.number();
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier();
                } else {
                    return Err(format!("Unexpected character: {}", c));
                }
            }
        }
        Ok(())
    }

    fn identifier(&mut self) {
        let mut text = self.source[self.current - 1].to_string();
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            text.push(self.advance());
        }

        // Handle multi-word operators/keywords

        // Peek ahead for "is greater than", etc.
        if text == "is" {
            // Check for "is equal to", "is greater than", etc.
            // Simplified for now: just basic keywords
        }

        let token = match text.as_str() {
            "let" => Token::Let,
            "be" => Token::Be,
            "define" => Token::Define,
            "give" => {
                if self.match_sequence("back") {
                    Token::GiveBack
                } else {
                    Token::Identifier(text)
                }
            }
            "if" => Token::If,
            "then" => Token::Then,
            "otherwise" => Token::Otherwise,
            "repeat" => Token::Repeat,
            "from" => Token::From,
            "to" => Token::To,
            "as" => Token::As,
            "for" => Token::For,
            "each" => Token::Each,
            "in" => Token::In,
            "while" => Token::While,
            "build" => Token::Build,
            "new" => Token::New,
            "say" => Token::Say,
            "log" => Token::Log,
            "warn" => Token::Warn,
            "error" => Token::Error,
            "plus" => Token::Plus,
            "minus" => Token::Minus,
            "times" => Token::Star,
            "divided" => {
                if self.match_sequence("by") {
                    Token::Slash
                } else {
                    Token::Identifier(text)
                }
            }
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "is" => {
                if self.match_sequence("greater than or equal to") {
                    Token::GreaterEqual
                } else if self.match_sequence("less than or equal to") {
                    Token::LessEqual
                } else if self.match_sequence("greater than") {
                    Token::Greater
                } else if self.match_sequence("less than") {
                    Token::Less
                } else if self.match_sequence("equal to") {
                    Token::EqualEqual
                } else if self.match_sequence("not equal to") {
                    Token::BangEqual
                } else {
                    Token::Identifier(text)
                }
            }
            "not" => {
                if self.match_sequence("equal to") {
                    Token::BangEqual
                } else {
                    Token::Not
                }
            }
            "nil" => Token::Nil,
            "and" => Token::And,
            "or" => Token::Or,
            _ => Token::Identifier(text),
        };
        self.tokens.push(token);
    }

    fn string(&mut self) -> Result<(), String> {
        let mut value = String::new();
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                return Err("Unterminated string".to_string());
            }
            value.push(self.advance());
        }

        if self.is_at_end() {
            return Err("Unterminated string".to_string());
        }

        self.advance(); // Closing quote
        self.tokens.push(Token::String(value));
        Ok(())
    }

    fn number(&mut self) {
        let mut text = self.source[self.current - 1].to_string();
        while self.peek().is_ascii_digit() {
            text.push(self.advance());
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            text.push(self.advance());
            while self.peek().is_ascii_digit() {
                text.push(self.advance());
            }
        }

        self.tokens.push(Token::Number(text.parse().unwrap()));
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        c
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn match_sequence(&mut self, suffix: &str) -> bool {
        let saved_current = self.current;
        // Skip whitespace
        while !self.is_at_end() && self.source[self.current].is_whitespace() {
            self.current += 1;
        }

        for c in suffix.chars() {
            if self.is_at_end() || self.source[self.current] != c {
                self.current = saved_current;
                return false;
            }
            self.current += 1;
        }

        // Ensure it's the end of word
        if !self.is_at_end() && self.source[self.current].is_alphanumeric() {
            self.current = saved_current;
            return false;
        }

        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
