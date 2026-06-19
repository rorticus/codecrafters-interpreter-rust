pub mod value;

use std::fmt::write;

use crate::{interpreter::value::Value, parser::expr::Expr};

pub enum InterpreterError {
    UnhandledException,
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::UnhandledException => write!(f, "Unhandled Exception"),
        }
    }
}

pub struct Interpreter {}

impl Interpreter {
    pub fn evaluate(expr: &Expr) -> Result<Value, InterpreterError> {
        match expr {
            Expr::Literal(literal) => Ok(Value::from_literal(literal)),
            _ => Err(InterpreterError::UnhandledException),
        }
    }
}
