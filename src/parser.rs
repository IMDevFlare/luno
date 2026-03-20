use crate::ast::*;
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Statement, String> {
        if self.match_token(&[Token::Let]) {
            self.let_declaration()
        } else if self.match_token(&[Token::Define]) {
            self.function_declaration()
        } else if self.match_token(&[Token::Build]) {
            self.class_declaration()
        } else {
            self.statement()
        }
    }

    fn let_declaration(&mut self) -> Result<Statement, String> {
        let name = match self.advance() {
            Token::Identifier(name) => name,
            _ => return Err("Expected variable name after 'let'".to_string()),
        };

        let mut type_annotation = None;
        if self.match_token(&[Token::Colon]) {
            type_annotation = Some(match self.advance() {
                Token::Identifier(t) => t,
                _ => return Err("Expected type name after ':'".to_string()),
            });
        }

        if !self.match_token(&[Token::Be, Token::Equal]) {
            return Err("Expected 'be' or '=' after variable name".to_string());
        }

        let initializer = self.expression()?;
        Ok(Statement::Let {
            name,
            type_annotation,
            initializer,
        })
    }

    fn function_declaration(&mut self) -> Result<Statement, String> {
        let name = match self.advance() {
            Token::Identifier(name) => name,
            _ => return Err("Expected function name".to_string()),
        };

        self.consume(Token::LeftParen, "Expected '(' after function name")?;
        let mut params = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                match self.advance() {
                    Token::Identifier(p) => params.push(p),
                    _ => return Err("Expected parameter name".to_string()),
                }
                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }
        }
        self.consume(Token::RightParen, "Expected ')' after parameters")?;

        self.consume(Token::LeftBrace, "Expected '{' before function body")?;
        let body = self.block()?;

        Ok(Statement::Function { name, params, body })
    }

    fn class_declaration(&mut self) -> Result<Statement, String> {
        let name = match self.advance() {
            Token::Identifier(name) => name,
            _ => return Err("Expected class name".to_string()),
        };

        // Simplified: classes just take constructor params in parens then brace
        self.consume(Token::LeftParen, "Expected '(' after class name")?;
        // skip params for now or handle them
        while !self.check(&Token::RightParen) && !self.is_at_end() {
            self.advance();
        }
        self.consume(Token::RightParen, "Expected ')'")?;

        self.consume(Token::LeftBrace, "Expected '{' before class body")?;
        let mut methods = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            methods.push(self.declaration()?);
        }
        self.consume(Token::RightBrace, "Expected '}' after class body")?;

        Ok(Statement::Class { name, methods })
    }

    fn statement(&mut self) -> Result<Statement, String> {
        if self.match_token(&[Token::If]) {
            self.if_statement()
        } else if self.match_token(&[Token::While]) {
            self.while_statement()
        } else if self.match_token(&[Token::Repeat]) {
            self.repeat_statement()
        } else if self.match_token(&[Token::Say]) {
            let expr = self.expression()?;
            Ok(Statement::Say(expr))
        } else if self.match_token(&[Token::GiveBack]) {
            let mut value = None;
            if !self.is_at_end() && !self.check(&Token::RightBrace) {
                value = Some(self.expression()?);
            }
            Ok(Statement::Return(value))
        } else if self.match_token(&[Token::LeftBrace]) {
            Ok(Statement::Expression(Expr::Literal(Value::Nil))) // Placeholder for blocks
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Statement, String> {
        let condition = self.expression()?;
        self.match_token(&[Token::Then]); // Optional 'then'

        self.consume(Token::LeftBrace, "Expected '{' after if condition")?;
        let then_branch = self.block()?;

        let mut else_branch = None;
        if self.match_token(&[Token::Otherwise]) {
            if self.match_token(&[Token::LeftBrace]) {
                else_branch = Some(self.block()?);
            } else {
                else_branch = Some(vec![self.statement()?]);
            }
        }

        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Statement, String> {
        let condition = self.expression()?;
        self.consume(Token::LeftBrace, "Expected '{' after while condition")?;
        let body = self.block()?;
        Ok(Statement::While { condition, body })
    }

    fn repeat_statement(&mut self) -> Result<Statement, String> {
        self.consume(Token::From, "Expected 'from' in repeat")?;
        let start = self.expression()?;
        self.consume(Token::To, "Expected 'to' in repeat")?;
        let end = self.expression()?;
        self.consume(Token::As, "Expected 'as' in repeat")?;
        let var_name = match self.advance() {
            Token::Identifier(name) => name,
            _ => return Err("Expected variable name in repeat".to_string()),
        };
        self.consume(Token::LeftBrace, "Expected '{' after repeat header")?;
        let body = self.block()?;
        Ok(Statement::Repeat {
            start,
            end,
            var_name,
            body,
        })
    }

    fn block(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(Token::RightBrace, "Expected '}' after block")?;
        Ok(statements)
    }

    fn expression_statement(&mut self) -> Result<Statement, String> {
        let expr = self.expression()?;
        Ok(Statement::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.or()
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;
        while self.match_token(&[Token::Or]) {
            let right = self.and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: Operator::Or,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;
        while self.match_token(&[Token::And]) {
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: Operator::And,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;
        while self.match_token(&[Token::EqualEqual, Token::BangEqual]) {
            let op = if self.previous() == Token::EqualEqual {
                Operator::Equal
            } else {
                Operator::NotEqual
            };
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;
        while self.match_token(&[
            Token::Greater,
            Token::GreaterEqual,
            Token::Less,
            Token::LessEqual,
        ]) {
            let op = match self.previous() {
                Token::Greater => Operator::GreaterThan,
                Token::GreaterEqual => Operator::GreaterEqual,
                Token::Less => Operator::LessThan,
                Token::LessEqual => Operator::LessEqual,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;
        while self.match_token(&[Token::Plus, Token::Minus, Token::PlusPlus]) {
            let op = match self.previous() {
                Token::Plus => Operator::Plus,
                Token::Minus => Operator::Minus,
                Token::PlusPlus => Operator::Concat,
                _ => unreachable!(),
            };
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;
        while self.match_token(&[Token::Star, Token::Slash]) {
            let op = if self.previous() == Token::Star {
                Operator::Multiply
            } else {
                Operator::Divide
            };
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_token(&[Token::Not, Token::Minus]) {
            let op = if self.previous() == Token::Not {
                UnaryOperator::Not
            } else {
                UnaryOperator::Negate
            };
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator: op,
                right: Box::new(right),
            });
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;
        loop {
            if self.match_token(&[Token::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[Token::Dot]) {
                let name = match self.advance() {
                    Token::Identifier(n) => n,
                    _ => return Err("Expected property name after '.'".to_string()),
                };
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let mut arguments = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }
        }
        self.consume(Token::RightParen, "Expected ')' after arguments")?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            arguments,
        })
    }

    fn primary(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Token::Bool(b) => Ok(Expr::Literal(Value::Bool(b))),
            Token::Nil => Ok(Expr::Literal(Value::Nil)),
            Token::Number(n) => {
                if n.fract() == 0.0 {
                    Ok(Expr::Literal(Value::Int(n as i64)))
                } else {
                    Ok(Expr::Literal(Value::Float(n)))
                }
            }
            Token::String(s) => Ok(Expr::Literal(Value::String(s))),
            Token::Identifier(name) => Ok(Expr::Variable(name)),
            Token::LeftParen => {
                let expr = self.expression()?;
                self.consume(Token::RightParen, "Expected ')' after expression")?;
                Ok(Expr::Grouping(Box::new(expr)))
            }
            Token::LeftBracket => {
                let mut elements = Vec::new();
                if !self.check(&Token::RightBracket) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_token(&[Token::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(Token::RightBracket, "Expected ']' after list")?;
                Ok(Expr::List(elements))
            }
            _ => Err(format!("Expected expression, found {:?}", self.previous())),
        }
    }

    fn match_token(&mut self, types: &[Token]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, t: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        matches!(
            (&self.peek(), t),
            (Token::Let, Token::Let)
                | (Token::Be, Token::Be)
                | (Token::Define, Token::Define)
                | (Token::GiveBack, Token::GiveBack)
                | (Token::If, Token::If)
                | (Token::Then, Token::Then)
                | (Token::Otherwise, Token::Otherwise)
                | (Token::Repeat, Token::Repeat)
                | (Token::From, Token::From)
                | (Token::To, Token::To)
                | (Token::As, Token::As)
                | (Token::For, Token::For)
                | (Token::Each, Token::Each)
                | (Token::In, Token::In)
                | (Token::While, Token::While)
                | (Token::Build, Token::Build)
                | (Token::New, Token::New)
                | (Token::Say, Token::Say)
                | (Token::Log, Token::Log)
                | (Token::Warn, Token::Warn)
                | (Token::Error, Token::Error)
                | (Token::Plus, Token::Plus)
                | (Token::Minus, Token::Minus)
                | (Token::Star, Token::Star)
                | (Token::Slash, Token::Slash)
                | (Token::Equal, Token::Equal)
                | (Token::EqualEqual, Token::EqualEqual)
                | (Token::BangEqual, Token::BangEqual)
                | (Token::Greater, Token::Greater)
                | (Token::GreaterEqual, Token::GreaterEqual)
                | (Token::Less, Token::Less)
                | (Token::LessEqual, Token::LessEqual)
                | (Token::And, Token::And)
                | (Token::Or, Token::Or)
                | (Token::Not, Token::Not)
                | (Token::PlusPlus, Token::PlusPlus)
                | (Token::LeftParen, Token::LeftParen)
                | (Token::RightParen, Token::RightParen)
                | (Token::LeftBrace, Token::LeftBrace)
                | (Token::RightBrace, Token::RightBrace)
                | (Token::LeftBracket, Token::LeftBracket)
                | (Token::RightBracket, Token::RightBracket)
                | (Token::Comma, Token::Comma)
                | (Token::Dot, Token::Dot)
                | (Token::Colon, Token::Colon)
                | (Token::Identifier(_), Token::Identifier(_))
                | (Token::String(_), Token::String(_))
                | (Token::Number(_), Token::Number(_))
                | (Token::Bool(_), Token::Bool(_))
                | (Token::Nil, Token::Nil)
        )
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, t: Token, msg: &str) -> Result<Token, String> {
        if self.check(&t) {
            Ok(self.advance())
        } else {
            Err(msg.to_string())
        }
    }
}
