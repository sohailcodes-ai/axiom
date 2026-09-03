use std::fmt;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    Message(String),
}

impl RuntimeError {
    pub fn new(msg: &str) -> Self {
        RuntimeError::Message(msg.to_string())
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::Message(msg) => write!(f, "runtime error: {}", msg),
        }
    }
}

impl std::error::Error for RuntimeError {}
