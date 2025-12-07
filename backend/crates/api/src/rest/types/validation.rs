//! Input validation types and helpers
//!
//! Provides the ValidateInput trait and helper functions for validating
//! request inputs against size/length limits.

use crate::config::limits::{
    MAX_ARRAY_ITEMS, MAX_DESCRIPTION_LENGTH, MAX_ENUM_VALUE_LENGTH, MAX_EXPRESSION_LENGTH,
    MAX_ID_LENGTH, MAX_NAME_LENGTH, MAX_PATH_LENGTH, MAX_TAG_LENGTH,
};

/// Validation error for input limits
#[derive(Debug)]
pub struct InputValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for InputValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for InputValidationError {}

/// Trait for validating request inputs
pub trait ValidateInput {
    fn validate_input(&self) -> Result<(), InputValidationError>;
}

// =============================================================================
// VALIDATION HELPER FUNCTIONS
// =============================================================================

/// Helper to validate string length
pub fn validate_string_length(
    value: &str,
    field: &str,
    max_length: usize,
) -> Result<(), InputValidationError> {
    if value.len() > max_length {
        return Err(InputValidationError {
            field: field.to_string(),
            message: format!(
                "exceeds maximum length of {} characters (got {})",
                max_length,
                value.len()
            ),
        });
    }
    Ok(())
}

/// Helper to validate optional string length
pub fn validate_optional_string_length(
    value: &Option<String>,
    field: &str,
    max_length: usize,
) -> Result<(), InputValidationError> {
    if let Some(s) = value {
        validate_string_length(s, field, max_length)?;
    }
    Ok(())
}

/// Helper to validate array length
pub fn validate_array_length<T>(
    value: &[T],
    field: &str,
    max_length: usize,
) -> Result<(), InputValidationError> {
    if value.len() > max_length {
        return Err(InputValidationError {
            field: field.to_string(),
            message: format!(
                "exceeds maximum count of {} items (got {})",
                max_length,
                value.len()
            ),
        });
    }
    Ok(())
}

/// Helper to validate required string (non-empty)
pub fn validate_required_string(value: &str, field: &str) -> Result<(), InputValidationError> {
    if value.is_empty() {
        return Err(InputValidationError {
            field: field.to_string(),
            message: "cannot be empty".to_string(),
        });
    }
    Ok(())
}

/// Helper to validate array is non-empty
pub fn validate_array_not_empty<T>(value: &[T], field: &str) -> Result<(), InputValidationError> {
    if value.is_empty() {
        return Err(InputValidationError {
            field: field.to_string(),
            message: "cannot be empty".to_string(),
        });
    }
    Ok(())
}

// =============================================================================
// CONVENIENCE FUNCTIONS WITH DEFAULT LIMITS
// =============================================================================

/// Validate ID field (max MAX_ID_LENGTH chars)
pub fn validate_id(value: &str, field: &str) -> Result<(), InputValidationError> {
    validate_string_length(value, field, MAX_ID_LENGTH)
}

/// Validate name field (max MAX_NAME_LENGTH chars)
pub fn validate_name(value: &str, field: &str) -> Result<(), InputValidationError> {
    validate_string_length(value, field, MAX_NAME_LENGTH)
}

/// Validate description field (max MAX_DESCRIPTION_LENGTH chars)
pub fn validate_description(value: &str, field: &str) -> Result<(), InputValidationError> {
    validate_string_length(value, field, MAX_DESCRIPTION_LENGTH)
}

/// Validate path field (max MAX_PATH_LENGTH chars)
pub fn validate_path(value: &str, field: &str) -> Result<(), InputValidationError> {
    validate_string_length(value, field, MAX_PATH_LENGTH)
}

/// Validate expression field (max MAX_EXPRESSION_LENGTH chars)
pub fn validate_expression(value: &str, field: &str) -> Result<(), InputValidationError> {
    validate_string_length(value, field, MAX_EXPRESSION_LENGTH)
}

/// Validate tag field (max MAX_TAG_LENGTH chars)
pub fn validate_tag(value: &str, field: &str) -> Result<(), InputValidationError> {
    validate_string_length(value, field, MAX_TAG_LENGTH)
}

/// Validate enum value field (max MAX_ENUM_VALUE_LENGTH chars)
pub fn validate_enum_value(value: &str, field: &str) -> Result<(), InputValidationError> {
    validate_string_length(value, field, MAX_ENUM_VALUE_LENGTH)
}

/// Validate array with default max items (MAX_ARRAY_ITEMS)
pub fn validate_array<T>(value: &[T], field: &str) -> Result<(), InputValidationError> {
    validate_array_length(value, field, MAX_ARRAY_ITEMS)
}
