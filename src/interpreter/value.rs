use crate::parser::expr::LiteralValue;

pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
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
            Value::Number(v) => *v != 0f64,
            Value::String(v) => v.is_empty(),
        }
    }

    pub fn as_number(&self) -> f64 {
        match self {
            Value::Nil => 0f64,
            Value::Boolean(v) => {
                if *v {
                    1f64
                } else {
                    0f64
                }
            }
            Value::Number(v) => *v,
            Value::String(v) => v.parse::<f64>().unwrap_or(0f64),
        }
    }
}
