use crate::interpreter::Signal;
use crate::interpreter::environment::Environment;
use crate::parser::expr::LiteralValue;
use crate::parser::stmt::Stmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct LoxClass {
    pub name: String,
    pub superclass: Option<Rc<LoxClass>>,
    pub methods: HashMap<String, Value>,
}

impl LoxClass {
    pub fn find_method(&self, name: &str) -> Option<Value> {
        if self.methods.contains_key(name) {
            return Some(self.methods.get(name).unwrap().clone());
        } else if let Some(superclass) = self.superclass.clone() {
            return superclass.find_method(name);
        } else {
            return None;
        }
    }
}

#[derive(Clone)]
pub struct LoxClassInstance {
    pub class: Rc<LoxClass>,
    pub fields: Rc<RefCell<HashMap<String, Value>>>,
}

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
        is_initializer: bool,
    },
    Class(Rc<LoxClass>),
    ClassInstance(Rc<LoxClassInstance>),
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
                ..
            } => write!(f, "<fn {}>", name),
            Value::Class(class) => write!(f, "{}", class.name),
            Value::ClassInstance(instance) => write!(f, "{} instance", instance.class.name),
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
