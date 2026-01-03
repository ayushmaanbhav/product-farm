//! Error types for AI Agent operations

use thiserror::Error;

/// Errors that can occur during AI agent operations
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Invalid natural language input: {0}")]
    InvalidInput(String),

    #[error("Failed to parse JSON Logic: {0}")]
    JsonLogicParseError(String),

    #[error("Rule validation failed: {0}")]
    ValidationError(String),

    #[error("Cycle detected in rule dependencies: {0}")]
    CycleDetected(String),

    #[error("Product not found: {0}")]
    ProductNotFound(String),

    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("Attribute not found: {0}")]
    AttributeNotFound(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Evaluation error: {0}")]
    EvaluationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("External API error: {0}")]
    ExternalApiError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Tool execution error: {0}")]
    ToolError(String),

    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),

    #[error("Maximum iterations reached: {0}")]
    MaxIterationsReached(usize),
}

pub type AgentResult<T> = Result<T, AgentError>;
