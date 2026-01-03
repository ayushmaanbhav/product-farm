//! Error types for LLM evaluation

use thiserror::Error;

/// Errors that can occur during LLM evaluation
#[derive(Debug, Error)]
pub enum LlmEvaluatorError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
}

pub type LlmEvaluatorResult<T> = Result<T, LlmEvaluatorError>;
