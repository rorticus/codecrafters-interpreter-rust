pub mod environment;
pub mod value;

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    interpreter::{environment::Environment, value::Value},
    lexer::{Token, TokenKind},
    parser::{expr::Expr, expr::ExprKind, stmt::Stmt},
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
                    },
                );

                Ok(())
            }
            Stmt::Class { name, .. } => {
                self.environment.define(
                    &name.lexeme,
                    Value::Class {
                        name: name.lexeme.to_string(),
                    },
                );
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
                self.call_function(expr, arguments).map_err(|e| match e {
                    Signal::Error(InterpreterError::RuntimeError(msg, _)) => {
                        Signal::Error(InterpreterError::RuntimeError(msg, expr.line))
                    }
                    other => other,
                })
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

    fn call_function(&mut self, identifier: &Expr, arguments: &Vec<Expr>) -> Result<Value, Signal> {
        let value = self.evaluate(identifier)?;

        match value {
            Value::NativeFunction(_, fn_call) => {
                let args: Result<Vec<Value>, Signal> =
                    arguments.iter().map(|arg| self.evaluate(arg)).collect();
                fn_call(args?)
            }
            Value::Function {
                name,
                params,
                body,
                closure,
            } => {
                let mut arg_vals = vec![];

                for arg in arguments {
                    arg_vals.push(self.evaluate(arg)?);
                }

                if arg_vals.len() != params.len() {
                    return Err(Signal::Error(InterpreterError::RuntimeError(
                        format!(
                            "Expected {} arguments but got {}.",
                            params.len(),
                            arg_vals.len()
                        ),
                        0,
                    )));
                }

                let saved_env = std::mem::replace(&mut self.environment, closure);
                self.environment.push();

                for (param, val) in params.iter().zip(arg_vals) {
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

                match result {
                    Ok(_) => Ok(Value::Nil),
                    Err(Signal::Return(v)) => Ok(v),
                    Err(e) => Err(e),
                }
            }
            _ => Err(Signal::Error(InterpreterError::RuntimeError(
                format!("Trying to call non-function"),
                0,
            ))),
        }
    }
}
