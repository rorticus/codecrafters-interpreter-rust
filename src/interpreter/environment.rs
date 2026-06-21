use crate::interpreter::Value;
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: &Value) {
        self.values.insert(name.to_string(), value.clone());
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(&name.to_string())
    }

    pub fn has(&self, name: &str) -> bool {
        self.values.contains_key(&name.to_string())
    }
}
