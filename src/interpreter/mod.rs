pub mod value;

use crate::{
    interpreter::value::Value,
    lexer::{Token, TokenKind},
    parser::expr::Expr,
};

pub enum InterpreterError {
    UnhandledException,
    Internal(String),
    RuntimeError(String),
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::UnhandledException => write!(f, "Unhandled Exception"),
            InterpreterError::Internal(msg) => write!(f, "Internal error: {}", msg),
            InterpreterError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub fn evaluate(&self, expr: &Expr) -> Result<Value, InterpreterError> {
        match expr {
            Expr::Literal(literal) => Ok(Value::from_literal(literal)),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Unary { operator, right } => self.eval_unary(operator, right),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.eval_binary(left, operator, right),
            _ => Err(InterpreterError::UnhandledException),
        }
    }

    fn eval_unary(&self, operator: &Token, right: &Expr) -> Result<Value, InterpreterError> {
        let value = self.evaluate(right)?;

        match operator.kind {
            TokenKind::Bang => Ok(Value::Boolean(!value.as_bool())),
            TokenKind::Minus => Ok(Value::Number(0f64 - value.as_number())),
            _ => Err(InterpreterError::Internal(format!(
                "Unhandled unary {}",
                operator.lexeme
            ))),
        }
    }

    fn eval_binary(
        &self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Value, InterpreterError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.kind {
            TokenKind::Star => Ok(Value::Number(left.as_number() * right.as_number())),
            TokenKind::Slash => Ok(Value::Number(left.as_number() / right.as_number())),
            TokenKind::Plus => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
                _ => Err(InterpreterError::RuntimeError(
                    "Operands must be two numbers or two strings.".to_string(),
                )),
            },
            TokenKind::Less => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l < r)),
                _ => Err(InterpreterError::RuntimeError(
                    "Operands must be two numbers or two strings.".to_string(),
                )),
            },
            TokenKind::LessEqual => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l <= r)),
                _ => Err(InterpreterError::RuntimeError(
                    "Operands must be two numbers or two strings.".to_string(),
                )),
            },
            TokenKind::Greater => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l > r)),
                _ => Err(InterpreterError::RuntimeError(
                    "Operands must be two numbers or two strings.".to_string(),
                )),
            },
            TokenKind::GreaterEqual => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Boolean(l >= r)),
                _ => Err(InterpreterError::RuntimeError(
                    "Operands must be two numbers or two strings.".to_string(),
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
            TokenKind::Minus => Ok(Value::Number(left.as_number() - right.as_number())),
            _ => Err(InterpreterError::Internal(format!(
                "Unhandled binary operation {}",
                operator.lexeme
            ))),
        }
    }
}
