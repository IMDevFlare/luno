use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Literal(Value),
    Variable(String),
    Binary {
        left: Box<Expr>,
        operator: Operator,
        right: Box<Expr>,
    },
    Unary {
        operator: UnaryOperator,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: String,
    },
    Grouping(Box<Expr>),
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Expression(Expr),
    Let {
        name: String,
        initializer: Expr,
        type_annotation: Option<String>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    If {
        condition: Expr,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
    },
    Repeat {
        start: Expr,
        end: Expr,
        var_name: String,
        body: Vec<Statement>,
    },
    Return(Option<Expr>),
    Class {
        name: String,
        methods: Vec<Statement>,
    },
    Say(Expr),
    Log {
        level: String,
        message: Expr,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    And,
    Or,
    Concat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UnaryOperator {
    Not,
    Negate,
}
