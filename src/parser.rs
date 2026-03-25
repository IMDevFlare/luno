// --- parser.rs — Luno Parser ---
// Builds a typed AST from a Vec<Token>.
// Uses recursive-descent parsing.

use crate::lexer::{StringPart, Token};

// --- AST Node Types ---

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Str(String),
    InterpolatedStr(Vec<StringPartExpr>),
    Bool(bool),
    Null,
    Identifier(String),
    SelfRef,

    // Operations
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Comparison {
        left: Box<Expr>,
        op: CmpOp,
        right: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        op: LogicOp,
        right: Box<Expr>,
    },

    // Access
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Attribute {
        object: Box<Expr>,
        name: String,
    },

    // Constructors
    ListLiteral(Vec<Expr>),
    MapLiteral(Vec<(Expr, Expr)>),
    SetLiteral(Vec<Expr>),

    // Lambda
    Lambda {
        params: Vec<Param>,
        body: Box<Expr>,
    },

    // Assignment expression (for things like self.x = ...)
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum StringPartExpr {
    Literal(String),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum CmpOp {
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
}

#[derive(Debug, Clone)]
pub enum LogicOp {
    And,
    Or,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Param {
    pub name: String,
    pub type_hint: Option<String>,
    pub default: Option<Expr>,
    pub variadic: bool,
}

// --- Statements ---

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Stmt {
    // Variable declarations
    Let {
        name: String,
        type_hint: Option<String>,
        value: Expr,
    },
    Const {
        name: String,
        type_hint: Option<String>,
        value: Expr,
    },

    // Assignment
    Assign {
        target: Expr,
        value: Expr,
    },
    AugAssign {
        target: Expr,
        op: BinOp,
        value: Expr,
    },

    // Expression statement
    ExprStmt(Expr),

    // Function definition
    FnDef {
        name: String,
        params: Vec<Param>,
        return_type: Option<String>,
        body: Vec<Stmt>,
    },

    // Return
    Return(Option<Expr>),

    // Control flow
    If {
        condition: Expr,
        body: Vec<Stmt>,
        elif_branches: Vec<(Expr, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,

    // Match
    Match {
        value: Expr,
        cases: Vec<(Expr, Vec<Stmt>)>,
    },

    // Class
    ClassDef {
        name: String,
        parent: Option<String>,
        body: Vec<Stmt>,
    },

    // Import
    Import {
        module: String,
    },
    FromImport {
        module: String,
        names: Vec<String>,
    },

    // Error handling
    TryCatch {
        try_body: Vec<Stmt>,
        catches: Vec<CatchClause>,
        finally_body: Option<Vec<Stmt>>,
    },
    Raise(Expr),
    ErrorDef {
        name: String,
        message: Expr,
    },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CatchClause {
    pub error_type: Option<String>,
    pub var_name: Option<String>,
    pub body: Vec<Stmt>,
}

// --- Parser ---

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // --- Entry point ---
    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();

        // Skip leading newlines
        self.skip_newlines();

        while !self.is_at_end() {
            stmts.push(self.parse_statement()?);
            self.skip_newlines();
        }

        Ok(stmts)
    }

    // --- Helpers ---

    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    #[allow(dead_code)]
    fn peek(&self) -> &Token {
        if self.pos + 1 < self.tokens.len() {
            &self.tokens[self.pos + 1]
        } else {
            &Token::Eof
        }
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<Token, String> {
        if self.current() == expected {
            Ok(self.advance())
        } else {
            Err(format!(
                "Expected {:?}, found {:?} at token position {}",
                expected,
                self.current(),
                self.pos
            ))
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.current() == &Token::Eof
    }

    fn skip_newlines(&mut self) {
        while !self.is_at_end() && self.current() == &Token::Newline {
            self.advance();
        }
    }

    // --- Statement parsing ---

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        match self.current() {
            Token::Let => self.parse_let(),
            Token::Const => self.parse_const(),
            Token::Fn => self.parse_fn_def(),
            Token::Return => self.parse_return(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::Break => {
                self.advance();
                Ok(Stmt::Break)
            }
            Token::Continue => {
                self.advance();
                Ok(Stmt::Continue)
            }
            Token::Match => self.parse_match(),
            Token::Class => self.parse_class(),
            Token::Import => self.parse_import(),
            Token::From => self.parse_from_import(),
            Token::Try => self.parse_try(),
            Token::Raise => self.parse_raise(),
            Token::ErrorKw => self.parse_error_def(),
            _ => self.parse_assignment_or_expr(),
        }
    }

    // --- let name [: type] = value ---
    fn parse_let(&mut self) -> Result<Stmt, String> {
        self.advance(); // skip 'let'
        let name = self.expect_identifier()?;
        let type_hint = if self.current() == &Token::Colon {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };
        self.expect(&Token::Assign)?;
        let value = self.parse_expression()?;
        Ok(Stmt::Let {
            name,
            type_hint,
            value,
        })
    }

    // --- const name [: type] = value ---
    fn parse_const(&mut self) -> Result<Stmt, String> {
        self.advance(); // skip 'const'
        let name = self.expect_identifier()?;
        let type_hint = if self.current() == &Token::Colon {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };
        self.expect(&Token::Assign)?;
        let value = self.parse_expression()?;
        Ok(Stmt::Const {
            name,
            type_hint,
            value,
        })
    }

    // --- fn name(params) [-> type]: body ---
    fn parse_fn_def(&mut self) -> Result<Stmt, String> {
        self.advance(); // skip 'fn'
        let name = self.expect_identifier()?;
        self.expect(&Token::LeftParen)?;
        let params = self.parse_params()?;
        self.expect(&Token::RightParen)?;

        let return_type = if self.current() == &Token::Arrow {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };

        self.expect(&Token::Colon)?;
        let body = self.parse_block()?;
        Ok(Stmt::FnDef {
            name,
            params,
            return_type,
            body,
        })
    }

    // --- Parse parameter list ---
    fn parse_params(&mut self) -> Result<Vec<Param>, String> {
        let mut params = Vec::new();
        if self.current() == &Token::RightParen {
            return Ok(params);
        }

        loop {
            let variadic = if self.current() == &Token::Star {
                self.advance();
                true
            } else {
                false
            };

            let name = if self.current() == &Token::Self_ {
                self.advance();
                "self".to_string()
            } else {
                self.expect_identifier()?
            };

            let type_hint = if self.current() == &Token::Colon {
                self.advance();
                Some(self.expect_identifier()?)
            } else {
                None
            };

            let default = if self.current() == &Token::Assign {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };

            params.push(Param {
                name,
                type_hint,
                default,
                variadic,
            });

            if self.current() != &Token::Comma {
                break;
            }
            self.advance(); // skip comma
        }

        Ok(params)
    }

    // --- return [expr] ---
    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance(); // skip 'return'
        if self.current() == &Token::Newline
            || self.current() == &Token::Eof
            || self.current() == &Token::Dedent
        {
            Ok(Stmt::Return(None))
        } else {
            let expr = self.parse_expression()?;
            Ok(Stmt::Return(Some(expr)))
        }
    }

    // --- if/elif/else ---
    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.advance(); // skip 'if'
        let condition = self.parse_expression()?;
        self.expect(&Token::Colon)?;
        let body = self.parse_block()?;

        let mut elif_branches = Vec::new();
        let mut else_body = None;

        self.skip_newlines();
        while self.current() == &Token::Elif {
            self.advance();
            let elif_cond = self.parse_expression()?;
            self.expect(&Token::Colon)?;
            let elif_body = self.parse_block()?;
            elif_branches.push((elif_cond, elif_body));
            self.skip_newlines();
        }

        if self.current() == &Token::Else {
            self.advance();
            self.expect(&Token::Colon)?;
            else_body = Some(self.parse_block()?);
        }

        Ok(Stmt::If {
            condition,
            body,
            elif_branches,
            else_body,
        })
    }

    // --- while condition: ---
    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.advance(); // skip 'while'
        let condition = self.parse_expression()?;
        self.expect(&Token::Colon)?;
        let body = self.parse_block()?;
        Ok(Stmt::While { condition, body })
    }

    // --- for var in iterable: ---
    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.advance(); // skip 'for'
        let var = self.expect_identifier()?;
        self.expect(&Token::In)?;
        let iterable = self.parse_expression()?;
        self.expect(&Token::Colon)?;
        let body = self.parse_block()?;
        Ok(Stmt::For {
            var,
            iterable,
            body,
        })
    }

    // --- match value: case ...: ---
    fn parse_match(&mut self) -> Result<Stmt, String> {
        self.advance(); // skip 'match'
        let value = self.parse_expression()?;
        self.expect(&Token::Colon)?;
        self.skip_newlines();
        self.expect(&Token::Indent)?;

        let mut cases = Vec::new();
        self.skip_newlines();

        while self.current() == &Token::Case {
            self.advance();
            let pattern = self.parse_expression()?;
            self.expect(&Token::Colon)?;
            let body = self.parse_block()?;
            cases.push((pattern, body));
            self.skip_newlines();
        }

        self.expect(&Token::Dedent)?;
        Ok(Stmt::Match { value, cases })
    }

    // --- class Name[(Parent)]: ---
    fn parse_class(&mut self) -> Result<Stmt, String> {
        self.advance(); // skip 'class'
        let name = self.expect_identifier()?;

        let parent = if self.current() == &Token::LeftParen {
            self.advance();
            let p = self.expect_identifier()?;
            self.expect(&Token::RightParen)?;
            Some(p)
        } else {
            None
        };

        self.expect(&Token::Colon)?;
        let body = self.parse_block()?;

        Ok(Stmt::ClassDef { name, parent, body })
    }

    // --- import module ---
    fn parse_import(&mut self) -> Result<Stmt, String> {
        self.advance(); // skip 'import'
        let module = self.expect_identifier()?;
        Ok(Stmt::Import { module })
    }

    // --- from module import name1, name2 ---
    fn parse_from_import(&mut self) -> Result<Stmt, String> {
        self.advance(); // skip 'from'
        let module = self.expect_identifier()?;
        self.expect(&Token::Import)?;
        let mut names = Vec::new();
        names.push(self.expect_identifier()?);
        while self.current() == &Token::Comma {
            self.advance();
            names.push(self.expect_identifier()?);
        }
        Ok(Stmt::FromImport { module, names })
    }

    // --- try/catch/finally ---
    fn parse_try(&mut self) -> Result<Stmt, String> {
        self.advance(); // skip 'try'
        self.expect(&Token::Colon)?;
        let try_body = self.parse_block()?;

        let mut catches = Vec::new();
        let mut finally_body = None;

        self.skip_newlines();
        while self.current() == &Token::Catch {
            self.advance();
            let error_type = if self.current() != &Token::Colon && self.current() != &Token::As {
                Some(self.expect_identifier()?)
            } else {
                None
            };
            let var_name = if self.current() == &Token::As {
                self.advance();
                Some(self.expect_identifier()?)
            } else {
                None
            };
            self.expect(&Token::Colon)?;
            let body = self.parse_block()?;
            catches.push(CatchClause {
                error_type,
                var_name,
                body,
            });
            self.skip_newlines();
        }

        if self.current() == &Token::Finally {
            self.advance();
            self.expect(&Token::Colon)?;
            finally_body = Some(self.parse_block()?);
        }

        Ok(Stmt::TryCatch {
            try_body,
            catches,
            finally_body,
        })
    }

    // --- raise expr ---
    fn parse_raise(&mut self) -> Result<Stmt, String> {
        self.advance(); // skip 'raise'
        let expr = self.parse_expression()?;
        Ok(Stmt::Raise(expr))
    }

    // --- error Name: "message" ---
    fn parse_error_def(&mut self) -> Result<Stmt, String> {
        self.advance(); // skip 'error'
        let name = self.expect_identifier()?;
        self.expect(&Token::Colon)?;
        let message = self.parse_expression()?;
        Ok(Stmt::ErrorDef { name, message })
    }

    // --- Assignment or expression statement ---
    fn parse_assignment_or_expr(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expression()?;

        match self.current() {
            Token::Assign => {
                self.advance();
                let value = self.parse_expression()?;
                Ok(Stmt::Assign {
                    target: expr,
                    value,
                })
            }
            Token::PlusAssign => {
                self.advance();
                let value = self.parse_expression()?;
                Ok(Stmt::AugAssign {
                    target: expr,
                    op: BinOp::Add,
                    value,
                })
            }
            Token::MinusAssign => {
                self.advance();
                let value = self.parse_expression()?;
                Ok(Stmt::AugAssign {
                    target: expr,
                    op: BinOp::Sub,
                    value,
                })
            }
            Token::StarAssign => {
                self.advance();
                let value = self.parse_expression()?;
                Ok(Stmt::AugAssign {
                    target: expr,
                    op: BinOp::Mul,
                    value,
                })
            }
            Token::SlashAssign => {
                self.advance();
                let value = self.parse_expression()?;
                Ok(Stmt::AugAssign {
                    target: expr,
                    op: BinOp::Div,
                    value,
                })
            }
            _ => Ok(Stmt::ExprStmt(expr)),
        }
    }

    // --- Expression parsing (precedence climbing) ---

    fn parse_expression(&mut self) -> Result<Expr, String> {
        // Lambda
        if self.current() == &Token::Lam {
            return self.parse_lambda();
        }
        self.parse_or()
    }

    fn parse_lambda(&mut self) -> Result<Expr, String> {
        self.advance(); // skip 'lam'
        let mut params = Vec::new();

        // Parse params until =>
        while self.current() != &Token::FatArrow {
            if !params.is_empty() {
                self.expect(&Token::Comma)?;
            }
            let name = self.expect_identifier()?;
            params.push(Param {
                name,
                type_hint: None,
                default: None,
                variadic: false,
            });
        }
        self.expect(&Token::FatArrow)?;
        let body = self.parse_expression()?;
        Ok(Expr::Lambda {
            params,
            body: Box::new(body),
        })
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.current() == &Token::Or {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Logical {
                left: Box::new(left),
                op: LogicOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_not()?;
        while self.current() == &Token::And {
            self.advance();
            let right = self.parse_not()?;
            left = Expr::Logical {
                left: Box::new(left),
                op: LogicOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Expr, String> {
        if self.current() == &Token::Not {
            self.advance();
            let operand = self.parse_not()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOp::Not,
                operand: Box::new(operand),
            });
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let left = self.parse_addition()?;

        let op = match self.current() {
            Token::Eq => CmpOp::Eq,
            Token::NotEq => CmpOp::NotEq,
            Token::Lt => CmpOp::Lt,
            Token::Gt => CmpOp::Gt,
            Token::LtEq => CmpOp::LtEq,
            Token::GtEq => CmpOp::GtEq,
            _ => return Ok(left),
        };

        self.advance();
        let right = self.parse_addition()?;
        Ok(Expr::Comparison {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }

    fn parse_addition(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplication()?;
        loop {
            let op = match self.current() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_power()?;
        loop {
            let op = match self.current() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_power()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expr, String> {
        let base = self.parse_unary()?;
        if self.current() == &Token::DoubleStar {
            self.advance();
            let exp = self.parse_power()?; // right-associative
            Ok(Expr::BinaryOp {
                left: Box::new(base),
                op: BinOp::Pow,
                right: Box::new(exp),
            })
        } else {
            Ok(base)
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.current() == &Token::Minus {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOp::Neg,
                operand: Box::new(operand),
            });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.current() {
                Token::LeftParen => {
                    self.advance();
                    let args = self.parse_arg_list()?;
                    self.expect(&Token::RightParen)?;
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }
                Token::LeftBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(&Token::RightBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Token::Dot => {
                    self.advance();
                    let name = self.expect_identifier()?;
                    expr = Expr::Attribute {
                        object: Box::new(expr),
                        name,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current().clone() {
            Token::Int(n) => {
                self.advance();
                Ok(Expr::Int(n))
            }
            Token::Float(n) => {
                self.advance();
                Ok(Expr::Float(n))
            }
            Token::Str(s) => {
                self.advance();
                Ok(Expr::Str(s))
            }
            Token::InterpolatedStr(parts) => {
                self.advance();
                let mut expr_parts = Vec::new();
                for part in parts {
                    match part {
                        StringPart::Literal(s) => {
                            expr_parts.push(StringPartExpr::Literal(s));
                        }
                        StringPart::Expr(tokens) => {
                            let mut inner_parser = Parser::new(
                                tokens
                                    .into_iter()
                                    .chain(std::iter::once(Token::Eof))
                                    .collect(),
                            );
                            let expr = inner_parser.parse_expression()?;
                            expr_parts.push(StringPartExpr::Expr(expr));
                        }
                    }
                }
                Ok(Expr::InterpolatedStr(expr_parts))
            }
            Token::True => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            Token::Null => {
                self.advance();
                Ok(Expr::Null)
            }
            Token::Self_ => {
                self.advance();
                Ok(Expr::SelfRef)
            }
            Token::Identifier(name) => {
                self.advance();
                Ok(Expr::Identifier(name))
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&Token::RightParen)?;
                Ok(expr)
            }
            Token::LeftBracket => self.parse_list_literal(),
            Token::LeftBrace => self.parse_map_or_set_literal(),
            _ => Err(format!(
                "Unexpected token {:?} at position {}",
                self.current(),
                self.pos
            )),
        }
    }

    // --- List literal [a, b, c] ---
    fn parse_list_literal(&mut self) -> Result<Expr, String> {
        self.advance(); // skip [
        let mut items = Vec::new();
        while self.current() != &Token::RightBracket {
            items.push(self.parse_expression()?);
            if self.current() != &Token::Comma {
                break;
            }
            self.advance();
        }
        self.expect(&Token::RightBracket)?;
        Ok(Expr::ListLiteral(items))
    }

    // --- Map {k: v, ...} or Set {a, b, ...} ---
    fn parse_map_or_set_literal(&mut self) -> Result<Expr, String> {
        self.advance(); // skip {
        if self.current() == &Token::RightBrace {
            self.advance();
            return Ok(Expr::MapLiteral(Vec::new()));
        }

        let first = self.parse_expression()?;

        // Check if this is a map (key: value) or set
        if self.current() == &Token::Colon {
            // Map
            self.advance();
            let first_val = self.parse_expression()?;
            let mut pairs = vec![(first, first_val)];
            while self.current() == &Token::Comma {
                self.advance();
                if self.current() == &Token::RightBrace {
                    break;
                }
                let k = self.parse_expression()?;
                self.expect(&Token::Colon)?;
                let v = self.parse_expression()?;
                pairs.push((k, v));
            }
            self.expect(&Token::RightBrace)?;
            Ok(Expr::MapLiteral(pairs))
        } else {
            // Set
            let mut items = vec![first];
            while self.current() == &Token::Comma {
                self.advance();
                if self.current() == &Token::RightBrace {
                    break;
                }
                items.push(self.parse_expression()?);
            }
            self.expect(&Token::RightBrace)?;
            Ok(Expr::SetLiteral(items))
        }
    }

    // --- Argument list for calls ---
    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if self.current() == &Token::RightParen {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expression()?);
            if self.current() != &Token::Comma {
                break;
            }
            self.advance();
        }
        Ok(args)
    }

    // --- Parse indented block ---
    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        self.skip_newlines();
        self.expect(&Token::Indent)?;
        let mut stmts = Vec::new();
        self.skip_newlines();

        while !self.is_at_end() && self.current() != &Token::Dedent {
            stmts.push(self.parse_statement()?);
            self.skip_newlines();
        }

        if self.current() == &Token::Dedent {
            self.advance();
        }

        Ok(stmts)
    }

    // --- Expect an identifier token ---
    fn expect_identifier(&mut self) -> Result<String, String> {
        match self.current().clone() {
            Token::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            _ => Err(format!(
                "Expected identifier, found {:?} at position {}",
                self.current(),
                self.pos
            )),
        }
    }
}
