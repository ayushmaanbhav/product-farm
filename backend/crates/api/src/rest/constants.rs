//! Constants for REST API
//!
//! This module re-exports constants from the centralized `config` module
//! for backward compatibility. New code should import directly from `crate::config`.

// Re-export all limits from config
pub use crate::config::limits::{
    DEFAULT_PAGE_SIZE, HEADER_CORRELATION_ID, HEADER_REQUEST_ID, KEY_SEPARATOR, MAX_DESCRIPTION_LENGTH,
    MAX_DISPLAY_NAME_LENGTH, MAX_DISPLAY_NAMES, MAX_ENUMERATION_VALUES, MAX_EXPRESSION_LENGTH,
    MAX_NAME_LENGTH, MAX_PRECISION, MAX_RELATED_ATTRIBUTES, MAX_REQUIRED_ATTRIBUTES,
    MAX_RULE_ATTRIBUTES, MAX_SCALE, MAX_TAG_LENGTH, MAX_TAGS, MAX_PAGE_SIZE,
};

// Re-export cache settings from config::server
pub use crate::config::server::DEFAULT_CACHE_TTL_SECS;

// Re-export storage config
pub use crate::config::storage::DEFAULT_CACHE_SIZE as MAX_CACHE_ENTRIES;

// Legacy aliases for backward compatibility
// (These were the old constant names - now unified)
pub use crate::config::limits::MAX_ID_LENGTH as MAX_PRODUCT_ID_LENGTH;
pub use crate::config::limits::KEY_SEPARATOR as FUNCTIONALITY_KEY_SEP;
pub use crate::config::limits::KEY_SEPARATOR as ENUMERATION_KEY_SEP;
