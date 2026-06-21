use crate::interpreter::Value;
use std::collections::HashMap;

pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: &str, value: &Value) {
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), value.clone());
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.scopes.last().unwrap().get(&name.to_string())
    }

    pub fn has(&self, name: &str) -> bool {
        self.scopes.last().unwrap().contains_key(&name.to_string())
    }
}
