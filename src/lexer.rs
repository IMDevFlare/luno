// --- lexer.rs — Luno Lexer ---
// Tokenizes Luno source code into a Vec<Token>.
// Handles indentation-based blocks by emitting Indent/Dedent tokens.

use std::fmt;

// --- Token Types ---

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Token {
    // Literals
    Int(i64),
    Float(f64),
    Str(String),
    InterpolatedStr(Vec<StringPart>),
    True,
    False,
    Null,
    Identifier(String),

    // Keywords
    Let,
    Const,
    Fn,
    Return,
    If,
    Elif,
    Else,
    For,
    In,
    While,
    Break,
    Continue,
    Match,
    Case,
    Class,
    Import,
    From,
    Try,
    Catch,
    Finally,
    Raise,
    ErrorKw,
    Lam,
    And,
    Or,
    Not,
    As,
    Self_,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    DoubleStar,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    Arrow,    // ->
    FatArrow, // =>
    Dot,
    DotDot, // ..

    // Delimiters
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    StarArgs, // *args marker

    // Structure
    Newline,
    Indent,
    Dedent,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Expr(Vec<Token>),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Int(n) => write!(f, "{}", n),
            Token::Float(n) => write!(f, "{}", n),
            Token::Str(s) => write!(f, "\"{}\"", s),
            Token::Identifier(s) => write!(f, "{}", s),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Null => write!(f, "null"),
            _ => write!(f, "{:?}", self),
        }
    }
}

// --- Lexer ---

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    pub line: usize,
    pub col: usize,
    indent_stack: Vec<usize>,
    pending_tokens: Vec<Token>,
    at_line_start: bool,
    paren_depth: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 0,
            indent_stack: vec![0],
            pending_tokens: Vec::new(),
            at_line_start: true,
            paren_depth: 0,
        }
    }

    // --- Main tokenization entry point ---
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens: Vec<Token> = Vec::new();

        loop {
            // Drain any pending indent/dedent tokens first
            while let Some(t) = self.pending_tokens.pop() {
                tokens.push(t);
            }

            if self.pos >= self.source.len() {
                break;
            }

            // Handle indentation at line start
            if self.at_line_start && self.paren_depth == 0 {
                self.handle_indentation(&mut tokens)?;
                self.at_line_start = false;
                continue;
            }

            let ch = self.current();

            // Skip spaces/tabs (not newlines)
            if ch == ' ' || ch == '\t' {
                self.advance();
                continue;
            }

            // Newline
            if ch == '\n' {
                if self.paren_depth == 0 {
                    // Only push newline if previous token isn't already a newline
                    if tokens.last() != Some(&Token::Newline)
                        && tokens.last() != Some(&Token::Indent)
                    {
                        tokens.push(Token::Newline);
                    }
                }
                self.advance();
                self.line += 1;
                self.col = 0;
                self.at_line_start = true;
                continue;
            }

            // Carriage return
            if ch == '\r' {
                self.advance();
                continue;
            }

            // Single-line comment: #
            // Multi-line comment: ## ... ##
            if ch == '#' {
                if self.peek() == Some('#') {
                    // multi-line comment
                    self.advance(); // skip first #
                    self.advance(); // skip second #
                    loop {
                        if self.pos >= self.source.len() {
                            return Err(format!(
                                "Line {}: unterminated multi-line comment",
                                self.line
                            ));
                        }
                        if self.current() == '#' && self.peek() == Some('#') {
                            self.advance();
                            self.advance();
                            break;
                        }
                        if self.current() == '\n' {
                            self.line += 1;
                            self.col = 0;
                        }
                        self.advance();
                    }
                } else {
                    // single-line comment
                    while self.pos < self.source.len() && self.current() != '\n' {
                        self.advance();
                    }
                }
                continue;
            }

            // Number literals
            if ch.is_ascii_digit() {
                tokens.push(self.read_number()?);
                continue;
            }

            // String literals (double or single quote)
            if ch == '"' || ch == '\'' {
                tokens.push(self.read_string(ch)?);
                continue;
            }

            // Backtick interpolated strings
            if ch == '`' {
                tokens.push(self.read_interpolated_string()?);
                continue;
            }

            // Identifiers and keywords
            if ch.is_alphabetic() || ch == '_' {
                tokens.push(self.read_identifier());
                continue;
            }

            // Operators and delimiters
            match ch {
                '+' => {
                    self.advance();
                    if self.current_opt() == Some('=') {
                        self.advance();
                        tokens.push(Token::PlusAssign);
                    } else {
                        tokens.push(Token::Plus);
                    }
                }
                '-' => {
                    self.advance();
                    if self.current_opt() == Some('>') {
                        self.advance();
                        tokens.push(Token::Arrow);
                    } else if self.current_opt() == Some('=') {
                        self.advance();
                        tokens.push(Token::MinusAssign);
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                '*' => {
                    self.advance();
                    if self.current_opt() == Some('*') {
                        self.advance();
                        tokens.push(Token::DoubleStar);
                    } else if self.current_opt() == Some('=') {
                        self.advance();
                        tokens.push(Token::StarAssign);
                    } else {
                        tokens.push(Token::Star);
                    }
                }
                '/' => {
                    self.advance();
                    if self.current_opt() == Some('=') {
                        self.advance();
                        tokens.push(Token::SlashAssign);
                    } else {
                        tokens.push(Token::Slash);
                    }
                }
                '%' => {
                    self.advance();
                    tokens.push(Token::Percent);
                }
                '=' => {
                    self.advance();
                    if self.current_opt() == Some('=') {
                        self.advance();
                        tokens.push(Token::Eq);
                    } else if self.current_opt() == Some('>') {
                        self.advance();
                        tokens.push(Token::FatArrow);
                    } else {
                        tokens.push(Token::Assign);
                    }
                }
                '!' => {
                    self.advance();
                    if self.current_opt() == Some('=') {
                        self.advance();
                        tokens.push(Token::NotEq);
                    } else {
                        tokens.push(Token::Not);
                    }
                }
                '<' => {
                    self.advance();
                    if self.current_opt() == Some('=') {
                        self.advance();
                        tokens.push(Token::LtEq);
                    } else {
                        tokens.push(Token::Lt);
                    }
                }
                '>' => {
                    self.advance();
                    if self.current_opt() == Some('=') {
                        self.advance();
                        tokens.push(Token::GtEq);
                    } else {
                        tokens.push(Token::Gt);
                    }
                }
                '.' => {
                    self.advance();
                    if self.current_opt() == Some('.') {
                        self.advance();
                        tokens.push(Token::DotDot);
                    } else {
                        tokens.push(Token::Dot);
                    }
                }
                '(' => {
                    self.advance();
                    self.paren_depth += 1;
                    tokens.push(Token::LeftParen);
                }
                ')' => {
                    self.advance();
                    if self.paren_depth > 0 {
                        self.paren_depth -= 1;
                    }
                    tokens.push(Token::RightParen);
                }
                '[' => {
                    self.advance();
                    self.paren_depth += 1;
                    tokens.push(Token::LeftBracket);
                }
                ']' => {
                    self.advance();
                    if self.paren_depth > 0 {
                        self.paren_depth -= 1;
                    }
                    tokens.push(Token::RightBracket);
                }
                '{' => {
                    self.advance();
                    self.paren_depth += 1;
                    tokens.push(Token::LeftBrace);
                }
                '}' => {
                    self.advance();
                    if self.paren_depth > 0 {
                        self.paren_depth -= 1;
                    }
                    tokens.push(Token::RightBrace);
                }
                ',' => {
                    self.advance();
                    tokens.push(Token::Comma);
                }
                ':' => {
                    self.advance();
                    tokens.push(Token::Colon);
                }
                _ => {
                    return Err(format!("Line {}: unexpected character '{}'", self.line, ch));
                }
            }
        }

        // Emit remaining dedents at EOF
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(Token::Dedent);
        }
        // Ensure trailing newline
        if tokens.last() != Some(&Token::Newline) && tokens.last() != Some(&Token::Dedent) {
            tokens.push(Token::Newline);
        }
        tokens.push(Token::Eof);
        Ok(tokens)
    }

    // --- Helpers ---

    fn current(&self) -> char {
        self.source[self.pos]
    }

    fn current_opt(&self) -> Option<char> {
        if self.pos < self.source.len() {
            Some(self.source[self.pos])
        } else {
            None
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos + 1 < self.source.len() {
            Some(self.source[self.pos + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
        self.col += 1;
    }

    // --- Indentation handling ---

    fn handle_indentation(&mut self, tokens: &mut Vec<Token>) -> Result<(), String> {
        let mut indent = 0;

        // Count leading spaces (tabs -> 4 spaces)
        while self.pos < self.source.len() {
            match self.source[self.pos] {
                ' ' => {
                    indent += 1;
                    self.advance();
                }
                '\t' => {
                    indent += 4;
                    self.advance();
                }
                _ => break,
            }
        }

        // Skip blank lines and comment-only lines
        if self.pos >= self.source.len()
            || self.source[self.pos] == '\n'
            || self.source[self.pos] == '\r'
        {
            return Ok(());
        }
        if self.source[self.pos] == '#' {
            return Ok(());
        }

        let current_indent = *self.indent_stack.last().unwrap();

        if indent > current_indent {
            self.indent_stack.push(indent);
            tokens.push(Token::Indent);
        } else if indent < current_indent {
            while *self.indent_stack.last().unwrap() > indent {
                self.indent_stack.pop();
                tokens.push(Token::Dedent);
            }
            if *self.indent_stack.last().unwrap() != indent {
                return Err(format!("Line {}: inconsistent indentation", self.line));
            }
        }

        Ok(())
    }

    // --- Number literal ---

    fn read_number(&mut self) -> Result<Token, String> {
        let mut num_str = String::new();
        let mut is_float = false;

        while self.pos < self.source.len()
            && (self.current().is_ascii_digit() || self.current() == '.' || self.current() == '_')
        {
            if self.current() == '.' {
                // Check if next char is also a dot (range operator)
                if self.peek() == Some('.') {
                    break;
                }
                if is_float {
                    return Err(format!("Line {}: invalid number literal", self.line));
                }
                is_float = true;
            }
            if self.current() != '_' {
                num_str.push(self.current());
            }
            self.advance();
        }

        if is_float {
            num_str
                .parse::<f64>()
                .map(Token::Float)
                .map_err(|_| format!("Line {}: invalid float literal", self.line))
        } else {
            num_str
                .parse::<i64>()
                .map(Token::Int)
                .map_err(|_| format!("Line {}: invalid integer literal", self.line))
        }
    }

    // --- String literal ---

    fn read_string(&mut self, quote: char) -> Result<Token, String> {
        self.advance(); // skip opening quote
        let mut s = String::new();

        while self.pos < self.source.len() && self.current() != quote {
            if self.current() == '\\' {
                self.advance();
                if self.pos >= self.source.len() {
                    return Err(format!("Line {}: unterminated string escape", self.line));
                }
                match self.current() {
                    'n' => s.push('\n'),
                    't' => s.push('\t'),
                    'r' => s.push('\r'),
                    '\\' => s.push('\\'),
                    '\'' => s.push('\''),
                    '"' => s.push('"'),
                    '0' => s.push('\0'),
                    _ => {
                        s.push('\\');
                        s.push(self.current());
                    }
                }
            } else {
                if self.current() == '\n' {
                    self.line += 1;
                    self.col = 0;
                }
                s.push(self.current());
            }
            self.advance();
        }

        if self.pos >= self.source.len() {
            return Err(format!("Line {}: unterminated string literal", self.line));
        }
        self.advance(); // skip closing quote
        Ok(Token::Str(s))
    }

    // --- Interpolated string (backtick) ---

    fn read_interpolated_string(&mut self) -> Result<Token, String> {
        self.advance(); // skip opening backtick
        let mut parts: Vec<StringPart> = Vec::new();
        let mut current_lit = String::new();

        while self.pos < self.source.len() && self.current() != '`' {
            if self.current() == '$' && self.peek() == Some('{') {
                // Save current literal part
                if !current_lit.is_empty() {
                    parts.push(StringPart::Literal(current_lit.clone()));
                    current_lit.clear();
                }
                self.advance(); // skip $
                self.advance(); // skip {

                // Collect tokens until matching }
                let mut depth = 1;
                let mut expr_src = String::new();
                while self.pos < self.source.len() && depth > 0 {
                    if self.current() == '{' {
                        depth += 1;
                    } else if self.current() == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    expr_src.push(self.current());
                    self.advance();
                }
                if depth != 0 {
                    return Err(format!("Line {}: unterminated interpolation", self.line));
                }
                self.advance(); // skip closing }

                // Tokenize the expression inside ${}
                let mut inner_lexer = Lexer::new(&expr_src);
                inner_lexer.at_line_start = false;
                let mut inner_tokens = inner_lexer.tokenize()?;
                // Remove trailing Eof and Newline
                inner_tokens.retain(|t| *t != Token::Eof && *t != Token::Newline);
                parts.push(StringPart::Expr(inner_tokens));
            } else if self.current() == '\\' {
                self.advance();
                if self.pos >= self.source.len() {
                    return Err(format!(
                        "Line {}: unterminated escape in interpolated string",
                        self.line
                    ));
                }
                match self.current() {
                    'n' => current_lit.push('\n'),
                    't' => current_lit.push('\t'),
                    '\\' => current_lit.push('\\'),
                    '`' => current_lit.push('`'),
                    '$' => current_lit.push('$'),
                    _ => {
                        current_lit.push('\\');
                        current_lit.push(self.current());
                    }
                }
                self.advance();
            } else {
                if self.current() == '\n' {
                    self.line += 1;
                    self.col = 0;
                }
                current_lit.push(self.current());
                self.advance();
            }
        }

        if self.pos >= self.source.len() {
            return Err(format!(
                "Line {}: unterminated interpolated string",
                self.line
            ));
        }
        self.advance(); // skip closing backtick

        if !current_lit.is_empty() {
            parts.push(StringPart::Literal(current_lit));
        }

        // Optimise: if it's just a single literal, return a plain string
        if parts.len() == 1 {
            if let StringPart::Literal(s) = &parts[0] {
                return Ok(Token::Str(s.clone()));
            }
        }

        Ok(Token::InterpolatedStr(parts))
    }

    // --- Identifier / keyword ---

    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();
        while self.pos < self.source.len()
            && (self.current().is_alphanumeric() || self.current() == '_')
        {
            ident.push(self.current());
            self.advance();
        }

        match ident.as_str() {
            "let" => Token::Let,
            "const" => Token::Const,
            "fn" => Token::Fn,
            "return" => Token::Return,
            "if" => Token::If,
            "elif" => Token::Elif,
            "else" => Token::Else,
            "for" => Token::For,
            "in" => Token::In,
            "while" => Token::While,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "match" => Token::Match,
            "case" => Token::Case,
            "class" => Token::Class,
            "import" => Token::Import,
            "from" => Token::From,
            "try" => Token::Try,
            "catch" => Token::Catch,
            "finally" => Token::Finally,
            "raise" => Token::Raise,
            "error" => Token::ErrorKw,
            "lam" => Token::Lam,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            "as" => Token::As,
            "self" => Token::Self_,
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            _ => Token::Identifier(ident),
        }
    }
}
