//! Error types for JSON Logic operations

use thiserror::Error;

/// Errors that can occur during JSON Logic parsing and evaluation
#[derive(Debug, Error)]
pub enum JsonLogicError {
    /// Unknown operation
    #[error("Unknown operation: {0}")]
    UnknownOperation(String),

    /// Invalid number of arguments
    #[error("Invalid argument count for '{op}': expected {expected}, got {actual}")]
    InvalidArgumentCount {
        op: String,
        expected: String,
        actual: usize,
    },

    /// Type mismatch during evaluation
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    /// Variable not found in context
    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    /// Invalid variable path
    #[error("Invalid variable path: {0}")]
    InvalidVariablePath(String),

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Invalid JSON structure
    #[error("Invalid JSON structure: {0}")]
    InvalidStructure(String),

    /// Compilation error
    #[error("Compilation error: {0}")]
    CompilationError(String),

    /// Runtime error
    #[error("Runtime error: {0}")]
    RuntimeError(String),

    /// Stack underflow during VM execution
    #[error("Stack underflow")]
    StackUnderflow,

    /// Stack overflow during VM execution
    #[error("Stack overflow")]
    StackOverflow,

    /// Invalid bytecode
    #[error("Invalid bytecode at offset {0}")]
    InvalidBytecode(usize),
}

/// Result type for JSON Logic operations
pub type JsonLogicResult<T> = Result<T, JsonLogicError>;
