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

                self.environment.define(name, val);

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
            Expr::Logical {
                left,
                operator,
                right,
            } => self.eval_logical(left, operator, right),
            Expr::Identifier(name) => match self.environment.get(&name.lexeme) {
                Some(v) => Ok(v.clone()),
                None => Err(InterpreterError::RuntimeError(
                    format!("Undeclared variable {}", name.lexeme),
                    name.line,
                )),
            },
            Expr::Assign { name, value } => {
                if !self.environment.has(&name.lexeme) {
                    return Err(InterpreterError::RuntimeError(
                        format!("Undeclared identifier {}", name.lexeme),
                        name.line,
                    ));
                } else {
                    let v = self.evaluate(value)?;
                    self.environment.assign(&name.lexeme, v.clone());
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

    fn eval_logical(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Value, InterpreterError> {
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
            _ => Err(InterpreterError::Internal(format!(
                "Unhandled logical operator {}",
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
