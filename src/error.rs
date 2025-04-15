use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum SchemeError {
    #[error("Parser Error: {0}")]
    Parser(String),
    #[error("Evaluation Error: {0}")]
    Eval(String),
    #[error("Runtime Error: {0}")]
    Runtime(String), // For general runtime issues like arity mismatch
    #[error("Type Error: Expected {expected}, found {found}")]
    Type { expected: String, found: String },
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    #[error("Not a procedure: {0}")]
    NotProcedure(String),
    #[error("Arity Mismatch: Expected {expected}, got {got}")]
    Arity { expected: String, got: usize },
}

pub type Result<T> = std::result::Result<T, SchemeError>;