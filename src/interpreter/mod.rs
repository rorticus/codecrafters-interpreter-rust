pub mod value;

use crate::{
    interpreter::value::Value,
    lexer::{Token, TokenKind},
    parser::expr::Expr,
};

pub enum InterpreterError {
    UnhandledException,
    Internal(String),
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::UnhandledException => write!(f, "Unhandled Exception"),
            InterpreterError::Internal(msg) => write!(f, "Internal error: {}", msg),
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
}
