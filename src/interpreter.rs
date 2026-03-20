use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum RuntimeValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
        closure: Environment,
    },
    Class {
        name: String,
        methods: HashMap<String, Statement>,
    },
    Instance {
        class_name: String,
        fields: HashMap<String, RuntimeValue>,
    },
    List(Vec<RuntimeValue>),
}

impl From<Value> for RuntimeValue {
    fn from(v: Value) -> Self {
        match v {
            Value::Int(i) => RuntimeValue::Int(i),
            Value::Float(f) => RuntimeValue::Float(f),
            Value::String(s) => RuntimeValue::String(s),
            Value::Bool(b) => RuntimeValue::Bool(b),
            Value::Nil => RuntimeValue::Nil,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, RuntimeValue>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn define(&mut self, name: String, value: RuntimeValue) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<RuntimeValue> {
        if let Some(value) = self.values.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }
}

pub struct Interpreter {
    pub environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Statement>) -> Result<(), String> {
        for stmt in statements {
            self.execute(stmt)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: Statement) -> Result<Option<RuntimeValue>, String> {
        match stmt {
            Statement::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(None)
            }
            Statement::Let {
                name, initializer, ..
            } => {
                let value = self.evaluate(initializer)?;
                self.environment.define(name, value);
                Ok(None)
            }
            Statement::Say(expr) => {
                let value = self.evaluate(expr)?;
                println!("{}", Self::stringify(value));
                Ok(None)
            }
            Statement::Function { name, params, body } => {
                let func = RuntimeValue::Function {
                    name: name.clone(),
                    params,
                    body,
                    closure: self.environment.clone(),
                };
                self.environment.define(name, func);
                Ok(None)
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond = self.evaluate(condition)?;
                if Self::is_truthy(cond) {
                    for stmt in then_branch {
                        self.execute(stmt)?;
                    }
                } else if let Some(branch) = else_branch {
                    for stmt in branch {
                        self.execute(stmt)?;
                    }
                }
                Ok(None)
            }
            Statement::While { condition, body } => {
                loop {
                    let cond = self.evaluate(condition.clone())?;
                    if !Self::is_truthy(cond) {
                        break;
                    }
                    for stmt in body.clone() {
                        self.execute(stmt)?;
                    }
                }
                Ok(None)
            }
            Statement::Repeat {
                start,
                end,
                var_name,
                body,
            } => {
                let s = self.evaluate(start)?;
                let e = self.evaluate(end)?;
                if let (RuntimeValue::Int(start_val), RuntimeValue::Int(end_val)) = (s, e) {
                    for i in start_val..=end_val {
                        self.environment
                            .define(var_name.clone(), RuntimeValue::Int(i));
                        for stmt in body.clone() {
                            self.execute(stmt)?;
                        }
                    }
                }
                Ok(None)
            }
            Statement::Return(expr) => {
                let value = if let Some(e) = expr {
                    self.evaluate(e)?
                } else {
                    RuntimeValue::Nil
                };
                Ok(Some(value))
            }
            Statement::Class { name, methods } => {
                let mut method_map = HashMap::new();
                for method in methods {
                    if let Statement::Function { name: m_name, .. } = &method {
                        method_map.insert(m_name.clone(), method.clone());
                    }
                }
                let class = RuntimeValue::Class {
                    name: name.clone(),
                    methods: method_map,
                };
                self.environment.define(name, class);
                Ok(None)
            }
            Statement::Log { level, message } => {
                let val = self.evaluate(message)?;
                println!("[{}] {}", level.to_uppercase(), Self::stringify(val));
                Ok(None)
            }
        }
    }

    fn evaluate(&mut self, expr: Expr) -> Result<RuntimeValue, String> {
        match expr {
            Expr::Literal(v) => Ok(v.into()),
            Expr::Variable(name) => self
                .environment
                .get(&name)
                .ok_or(format!("Undefined variable '{}'", name)),
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;
                match operator {
                    Operator::Plus => self.add(left, right),
                    Operator::Minus => self.subtract(left, right),
                    Operator::Multiply => self.multiply(left, right),
                    Operator::Divide => self.divide(left, right),
                    Operator::Concat => self.concat(left, right),
                    Operator::GreaterThan => Ok(RuntimeValue::Bool(self.compare(left, right)? > 0)),
                    Operator::LessThan => Ok(RuntimeValue::Bool(self.compare(left, right)? < 0)),
                    Operator::GreaterEqual => {
                        Ok(RuntimeValue::Bool(self.compare(left, right)? >= 0))
                    }
                    Operator::LessEqual => Ok(RuntimeValue::Bool(self.compare(left, right)? <= 0)),
                    Operator::Equal => Ok(RuntimeValue::Bool(self.is_equal(left, right))),
                    Operator::NotEqual => Ok(RuntimeValue::Bool(!self.is_equal(left, right))),
                    _ => Err(format!("Operator {:?} not supported", operator)),
                }
            }
            Expr::Call { callee, arguments } => {
                let func = self.evaluate(*callee)?;
                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.evaluate(arg)?);
                }
                self.call(func, args)
            }
            Expr::Grouping(expr) => self.evaluate(*expr),
            Expr::List(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.evaluate(elem)?);
                }
                Ok(RuntimeValue::List(values))
            }
            Expr::Get { object, name } => {
                let obj = self.evaluate(*object)?;
                match obj {
                    RuntimeValue::Instance { fields, .. } => fields
                        .get(&name)
                        .cloned()
                        .ok_or(format!("Undefined property '{}'", name)),
                    _ => Err("Can only access properties on instances".to_string()),
                }
            }
            _ => Err("Expression not yet implemented".to_string()),
        }
    }

    fn call(
        &mut self,
        func: RuntimeValue,
        args: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, String> {
        match func {
            RuntimeValue::Function {
                params,
                body,
                closure,
                ..
            } => {
                if args.len() != params.len() {
                    return Err(format!(
                        "Expected {} arguments but got {}",
                        params.len(),
                        args.len()
                    ));
                }
                let mut env = closure;
                env.parent = Some(Box::new(self.environment.clone()));
                let previous = self.environment.clone();
                self.environment = env;

                for (param, arg) in params.iter().zip(args) {
                    self.environment.define(param.clone(), arg);
                }

                let mut result = RuntimeValue::Nil;
                for stmt in body {
                    if let Some(ret) = self.execute(stmt)? {
                        result = ret;
                        break;
                    }
                }

                self.environment = previous;
                Ok(result)
            }
            _ => Err("Can only call functions".to_string()),
        }
    }

    fn add(&self, left: RuntimeValue, right: RuntimeValue) -> Result<RuntimeValue, String> {
        match (left, right) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a + b)),
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => Ok(RuntimeValue::Float(a + b)),
            (RuntimeValue::Int(a), RuntimeValue::Float(b)) => Ok(RuntimeValue::Float(a as f64 + b)),
            (RuntimeValue::Float(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Float(a + b as f64)),
            _ => Err("Operands must be numbers for addition".to_string()),
        }
    }

    fn subtract(&self, left: RuntimeValue, right: RuntimeValue) -> Result<RuntimeValue, String> {
        match (left, right) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a - b)),
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => Ok(RuntimeValue::Float(a - b)),
            _ => Err("Operands must be numbers for subtraction".to_string()),
        }
    }

    fn multiply(&self, left: RuntimeValue, right: RuntimeValue) -> Result<RuntimeValue, String> {
        match (left, right) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(RuntimeValue::Int(a * b)),
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => Ok(RuntimeValue::Float(a * b)),
            _ => Err("Operands must be numbers for multiplication".to_string()),
        }
    }

    fn divide(&self, left: RuntimeValue, right: RuntimeValue) -> Result<RuntimeValue, String> {
        match (left, right) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => {
                if b == 0 {
                    return Err("Division by zero".to_string());
                }
                Ok(RuntimeValue::Int(a / b))
            }
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => {
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(RuntimeValue::Float(a / b))
            }
            _ => Err("Operands must be numbers for division".to_string()),
        }
    }

    fn concat(&self, left: RuntimeValue, right: RuntimeValue) -> Result<RuntimeValue, String> {
        Ok(RuntimeValue::String(format!(
            "{}{}",
            Self::stringify(left),
            Self::stringify(right)
        )))
    }

    fn compare(&self, left: RuntimeValue, right: RuntimeValue) -> Result<i8, String> {
        match (left, right) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => Ok(if a > b {
                1
            } else if a < b {
                -1
            } else {
                0
            }),
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => Ok(if a > b {
                1
            } else if a < b {
                -1
            } else {
                0
            }),
            _ => Err("Operands must be the same type for comparison".to_string()),
        }
    }

    fn is_equal(&self, left: RuntimeValue, right: RuntimeValue) -> bool {
        match (left, right) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => a == b,
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => a == b,
            (RuntimeValue::String(a), RuntimeValue::String(b)) => a == b,
            (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => a == b,
            (RuntimeValue::Nil, RuntimeValue::Nil) => true,
            _ => false,
        }
    }

    fn is_truthy(v: RuntimeValue) -> bool {
        match v {
            RuntimeValue::Bool(b) => b,
            RuntimeValue::Nil => false,
            _ => true,
        }
    }

    fn stringify(v: RuntimeValue) -> String {
        match v {
            RuntimeValue::Int(i) => i.to_string(),
            RuntimeValue::Float(f) => f.to_string(),
            RuntimeValue::String(s) => s,
            RuntimeValue::Bool(b) => b.to_string(),
            RuntimeValue::Nil => "nil".to_string(),
            RuntimeValue::Function { name, .. } => format!("<function {}>", name),
            _ => "object".to_string(),
        }
    }
}
