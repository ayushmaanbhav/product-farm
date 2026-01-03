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

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Server error ({status}): {message}")]
    ServerError { status: u16, message: String },

    #[error("Timeout: {0}")]
    Timeout(String),
}

impl LlmEvaluatorError {
    /// Returns true if this error is transient and should be retried
    ///
    /// Retryable errors:
    /// - NetworkError: Connection issues, DNS failures
    /// - RateLimitError: 429 responses (should retry with backoff)
    /// - ServerError: 5xx responses (temporary server issues)
    /// - Timeout: Request took too long
    ///
    /// Non-retryable errors:
    /// - ConfigError: Invalid configuration (won't be fixed by retry)
    /// - ParseError: Response parsing failed (won't be fixed by retry)
    /// - ApiError: 4xx errors like auth failure, bad request
    /// - FeatureNotEnabled: Feature flag not enabled
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            LlmEvaluatorError::NetworkError(_)
                | LlmEvaluatorError::RateLimitError(_)
                | LlmEvaluatorError::ServerError { .. }
                | LlmEvaluatorError::Timeout(_)
        )
    }

    /// Create a network error
    pub fn network(msg: impl Into<String>) -> Self {
        LlmEvaluatorError::NetworkError(msg.into())
    }

    /// Create a rate limit error
    pub fn rate_limit(msg: impl Into<String>) -> Self {
        LlmEvaluatorError::RateLimitError(msg.into())
    }

    /// Create a server error with status code
    pub fn server_error(status: u16, msg: impl Into<String>) -> Self {
        LlmEvaluatorError::ServerError {
            status,
            message: msg.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(msg: impl Into<String>) -> Self {
        LlmEvaluatorError::Timeout(msg.into())
    }
}

pub type LlmEvaluatorResult<T> = Result<T, LlmEvaluatorError>;

/// Convert LlmEvaluatorError to CoreError, preserving error details
impl From<LlmEvaluatorError> for product_farm_core::CoreError {
    fn from(e: LlmEvaluatorError) -> Self {
        let is_retryable = e.is_retryable();
        let (error_type, message, status_code) = match &e {
            LlmEvaluatorError::ConfigError(msg) => ("config".to_string(), msg.clone(), None),
            LlmEvaluatorError::ApiError(msg) => ("api".to_string(), msg.clone(), None),
            LlmEvaluatorError::ParseError(msg) => ("parse".to_string(), msg.clone(), None),
            LlmEvaluatorError::FeatureNotEnabled(msg) => ("feature".to_string(), msg.clone(), None),
            LlmEvaluatorError::NetworkError(msg) => ("network".to_string(), msg.clone(), None),
            LlmEvaluatorError::RateLimitError(msg) => ("rate_limit".to_string(), msg.clone(), None),
            LlmEvaluatorError::ServerError { status, message } => {
                ("server".to_string(), message.clone(), Some(*status))
            }
            LlmEvaluatorError::Timeout(msg) => ("timeout".to_string(), msg.clone(), None),
        };
        product_farm_core::CoreError::LlmError {
            error_type,
            message,
            status_code,
            is_retryable,
        }
    }
}
