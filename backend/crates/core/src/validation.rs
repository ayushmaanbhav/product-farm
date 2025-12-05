//! Validation patterns and constants
//!
//! This module defines all validation regex patterns matching the legacy Kotlin implementation.
//! All patterns enforce strict naming conventions for IDs, paths, and other identifiers.

use once_cell::sync::Lazy;
use regex::Regex;

/// Component separator used in paths (legacy format)
pub const COMPONENT_SEPARATOR: char = ':';

/// Human-readable format component separator
pub const HUMAN_FORMAT_COMPONENT_SEPARATOR: char = '.';

/// Original format component separator
pub const ORIGINAL_FORMAT_COMPONENT_SEPARATOR: char = '.';

/// Attribute name separator
pub const ATTRIBUTE_NAME_SEPARATOR: char = '.';

/// Abstract path marker
pub const ABSTRACT_PATH_NAME: &str = "abstract-path";

// =============================================================================
// REGEX PATTERNS
// =============================================================================

/// Product ID regex pattern string
/// Format: Starts with letter, followed by alphanumeric or underscore-letter/digit combinations
/// Max length: 51 characters
pub const PRODUCT_ID_PATTERN: &str = r"^[a-zA-Z]([_][a-zA-Z0-9]|[a-zA-Z0-9]){0,50}$";

/// Product name regex pattern string
/// Format: Letters, numbers, spaces, common punctuation
/// Max length: 50 characters
pub const PRODUCT_NAME_PATTERN: &str = r"^[a-zA-Z0-9,.\-_:' ]{0,50}$";

/// Component type regex pattern string
/// Format: lowercase letters, hyphens allowed (not consecutive)
/// Max length: 51 characters
pub const COMPONENT_TYPE_PATTERN: &str = r"^[a-z]([-][a-z]|[a-z]){0,50}$";

/// Component ID regex pattern string
/// Format: lowercase letters/numbers, hyphens allowed (not consecutive)
/// Max length: 51 characters
pub const COMPONENT_ID_PATTERN: &str = r"^[a-z]([-][a-z0-9]|[a-z0-9]){0,50}$";

/// Attribute name regex pattern string
/// Format: lowercase letters, dots and hyphens allowed
/// Max length: 101 characters
pub const ATTRIBUTE_NAME_PATTERN: &str = r"^[a-z]([.][a-z]|[-][a-z0-9]|[a-z0-9]){0,100}$";

/// Display name regex pattern string
/// Format: lowercase letters, dots and hyphens allowed
/// Max length: 201 characters
pub const DISPLAY_NAME_PATTERN: &str = r"^[a-z]([.][a-z]|[-][a-z0-9]|[a-z0-9]){0,200}$";

/// Original attribute name regex pattern (mixed case)
pub const ORIGINAL_ATTRIBUTE_NAME_PATTERN: &str = r"^[a-zA-Z]([.][a-zA-Z]|[a-zA-Z0-9]){0,100}$";

/// Original component type regex pattern (mixed case with underscores)
pub const ORIGINAL_COMPONENT_TYPE_PATTERN: &str = r"^[a-zA-Z]([_][a-zA-Z]|[a-zA-Z]){0,50}$";

/// Original component ID regex pattern (mixed case with various separators)
pub const ORIGINAL_COMPONENT_ID_PATTERN: &str = r"^[a-zA-Z]([-_()][a-zA-Z0-9]|[a-zA-Z0-9]){0,50}$";

/// Tag regex pattern string
/// Format: lowercase letters, hyphens allowed
/// Max length: 51 characters
pub const TAG_PATTERN: &str = r"^[a-z]([-][a-z]|[a-z]){0,50}$";

/// Datatype regex pattern string
/// Format: lowercase letters, hyphens allowed
/// Max length: 51 characters
pub const DATATYPE_PATTERN: &str = r"^[a-z]([-][a-z]|[a-z]){0,50}$";

/// Functionality name regex pattern string
/// Format: lowercase letters, hyphens allowed
/// Max length: 51 characters
pub const FUNCTIONALITY_NAME_PATTERN: &str = r"^[a-z]([-][a-z]|[a-z]){0,50}$";

/// Enumeration name regex pattern string
/// Format: lowercase letters, hyphens allowed
/// Max length: 51 characters
pub const ENUMERATION_NAME_PATTERN: &str = r"^[a-z]([-][a-z]|[a-z]){0,50}$";

/// Enumeration value regex pattern string
/// Format: lowercase letters, hyphens allowed
/// Max length: 51 characters
pub const ENUMERATION_VALUE_PATTERN: &str = r"^[a-z]([-][a-z]|[a-z]){0,50}$";

/// Description regex pattern string
/// Format: alphanumeric and common punctuation/whitespace
/// Max length: 200 characters
pub const DESCRIPTION_PATTERN: &str = r#"^[a-zA-Z0-9,.<>/?*()&#;\-_=+:'"!\[\]{}\s]{0,200}$"#;

/// Relationship name regex pattern string
/// Format: lowercase letters, hyphens allowed
/// Max length: 51 characters
pub const RELATIONSHIP_NAME_PATTERN: &str = r"^[a-z]([-][a-z]|[a-z]){0,50}$";

/// Rule type regex pattern string (same as functionality name)
pub const RULE_TYPE_PATTERN: &str = r"^[a-z]([-][a-z]|[a-z]){0,50}$";

/// UUID regex pattern (lowercase hex, no hyphens)
pub const UUID_PATTERN: &str = r"^[a-f0-9]{32}$";

// =============================================================================
// COMPILED REGEX INSTANCES
// =============================================================================

/// Compiled product ID regex
pub static PRODUCT_ID_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(PRODUCT_ID_PATTERN).expect("Invalid PRODUCT_ID_PATTERN"));

/// Compiled product name regex
pub static PRODUCT_NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(PRODUCT_NAME_PATTERN).expect("Invalid PRODUCT_NAME_PATTERN"));

/// Compiled component type regex
pub static COMPONENT_TYPE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(COMPONENT_TYPE_PATTERN).expect("Invalid COMPONENT_TYPE_PATTERN"));

/// Compiled component ID regex
pub static COMPONENT_ID_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(COMPONENT_ID_PATTERN).expect("Invalid COMPONENT_ID_PATTERN"));

/// Compiled attribute name regex
pub static ATTRIBUTE_NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(ATTRIBUTE_NAME_PATTERN).expect("Invalid ATTRIBUTE_NAME_PATTERN"));

/// Compiled display name regex
pub static DISPLAY_NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(DISPLAY_NAME_PATTERN).expect("Invalid DISPLAY_NAME_PATTERN"));

/// Compiled tag regex
pub static TAG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(TAG_PATTERN).expect("Invalid TAG_PATTERN"));

/// Compiled datatype regex
pub static DATATYPE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(DATATYPE_PATTERN).expect("Invalid DATATYPE_PATTERN"));

/// Compiled functionality name regex
pub static FUNCTIONALITY_NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(FUNCTIONALITY_NAME_PATTERN).expect("Invalid FUNCTIONALITY_NAME_PATTERN"));

/// Compiled enumeration name regex
pub static ENUMERATION_NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(ENUMERATION_NAME_PATTERN).expect("Invalid ENUMERATION_NAME_PATTERN"));

/// Compiled enumeration value regex
pub static ENUMERATION_VALUE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(ENUMERATION_VALUE_PATTERN).expect("Invalid ENUMERATION_VALUE_PATTERN"));

/// Compiled description regex
pub static DESCRIPTION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(DESCRIPTION_PATTERN).expect("Invalid DESCRIPTION_PATTERN"));

/// Compiled rule type regex
pub static RULE_TYPE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(RULE_TYPE_PATTERN).expect("Invalid RULE_TYPE_PATTERN"));

// =============================================================================
// VALIDATION FUNCTIONS
// =============================================================================

/// Validate a product ID
pub fn is_valid_product_id(id: &str) -> bool {
    PRODUCT_ID_REGEX.is_match(id)
}

/// Validate a product name
pub fn is_valid_product_name(name: &str) -> bool {
    PRODUCT_NAME_REGEX.is_match(name)
}

/// Validate a component type
pub fn is_valid_component_type(ct: &str) -> bool {
    COMPONENT_TYPE_REGEX.is_match(ct)
}

/// Validate a component ID
pub fn is_valid_component_id(id: &str) -> bool {
    COMPONENT_ID_REGEX.is_match(id)
}

/// Validate an attribute name
pub fn is_valid_attribute_name(name: &str) -> bool {
    ATTRIBUTE_NAME_REGEX.is_match(name)
}

/// Validate a display name
pub fn is_valid_display_name(name: &str) -> bool {
    DISPLAY_NAME_REGEX.is_match(name)
}

/// Validate a tag
pub fn is_valid_tag(tag: &str) -> bool {
    TAG_REGEX.is_match(tag)
}

/// Validate a datatype name
pub fn is_valid_datatype(name: &str) -> bool {
    DATATYPE_REGEX.is_match(name)
}

/// Validate a functionality name
pub fn is_valid_functionality_name(name: &str) -> bool {
    FUNCTIONALITY_NAME_REGEX.is_match(name)
}

/// Validate an enumeration name
pub fn is_valid_enumeration_name(name: &str) -> bool {
    ENUMERATION_NAME_REGEX.is_match(name)
}

/// Validate an enumeration value
pub fn is_valid_enumeration_value(value: &str) -> bool {
    ENUMERATION_VALUE_REGEX.is_match(value)
}

/// Validate a description
pub fn is_valid_description(desc: &str) -> bool {
    DESCRIPTION_REGEX.is_match(desc)
}

/// Validate a rule type
pub fn is_valid_rule_type(rule_type: &str) -> bool {
    RULE_TYPE_REGEX.is_match(rule_type)
}

// =============================================================================
// PATH VALIDATION
// =============================================================================

/// Validate a full path (productId:componentType:componentId:attributeName)
pub fn is_valid_path(path: &str) -> bool {
    let parts: Vec<&str> = path.split(COMPONENT_SEPARATOR).collect();
    if parts.len() != 4 {
        return false;
    }

    is_valid_product_id(parts[0])
        && is_valid_component_type(parts[1])
        && is_valid_component_id(parts[2])
        && is_valid_attribute_name(parts[3])
}

/// Validate an abstract path (productId:abstract-path:componentType[:componentId]:attributeName)
pub fn is_valid_abstract_path(path: &str) -> bool {
    let parts: Vec<&str> = path.split(COMPONENT_SEPARATOR).collect();

    // Must have at least 4 parts: productId:abstract-path:componentType:attributeName
    // Or 5 parts: productId:abstract-path:componentType:componentId:attributeName
    if parts.len() < 4 || parts.len() > 5 {
        return false;
    }

    // Second part must be "abstract-path"
    if parts[1] != ABSTRACT_PATH_NAME {
        return false;
    }

    if parts.len() == 4 {
        // Without componentId
        is_valid_product_id(parts[0])
            && is_valid_component_type(parts[2])
            && is_valid_attribute_name(parts[3])
    } else {
        // With componentId
        is_valid_product_id(parts[0])
            && is_valid_component_type(parts[2])
            && is_valid_component_id(parts[3])
            && is_valid_attribute_name(parts[4])
    }
}

/// Parse a concrete path into its components
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPath {
    pub product_id: String,
    pub component_type: String,
    pub component_id: String,
    pub attribute_name: String,
}

impl ParsedPath {
    /// Parse a path string into components
    pub fn parse(path: &str) -> Option<Self> {
        let parts: Vec<&str> = path.split(COMPONENT_SEPARATOR).collect();
        if parts.len() != 4 {
            return None;
        }

        Some(Self {
            product_id: parts[0].to_string(),
            component_type: parts[1].to_string(),
            component_id: parts[2].to_string(),
            attribute_name: parts[3].to_string(),
        })
    }

    /// Format back to path string
    pub fn to_path(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}",
            self.product_id,
            COMPONENT_SEPARATOR,
            self.component_type,
            COMPONENT_SEPARATOR,
            self.component_id,
            COMPONENT_SEPARATOR,
            self.attribute_name
        )
    }
}

/// Parse an abstract path into its components
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAbstractPath {
    pub product_id: String,
    pub component_type: String,
    pub component_id: Option<String>,
    pub attribute_name: String,
}

impl ParsedAbstractPath {
    /// Parse an abstract path string into components
    pub fn parse(path: &str) -> Option<Self> {
        let parts: Vec<&str> = path.split(COMPONENT_SEPARATOR).collect();

        if parts.len() < 4 || parts.len() > 5 || parts[1] != ABSTRACT_PATH_NAME {
            return None;
        }

        if parts.len() == 4 {
            Some(Self {
                product_id: parts[0].to_string(),
                component_type: parts[2].to_string(),
                component_id: None,
                attribute_name: parts[3].to_string(),
            })
        } else {
            Some(Self {
                product_id: parts[0].to_string(),
                component_type: parts[2].to_string(),
                component_id: Some(parts[3].to_string()),
                attribute_name: parts[4].to_string(),
            })
        }
    }

    /// Format back to abstract path string
    pub fn to_path(&self) -> String {
        match &self.component_id {
            Some(cid) => format!(
                "{}{}{}{}{}{}{}{}{}",
                self.product_id,
                COMPONENT_SEPARATOR,
                ABSTRACT_PATH_NAME,
                COMPONENT_SEPARATOR,
                self.component_type,
                COMPONENT_SEPARATOR,
                cid,
                COMPONENT_SEPARATOR,
                self.attribute_name
            ),
            None => format!(
                "{}{}{}{}{}{}{}",
                self.product_id,
                COMPONENT_SEPARATOR,
                ABSTRACT_PATH_NAME,
                COMPONENT_SEPARATOR,
                self.component_type,
                COMPONENT_SEPARATOR,
                self.attribute_name
            ),
        }
    }

    /// Convert an abstract path to a concrete path with a specific component ID
    pub fn to_concrete_path(&self, component_id: &str) -> ParsedPath {
        ParsedPath {
            product_id: self.product_id.clone(),
            component_type: self.component_type.clone(),
            component_id: self.component_id.clone().unwrap_or_else(|| component_id.to_string()),
            attribute_name: self.attribute_name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_id_validation() {
        // Valid
        assert!(is_valid_product_id("myProduct"));
        assert!(is_valid_product_id("product123"));
        assert!(is_valid_product_id("my_product"));
        assert!(is_valid_product_id("A"));
        assert!(is_valid_product_id("Product_2024_v1"));

        // Invalid
        assert!(!is_valid_product_id("")); // Empty
        assert!(!is_valid_product_id("123product")); // Starts with number
        assert!(!is_valid_product_id("_product")); // Starts with underscore
        assert!(!is_valid_product_id("product-name")); // Contains hyphen (not allowed for product ID)
    }

    #[test]
    fn test_component_type_validation() {
        // Valid
        assert!(is_valid_component_type("cover"));
        assert!(is_valid_component_type("market-data"));
        assert!(is_valid_component_type("a"));

        // Invalid
        assert!(!is_valid_component_type("")); // Empty
        assert!(!is_valid_component_type("Cover")); // Uppercase
        assert!(!is_valid_component_type("123type")); // Starts with number
        assert!(!is_valid_component_type("cover_type")); // Underscore not allowed
    }

    #[test]
    fn test_path_validation() {
        // Valid paths
        assert!(is_valid_path("myProduct:cover:basic:premium"));
        assert!(is_valid_path("insuranceV1:market-data:source1:current-price"));

        // Invalid paths
        assert!(!is_valid_path("myProduct:cover:basic")); // Missing attribute
        assert!(!is_valid_path("myProduct.cover.basic.premium")); // Wrong separator
        assert!(!is_valid_path("123:cover:basic:premium")); // Invalid product ID
    }

    #[test]
    fn test_abstract_path_validation() {
        // Valid without component ID
        assert!(is_valid_abstract_path("myProduct:abstract-path:cover:premium"));

        // Valid with component ID
        assert!(is_valid_abstract_path("myProduct:abstract-path:cover:basic:premium"));

        // Invalid
        assert!(!is_valid_abstract_path("myProduct:cover:premium")); // Missing abstract-path marker
        assert!(!is_valid_abstract_path("myProduct:wrong-marker:cover:premium")); // Wrong marker
    }

    #[test]
    fn test_parsed_path() {
        let path = "myProduct:cover:basic:premium";
        let parsed = ParsedPath::parse(path).unwrap();

        assert_eq!(parsed.product_id, "myProduct");
        assert_eq!(parsed.component_type, "cover");
        assert_eq!(parsed.component_id, "basic");
        assert_eq!(parsed.attribute_name, "premium");
        assert_eq!(parsed.to_path(), path);
    }

    #[test]
    fn test_parsed_abstract_path() {
        // Without component ID
        let path = "myProduct:abstract-path:cover:premium";
        let parsed = ParsedAbstractPath::parse(path).unwrap();

        assert_eq!(parsed.product_id, "myProduct");
        assert_eq!(parsed.component_type, "cover");
        assert_eq!(parsed.component_id, None);
        assert_eq!(parsed.attribute_name, "premium");
        assert_eq!(parsed.to_path(), path);

        // With component ID
        let path = "myProduct:abstract-path:cover:basic:premium";
        let parsed = ParsedAbstractPath::parse(path).unwrap();

        assert_eq!(parsed.product_id, "myProduct");
        assert_eq!(parsed.component_type, "cover");
        assert_eq!(parsed.component_id, Some("basic".to_string()));
        assert_eq!(parsed.attribute_name, "premium");
        assert_eq!(parsed.to_path(), path);
    }

    #[test]
    fn test_tag_validation() {
        assert!(is_valid_tag("input"));
        assert!(is_valid_tag("market-data"));
        assert!(!is_valid_tag("Input")); // Uppercase
        assert!(!is_valid_tag("input_data")); // Underscore
    }

    #[test]
    fn test_enumeration_validation() {
        assert!(is_valid_enumeration_name("signal-type"));
        assert!(is_valid_enumeration_value("buy"));
        assert!(!is_valid_enumeration_name("Signal_Type")); // Wrong case and underscore
    }
}
