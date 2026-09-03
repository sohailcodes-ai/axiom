use std::fmt;

#[derive(Debug, Clone)]
pub struct CompileError {
    pub message: String,
    pub filename: String,
    pub line: usize,
    pub column: usize,
}

impl CompileError {
    pub fn new(message: &str, filename: &str, line: usize, column: usize) -> Self {
        CompileError {
            message: message.to_string(),
            filename: filename.to_string(),
            line,
            column,
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.filename, self.line, self.message)
    }
}

impl std::error::Error for CompileError {}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    pub fn new(message: &str) -> Self {
        RuntimeError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "runtime error: {}", self.message)
    }
}

impl std::error::Error for RuntimeError {}
