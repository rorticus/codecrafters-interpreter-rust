pub mod environment;
pub mod value;

use crate::{
    interpreter::{environment::Environment, value::Value},
    lexer::{Token, TokenKind},
    parser::{expr::Expr, stmt::Stmt},
};

pub enum InterpreterError {
    Internal(String),
    RuntimeError(String, usize),
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
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), InterpreterError> {
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

                self.environment.define(name, &val);

                Ok(())
            }
            Stmt::Block(statements) => {
                for stmt in statements {
                    self.execute(stmt)?;
                }

                Ok(())
            }
        }
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, InterpreterError> {
        match expr {
            Expr::Literal(literal) => Ok(Value::from_literal(literal)),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Unary { operator, right } => self.eval_unary(operator, right),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.eval_binary(left, operator, right),
            Expr::Identifier(name) => match self.environment.get(name) {
                Some(v) => Ok(v.clone()),
                None => Err(InterpreterError::RuntimeError(
                    format!("Undeclared variable {}", name),
                    0,
                )),
            },
            Expr::Assign { name, value } => {
                if !self.environment.has(name) {
                    return Err(InterpreterError::RuntimeError(
                        format!("Undeclared identifier {}", name),
                        0,
                    ));
                } else {
                    let v = self.evaluate(value)?;
                    self.environment.define(name, &v);
                    Ok(v)
                }
            }
        }
    }

    fn eval_unary(&mut self, operator: &Token, right: &Expr) -> Result<Value, InterpreterError> {
        let value = self.evaluate(right)?;

        match operator.kind {
            TokenKind::Bang => Ok(Value::Boolean(!value.as_bool())),
            TokenKind::Minus => match value {
                Value::Number(r) => Ok(Value::Number(-r)),
                _ => Err(InterpreterError::RuntimeError(
                    "Operand must be a number.".to_string(),
                    operator.line,
                )),
            },
            _ => Err(InterpreterError::Internal(format!(
                "Unhandled unary {}",
                operator.lexeme
            ))),
        }
    }

    fn eval_binary(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Value, InterpreterError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.kind {
            TokenKind::Star => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                _ => Err(InterpreterError::RuntimeError(
                    "Operands must be numbers.".to_string(),
                    operator.line,
                )),
            },
            TokenKind::Slash => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
                _ => Err(InterpreterError::RuntimeError(
                    "Operands must be numbers.".to_string(),
                    operator.line,
                )),
            },
            TokenKind::Plus => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
                _ => Err(InterpreterError::RuntimeError(
                    "Operands must be two numbers or two strings.".to_string(),
                    operator.line,
                )),
            },
            TokenKind::Less => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l < r)),
                _ => Err(InterpreterError::RuntimeError(
                    "Operands must be numbers.".to_string(),
                    operator.line,
                )),
            },
            TokenKind::LessEqual => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l <= r)),
                _ => Err(InterpreterError::RuntimeError(
                    "Operands must be numbers.".to_string(),
                    operator.line,
                )),
            },
            TokenKind::Greater => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l > r)),
                _ => Err(InterpreterError::RuntimeError(
                    "Operands must be numbers.".to_string(),
                    operator.line,
                )),
            },
            TokenKind::GreaterEqual => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l >= r)),
                _ => Err(InterpreterError::RuntimeError(
                    "Operands must be numbers.".to_string(),
                    operator.line,
                )),
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
                _ => Err(InterpreterError::RuntimeError(
                    "Operands must be numbers.".to_string(),
                    operator.line,
                )),
            },
            _ => Err(InterpreterError::Internal(format!(
                "Unhandled binary operator {}",
                operator.lexeme
            ))),
        }
    }
}
