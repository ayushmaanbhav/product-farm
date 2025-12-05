//! Error types for the Product-FARM system

use thiserror::Error;

use crate::{AttributeId, ProductId, RuleId};

/// Core errors for the Product-FARM system
#[derive(Debug, Error)]
pub enum CoreError {
    /// Product not found
    #[error("Product not found: {0:?}")]
    ProductNotFound(ProductId),

    /// Attribute not found
    #[error("Attribute not found: {0:?}")]
    AttributeNotFound(AttributeId),

    /// Rule not found
    #[error("Rule not found: {0:?}")]
    RuleNotFound(RuleId),

    /// Invalid state transition
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    /// Validation error (generic)
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Field-specific validation failure
    #[error("Validation failed for field '{field}': {message}")]
    ValidationFailed { field: String, message: String },

    /// Cyclic dependency detected
    #[error("Cyclic dependency detected in rule graph: {0}")]
    CyclicDependency(String),

    /// Type mismatch
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    /// Missing required attribute
    #[error("Missing required attribute: {0:?}")]
    MissingAttribute(AttributeId),

    /// Entity immutable - cannot modify
    #[error("Entity '{entity_type}' with ID '{id}' is immutable and cannot be modified")]
    Immutable { entity_type: String, id: String },

    /// Duplicate entity
    #[error("Duplicate {entity_type} with ID '{id}' already exists")]
    DuplicateEntity { entity_type: String, id: String },

    /// Invalid path format
    #[error("Invalid path format: {0}")]
    InvalidPath(String),

    /// Enumeration value not found
    #[error("Enumeration value '{value}' not found in enum '{enum_name}'")]
    InvalidEnumValue { enum_name: String, value: String },

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for CoreError {
    fn from(e: serde_json::Error) -> Self {
        CoreError::SerializationError(e.to_string())
    }
}

/// Result type alias for CoreError
pub type CoreResult<T> = Result<T, CoreError>;
