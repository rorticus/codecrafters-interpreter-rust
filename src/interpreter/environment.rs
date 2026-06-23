use crate::interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type Scope = Rc<RefCell<HashMap<String, Value>>>;

#[derive(Clone)]
pub struct Environment {
    scopes: Vec<Scope>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            scopes: vec![Rc::new(RefCell::new(HashMap::new()))],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(Rc::new(RefCell::new(HashMap::new())));
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.borrow().get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        // walk scopes inward→outward to find where the variable was declared
        for scope in self.scopes.iter().rev() {
            if scope.borrow().contains_key(name) {
                scope.borrow_mut().insert(name.to_string(), value);
                return true;
            }
        }
        false
    }

    pub fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.scopes
            .last()
            .unwrap()
            .borrow_mut()
            .insert(name.to_string(), value);
    }
}
