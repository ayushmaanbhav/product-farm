//! Error types for REST API
//!
//! Provides consistent error handling and HTTP status code mapping.
//! Follows the GenericResponse/ErrorDetail pattern from the legacy Kotlin code.

use std::collections::HashMap;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

// =============================================================================
// ERROR DETAIL - Structured field-level errors
// =============================================================================

/// Detailed error information for a specific field or constraint violation.
/// Follows the Kotlin ErrorDetail pattern for consistent API responses.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDetail {
    /// Error code (e.g., "REQUIRED", "INVALID_FORMAT", "TOO_LONG")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Field name that caused the error (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// Additional parameters for error message interpolation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, String>>,
}

impl ErrorDetail {
    /// Create a simple error detail with code and message
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            field: None,
            params: None,
        }
    }

    /// Create a field-specific error detail
    pub fn for_field(
        code: impl Into<String>,
        message: impl Into<String>,
        field: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            field: Some(field.into()),
            params: None,
        }
    }

    /// Add a parameter for message interpolation
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }
}

// =============================================================================
// ERROR RESPONSE - API error envelope
// =============================================================================

/// API error response body following REST best practices.
/// Includes HTTP status code, message, and detailed errors.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// HTTP status code
    pub status_code: u16,
    /// Primary error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Detailed field-level errors
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ErrorDetail>,
}

impl ErrorResponse {
    /// Create error response with just a message
    pub fn message(status: StatusCode, msg: impl Into<String>) -> Self {
        Self {
            status_code: status.as_u16(),
            message: Some(msg.into()),
            errors: Vec::new(),
        }
    }

    /// Create error response with detailed errors
    pub fn with_errors(status: StatusCode, errors: Vec<ErrorDetail>) -> Self {
        Self {
            status_code: status.as_u16(),
            message: None,
            errors,
        }
    }

    /// Create error response with message and errors
    pub fn with_message_and_errors(
        status: StatusCode,
        msg: impl Into<String>,
        errors: Vec<ErrorDetail>,
    ) -> Self {
        Self {
            status_code: status.as_u16(),
            message: Some(msg.into()),
            errors,
        }
    }
}

// =============================================================================
// API ERROR - Error variants
// =============================================================================

/// API error types that map to HTTP status codes.
/// Provides a type-safe way to represent different error conditions.
#[derive(Debug)]
pub enum ApiError {
    /// Resource not found (404)
    NotFound(String),
    /// Invalid request data (400)
    BadRequest(String),
    /// Resource already exists (409)
    Conflict(String),
    /// Precondition failed (412) - e.g., wrong state for operation
    PreconditionFailed(String),
    /// Multiple validation errors (400)
    ValidationFailed(Vec<ErrorDetail>),
    /// Internal server error (500)
    Internal(String),
    /// Feature not implemented (501)
    NotImplemented(String),
}

impl ApiError {
    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create a bad request error
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    /// Create a conflict error
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    /// Create a precondition failed error
    pub fn precondition_failed(msg: impl Into<String>) -> Self {
        Self::PreconditionFailed(msg.into())
    }

    /// Create a validation failed error from error details
    pub fn validation_failed(errors: Vec<ErrorDetail>) -> Self {
        Self::ValidationFailed(errors)
    }

    /// Create a validation failed error from simple strings
    pub fn validation_errors<S: Into<String>>(errors: impl IntoIterator<Item = S>) -> Self {
        Self::ValidationFailed(
            errors
                .into_iter()
                .map(|e| ErrorDetail::new("VALIDATION_ERROR", e))
                .collect(),
        )
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Create a not implemented error
    pub fn not_implemented(msg: impl Into<String>) -> Self {
        Self::NotImplemented(msg.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, response) = match self {
            ApiError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::message(StatusCode::NOT_FOUND, msg),
            ),
            ApiError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::message(StatusCode::BAD_REQUEST, msg),
            ),
            ApiError::Conflict(msg) => (
                StatusCode::CONFLICT,
                ErrorResponse::message(StatusCode::CONFLICT, msg),
            ),
            ApiError::PreconditionFailed(msg) => (
                StatusCode::PRECONDITION_FAILED,
                ErrorResponse::message(StatusCode::PRECONDITION_FAILED, msg),
            ),
            ApiError::ValidationFailed(errors) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::with_message_and_errors(
                    StatusCode::BAD_REQUEST,
                    "Validation failed",
                    errors,
                ),
            ),
            ApiError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::message(StatusCode::INTERNAL_SERVER_ERROR, msg),
            ),
            ApiError::NotImplemented(msg) => (
                StatusCode::NOT_IMPLEMENTED,
                ErrorResponse::message(StatusCode::NOT_IMPLEMENTED, msg),
            ),
        };

        (status, Json(response)).into_response()
    }
}

// =============================================================================
// ERROR CONVERSIONS
// =============================================================================

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest(format!("Invalid JSON: {}", err))
    }
}

impl From<super::types::InputValidationError> for ApiError {
    fn from(err: super::types::InputValidationError) -> Self {
        ApiError::ValidationFailed(vec![ErrorDetail::for_field(
            codes::TOO_LONG,
            err.message,
            err.field,
        )])
    }
}

// =============================================================================
// RESULT TYPE ALIAS
// =============================================================================

/// Result type alias for REST handlers
pub type ApiResult<T> = Result<T, ApiError>;

// =============================================================================
// ERROR CODES - Standard error codes for consistency
// =============================================================================

/// Standard error codes for consistency across API
pub mod codes {
    pub const REQUIRED: &str = "REQUIRED";
    pub const INVALID_FORMAT: &str = "INVALID_FORMAT";
    pub const TOO_LONG: &str = "TOO_LONG";
    pub const TOO_SHORT: &str = "TOO_SHORT";
    pub const OUT_OF_RANGE: &str = "OUT_OF_RANGE";
    pub const INVALID_VALUE: &str = "INVALID_VALUE";
    pub const DUPLICATE: &str = "DUPLICATE";
    pub const NOT_FOUND: &str = "NOT_FOUND";
    pub const IMMUTABLE: &str = "IMMUTABLE";
    pub const INVALID_STATE: &str = "INVALID_STATE";
    pub const DEPENDENCY_ERROR: &str = "DEPENDENCY_ERROR";
}
