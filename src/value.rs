use crate::env::Environment;
use crate::error::{Result, SchemeError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

// Type alias for built-in functions
pub type BuiltinFn = fn(&[Value], Rc<RefCell<Environment>>) -> Result<Value>;

#[derive(Clone)]
pub enum Value {
    Integer(i64),
    Bool(bool),
    Symbol(String),
    String(String),
    Nil,
    List(Vec<Value>),
    Array(Rc<RefCell<Vec<Value>>>), // Rc for sharing, RefCell for interior mutability
    Map(Rc<RefCell<HashMap<String, Value>>>), // Keys are strings, values are Values
    Lambda {
        params: Rc<Vec<String>>,
        body: Rc<Value>, // Body is usually a single expression, often (begin ...)
        env: Rc<RefCell<Environment>>, // Closure environment
    },
    Builtin(BuiltinFn, String), // Store name for display
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::String(s) => write!(f, "\"{}\"", s), // Display with quotes
            Value::Nil => write!(f, "()"),
            Value::List(lst) => {
                let strs: Vec<String> = lst.iter().map(|v| format!("{:?}", v)).collect();
                write!(f, "({})", strs.join(" "))
            }
            Value::Array(arr) => {
                let borrowed = arr.borrow();
                let strs: Vec<String> = borrowed.iter().map(|v| format!("{:?}", v)).collect();
                write!(f, "[{}]", strs.join(", "))
            }
            Value::Map(map) => {
                let borrowed = map.borrow();
                let strs: Vec<String> = borrowed
                    .iter()
                    .map(|(k, v)| format!("{}: {:?}", k, v))
                    .collect();
                write!(f, "{{{}}}", strs.join(", "))
            }
            Value::Lambda { params, .. } => write!(f, "#<procedure:{}>", params.join(" ")),
            Value::Builtin(_, name) => write!(f, "#<builtin:{}>", name),
        }
    }
}

// PartialEq for basic comparisons (useful for tests, equal?)
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::List(a), Value::List(b)) => a == b, // Recursive PartialEq
            (Value::Array(a), Value::Array(b)) => Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow(), // Structural for arrays
            (Value::Map(a), Value::Map(b)) => Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow(), // Structural for maps
            // Lambdas and Builtins are generally compared by identity (pointer equality) in Scheme (eq?)
            // If we needed structural comparison for functions, it'd be complex.
            _ => false, // Different types are not equal
        }
    }
}

impl Value {
    pub fn type_name(&self) -> String {
        match self {
            Value::Integer(_) => "integer".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::Symbol(_) => "symbol".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Nil => "nil".to_string(),
            Value::List(_) => "list".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Map(_) => "map".to_string(),
            Value::Lambda { .. } => "procedure".to_string(),
            Value::Builtin(_, _) => "procedure".to_string(),
        }
    }

    // Helper for creating errors
    pub fn type_error(expected: &str, found: &Value) -> SchemeError {
        SchemeError::Type {
            expected: expected.to_string(),
            found: found.type_name(),
        }
    }
}