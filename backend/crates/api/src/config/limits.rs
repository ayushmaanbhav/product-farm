//! Centralized size and length limits for the API
//!
//! These limits serve two purposes:
//! 1. **DoS Prevention**: Request size limits to prevent resource exhaustion
//! 2. **Business Constraints**: Field length limits for data integrity
//!
//! # Usage
//!
//! ```rust
//! use product_farm_api::config::limits::{MAX_NAME_LENGTH, MAX_ID_LENGTH};
//!
//! fn validate_name(name: &str) -> bool {
//!     name.len() <= MAX_NAME_LENGTH
//! }
//! ```

// =============================================================================
// STRING LENGTH LIMITS
// =============================================================================

/// Maximum length for entity IDs (products, datatypes, rules, etc.)
///
/// IDs should be short, readable identifiers. 128 chars allows for
/// namespaced IDs like "org-name/product-name/v1.2.3"
pub const MAX_ID_LENGTH: usize = 128;

/// Maximum length for entity names
///
/// Names are human-readable labels. 256 chars is sufficient for
/// descriptive names while preventing abuse.
pub const MAX_NAME_LENGTH: usize = 256;

/// Maximum length for descriptions
///
/// Descriptions can be longer than names. 4KB allows for detailed
/// explanations without enabling document storage abuse.
pub const MAX_DESCRIPTION_LENGTH: usize = 4096;

/// Maximum length for display names
///
/// Display names are shown in UI. 200 chars fits most UI layouts.
pub const MAX_DISPLAY_NAME_LENGTH: usize = 200;

/// Maximum length for tags
///
/// Tags are short categorization labels. 64 chars is generous for tags.
pub const MAX_TAG_LENGTH: usize = 64;

/// Maximum length for enumeration values
///
/// Enum values are typically short identifiers or labels.
pub const MAX_ENUM_VALUE_LENGTH: usize = 256;

/// Maximum length for expressions (JSON Logic)
///
/// Expressions can be complex. 64KB allows for sophisticated rules
/// while preventing resource exhaustion from huge expressions.
pub const MAX_EXPRESSION_LENGTH: usize = 65536; // 64KB

/// Maximum length for path fields
///
/// Paths like "product:abstract-path:component:id:attribute" are structured.
/// 512 chars handles deeply nested paths.
pub const MAX_PATH_LENGTH: usize = 512;

/// Maximum length for component types and IDs
///
/// Component identifiers should be concise. 64 chars is sufficient.
pub const MAX_COMPONENT_LENGTH: usize = 64;

/// Maximum length for attribute names
///
/// Attribute names should be concise identifiers.
pub const MAX_ATTRIBUTE_NAME_LENGTH: usize = 64;

/// Maximum length for template types
///
/// Template types like "loan", "insurance", etc.
pub const MAX_TEMPLATE_TYPE_LENGTH: usize = 64;

/// Maximum length for regex patterns (ReDoS prevention)
///
/// Long regex patterns can cause exponential backtracking (ReDoS).
/// 1KB is generous for validation patterns.
pub const MAX_PATTERN_LENGTH: usize = 1024;

// =============================================================================
// ARRAY SIZE LIMITS
// =============================================================================

/// Maximum number of items in any array (DoS prevention)
///
/// Prevents memory exhaustion from huge arrays in requests.
pub const MAX_ARRAY_ITEMS: usize = 1000;

/// Maximum display names per attribute
///
/// Attributes can have locale-specific display names.
pub const MAX_DISPLAY_NAMES: usize = 10;

/// Maximum tags per attribute
///
/// Tags for categorization and filtering.
pub const MAX_TAGS: usize = 20;

/// Maximum related attributes per attribute
///
/// Cross-references between attributes.
pub const MAX_RELATED_ATTRIBUTES: usize = 50;

/// Maximum input/output attributes per rule
///
/// Complex rules may reference many attributes.
pub const MAX_RULE_ATTRIBUTES: usize = 100;

/// Maximum required attributes per functionality
///
/// Functionalities define attribute requirements.
pub const MAX_REQUIRED_ATTRIBUTES: usize = 100;

/// Maximum values per enumeration
///
/// Enumerations define discrete value sets.
pub const MAX_ENUMERATION_VALUES: usize = 1000;

// =============================================================================
// NUMERIC LIMITS
// =============================================================================

/// Maximum precision for DECIMAL types
///
/// Standard SQL DECIMAL precision limit (38 digits).
pub const MAX_PRECISION: u8 = 38;

/// Maximum scale for DECIMAL types
///
/// Scale cannot exceed precision.
pub const MAX_SCALE: u8 = 38;

// =============================================================================
// PAGINATION LIMITS
// =============================================================================

/// Default page size for list operations
///
/// Reasonable default for most list views.
pub const DEFAULT_PAGE_SIZE: usize = 20;

/// Maximum page size for list operations
///
/// Prevents fetching too much data in one request.
pub const MAX_PAGE_SIZE: usize = 100;

// =============================================================================
// REGEX SIZE LIMITS (ReDoS Prevention)
// =============================================================================

/// Maximum compiled regex size (10MB)
///
/// Prevents memory exhaustion from pathological regex patterns.
/// The regex crate enforces this limit during compilation.
pub const MAX_REGEX_SIZE: usize = 10 * 1024 * 1024;

// =============================================================================
// KEY FORMAT PATTERNS
// =============================================================================

/// Separator for composite keys (e.g., functionality key: "{product_id}:{name}")
pub const KEY_SEPARATOR: char = ':';

// =============================================================================
// HTTP HEADERS
// =============================================================================

/// Request ID header name for distributed tracing
pub const HEADER_REQUEST_ID: &str = "X-Request-Id";

/// Correlation ID header name for request chain tracking
pub const HEADER_CORRELATION_ID: &str = "X-Correlation-Id";
