//! Input validation for REST API
//!
//! Centralizes validation logic following the Kotlin service pattern.
//! Uses core library validation where available.

use product_farm_core::validation;

use super::constants::*;
use super::error::{codes, ApiError, ApiResult, ErrorDetail};
use super::types::DatatypeConstraintsJson;

// =============================================================================
// VALIDATION SERVICE
// =============================================================================

/// Validates input data for REST API requests.
/// Centralizes validation logic to ensure consistency.
pub struct Validator;

impl Validator {
    /// Validate a product ID
    pub fn validate_product_id(id: &str) -> ApiResult<()> {
        if id.is_empty() {
            return Err(ApiError::BadRequest("Product ID is required".to_string()));
        }

        if id.len() > MAX_PRODUCT_ID_LENGTH {
            return Err(ApiError::BadRequest(format!(
                "Product ID exceeds maximum length of {} characters",
                MAX_PRODUCT_ID_LENGTH
            )));
        }

        if !validation::is_valid_product_id(id) {
            return Err(ApiError::BadRequest(format!(
                "Invalid product ID '{}'. Must be alphanumeric with hyphens.",
                id
            )));
        }

        Ok(())
    }

    /// Validate a name field (product name, functionality name, etc.)
    pub fn validate_name(name: &str, field: &str) -> ApiResult<()> {
        if name.is_empty() {
            return Err(ApiError::BadRequest(format!("{} is required", field)));
        }

        if name.len() > MAX_NAME_LENGTH {
            return Err(ApiError::BadRequest(format!(
                "{} exceeds maximum length of {} characters",
                field, MAX_NAME_LENGTH
            )));
        }

        Ok(())
    }

    /// Validate a description field
    pub fn validate_description(description: Option<&str>) -> ApiResult<()> {
        if let Some(desc) = description {
            if desc.len() > MAX_DESCRIPTION_LENGTH {
                return Err(ApiError::BadRequest(format!(
                    "Description exceeds maximum length of {} characters",
                    MAX_DESCRIPTION_LENGTH
                )));
            }
        }
        Ok(())
    }

    /// Validate datatype constraints
    pub fn validate_datatype_constraints(
        constraints: Option<&DatatypeConstraintsJson>,
    ) -> ApiResult<()> {
        let Some(c) = constraints else {
            return Ok(());
        };

        let mut errors = Vec::new();

        // Validate min <= max
        if let (Some(min), Some(max)) = (c.min, c.max) {
            if min > max {
                errors.push(ErrorDetail::for_field(
                    codes::INVALID_VALUE,
                    "Constraint 'min' cannot be greater than 'max'",
                    "constraints.min",
                ));
            }
        }

        // Validate min_length <= max_length
        if let (Some(min_len), Some(max_len)) = (c.min_length, c.max_length) {
            if min_len > max_len {
                errors.push(ErrorDetail::for_field(
                    codes::INVALID_VALUE,
                    "Constraint 'minLength' cannot be greater than 'maxLength'",
                    "constraints.minLength",
                ));
            }
        }

        // Validate precision bounds
        if let Some(p) = c.precision {
            if p < 0 || p > MAX_PRECISION as i32 {
                errors.push(ErrorDetail::for_field(
                    codes::OUT_OF_RANGE,
                    format!("Precision must be between 0 and {}", MAX_PRECISION),
                    "constraints.precision",
                ));
            }
        }

        // Validate scale bounds
        if let Some(s) = c.scale {
            if s < 0 || s > MAX_SCALE as i32 {
                errors.push(ErrorDetail::for_field(
                    codes::OUT_OF_RANGE,
                    format!("Scale must be between 0 and {}", MAX_SCALE),
                    "constraints.scale",
                ));
            }
        }

        // Validate scale <= precision
        if let (Some(p), Some(s)) = (c.precision, c.scale) {
            if s > p {
                errors.push(ErrorDetail::for_field(
                    codes::INVALID_VALUE,
                    "Scale cannot be greater than precision",
                    "constraints.scale",
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ApiError::ValidationFailed(errors))
        }
    }

    /// Validate an abstract attribute path
    pub fn validate_abstract_path(path: &str) -> ApiResult<()> {
        if path.is_empty() {
            return Err(ApiError::BadRequest(
                "Abstract path is required".to_string(),
            ));
        }

        // Abstract path format: {productId}:{componentType}:{attributeName}
        let parts: Vec<&str> = path.split(':').collect();
        if parts.len() < 3 {
            return Err(ApiError::BadRequest(format!(
                "Invalid abstract path '{}'. Expected format: productId:componentType:attributeName",
                path
            )));
        }

        Ok(())
    }

    /// Validate a concrete attribute path
    pub fn validate_concrete_path(path: &str) -> ApiResult<()> {
        if path.is_empty() {
            return Err(ApiError::BadRequest("Path is required".to_string()));
        }

        // Concrete path format: {productId}:{componentType}:{componentId}:{attributeName}
        let parts: Vec<&str> = path.split(':').collect();
        if parts.len() < 4 {
            return Err(ApiError::BadRequest(format!(
                "Invalid path '{}'. Expected format: productId:componentType:componentId:attributeName",
                path
            )));
        }

        Ok(())
    }

    /// Validate display names array
    pub fn validate_display_names<T>(display_names: &[T]) -> ApiResult<()> {
        if display_names.len() > MAX_DISPLAY_NAMES {
            return Err(ApiError::BadRequest(format!(
                "Too many display names. Maximum is {}",
                MAX_DISPLAY_NAMES
            )));
        }
        Ok(())
    }

    /// Validate tags array
    pub fn validate_tags(tags: &[String]) -> ApiResult<()> {
        if tags.len() > MAX_TAGS {
            return Err(ApiError::BadRequest(format!(
                "Too many tags. Maximum is {}",
                MAX_TAGS
            )));
        }

        for tag in tags {
            if tag.len() > MAX_TAG_LENGTH {
                return Err(ApiError::BadRequest(format!(
                    "Tag '{}' exceeds maximum length of {} characters",
                    tag, MAX_TAG_LENGTH
                )));
            }
        }

        Ok(())
    }

    /// Validate enumeration values
    pub fn validate_enumeration_values(values: &[String]) -> ApiResult<()> {
        if values.is_empty() {
            return Err(ApiError::BadRequest(
                "Enumeration must have at least one value".to_string(),
            ));
        }

        if values.len() > MAX_ENUMERATION_VALUES {
            return Err(ApiError::BadRequest(format!(
                "Too many enumeration values. Maximum is {}",
                MAX_ENUMERATION_VALUES
            )));
        }

        Ok(())
    }

    /// Validate a rule expression
    pub fn validate_expression(expression: &str) -> ApiResult<()> {
        if expression.is_empty() {
            return Err(ApiError::BadRequest(
                "Rule expression is required".to_string(),
            ));
        }

        if expression.len() > MAX_EXPRESSION_LENGTH {
            return Err(ApiError::BadRequest(format!(
                "Rule expression exceeds maximum length of {} characters",
                MAX_EXPRESSION_LENGTH
            )));
        }

        Ok(())
    }
}

// =============================================================================
// REQUEST VALIDATION
// =============================================================================

/// Validate create product request
pub fn validate_create_product(id: &str, name: &str, description: Option<&str>) -> ApiResult<()> {
    Validator::validate_product_id(id)?;
    Validator::validate_name(name, "Product name")?;
    Validator::validate_description(description)?;
    Ok(())
}

/// Validate create datatype request
pub fn validate_create_datatype(
    id: &str,
    constraints: Option<&DatatypeConstraintsJson>,
) -> ApiResult<()> {
    Validator::validate_name(id, "Datatype ID")?;
    Validator::validate_datatype_constraints(constraints)?;
    Ok(())
}

/// Validate create functionality request
pub fn validate_create_functionality(name: &str, description: Option<&str>) -> ApiResult<()> {
    Validator::validate_name(name, "Functionality name")?;
    Validator::validate_description(description)?;
    Ok(())
}
