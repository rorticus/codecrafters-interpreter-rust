use crate::interpreter::Signal;
use crate::interpreter::environment::Environment;
use crate::parser::expr::LiteralValue;
use crate::parser::stmt::Stmt;

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    NativeFunction(&'static str, fn(Vec<Value>) -> Result<Value, Signal>),
    Function {
        name: String,
        params: Vec<String>,
        body: Stmt,
        closure: Environment,
    },
    Class {
        name: String,
    },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::NativeFunction(name, _) => write!(f, "<native fn {}>", name),
            Value::Function {
                name,
                params,
                body,
                closure,
            } => write!(f, "<fn {}>", name),
            Value::Class { name } => write!(f, "{}", name),
        }
    }
}

impl Value {
    pub fn from_literal(literal: &LiteralValue) -> Self {
        match literal {
            LiteralValue::Nil => Value::Nil,
            LiteralValue::Boolean(b) => Value::Boolean(*b),
            LiteralValue::Number(n) => Value::Number(*n),
            LiteralValue::String(s) => Value::String(s.clone()),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Boolean(v) => *v,
            _ => true,
        }
    }
}
