//! Core type aliases and identifiers
//!
//! ## Path Formats (Legacy Compatible)
//!
//! - **Concrete Path**: `{productId}:{componentType}:{componentId}:{attributeName}`
//!   - Example: `insuranceV1:cover:basic:premium`
//!   - Component separator: `:` (colon)
//!
//! - **Abstract Path**: `{productId}:abstract-path:{componentType}[:{componentId}]:{attributeName}`
//!   - Example without component ID: `insuranceV1:abstract-path:cover:premium`
//!   - Example with component ID: `insuranceV1:abstract-path:cover:basic:premium`

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::validation::{self, COMPONENT_SEPARATOR, ABSTRACT_PATH_NAME};

/// Unique identifier for products
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductId(pub String);

impl ProductId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validate that this is a valid product ID
    pub fn is_valid(&self) -> bool {
        validation::is_valid_product_id(&self.0)
    }
}

impl From<String> for ProductId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ProductId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for ProductId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for attributes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttributeId(pub String);

impl AttributeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for AttributeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AttributeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Unique identifier for rules
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuleId(pub Uuid);

impl RuleId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Create from a string (parses as UUID or generates deterministic UUID from string)
    pub fn from_string(s: &str) -> Self {
        if let Ok(uuid) = Uuid::parse_str(s) {
            Self(uuid)
        } else {
            // Generate a deterministic UUID v5 from the string using DNS namespace
            Self(Uuid::new_v5(&Uuid::NAMESPACE_DNS, s.as_bytes()))
        }
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Get as hex string without hyphens (legacy format)
    pub fn to_hex_string(&self) -> String {
        self.0.as_simple().to_string()
    }
}

impl Default for RuleId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for functionalities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionalityId(pub String);

impl FunctionalityId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validate that this is a valid functionality name
    pub fn is_valid(&self) -> bool {
        validation::is_valid_functionality_name(&self.0)
    }
}

impl From<String> for FunctionalityId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for FunctionalityId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for FunctionalityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for data types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DataTypeId(pub String);

impl DataTypeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validate that this is a valid datatype name
    pub fn is_valid(&self) -> bool {
        validation::is_valid_datatype(&self.0)
    }
}

impl From<String> for DataTypeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for DataTypeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for DataTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Abstract path for attribute templates
///
/// Format: `{productId}:abstract-path:{componentType}[:{componentId}]:{attributeName}`
///
/// Examples:
/// - Without component ID: `insuranceV1:abstract-path:cover:premium`
/// - With component ID: `insuranceV1:abstract-path:cover:basic:premium`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AbstractPath(pub String);

impl AbstractPath {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Build an abstract path from components
    pub fn build(
        product_id: &str,
        component_type: &str,
        component_id: Option<&str>,
        attribute_name: &str,
    ) -> Self {
        let path = match component_id {
            Some(cid) => format!(
                "{}{}{}{}{}{}{}{}{}",
                product_id,
                COMPONENT_SEPARATOR,
                ABSTRACT_PATH_NAME,
                COMPONENT_SEPARATOR,
                component_type,
                COMPONENT_SEPARATOR,
                cid,
                COMPONENT_SEPARATOR,
                attribute_name
            ),
            None => format!(
                "{}{}{}{}{}{}{}",
                product_id,
                COMPONENT_SEPARATOR,
                ABSTRACT_PATH_NAME,
                COMPONENT_SEPARATOR,
                component_type,
                COMPONENT_SEPARATOR,
                attribute_name
            ),
        };
        Self(path)
    }

    /// Validate that this is a valid abstract path
    pub fn is_valid(&self) -> bool {
        validation::is_valid_abstract_path(&self.0)
    }

    /// Parse the abstract path into components
    pub fn parse(&self) -> Option<validation::ParsedAbstractPath> {
        validation::ParsedAbstractPath::parse(&self.0)
    }

    /// Get the product ID from the path
    pub fn product_id(&self) -> Option<String> {
        self.parse().map(|p| p.product_id)
    }

    /// Get the component type from the path
    pub fn component_type(&self) -> Option<String> {
        self.parse().map(|p| p.component_type)
    }
}

impl From<String> for AbstractPath {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AbstractPath {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for AbstractPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Concrete path for attribute instances
///
/// Format: `{productId}:{componentType}:{componentId}:{attributeName}`
///
/// Example: `insuranceV1:cover:basic:premium`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConcretePath(pub String);

impl ConcretePath {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Build a concrete path from components
    pub fn build(
        product_id: &str,
        component_type: &str,
        component_id: &str,
        attribute_name: &str,
    ) -> Self {
        Self(format!(
            "{}{}{}{}{}{}{}",
            product_id,
            COMPONENT_SEPARATOR,
            component_type,
            COMPONENT_SEPARATOR,
            component_id,
            COMPONENT_SEPARATOR,
            attribute_name
        ))
    }

    /// Validate that this is a valid concrete path
    pub fn is_valid(&self) -> bool {
        validation::is_valid_path(&self.0)
    }

    /// Parse the concrete path into components
    pub fn parse(&self) -> Option<ParsedConcretePath> {
        validation::ParsedPath::parse(&self.0).map(|p| ParsedConcretePath {
            product_id: p.product_id,
            component_type: p.component_type,
            component_id: p.component_id,
            attribute_name: p.attribute_name,
        })
    }

    /// Get the product ID from the path
    pub fn product_id(&self) -> Option<String> {
        self.parse().map(|p| p.product_id)
    }

    /// Get the component type from the path
    pub fn component_type(&self) -> Option<String> {
        self.parse().map(|p| p.component_type)
    }

    /// Get the component ID from the path
    pub fn component_id(&self) -> Option<String> {
        self.parse().map(|p| p.component_id)
    }
}

impl From<String> for ConcretePath {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ConcretePath {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for ConcretePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Parsed components of a concrete path
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedConcretePath {
    pub product_id: String,
    pub component_type: String,
    pub component_id: String,
    pub attribute_name: String,
}

impl ParsedConcretePath {
    /// Convert back to concrete path string
    pub fn to_path(&self) -> ConcretePath {
        ConcretePath::build(
            &self.product_id,
            &self.component_type,
            &self.component_id,
            &self.attribute_name,
        )
    }

    /// Get the corresponding abstract path
    pub fn to_abstract_path(&self) -> AbstractPath {
        AbstractPath::build(
            &self.product_id,
            &self.component_type,
            Some(&self.component_id),
            &self.attribute_name,
        )
    }
}

/// Tag for categorizing attributes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tag(pub String);

impl Tag {
    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validate that this is a valid tag
    pub fn is_valid(&self) -> bool {
        validation::is_valid_tag(&self.0)
    }
}

impl From<String> for Tag {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Tag {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_id() {
        let id = ProductId::new("insuranceV1");
        assert!(id.is_valid());
        assert_eq!(id.as_str(), "insuranceV1");

        let invalid = ProductId::new("123-invalid");
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_abstract_path_build() {
        // Without component ID
        let path = AbstractPath::build("myProduct", "cover", None, "premium");
        assert_eq!(path.as_str(), "myProduct:abstract-path:cover:premium");
        assert!(path.is_valid());

        // With component ID
        let path = AbstractPath::build("myProduct", "cover", Some("basic"), "premium");
        assert_eq!(path.as_str(), "myProduct:abstract-path:cover:basic:premium");
        assert!(path.is_valid());
    }

    #[test]
    fn test_abstract_path_parse() {
        // Without component ID
        let path = AbstractPath::new("myProduct:abstract-path:cover:premium");
        let parsed = path.parse().unwrap();
        assert_eq!(parsed.product_id, "myProduct");
        assert_eq!(parsed.component_type, "cover");
        assert_eq!(parsed.component_id, None);
        assert_eq!(parsed.attribute_name, "premium");

        // With component ID
        let path = AbstractPath::new("myProduct:abstract-path:cover:basic:premium");
        let parsed = path.parse().unwrap();
        assert_eq!(parsed.product_id, "myProduct");
        assert_eq!(parsed.component_type, "cover");
        assert_eq!(parsed.component_id, Some("basic".to_string()));
        assert_eq!(parsed.attribute_name, "premium");
    }

    #[test]
    fn test_concrete_path_build() {
        let path = ConcretePath::build("myProduct", "cover", "basic", "premium");
        assert_eq!(path.as_str(), "myProduct:cover:basic:premium");
        assert!(path.is_valid());
    }

    #[test]
    fn test_concrete_path_parse() {
        let path = ConcretePath::new("myProduct:cover:basic:premium");
        let parsed = path.parse().unwrap();
        assert_eq!(parsed.product_id, "myProduct");
        assert_eq!(parsed.component_type, "cover");
        assert_eq!(parsed.component_id, "basic");
        assert_eq!(parsed.attribute_name, "premium");

        // Roundtrip
        let roundtrip = parsed.to_path();
        assert_eq!(roundtrip.as_str(), path.as_str());
    }

    #[test]
    fn test_path_conversion() {
        let concrete = ConcretePath::new("myProduct:cover:basic:premium");
        let parsed = concrete.parse().unwrap();

        // Convert to abstract path
        let abstract_path = parsed.to_abstract_path();
        assert_eq!(abstract_path.as_str(), "myProduct:abstract-path:cover:basic:premium");
    }

    #[test]
    fn test_tag_validation() {
        let valid = Tag::new("input");
        assert!(valid.is_valid());

        let valid_hyphen = Tag::new("market-data");
        assert!(valid_hyphen.is_valid());

        let invalid_uppercase = Tag::new("Input");
        assert!(!invalid_uppercase.is_valid());

        let invalid_underscore = Tag::new("input_data");
        assert!(!invalid_underscore.is_valid());
    }

    #[test]
    fn test_functionality_id() {
        let valid = FunctionalityId::new("premium-calculation");
        assert!(valid.is_valid());

        let invalid = FunctionalityId::new("PREMIUM_CALCULATION");
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_datatype_id() {
        let valid = DataTypeId::new("decimal");
        assert!(valid.is_valid());

        let valid_hyphen = DataTypeId::new("date-time");
        assert!(valid_hyphen.is_valid());

        let invalid = DataTypeId::new("DateTime");
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_rule_id() {
        let id = RuleId::new();
        let hex = id.to_hex_string();
        assert_eq!(hex.len(), 32); // UUID without hyphens is 32 chars

        // Parse from string
        let id2 = RuleId::from_string(&id.as_uuid().to_string());
        assert_eq!(id2.as_uuid(), id.as_uuid());
    }
}
