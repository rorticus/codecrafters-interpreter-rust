pub mod environment;
pub mod value;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    interpreter::{environment::Environment, value::Value},
    lexer::{Token, TokenKind},
    parser::{expr::Expr, stmt::Stmt},
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
}

impl Interpreter {
    pub fn new() -> Self {
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

        Interpreter { environment }
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
                for stmt in statements {
                    self.execute(stmt)?;
                }
                self.environment.pop();

                Ok(())
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

                if let Some(init) = initializer {
                    self.execute(init)?;
                }

                loop {
                    if let Some(cond) = condition {
                        let result = self.evaluate(cond)?;
                        if !result.as_bool() {
                            break;
                        }
                    }

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

                    if let Some(inc) = increment {
                        self.evaluate(inc)?;
                    }
                }

                self.environment.pop();

                Ok(())
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
                    },
                );

                Ok(())
            }
        }
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, Signal> {
        match expr {
            Expr::Literal(literal) => Ok(Value::from_literal(literal)),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Unary { operator, right } => self.eval_unary(operator, right),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.eval_binary(left, operator, right),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.eval_logical(left, operator, right),
            Expr::Identifier(name) => match self.environment.get(&name.lexeme) {
                Some(v) => Ok(v.clone()),
                None => Err(Signal::Error(InterpreterError::RuntimeError(
                    format!("Undeclared variable {}", name.lexeme),
                    name.line,
                ))),
            },
            Expr::Assign { name, value } => {
                if !self.environment.has(&name.lexeme) {
                    return Err(Signal::Error(InterpreterError::RuntimeError(
                        format!("Undeclared identifier {}", name.lexeme),
                        name.line,
                    )));
                } else {
                    let v = self.evaluate(value)?;
                    self.environment.assign(&name.lexeme, v.clone());
                    Ok(v)
                }
            }
            Expr::Call { expr, arguments } => self.call_function(expr, arguments),
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
            Value::Function { name, params, body } => {
                self.environment.push();

                if arguments.len() != params.len() {
                    return Err(Signal::Error(InterpreterError::RuntimeError(
                        format!("Incorrect number of arguments to function"),
                        0,
                    )));
                }

                for i in 0..arguments.len() {
                    let arg_val = self.evaluate(&arguments[i])?;
                    self.environment.define(params[i].as_str(), arg_val);
                }

                let result = self.execute(&body);

                self.environment.pop();

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
