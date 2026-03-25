// --- interpreter.rs — Luno Tree-Walking Interpreter ---

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

type NativeFn = Rc<dyn Fn(Vec<Value>) -> Result<Value, LunoError>>;

use crate::parser::*;

// --- Value type ---

pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
    List(Rc<RefCell<Vec<Value>>>),
    Map(Rc<RefCell<HashMap<String, Value>>>),
    Function {
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
        env: Env,
    },
    NativeFunction {
        name: String,
        func: NativeFn,
    },
    Class {
        name: String,
        parent: Option<Box<Value>>,
        methods: HashMap<String, Value>,
    },
    Instance {
        class_name: String,
        fields: Rc<RefCell<HashMap<String, Value>>>,
        methods: HashMap<String, Value>,
    },
    Error {
        name: String,
        message: String,
    },
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Int(n) => Value::Int(*n),
            Value::Float(n) => Value::Float(*n),
            Value::Str(s) => Value::Str(s.clone()),
            Value::Bool(b) => Value::Bool(*b),
            Value::Null => Value::Null,
            Value::List(l) => Value::List(l.clone()),
            Value::Map(m) => Value::Map(m.clone()),
            Value::Function {
                name,
                params,
                body,
                env,
            } => Value::Function {
                name: name.clone(),
                params: params.clone(),
                body: body.clone(),
                env: env.clone(),
            },
            Value::NativeFunction { name, func } => Value::NativeFunction {
                name: name.clone(),
                func: func.clone(),
            },
            Value::Class {
                name,
                parent,
                methods,
            } => Value::Class {
                name: name.clone(),
                parent: parent.clone(),
                methods: methods.clone(),
            },
            Value::Instance {
                class_name,
                fields,
                methods,
            } => Value::Instance {
                class_name: class_name.clone(),
                fields: fields.clone(),
                methods: methods.clone(),
            },
            Value::Error { name, message } => Value::Error {
                name: name.clone(),
                message: message.clone(),
            },
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::List(items) => {
                let items = items.borrow();
                let s: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", s.join(", "))
            }
            Value::Map(map) => {
                let map = map.borrow();
                let s: Vec<String> = map.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", s.join(", "))
            }
            Value::Function { name, .. } => write!(f, "<fn {}>", name),
            Value::NativeFunction { name, .. } => write!(f, "<builtin {}>", name),
            Value::Class { name, .. } => write!(f, "<class {}>", name),
            Value::Instance { class_name, .. } => write!(f, "<{} instance>", class_name),
            Value::Error { name, message } => write!(f, "{}: {}", name, message),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Int(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.borrow().is_empty(),
            _ => true,
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Str(_) => "str",
            Value::Bool(_) => "bool",
            Value::Null => "null",
            Value::List(_) => "list",
            Value::Map(_) => "map",
            Value::Function { .. } | Value::NativeFunction { .. } => "function",
            Value::Class { .. } => "class",
            Value::Instance { .. } => "instance",
            Value::Error { .. } => "error",
        }
    }
}

// --- Error type ---

#[derive(Debug, Clone)]
pub enum LunoError {
    Runtime(String),
    Return(Value),
    Break,
    Continue,
    Raised(String, String), // (error_name, message)
}

impl fmt::Display for LunoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LunoError::Runtime(msg) => write!(f, "RuntimeError: {}", msg),
            LunoError::Return(_) => write!(f, "return outside function"),
            LunoError::Break => write!(f, "break outside loop"),
            LunoError::Continue => write!(f, "continue outside loop"),
            LunoError::Raised(name, msg) => write!(f, "{}: {}", name, msg),
        }
    }
}

// --- Environment ---

#[derive(Debug, Clone)]
pub struct EnvInner {
    vars: HashMap<String, Value>,
    consts: HashMap<String, bool>,
    parent: Option<Env>,
}

pub type Env = Rc<RefCell<EnvInner>>;

pub fn new_env(parent: Option<Env>) -> Env {
    Rc::new(RefCell::new(EnvInner {
        vars: HashMap::new(),
        consts: HashMap::new(),
        parent,
    }))
}

pub fn env_get(env: &Env, name: &str) -> Option<Value> {
    let inner = env.borrow();
    if let Some(val) = inner.vars.get(name) {
        Some(val.clone())
    } else if let Some(parent) = &inner.parent {
        env_get(parent, name)
    } else {
        None
    }
}

pub fn env_set(env: &Env, name: &str, value: Value) -> Result<(), LunoError> {
    // Check if it's a const in current scope
    {
        let inner = env.borrow();
        if inner.consts.get(name) == Some(&true) {
            return Err(LunoError::Runtime(format!(
                "Cannot reassign const '{}'",
                name
            )));
        }
    }
    // Try to find in current or parent scopes
    {
        let inner = env.borrow();
        if inner.vars.contains_key(name) {
            drop(inner);
            env.borrow_mut().vars.insert(name.to_string(), value);
            return Ok(());
        }
        if let Some(parent) = inner.parent.clone() {
            drop(inner);
            return env_set(&parent, name, value);
        }
    }
    // Define in current scope
    env.borrow_mut().vars.insert(name.to_string(), value);
    Ok(())
}

pub fn env_define(env: &Env, name: &str, value: Value, is_const: bool) {
    let mut inner = env.borrow_mut();
    inner.vars.insert(name.to_string(), value);
    if is_const {
        inner.consts.insert(name.to_string(), true);
    }
}

// --- Interpreter ---

pub struct Interpreter {
    pub global_env: Env,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = new_env(None);
        register_builtins(&env);
        Interpreter { global_env: env }
    }

    pub fn run(&mut self, stmts: &[Stmt]) -> Result<(), LunoError> {
        for stmt in stmts {
            self.exec_stmt(stmt, &self.global_env.clone())?;
        }
        Ok(())
    }

    // --- Statement execution ---

    fn exec_stmt(&mut self, stmt: &Stmt, env: &Env) -> Result<(), LunoError> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let val = self.eval_expr(value, env)?;
                env_define(env, name, val, false);
            }
            Stmt::Const { name, value, .. } => {
                let val = self.eval_expr(value, env)?;
                env_define(env, name, val, true);
            }
            Stmt::Assign { target, value } => {
                let val = self.eval_expr(value, env)?;
                self.assign_target(target, val, env)?;
            }
            Stmt::AugAssign { target, op, value } => {
                let current = self.eval_expr(target, env)?;
                let rhs = self.eval_expr(value, env)?;
                let result = self.apply_binop(op, &current, &rhs)?;
                self.assign_target(target, result, env)?;
            }
            Stmt::ExprStmt(expr) => {
                self.eval_expr(expr, env)?;
            }
            Stmt::FnDef {
                name, params, body, ..
            } => {
                let func = Value::Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    env: env.clone(),
                };
                env_define(env, name, func, false);
            }
            Stmt::Return(expr) => {
                let val = match expr {
                    Some(e) => self.eval_expr(e, env)?,
                    None => Value::Null,
                };
                return Err(LunoError::Return(val));
            }
            Stmt::If {
                condition,
                body,
                elif_branches,
                else_body,
            } => {
                let cond = self.eval_expr(condition, env)?;
                if cond.is_truthy() {
                    self.exec_block(body, env)?;
                } else {
                    let mut handled = false;
                    for (elif_cond, elif_body) in elif_branches {
                        let c = self.eval_expr(elif_cond, env)?;
                        if c.is_truthy() {
                            self.exec_block(elif_body, env)?;
                            handled = true;
                            break;
                        }
                    }
                    if !handled {
                        if let Some(eb) = else_body {
                            self.exec_block(eb, env)?;
                        }
                    }
                }
            }
            Stmt::While { condition, body } => loop {
                let cond = self.eval_expr(condition, env)?;
                if !cond.is_truthy() {
                    break;
                }
                match self.exec_block(body, env) {
                    Err(LunoError::Break) => break,
                    Err(LunoError::Continue) => continue,
                    Err(e) => return Err(e),
                    Ok(_) => {}
                }
            },
            Stmt::For {
                var,
                iterable,
                body,
            } => {
                let iter_val = self.eval_expr(iterable, env)?;
                let items = self.value_to_iter(&iter_val)?;
                for item in items {
                    env_define(env, var, item, false);
                    match self.exec_block(body, env) {
                        Err(LunoError::Break) => break,
                        Err(LunoError::Continue) => continue,
                        Err(e) => return Err(e),
                        Ok(_) => {}
                    }
                }
            }
            Stmt::Break => return Err(LunoError::Break),
            Stmt::Continue => return Err(LunoError::Continue),
            Stmt::Match { value, cases } => {
                let val = self.eval_expr(value, env)?;
                for (pattern, body) in cases {
                    let pat_val = self.eval_expr(pattern, env)?;
                    if self.values_equal(&val, &pat_val) {
                        self.exec_block(body, env)?;
                        break;
                    }
                }
            }
            Stmt::ClassDef { name, parent, body } => {
                let parent_val = if let Some(p) = parent {
                    match env_get(env, p) {
                        Some(v @ Value::Class { .. }) => Some(Box::new(v)),
                        _ => {
                            return Err(LunoError::Runtime(format!(
                                "Parent class '{}' not found",
                                p
                            )))
                        }
                    }
                } else {
                    None
                };
                let class_env = new_env(Some(env.clone()));
                let mut methods = HashMap::new();
                for stmt in body {
                    if let Stmt::FnDef {
                        name: mname,
                        params,
                        body: mbody,
                        ..
                    } = stmt
                    {
                        methods.insert(
                            mname.clone(),
                            Value::Function {
                                name: mname.clone(),
                                params: params.clone(),
                                body: mbody.clone(),
                                env: class_env.clone(),
                            },
                        );
                    }
                }
                // Inherit methods from parent
                if let Some(ref pv) = parent_val {
                    if let Value::Class { methods: pm, .. } = pv.as_ref() {
                        for (k, v) in pm {
                            methods.entry(k.clone()).or_insert_with(|| v.clone());
                        }
                    }
                }
                let class = Value::Class {
                    name: name.clone(),
                    parent: parent_val,
                    methods,
                };
                env_define(env, name, class, false);
            }
            Stmt::Import { module } => {
                if module == "math" {
                    register_math_module(env);
                } else {
                    return Err(LunoError::Runtime(format!("Module '{}' not found", module)));
                }
            }
            Stmt::FromImport { module, names } => {
                if module == "math" {
                    register_math_module(env);
                    let _ = names; // all math functions already registered
                } else {
                    return Err(LunoError::Runtime(format!("Module '{}' not found", module)));
                }
            }
            Stmt::TryCatch {
                try_body,
                catches,
                finally_body,
            } => {
                let result = self.exec_block(try_body, env);
                match result {
                    Err(LunoError::Raised(ename, emsg)) => {
                        let mut caught = false;
                        for c in catches {
                            let matches = c.error_type.as_ref().map_or(true, |t| t == &ename);
                            if matches {
                                if let Some(vn) = &c.var_name {
                                    let err_val = Value::Error {
                                        name: ename.clone(),
                                        message: emsg.clone(),
                                    };
                                    env_define(env, vn, err_val, false);
                                }
                                self.exec_block(&c.body, env)?;
                                caught = true;
                                break;
                            }
                        }
                        if !caught {
                            if let Some(fb) = finally_body {
                                self.exec_block(fb, env)?;
                            }
                            return Err(LunoError::Raised(ename, emsg));
                        }
                    }
                    Err(e) => {
                        if let Some(fb) = finally_body {
                            self.exec_block(fb, env)?;
                        }
                        return Err(e);
                    }
                    Ok(_) => {}
                }
                if let Some(fb) = finally_body {
                    self.exec_block(fb, env)?;
                }
            }
            Stmt::Raise(expr) => {
                let val = self.eval_expr(expr, env)?;
                match val {
                    Value::Error { name, message } => return Err(LunoError::Raised(name, message)),
                    Value::Str(s) => return Err(LunoError::Raised("Error".into(), s)),
                    _ => return Err(LunoError::Raised("Error".into(), format!("{}", val))),
                }
            }
            Stmt::ErrorDef { name, message } => {
                let msg = self.eval_expr(message, env)?;
                let err = Value::Error {
                    name: name.clone(),
                    message: format!("{}", msg),
                };
                env_define(env, name, err, false);
            }
        }
        Ok(())
    }

    fn exec_block(&mut self, stmts: &[Stmt], env: &Env) -> Result<(), LunoError> {
        let block_env = new_env(Some(env.clone()));
        for stmt in stmts {
            self.exec_stmt(stmt, &block_env)?;
        }
        Ok(())
    }

    fn assign_target(&mut self, target: &Expr, value: Value, env: &Env) -> Result<(), LunoError> {
        match target {
            Expr::Identifier(name) => {
                env_set(env, name, value)?;
            }
            Expr::Attribute { object, name } => {
                let obj = self.eval_expr(object, env)?;
                if let Value::Instance { fields, .. } = obj {
                    fields.borrow_mut().insert(name.clone(), value);
                } else {
                    return Err(LunoError::Runtime(
                        "Cannot set attribute on non-instance".into(),
                    ));
                }
            }
            Expr::Index { object, index } => {
                let obj = self.eval_expr(object, env)?;
                let idx = self.eval_expr(index, env)?;
                if let Value::List(list) = obj {
                    if let Value::Int(i) = idx {
                        let mut l = list.borrow_mut();
                        let len = l.len() as i64;
                        let actual = if i < 0 {
                            (len + i) as usize
                        } else {
                            i as usize
                        };
                        if actual < l.len() {
                            l[actual] = value;
                        } else {
                            return Err(LunoError::Runtime("Index out of bounds".into()));
                        }
                    }
                } else if let Value::Map(map) = obj {
                    if let Value::Str(k) = idx {
                        map.borrow_mut().insert(k, value);
                    }
                }
            }
            _ => return Err(LunoError::Runtime("Invalid assignment target".into())),
        }
        Ok(())
    }

    // --- Expression evaluation ---

    fn eval_expr(&mut self, expr: &Expr, env: &Env) -> Result<Value, LunoError> {
        match expr {
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Float(n) => Ok(Value::Float(*n)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Null => Ok(Value::Null),
            Expr::SelfRef => {
                env_get(env, "self").ok_or_else(|| LunoError::Runtime("'self' not in scope".into()))
            }
            Expr::Identifier(name) => env_get(env, name)
                .ok_or_else(|| LunoError::Runtime(format!("Undefined variable '{}'", name))),
            Expr::InterpolatedStr(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        StringPartExpr::Literal(s) => result.push_str(s),
                        StringPartExpr::Expr(e) => {
                            let val = self.eval_expr(e, env)?;
                            result.push_str(&format!("{}", val));
                        }
                    }
                }
                Ok(Value::Str(result))
            }
            Expr::BinaryOp { left, op, right } => {
                let l = self.eval_expr(left, env)?;
                let r = self.eval_expr(right, env)?;
                self.apply_binop(op, &l, &r)
            }
            Expr::UnaryOp { op, operand } => {
                let val = self.eval_expr(operand, env)?;
                match op {
                    UnaryOp::Neg => match val {
                        Value::Int(n) => Ok(Value::Int(-n)),
                        Value::Float(n) => Ok(Value::Float(-n)),
                        _ => Err(LunoError::Runtime("Cannot negate non-number".into())),
                    },
                    UnaryOp::Not => Ok(Value::Bool(!val.is_truthy())),
                }
            }
            Expr::Comparison { left, op, right } => {
                let l = self.eval_expr(left, env)?;
                let r = self.eval_expr(right, env)?;
                Ok(Value::Bool(self.apply_cmp(op, &l, &r)?))
            }
            Expr::Logical { left, op, right } => {
                let l = self.eval_expr(left, env)?;
                match op {
                    LogicOp::And => {
                        if !l.is_truthy() {
                            Ok(l)
                        } else {
                            self.eval_expr(right, env)
                        }
                    }
                    LogicOp::Or => {
                        if l.is_truthy() {
                            Ok(l)
                        } else {
                            self.eval_expr(right, env)
                        }
                    }
                }
            }
            Expr::Call { callee, args } => {
                let func = self.eval_expr(callee, env)?;
                let mut arg_vals = Vec::new();
                for a in args {
                    arg_vals.push(self.eval_expr(a, env)?);
                }
                self.call_value(&func, arg_vals, env)
            }
            Expr::Index { object, index } => {
                let obj = self.eval_expr(object, env)?;
                let idx = self.eval_expr(index, env)?;
                self.index_value(&obj, &idx)
            }
            Expr::Attribute { object, name } => {
                let obj = self.eval_expr(object, env)?;
                self.get_attribute(&obj, name, env)
            }
            Expr::ListLiteral(items) => {
                let mut vals = Vec::new();
                for item in items {
                    vals.push(self.eval_expr(item, env)?);
                }
                Ok(Value::List(Rc::new(RefCell::new(vals))))
            }
            Expr::MapLiteral(pairs) => {
                let mut map = HashMap::new();
                for (k, v) in pairs {
                    let key = self.eval_expr(k, env)?;
                    let val = self.eval_expr(v, env)?;
                    map.insert(format!("{}", key), val);
                }
                Ok(Value::Map(Rc::new(RefCell::new(map))))
            }
            Expr::SetLiteral(items) => {
                // Represent sets as lists with unique values for now
                let mut vals = Vec::new();
                for item in items {
                    vals.push(self.eval_expr(item, env)?);
                }
                Ok(Value::List(Rc::new(RefCell::new(vals))))
            }
            Expr::Lambda { params, body } => Ok(Value::Function {
                name: "<lambda>".into(),
                params: params.clone(),
                body: vec![Stmt::Return(Some(*body.clone()))],
                env: env.clone(),
            }),
            Expr::Assign { target, value } => {
                let val = self.eval_expr(value, env)?;
                self.assign_target(target, val.clone(), env)?;
                Ok(val)
            }
        }
    }

    // --- Calling functions/classes ---

    fn call_value(
        &mut self,
        callee: &Value,
        args: Vec<Value>,
        _call_env: &Env,
    ) -> Result<Value, LunoError> {
        match callee {
            Value::Function {
                params,
                body,
                env: closure_env,
                ..
            } => {
                let fn_env = new_env(Some(closure_env.clone()));
                for (i, param) in params.iter().enumerate() {
                    if param.name == "self" {
                        continue;
                    }
                    if param.variadic {
                        let rest: Vec<Value> = args[i..].to_vec();
                        env_define(
                            &fn_env,
                            &param.name,
                            Value::List(Rc::new(RefCell::new(rest))),
                            false,
                        );
                        break;
                    }
                    let val = if i < args.len() {
                        args[i].clone()
                    } else if let Some(def) = &param.default {
                        self.eval_expr(def, &fn_env)?
                    } else {
                        Value::Null
                    };
                    env_define(&fn_env, &param.name, val, false);
                }
                match self.exec_fn_body(body, &fn_env) {
                    Ok(_) => Ok(Value::Null),
                    Err(LunoError::Return(v)) => Ok(v),
                    Err(e) => Err(e),
                }
            }
            Value::NativeFunction { func, .. } => func(args),
            Value::Class { name, methods, .. } => {
                let fields = Rc::new(RefCell::new(HashMap::new()));
                let instance = Value::Instance {
                    class_name: name.clone(),
                    fields: fields.clone(),
                    methods: methods.clone(),
                };
                // Call init if it exists
                if let Some(init_fn) = methods.get("init") {
                    if let Value::Function {
                        params,
                        body,
                        env: closure_env,
                        ..
                    } = init_fn
                    {
                        let fn_env = new_env(Some(closure_env.clone()));
                        env_define(&fn_env, "self", instance.clone(), false);
                        for (i, param) in params.iter().enumerate() {
                            if param.name == "self" {
                                continue;
                            }
                            let arg_idx = i - 1; // skip self
                            let val = if arg_idx < args.len() {
                                args[arg_idx].clone()
                            } else {
                                Value::Null
                            };
                            env_define(&fn_env, &param.name, val, false);
                        }
                        match self.exec_fn_body(body, &fn_env) {
                            Ok(_) | Err(LunoError::Return(Value::Null)) => {}
                            Err(LunoError::Return(_)) => {}
                            Err(e) => return Err(e),
                        }
                    }
                }
                // Rebuild instance with updated fields
                Ok(Value::Instance {
                    class_name: name.clone(),
                    fields,
                    methods: methods.clone(),
                })
            }
            _ => Err(LunoError::Runtime(format!("'{}' is not callable", callee))),
        }
    }

    fn exec_fn_body(&mut self, body: &[Stmt], env: &Env) -> Result<(), LunoError> {
        for stmt in body {
            self.exec_stmt(stmt, env)?;
        }
        Ok(())
    }

    // --- Attribute access ---

    fn get_attribute(&mut self, obj: &Value, name: &str, _env: &Env) -> Result<Value, LunoError> {
        match obj {
            Value::Instance {
                fields, methods, ..
            } => {
                if let Some(v) = fields.borrow().get(name) {
                    return Ok(v.clone());
                }
                if let Some(method) = methods.get(name) {
                    // Bind self
                    if let Value::Function {
                        name: mname,
                        params,
                        body,
                        env: menv,
                    } = method
                    {
                        let bound_env = new_env(Some(menv.clone()));
                        env_define(&bound_env, "self", obj.clone(), false);
                        return Ok(Value::Function {
                            name: mname.clone(),
                            params: params.clone(),
                            body: body.clone(),
                            env: bound_env,
                        });
                    }
                    return Ok(method.clone());
                }
                Err(LunoError::Runtime(format!(
                    "No attribute '{}' on instance",
                    name
                )))
            }
            Value::Str(s) => {
                // String methods
                match name {
                    "length" => Ok(Value::Int(s.len() as i64)),
                    "upper" => {
                        let s = s.clone();
                        Ok(Value::NativeFunction {
                            name: "upper".into(),
                            func: Rc::new(move |_| Ok(Value::Str(s.to_uppercase()))),
                        })
                    }
                    "lower" => {
                        let s = s.clone();
                        Ok(Value::NativeFunction {
                            name: "lower".into(),
                            func: Rc::new(move |_| Ok(Value::Str(s.to_lowercase()))),
                        })
                    }
                    _ => Err(LunoError::Runtime(format!(
                        "No attribute '{}' on str",
                        name
                    ))),
                }
            }
            Value::List(list) => match name {
                "length" => Ok(Value::Int(list.borrow().len() as i64)),
                "append" => {
                    let list = list.clone();
                    Ok(Value::NativeFunction {
                        name: "append".into(),
                        func: Rc::new(move |args| {
                            if let Some(v) = args.into_iter().next() {
                                list.borrow_mut().push(v);
                            }
                            Ok(Value::Null)
                        }),
                    })
                }
                "pop" => {
                    let list = list.clone();
                    Ok(Value::NativeFunction {
                        name: "pop".into(),
                        func: Rc::new(move |_| Ok(list.borrow_mut().pop().unwrap_or(Value::Null))),
                    })
                }
                _ => Err(LunoError::Runtime(format!(
                    "No attribute '{}' on list",
                    name
                ))),
            },
            Value::Map(map) => {
                if let Some(v) = map.borrow().get(name) {
                    return Ok(v.clone());
                }
                match name {
                    "keys" => {
                        let keys: Vec<Value> =
                            map.borrow().keys().map(|k| Value::Str(k.clone())).collect();
                        Ok(Value::List(Rc::new(RefCell::new(keys))))
                    }
                    "values" => {
                        let vals: Vec<Value> = map.borrow().values().cloned().collect();
                        Ok(Value::List(Rc::new(RefCell::new(vals))))
                    }
                    _ => Err(LunoError::Runtime(format!("No key '{}' in map", name))),
                }
            }
            _ => Err(LunoError::Runtime(format!(
                "Cannot access attribute '{}' on {}",
                name,
                obj.type_name()
            ))),
        }
    }

    // --- Binary operations ---

    fn apply_binop(&self, op: &BinOp, left: &Value, right: &Value) -> Result<Value, LunoError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => match op {
                BinOp::Add => Ok(Value::Int(a + b)),
                BinOp::Sub => Ok(Value::Int(a - b)),
                BinOp::Mul => Ok(Value::Int(a * b)),
                BinOp::Div => {
                    if *b == 0 {
                        return Err(LunoError::Runtime("Division by zero".into()));
                    }
                    Ok(Value::Int(a / b))
                }
                BinOp::Mod => Ok(Value::Int(a % b)),
                BinOp::Pow => Ok(Value::Int(a.pow(*b as u32))),
            },
            (Value::Float(a), Value::Float(b)) => match op {
                BinOp::Add => Ok(Value::Float(a + b)),
                BinOp::Sub => Ok(Value::Float(a - b)),
                BinOp::Mul => Ok(Value::Float(a * b)),
                BinOp::Div => Ok(Value::Float(a / b)),
                BinOp::Mod => Ok(Value::Float(a % b)),
                BinOp::Pow => Ok(Value::Float(a.powf(*b))),
            },
            (Value::Int(a), Value::Float(b)) => {
                self.apply_binop(op, &Value::Float(*a as f64), &Value::Float(*b))
            }
            (Value::Float(a), Value::Int(b)) => {
                self.apply_binop(op, &Value::Float(*a), &Value::Float(*b as f64))
            }
            (Value::Str(a), Value::Str(b)) if matches!(op, BinOp::Add) => {
                Ok(Value::Str(format!("{}{}", a, b)))
            }
            (Value::Str(a), Value::Int(b)) if matches!(op, BinOp::Mul) => {
                Ok(Value::Str(a.repeat(*b as usize)))
            }
            _ => Err(LunoError::Runtime(format!(
                "Unsupported operation {:?} between {} and {}",
                op,
                left.type_name(),
                right.type_name()
            ))),
        }
    }

    fn apply_cmp(&self, op: &CmpOp, left: &Value, right: &Value) -> Result<bool, LunoError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(match op {
                CmpOp::Eq => a == b,
                CmpOp::NotEq => a != b,
                CmpOp::Lt => a < b,
                CmpOp::Gt => a > b,
                CmpOp::LtEq => a <= b,
                CmpOp::GtEq => a >= b,
            }),
            (Value::Float(a), Value::Float(b)) => Ok(match op {
                CmpOp::Eq => a == b,
                CmpOp::NotEq => a != b,
                CmpOp::Lt => a < b,
                CmpOp::Gt => a > b,
                CmpOp::LtEq => a <= b,
                CmpOp::GtEq => a >= b,
            }),
            (Value::Int(a), Value::Float(b)) => {
                self.apply_cmp(op, &Value::Float(*a as f64), &Value::Float(*b))
            }
            (Value::Float(a), Value::Int(b)) => {
                self.apply_cmp(op, &Value::Float(*a), &Value::Float(*b as f64))
            }
            (Value::Str(a), Value::Str(b)) => Ok(match op {
                CmpOp::Eq => a == b,
                CmpOp::NotEq => a != b,
                CmpOp::Lt => a < b,
                CmpOp::Gt => a > b,
                CmpOp::LtEq => a <= b,
                CmpOp::GtEq => a >= b,
            }),
            (Value::Bool(a), Value::Bool(b)) => Ok(match op {
                CmpOp::Eq => a == b,
                CmpOp::NotEq => a != b,
                _ => {
                    return Err(LunoError::Runtime(
                        "Cannot compare booleans with <, >".into(),
                    ))
                }
            }),
            (Value::Null, Value::Null) => Ok(matches!(op, CmpOp::Eq | CmpOp::LtEq | CmpOp::GtEq)),
            _ => Ok(matches!(op, CmpOp::NotEq)),
        }
    }

    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        self.apply_cmp(&CmpOp::Eq, a, b).unwrap_or(false)
    }

    // --- Indexing ---

    fn index_value(&self, obj: &Value, idx: &Value) -> Result<Value, LunoError> {
        match (obj, idx) {
            (Value::List(list), Value::Int(i)) => {
                let list = list.borrow();
                let len = list.len() as i64;
                let actual = if *i < 0 {
                    (len + i) as usize
                } else {
                    *i as usize
                };
                list.get(actual)
                    .cloned()
                    .ok_or_else(|| LunoError::Runtime("Index out of bounds".into()))
            }
            (Value::Map(map), Value::Str(k)) => map
                .borrow()
                .get(k)
                .cloned()
                .ok_or_else(|| LunoError::Runtime(format!("Key '{}' not found", k))),
            (Value::Str(s), Value::Int(i)) => {
                let len = s.len() as i64;
                let actual = if *i < 0 {
                    (len + i) as usize
                } else {
                    *i as usize
                };
                s.chars()
                    .nth(actual)
                    .map(|c| Value::Str(c.to_string()))
                    .ok_or_else(|| LunoError::Runtime("Index out of bounds".into()))
            }
            _ => Err(LunoError::Runtime("Invalid index operation".into())),
        }
    }

    // --- Iteration ---

    fn value_to_iter(&self, val: &Value) -> Result<Vec<Value>, LunoError> {
        match val {
            Value::List(list) => Ok(list.borrow().clone()),
            Value::Str(s) => Ok(s.chars().map(|c| Value::Str(c.to_string())).collect()),
            Value::Map(map) => Ok(map.borrow().keys().map(|k| Value::Str(k.clone())).collect()),
            _ => Err(LunoError::Runtime(format!(
                "'{}' is not iterable",
                val.type_name()
            ))),
        }
    }
}

// --- Built-in functions ---

fn register_builtins(env: &Env) {
    let builtins: Vec<(&str, NativeFn)> = vec![
        (
            "print",
            Rc::new(|args| {
                let s: Vec<String> = args.iter().map(|a| format!("{}", a)).collect();
                println!("{}", s.join(" "));
                Ok(Value::Null)
            }),
        ),
        (
            "input",
            Rc::new(|args| {
                if let Some(prompt) = args.first() {
                    eprint!("{}", prompt);
                }
                let mut line = String::new();
                std::io::stdin().read_line(&mut line).ok();
                Ok(Value::Str(line.trim_end().to_string()))
            }),
        ),
        (
            "len",
            Rc::new(|args| match args.first() {
                Some(Value::Str(s)) => Ok(Value::Int(s.len() as i64)),
                Some(Value::List(l)) => Ok(Value::Int(l.borrow().len() as i64)),
                Some(Value::Map(m)) => Ok(Value::Int(m.borrow().len() as i64)),
                _ => Err(LunoError::Runtime(
                    "len() requires a str, list, or map".into(),
                )),
            }),
        ),
        (
            "range",
            Rc::new(|args| {
                let (start, end, step) = match args.len() {
                    1 => (
                        0,
                        match &args[0] {
                            Value::Int(n) => *n,
                            _ => {
                                return Err(LunoError::Runtime("range() requires int args".into()))
                            }
                        },
                        1,
                    ),
                    2 => (
                        match &args[0] {
                            Value::Int(n) => *n,
                            _ => {
                                return Err(LunoError::Runtime("range() requires int args".into()))
                            }
                        },
                        match &args[1] {
                            Value::Int(n) => *n,
                            _ => {
                                return Err(LunoError::Runtime("range() requires int args".into()))
                            }
                        },
                        1,
                    ),
                    3 => (
                        match &args[0] {
                            Value::Int(n) => *n,
                            _ => {
                                return Err(LunoError::Runtime("range() requires int args".into()))
                            }
                        },
                        match &args[1] {
                            Value::Int(n) => *n,
                            _ => {
                                return Err(LunoError::Runtime("range() requires int args".into()))
                            }
                        },
                        match &args[2] {
                            Value::Int(n) => *n,
                            _ => {
                                return Err(LunoError::Runtime("range() requires int args".into()))
                            }
                        },
                    ),
                    _ => return Err(LunoError::Runtime("range() takes 1-3 arguments".into())),
                };
                let mut items = Vec::new();
                let mut i = start;
                while (step > 0 && i < end) || (step < 0 && i > end) {
                    items.push(Value::Int(i));
                    i += step;
                }
                Ok(Value::List(Rc::new(RefCell::new(items))))
            }),
        ),
        (
            "type",
            Rc::new(|args| match args.first() {
                Some(v) => Ok(Value::Str(v.type_name().to_string())),
                None => Err(LunoError::Runtime("type() requires an argument".into())),
            }),
        ),
        (
            "str",
            Rc::new(|args| match args.first() {
                Some(v) => Ok(Value::Str(format!("{}", v))),
                None => Ok(Value::Str(String::new())),
            }),
        ),
        (
            "int",
            Rc::new(|args| match args.first() {
                Some(Value::Int(n)) => Ok(Value::Int(*n)),
                Some(Value::Float(n)) => Ok(Value::Int(*n as i64)),
                Some(Value::Str(s)) => s
                    .parse::<i64>()
                    .map(Value::Int)
                    .map_err(|_| LunoError::Runtime(format!("Cannot convert '{}' to int", s))),
                Some(Value::Bool(b)) => Ok(Value::Int(if *b { 1 } else { 0 })),
                _ => Err(LunoError::Runtime(
                    "int() requires a convertible argument".into(),
                )),
            }),
        ),
        (
            "float",
            Rc::new(|args| match args.first() {
                Some(Value::Float(n)) => Ok(Value::Float(*n)),
                Some(Value::Int(n)) => Ok(Value::Float(*n as f64)),
                Some(Value::Str(s)) => s
                    .parse::<f64>()
                    .map(Value::Float)
                    .map_err(|_| LunoError::Runtime(format!("Cannot convert '{}' to float", s))),
                _ => Err(LunoError::Runtime(
                    "float() requires a convertible argument".into(),
                )),
            }),
        ),
        (
            "abs",
            Rc::new(|args| match args.first() {
                Some(Value::Int(n)) => Ok(Value::Int(n.abs())),
                Some(Value::Float(n)) => Ok(Value::Float(n.abs())),
                _ => Err(LunoError::Runtime("abs() requires a number".into())),
            }),
        ),
        (
            "min",
            Rc::new(|args| {
                if args.is_empty() {
                    return Err(LunoError::Runtime("min() requires arguments".into()));
                }
                let mut best = args[0].clone();
                for a in &args[1..] {
                    if let (Value::Int(a_val), Value::Int(b_val)) = (a, &best) {
                        if a_val < b_val {
                            best = a.clone();
                        }
                    } else if let (Value::Float(a_val), Value::Float(b_val)) = (a, &best) {
                        if a_val < b_val {
                            best = a.clone();
                        }
                    }
                }
                Ok(best)
            }),
        ),
        (
            "max",
            Rc::new(|args| {
                if args.is_empty() {
                    return Err(LunoError::Runtime("max() requires arguments".into()));
                }
                let mut best = args[0].clone();
                for a in &args[1..] {
                    if let (Value::Int(a_val), Value::Int(b_val)) = (a, &best) {
                        if a_val > b_val {
                            best = a.clone();
                        }
                    } else if let (Value::Float(a_val), Value::Float(b_val)) = (a, &best) {
                        if a_val > b_val {
                            best = a.clone();
                        }
                    }
                }
                Ok(best)
            }),
        ),
    ];

    for (name, func) in builtins {
        env_define(
            env,
            name,
            Value::NativeFunction {
                name: name.to_string(),
                func,
            },
            false,
        );
    }
}

fn register_math_module(env: &Env) {
    let math_fns: Vec<(&str, NativeFn)> = vec![
        (
            "sqrt",
            Rc::new(|args| match args.first() {
                Some(Value::Float(n)) => Ok(Value::Float(n.sqrt())),
                Some(Value::Int(n)) => Ok(Value::Float((*n as f64).sqrt())),
                _ => Err(LunoError::Runtime("sqrt() requires a number".into())),
            }),
        ),
        (
            "floor",
            Rc::new(|args| match args.first() {
                Some(Value::Float(n)) => Ok(Value::Int(n.floor() as i64)),
                Some(Value::Int(n)) => Ok(Value::Int(*n)),
                _ => Err(LunoError::Runtime("floor() requires a number".into())),
            }),
        ),
        (
            "ceil",
            Rc::new(|args| match args.first() {
                Some(Value::Float(n)) => Ok(Value::Int(n.ceil() as i64)),
                Some(Value::Int(n)) => Ok(Value::Int(*n)),
                _ => Err(LunoError::Runtime("ceil() requires a number".into())),
            }),
        ),
        (
            "sin",
            Rc::new(|args| match args.first() {
                Some(Value::Float(n)) => Ok(Value::Float(n.sin())),
                Some(Value::Int(n)) => Ok(Value::Float((*n as f64).sin())),
                _ => Err(LunoError::Runtime("sin() requires a number".into())),
            }),
        ),
        (
            "cos",
            Rc::new(|args| match args.first() {
                Some(Value::Float(n)) => Ok(Value::Float(n.cos())),
                Some(Value::Int(n)) => Ok(Value::Float((*n as f64).cos())),
                _ => Err(LunoError::Runtime("cos() requires a number".into())),
            }),
        ),
        ("pi", Rc::new(|_| Ok(Value::Float(std::f64::consts::PI)))),
    ];

    for (name, func) in math_fns {
        env_define(
            env,
            name,
            Value::NativeFunction {
                name: name.to_string(),
                func,
            },
            false,
        );
    }
}
