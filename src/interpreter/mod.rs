pub mod environment;
pub mod value;

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    interpreter::{
        environment::Environment,
        value::{LoxClass, LoxClassInstance, Value},
    },
    lexer::{Token, TokenKind},
    parser::{
        expr::{Expr, ExprKind},
        stmt::Stmt,
    },
};

pub enum InterpreterError {
    Internal(String),
    RuntimeError(String, usize),
}

pub enum Signal {
    Break(usize),
    Continue(usize),
    Return(Value),
    Error(InterpreterError),
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::Internal(msg) => write!(f, "Internal error: {}", msg),
            InterpreterError::RuntimeError(msg, line) => write!(f, "{}\n[Line {}]", msg, line),
        }
    }
}

pub struct Interpreter {
    environment: Environment,
    depths: HashMap<usize, usize>,
}

impl Interpreter {
    pub fn new(depths: HashMap<usize, usize>) -> Self {
        let mut environment = Environment::new();

        environment.define(
            "clock",
            Value::NativeFunction("clock", |_| {
                Ok(Value::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                ))
            }),
        );

        Interpreter {
            environment,
            depths,
        }
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), Signal> {
        match stmt {
            Stmt::Expression(e) => {
                self.evaluate(e)?;
                Ok(())
            }
            Stmt::Print(e) => {
                let value = self.evaluate(e)?;
                println!("{value}");
                Ok(())
            }
            Stmt::Declaration(name, value) => {
                let val = if let Some(v) = value {
                    self.evaluate(v)?
                } else {
                    Value::Nil
                };

                self.environment.define(&name.lexeme, val);

                Ok(())
            }
            Stmt::Block(statements) => {
                self.environment.push();
                let mut result = Ok(());
                for stmt in statements {
                    result = self.execute(stmt);
                    if result.is_err() {
                        break;
                    }
                }
                self.environment.pop(); // always runs, even on Signal::Return
                result
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let value = self.evaluate(condition)?;

                if value.as_bool() {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }

                Ok(())
            }
            Stmt::While { condition, block } => {
                loop {
                    let value = self.evaluate(condition)?;

                    if value.as_bool() {
                        match self.execute(block) {
                            Err(Signal::Break(_)) => {
                                break;
                            }
                            Err(Signal::Continue(_)) => {
                                continue;
                            }
                            Err(e) => return Err(e),
                            _ => {}
                        }
                    } else {
                        break;
                    }
                }

                Ok(())
            }
            Stmt::For {
                initializer,
                condition,
                increment,
                block,
            } => {
                self.environment.push();

                let result = 'for_loop: {
                    if let Some(init) = initializer {
                        if let Err(e) = self.execute(init) {
                            break 'for_loop Err(e);
                        }
                    }

                    loop {
                        if let Some(cond) = condition {
                            match self.evaluate(cond) {
                                Ok(v) if !v.as_bool() => break,
                                Ok(_) => {}
                                Err(e) => break 'for_loop Err(e),
                            }
                        }

                        match self.execute(block) {
                            Err(Signal::Break(_)) => break,
                            Err(Signal::Continue(_)) => {
                                if let Some(inc) = increment {
                                    if let Err(e) = self.evaluate(inc) {
                                        break 'for_loop Err(e);
                                    }
                                }
                                continue;
                            }
                            Err(e) => break 'for_loop Err(e),
                            Ok(_) => {}
                        }

                        if let Some(inc) = increment {
                            if let Err(e) = self.evaluate(inc) {
                                break 'for_loop Err(e);
                            }
                        }
                    }

                    Ok(())
                };

                self.environment.pop();
                result
            }
            Stmt::Break(t) => Err(Signal::Break(t.line)),
            Stmt::Continue(t) => Err(Signal::Continue(t.line)),
            Stmt::Return(expr) => match expr {
                Some(expr) => {
                    let result = self.evaluate(expr)?;
                    Err(Signal::Return(result))
                }
                None => Err(Signal::Return(Value::Nil)),
            },
            Stmt::Function { name, params, body } => {
                self.environment.define(
                    &name.lexeme,
                    Value::Function {
                        name: name.lexeme.clone(),
                        params: params.iter().map(|p| p.lexeme.clone()).collect(),
                        body: *body.clone(),
                        closure: self.environment.clone(),
                        is_initializer: false,
                    },
                );

                Ok(())
            }
            Stmt::Class {
                name,
                methods,
                superclass,
            } => {
                let mut class_methods = HashMap::new();

                let resolved_superclass = if let Some(sc) = superclass {
                    let klass = self.evaluate(sc)?;
                    if let Value::Class(rc) = klass {
                        Some(rc)
                    } else {
                        return Err(Signal::Error(InterpreterError::RuntimeError(
                            format!("superclass is not a class"),
                            sc.line,
                        )));
                    }
                } else {
                    None
                };

                if let Some(ref rc) = resolved_superclass {
                    self.environment.push();
                    self.environment.define("super", Value::Class(rc.clone()));
                }

                for stmt in methods {
                    match stmt {
                        Stmt::Function { name, params, body } => {
                            let method = Value::Function {
                                name: name.lexeme.clone(),
                                params: params.iter().map(|t| t.lexeme.clone()).collect(),
                                body: *body.clone(),
                                closure: self.environment.clone(),
                                is_initializer: name.lexeme == "init",
                            };

                            class_methods.insert(name.lexeme.clone(), method);
                        }
                        _ => {}
                    }
                }

                let lox_class = LoxClass {
                    name: name.lexeme.to_string(),
                    methods: class_methods,
                    superclass: resolved_superclass.clone(),
                };

                if resolved_superclass.is_some() {
                    self.environment.pop();
                }

                self.environment
                    .define(&name.lexeme, Value::Class(Rc::new(lox_class)));

                Ok(())
            }
        }
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, Signal> {
        match &expr.kind {
            ExprKind::Literal(literal) => Ok(Value::from_literal(literal)),
            ExprKind::Grouping(expr) => self.evaluate(expr),
            ExprKind::Unary { operator, right } => self.eval_unary(operator, right),
            ExprKind::Binary {
                left,
                operator,
                right,
            } => self.eval_binary(left, operator, right),
            ExprKind::Logical {
                left,
                operator,
                right,
            } => self.eval_logical(left, operator, right),
            ExprKind::Identifier(name) => {
                if let Some(&depth) = self.depths.get(&expr.id) {
                    self.environment.get_at(depth, &name.lexeme).ok_or_else(|| {
                        Signal::Error(InterpreterError::RuntimeError(
                            format!("Undefined variable {}", name.lexeme),
                            name.line,
                        ))
                    })
                } else {
                    self.environment.get_global(&name.lexeme).ok_or_else(|| {
                        Signal::Error(InterpreterError::RuntimeError(
                            format!("Undeclared variable {}", name.lexeme),
                            name.line,
                        ))
                    })
                }
            }
            ExprKind::Assign { name, value } => {
                let v = self.evaluate(value)?;

                if let Some(&depth) = self.depths.get(&expr.id) {
                    if !self.environment.assign_at(depth, &name.lexeme, v.clone()) {
                        return Err(Signal::Error(InterpreterError::RuntimeError(
                            format!("Undeclared identifier {}", name.lexeme),
                            name.line,
                        )));
                    }
                } else {
                    self.environment.assign_global(&name.lexeme, v.clone());
                }

                Ok(v)
            }
            ExprKind::Call { expr, arguments } => {
                self.call_function_expr(expr, arguments)
                    .map_err(|e| match e {
                        Signal::Error(InterpreterError::RuntimeError(msg, _)) => {
                            Signal::Error(InterpreterError::RuntimeError(msg, expr.line))
                        }
                        other => other,
                    })
            }
            ExprKind::Get { object, name } => {
                let val = self.evaluate(object)?;

                match val {
                    Value::ClassInstance(class_instance) => {
                        let fields = class_instance.fields.borrow();

                        if fields.contains_key(&name.lexeme) {
                            Ok(fields.get(&name.lexeme).unwrap().clone())
                        } else if let Some(method) = class_instance.class.find_method(&name.lexeme)
                        {
                            Ok(self
                                .bind_method(&method, Value::ClassInstance(class_instance.clone())))
                        } else {
                            Err(Signal::Error(InterpreterError::RuntimeError(
                                format!("Undefined property '{}'", name.lexeme),
                                name.line,
                            )))
                        }
                    }
                    _ => Err(Signal::Error(InterpreterError::RuntimeError(
                        format!("cannot access member variable on non class variable"),
                        expr.line,
                    ))),
                }
            }
            ExprKind::Set {
                object,
                name,
                value,
            } => {
                let obj = self.evaluate(object)?;

                match obj {
                    Value::ClassInstance(lox_instance) => {
                        let mut fields = lox_instance.fields.borrow_mut();

                        let value = self.evaluate(value)?;

                        fields.insert(name.lexeme.clone(), value.clone());

                        Ok(value)
                    }
                    _ => Err(Signal::Error(InterpreterError::RuntimeError(
                        "cannot set field on non class variable".to_string(),
                        name.line,
                    ))),
                }
            }
            ExprKind::This(t) => {
                if let Some(&depth) = self.depths.get(&expr.id) {
                    return self.environment.get_at(depth, &t.lexeme).ok_or_else(|| {
                        Signal::Error(InterpreterError::RuntimeError(
                            format!("Invalid usage of this"),
                            t.line,
                        ))
                    });
                }

                Err(Signal::Error(InterpreterError::RuntimeError(
                    "Invalid usage of this".to_string(),
                    t.line,
                )))
            }
            ExprKind::Super(t, m) => {
                if let Some(&depth) = self.depths.get(&expr.id) {
                    let superclass = self.environment.get_at(depth, &t.lexeme);
                    if let Some(Value::Class(superclass)) = superclass {
                        let object = self.environment.get_at(depth - 1, "this");
                        if object.is_none() {
                            return Err(Signal::Error(InterpreterError::RuntimeError(
                                format!("this not found"),
                                t.line,
                            )));
                        }

                        let method = superclass.find_method(&m.lexeme);
                        if method.is_none() {
                            return Err(Signal::Error(InterpreterError::RuntimeError(
                                format!("Undefined property '{}'.", m.lexeme),
                                t.line,
                            )));
                        }

                        return Ok(self.bind_method(&method.unwrap(), object.unwrap()));
                    } else {
                        return Err(Signal::Error(InterpreterError::RuntimeError(
                            format!("Invalid usage of super"),
                            t.line,
                        )));
                    }
                }

                Err(Signal::Error(InterpreterError::RuntimeError(
                    "Invalid usage of t".to_string(),
                    t.line,
                )))
            }
        }
    }

    fn eval_unary(&mut self, operator: &Token, right: &Expr) -> Result<Value, Signal> {
        let value = self.evaluate(right)?;

        match operator.kind {
            TokenKind::Bang => Ok(Value::Boolean(!value.as_bool())),
            TokenKind::Minus => match value {
                Value::Number(r) => Ok(Value::Number(-r)),
                _ => Err(Signal::Error(InterpreterError::RuntimeError(
                    "Operand must be a number.".to_string(),
                    operator.line,
                ))),
            },
            _ => Err(Signal::Error(InterpreterError::Internal(format!(
                "Unhandled unary {}",
                operator.lexeme
            )))),
        }
    }

    fn eval_logical(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Value, Signal> {
        match operator.kind {
            TokenKind::Or => {
                let left = self.evaluate(left)?;

                if left.as_bool() {
                    Ok(left)
                } else {
                    self.evaluate(right)
                }
            }
            TokenKind::And => {
                let left = self.evaluate(left)?;

                if !left.as_bool() {
                    Ok(left)
                } else {
                    self.evaluate(right)
                }
            }
            _ => Err(Signal::Error(InterpreterError::Internal(format!(
                "Unhandled logical operator {}",
                operator.lexeme
            )))),
        }
    }

    fn eval_binary(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Value, Signal> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.kind {
            TokenKind::Star => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                _ => Err(Signal::Error(InterpreterError::RuntimeError(
                    "Operands must be numbers.".to_string(),
                    operator.line,
                ))),
            },
            TokenKind::Slash => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
                _ => Err(Signal::Error(InterpreterError::RuntimeError(
                    "Operands must be numbers.".to_string(),
                    operator.line,
                ))),
            },
            TokenKind::Plus => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
                _ => Err(Signal::Error(InterpreterError::RuntimeError(
                    "Operands must be two numbers or two strings.".to_string(),
                    operator.line,
                ))),
            },
            TokenKind::Less => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l < r)),
                _ => Err(Signal::Error(InterpreterError::RuntimeError(
                    "Operands must be numbers.".to_string(),
                    operator.line,
                ))),
            },
            TokenKind::LessEqual => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l <= r)),
                _ => Err(Signal::Error(InterpreterError::RuntimeError(
                    "Operands must be numbers.".to_string(),
                    operator.line,
                ))),
            },
            TokenKind::Greater => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l > r)),
                _ => Err(Signal::Error(InterpreterError::RuntimeError(
                    "Operands must be numbers.".to_string(),
                    operator.line,
                ))),
            },
            TokenKind::GreaterEqual => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l >= r)),
                _ => Err(Signal::Error(InterpreterError::RuntimeError(
                    "Operands must be numbers.".to_string(),
                    operator.line,
                ))),
            },
            TokenKind::EqualEqual => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l == r)),
                (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l == r)),
                (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l == r)),
                _ => Ok(Value::Boolean(false)),
            },
            TokenKind::BangEqual => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l != r)),
                (Value::String(l), Value::String(r)) => Ok(Value::Boolean(l != r)),
                (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l != r)),
                _ => Ok(Value::Boolean(false)),
            },
            TokenKind::Minus => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                _ => Err(Signal::Error(InterpreterError::RuntimeError(
                    "Operands must be numbers.".to_string(),
                    operator.line,
                ))),
            },
            _ => Err(Signal::Error(InterpreterError::Internal(format!(
                "Unhandled binary operator {}",
                operator.lexeme
            )))),
        }
    }

    fn call_function(&mut self, func: &Value, arguments: Vec<Value>) -> Result<Value, Signal> {
        match func {
            Value::NativeFunction(_, fn_call) => fn_call(arguments),
            Value::Function {
                name,
                params,
                body,
                closure,
                is_initializer,
            } => {
                if arguments.len() != params.len() {
                    return Err(Signal::Error(InterpreterError::RuntimeError(
                        format!(
                            "Expected {} arguments but got {}.",
                            params.len(),
                            arguments.len()
                        ),
                        0,
                    )));
                }

                let saved_env = std::mem::replace(&mut self.environment, closure.clone());
                self.environment.push();

                for (param, val) in params.iter().zip(arguments) {
                    self.environment.define(param, val);
                }

                let result = if let Stmt::Block(stmts) = body {
                    let mut result = Ok(());
                    for stmt in stmts {
                        result = self.execute(&stmt);
                        if result.is_err() {
                            break;
                        }
                    }
                    result
                } else {
                    self.execute(&body)
                };

                self.environment.pop();
                self.environment = saved_env;

                if *is_initializer && result.is_ok() {
                    return Ok(closure.get_at(0, "this").unwrap());
                }

                match result {
                    Ok(_) => Ok(Value::Nil),
                    Err(Signal::Return(v)) => Ok(v),
                    Err(e) => Err(e),
                }
            }
            Value::Class(lox_class) => {
                let class_instance = LoxClassInstance {
                    class: lox_class.clone(),
                    fields: Rc::new(RefCell::new(HashMap::new())),
                };

                let value = Value::ClassInstance(Rc::new(class_instance));

                if let Some(init_method) = lox_class.find_method("init") {
                    let bound_init = self.bind_method(&init_method, value.clone());
                    self.call_function(&bound_init, arguments)?;
                }

                Ok(value)
            }
            _ => Err(Signal::Error(InterpreterError::RuntimeError(
                format!("Trying to call non-function"),
                0,
            ))),
        }
    }

    fn call_function_expr(
        &mut self,
        identifier: &Expr,
        arguments: &Vec<Expr>,
    ) -> Result<Value, Signal> {
        let value = self.evaluate(identifier)?;
        let args: Result<Vec<Value>, Signal> =
            arguments.iter().map(|arg| self.evaluate(arg)).collect();

        self.call_function(&value, args?)
    }

    fn bind_method(&self, method: &Value, instance: Value) -> Value {
        if let Value::Function {
            name,
            params,
            body,
            closure,
            is_initializer,
        } = method
        {
            let mut bound_closure = closure.clone();
            bound_closure.push(); // new innermost scope
            bound_closure.define("this", instance); // inject `this`
            Value::Function {
                name: name.clone(),
                params: params.clone(),
                body: body.clone(),
                closure: bound_closure,
                is_initializer: *is_initializer,
            }
        } else {
            method.clone()
        }
    }
}
