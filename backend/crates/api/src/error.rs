//! Error types for API operations

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Unauthorized")]
    Unauthorized,
}

pub type ApiResult<T> = Result<T, ApiError>;
