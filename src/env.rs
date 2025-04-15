use crate::value::Value;
use crate::error::{Result, SchemeError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Environment {
    bindings: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    pub fn new_child(parent: Rc<RefCell<Environment>>) -> Self {
        Environment {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    // Looks up in current scope only
    pub fn lookup_local(&self, name: &str) -> Option<Value> {
        self.bindings.get(name).cloned()
    }

    // Looks up recursively through parent scopes
    pub fn lookup(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.bindings.get(name) {
            Some(value.clone())
        } else if let Some(parent_env) = &self.parent {
            parent_env.borrow().lookup(name)
        } else {
            None
        }
    }

     // Sets an existing variable, searching up the scope chain
    pub fn set(&mut self, name: &str, value: Value) -> Result<()> {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent_env) = &self.parent {
            parent_env.borrow_mut().set(name, value)
        } else {
            Err(SchemeError::UndefinedVariable(name.to_string()))
        }
    }
}