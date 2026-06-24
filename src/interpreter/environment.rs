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

    pub fn get_at(&self, depth: usize, name: &str) -> Option<Value> {
        let idx = self.scopes.len() - 1 - depth;
        self.scopes.get(idx)?.borrow().get(name).cloned()
    }

    pub fn assign_at(&mut self, depth: usize, name: &str, value: Value) -> bool {
        let idx = self.scopes.len() - 1 - depth;

        if let Some(scope) = self.scopes.get(idx) {
            scope.borrow_mut().insert(name.to_string(), value);
            true
        } else {
            false
        }
    }

    pub fn get_global(&self, name: &str) -> Option<Value> {
        self.scopes[0].borrow().get(name).cloned()
    }

    pub fn assign_global(&mut self, name: &str, value: Value) -> bool {
        if self.scopes[0].borrow().contains_key(name) {
            self.scopes[0].borrow_mut().insert(name.to_string(), value);
            true
        } else {
            false
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.scopes
            .last()
            .unwrap()
            .borrow_mut()
            .insert(name.to_string(), value);
    }
}
