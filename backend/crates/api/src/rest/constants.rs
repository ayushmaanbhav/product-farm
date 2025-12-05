//! Constants for REST API
//!
//! Centralized constants to avoid magic values throughout the codebase.

// =============================================================================
// PAGINATION
// =============================================================================

/// Default page size for list operations
pub const DEFAULT_PAGE_SIZE: usize = 20;

/// Maximum allowed page size
pub const MAX_PAGE_SIZE: usize = 100;

// =============================================================================
// STRING LENGTH LIMITS
// =============================================================================

/// Maximum length for product ID
pub const MAX_PRODUCT_ID_LENGTH: usize = 50;

/// Maximum length for entity names (product, functionality, datatype, etc.)
pub const MAX_NAME_LENGTH: usize = 100;

/// Maximum length for descriptions
pub const MAX_DESCRIPTION_LENGTH: usize = 500;

/// Maximum length for display names
pub const MAX_DISPLAY_NAME_LENGTH: usize = 200;

/// Maximum length for tags
pub const MAX_TAG_LENGTH: usize = 50;

/// Maximum length for rule expressions
pub const MAX_EXPRESSION_LENGTH: usize = 10_000;

// =============================================================================
// ARRAY SIZE LIMITS
// =============================================================================

/// Maximum number of display names per attribute
pub const MAX_DISPLAY_NAMES: usize = 10;

/// Maximum number of tags per attribute
pub const MAX_TAGS: usize = 20;

/// Maximum number of related attributes
pub const MAX_RELATED_ATTRIBUTES: usize = 50;

/// Maximum number of input/output attributes per rule
pub const MAX_RULE_ATTRIBUTES: usize = 100;

/// Maximum number of required attributes per functionality
pub const MAX_REQUIRED_ATTRIBUTES: usize = 100;

/// Maximum number of values in enumeration
pub const MAX_ENUMERATION_VALUES: usize = 1000;

// =============================================================================
// NUMERIC LIMITS
// =============================================================================

/// Maximum precision for decimal types
pub const MAX_PRECISION: u8 = 38;

/// Maximum scale for decimal types
pub const MAX_SCALE: u8 = 38;

// =============================================================================
// KEY FORMAT PATTERNS
// =============================================================================

/// Format for functionality store key: "{product_id}:{name}"
pub const FUNCTIONALITY_KEY_SEP: char = ':';

/// Format for enumeration store key: "{template_type}:{name}"
pub const ENUMERATION_KEY_SEP: char = ':';

// =============================================================================
// HTTP HEADERS
// =============================================================================

/// Request ID header name
pub const HEADER_REQUEST_ID: &str = "X-Request-Id";

/// Correlation ID header name
pub const HEADER_CORRELATION_ID: &str = "X-Correlation-Id";

// =============================================================================
// CACHE SETTINGS
// =============================================================================

/// Default cache TTL in seconds
pub const DEFAULT_CACHE_TTL_SECS: u64 = 300; // 5 minutes

/// Maximum cache entries
pub const MAX_CACHE_ENTRIES: usize = 10_000;
