//! Attribute definitions - both abstract (templates) and concrete (instances)
//!
//! ## Attribute Value Types (Legacy Compatible)
//!
//! - **FIXED_VALUE**: value must NOT be null AND rule must be null
//! - **RULE_DRIVEN**: value must be null AND rule must NOT be null
//! - **JUST_DEFINITION**: value must be null AND rule must be null (schema only)
//!
//! ## Path Formats
//!
//! - **Concrete Path**: `{productId}:{componentType}:{componentId}:{attributeName}`
//! - **Abstract Path**: `{productId}:abstract-path:{componentType}[:{componentId}]:{attributeName}`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    validation, AbstractPath, ConcretePath, CoreError, CoreResult, DataTypeId, ProductId,
    RuleId, Tag, Value,
};

// =============================================================================
// ENUMS
// =============================================================================

/// Attribute value type - determines how the attribute gets its value
///
/// This enum matches the legacy Kotlin AttributeValueType exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum AttributeValueType {
    /// Fixed value attribute - has a static value, no rule
    /// Constraint: value IS NOT NULL AND rule IS NULL
    #[default]
    FixedValue,
    /// Rule-driven attribute - computed by a rule, no static value
    /// Constraint: value IS NULL AND rule IS NOT NULL
    RuleDriven,
    /// Just definition - schema only, no value and no rule
    /// Constraint: value IS NULL AND rule IS NULL
    JustDefinition,
}


impl AttributeValueType {
    /// Check if this type requires a value
    pub fn requires_value(&self) -> bool {
        matches!(self, AttributeValueType::FixedValue)
    }

    /// Check if this type requires a rule
    pub fn requires_rule(&self) -> bool {
        matches!(self, AttributeValueType::RuleDriven)
    }

    /// Check if this type forbids both value and rule
    pub fn is_definition_only(&self) -> bool {
        matches!(self, AttributeValueType::JustDefinition)
    }
}

/// Display name format - how the display name should be formatted
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum DisplayNameFormat {
    /// System format (lowercase, hyphens)
    #[default]
    System,
    /// Original format from source (preserved case)
    Original,
    /// Human-readable format (Title Case, spaces)
    Human,
}


/// Attribute relationship type - defines how abstract attributes relate
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttributeRelationshipType {
    /// Simple enumeration reference
    Enumeration,
    /// Key of a key-value enumeration
    KeyEnumeration,
    /// Value of a key-value enumeration
    ValueEnumeration,
}

// =============================================================================
// DISPLAY NAME
// =============================================================================

/// Structured display name with format and ordering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDisplayName {
    /// The product this display name belongs to
    pub product_id: ProductId,
    /// The display name text
    pub display_name: String,
    /// Associated abstract path (for abstract attributes)
    pub abstract_path: Option<AbstractPath>,
    /// Associated concrete path (for concrete attributes)
    pub path: Option<ConcretePath>,
    /// Format of this display name
    pub display_name_format: DisplayNameFormat,
    /// Order for sorting display names
    pub order: i32,
}

impl AttributeDisplayName {
    /// Create a display name for an abstract attribute
    pub fn for_abstract(
        product_id: impl Into<ProductId>,
        abstract_path: impl Into<AbstractPath>,
        display_name: impl Into<String>,
        format: DisplayNameFormat,
        order: i32,
    ) -> Self {
        Self {
            product_id: product_id.into(),
            display_name: display_name.into(),
            abstract_path: Some(abstract_path.into()),
            path: None,
            display_name_format: format,
            order,
        }
    }

    /// Create a display name for a concrete attribute
    pub fn for_concrete(
        product_id: impl Into<ProductId>,
        path: impl Into<ConcretePath>,
        display_name: impl Into<String>,
        format: DisplayNameFormat,
        order: i32,
    ) -> Self {
        Self {
            product_id: product_id.into(),
            display_name: display_name.into(),
            abstract_path: None,
            path: Some(path.into()),
            display_name_format: format,
            order,
        }
    }
}

// =============================================================================
// RELATED ATTRIBUTE
// =============================================================================

/// Relationship between abstract attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractAttributeRelatedAttribute {
    /// The source abstract attribute path
    pub abstract_path: AbstractPath,
    /// The referenced abstract attribute path
    pub reference_abstract_path: AbstractPath,
    /// The type of relationship
    pub relationship: AttributeRelationshipType,
    /// Order for sorting
    pub order: i32,
}

impl AbstractAttributeRelatedAttribute {
    pub fn new(
        abstract_path: impl Into<AbstractPath>,
        reference: impl Into<AbstractPath>,
        relationship: AttributeRelationshipType,
        order: i32,
    ) -> Self {
        Self {
            abstract_path: abstract_path.into(),
            reference_abstract_path: reference.into(),
            relationship,
            order,
        }
    }
}

// =============================================================================
// TAG
// =============================================================================

/// Tag relationship with ordering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractAttributeTag {
    /// The abstract attribute path
    pub abstract_path: AbstractPath,
    /// The tag
    pub tag: Tag,
    /// Product ID
    pub product_id: ProductId,
    /// Order for sorting
    pub order: i32,
}

impl AbstractAttributeTag {
    pub fn new(
        abstract_path: impl Into<AbstractPath>,
        tag: impl Into<Tag>,
        product_id: impl Into<ProductId>,
        order: i32,
    ) -> Self {
        Self {
            abstract_path: abstract_path.into(),
            tag: tag.into(),
            product_id: product_id.into(),
            order,
        }
    }
}

// =============================================================================
// ABSTRACT ATTRIBUTE
// =============================================================================

/// Abstract attribute - a template for attribute definitions
/// These define the schema for attributes that can be instantiated per product
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractAttribute {
    /// Abstract path: {productId}:abstract-path:{componentType}[:{componentId}]:{attributeName}
    pub abstract_path: AbstractPath,
    /// Product this attribute belongs to
    pub product_id: ProductId,
    /// Component type (e.g., "cover", "customer", "position")
    pub component_type: String,
    /// Optional component ID (for specific instances)
    pub component_id: Option<String>,
    /// Data type reference
    pub datatype_id: DataTypeId,
    /// Enum name (required when datatype is enum type)
    pub enum_name: Option<String>,
    /// Constraint expression (JSON Logic for validation)
    pub constraint_expression: Option<serde_json::Value>,
    /// Whether this attribute can be modified after product approval
    pub immutable: bool,
    /// Human-readable description
    pub description: Option<String>,
    /// Display names for user-facing representation (ordered)
    pub display_names: Vec<AttributeDisplayName>,
    /// Tags for categorization and querying (ordered)
    pub tags: Vec<AbstractAttributeTag>,
    /// Related attributes (ordered)
    pub related_attributes: Vec<AbstractAttributeRelatedAttribute>,
}

impl AbstractAttribute {
    /// Create a new abstract attribute without validation.
    ///
    /// Note: Prefer `try_new()` for API boundaries to ensure valid data.
    pub fn new(
        abstract_path: impl Into<AbstractPath>,
        product_id: impl Into<ProductId>,
        component_type: impl Into<String>,
        datatype_id: impl Into<DataTypeId>,
    ) -> Self {
        Self {
            abstract_path: abstract_path.into(),
            product_id: product_id.into(),
            component_type: component_type.into(),
            component_id: None,
            datatype_id: datatype_id.into(),
            enum_name: None,
            constraint_expression: None,
            immutable: false,
            description: None,
            display_names: Vec::new(),
            tags: Vec::new(),
            related_attributes: Vec::new(),
        }
    }

    /// Create a new abstract attribute with validation.
    ///
    /// Returns an error if the abstract path or component type is invalid.
    /// Use this at API boundaries to ensure data integrity.
    pub fn try_new(
        abstract_path: impl Into<AbstractPath>,
        product_id: impl Into<ProductId>,
        component_type: impl Into<String>,
        datatype_id: impl Into<DataTypeId>,
    ) -> CoreResult<Self> {
        let attr = Self::new(abstract_path, product_id, component_type, datatype_id);
        attr.validate()?;
        Ok(attr)
    }

    /// Set component ID
    pub fn with_component_id(mut self, id: impl Into<String>) -> Self {
        self.component_id = Some(id.into());
        self
    }

    /// Set enum name (for enum datatypes)
    pub fn with_enum(mut self, enum_name: impl Into<String>) -> Self {
        self.enum_name = Some(enum_name.into());
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Mark as immutable
    pub fn immutable(mut self) -> Self {
        self.immutable = true;
        self
    }

    /// Set constraint expression
    pub fn with_constraint(mut self, expr: serde_json::Value) -> Self {
        self.constraint_expression = Some(expr);
        self
    }

    /// Add a display name
    pub fn with_display_name(mut self, display_name: AttributeDisplayName) -> Self {
        self.display_names.push(display_name);
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: AbstractAttributeTag) -> Self {
        self.tags.push(tag);
        self
    }

    /// Add a simple tag by name
    pub fn with_tag_name(mut self, tag_name: impl Into<Tag>, order: i32) -> Self {
        self.tags.push(AbstractAttributeTag::new(
            self.abstract_path.clone(),
            tag_name,
            self.product_id.clone(),
            order,
        ));
        self
    }

    /// Add a related attribute
    pub fn with_related_attribute(mut self, related: AbstractAttributeRelatedAttribute) -> Self {
        self.related_attributes.push(related);
        self
    }

    /// Get primary display name
    pub fn primary_display_name(&self) -> Option<&AttributeDisplayName> {
        self.display_names.first()
    }

    /// Check if attribute has a specific tag
    pub fn has_tag(&self, tag_name: &str) -> bool {
        self.tags.iter().any(|t| t.tag.as_str() == tag_name)
    }

    /// Check if modifications should be blocked
    pub fn check_modifiable(&self) -> CoreResult<()> {
        if self.immutable {
            return Err(CoreError::Immutable {
                entity_type: "AbstractAttribute".to_string(),
                id: self.abstract_path.as_str().to_string(),
            });
        }
        Ok(())
    }

    /// Check if attribute is editable
    pub fn is_editable(&self) -> bool {
        !self.immutable
    }

    /// Validate the abstract attribute
    pub fn validate(&self) -> CoreResult<()> {
        // Validate abstract path format
        if !validation::is_valid_abstract_path(self.abstract_path.as_str()) {
            return Err(CoreError::InvalidPath(format!(
                "Invalid abstract path: {}",
                self.abstract_path.as_str()
            )));
        }

        // Validate component type
        if !validation::is_valid_component_type(&self.component_type) {
            return Err(CoreError::ValidationFailed {
                field: "component_type".to_string(),
                message: format!(
                    "Component type '{}' does not match required pattern",
                    self.component_type
                ),
            });
        }

        // Validate component ID if present
        if let Some(cid) = &self.component_id {
            if !validation::is_valid_component_id(cid) {
                return Err(CoreError::ValidationFailed {
                    field: "component_id".to_string(),
                    message: format!(
                        "Component ID '{}' does not match required pattern",
                        cid
                    ),
                });
            }
        }

        // Validate datatype
        if !validation::is_valid_datatype(self.datatype_id.as_str()) {
            return Err(CoreError::ValidationFailed {
                field: "datatype_id".to_string(),
                message: format!(
                    "Datatype '{}' does not match required pattern",
                    self.datatype_id.as_str()
                ),
            });
        }

        // Validate tags
        for tag in &self.tags {
            if !validation::is_valid_tag(tag.tag.as_str()) {
                return Err(CoreError::ValidationFailed {
                    field: "tag".to_string(),
                    message: format!(
                        "Tag '{}' does not match required pattern",
                        tag.tag.as_str()
                    ),
                });
            }
        }

        // Validate description if present
        if let Some(desc) = &self.description {
            if !validation::is_valid_description(desc) {
                return Err(CoreError::ValidationFailed {
                    field: "description".to_string(),
                    message: "Description does not match required pattern".to_string(),
                });
            }
        }

        Ok(())
    }
}

// =============================================================================
// CONCRETE ATTRIBUTE
// =============================================================================

/// Concrete attribute - an instance of an abstract attribute with a value or rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    /// Concrete path: {productId}:{componentType}:{componentId}:{attributeName}
    pub path: ConcretePath,
    /// Reference to abstract attribute template
    pub abstract_path: AbstractPath,
    /// Product this attribute belongs to
    pub product_id: ProductId,
    /// The value type (FIXED_VALUE, RULE_DRIVEN, or JUST_DEFINITION)
    pub value_type: AttributeValueType,
    /// Static value (required when value_type is FIXED_VALUE)
    pub value: Option<Value>,
    /// Rule ID (required when value_type is RULE_DRIVEN)
    pub rule_id: Option<RuleId>,
    /// Display names (can override abstract attribute)
    pub display_names: Vec<AttributeDisplayName>,
    /// When the attribute was created
    pub created_at: DateTime<Utc>,
    /// When the attribute was last updated
    pub updated_at: DateTime<Utc>,
}

impl Attribute {
    /// Create a new attribute with a fixed value
    pub fn new_fixed_value(
        path: impl Into<ConcretePath>,
        abstract_path: impl Into<AbstractPath>,
        product_id: impl Into<ProductId>,
        value: Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            path: path.into(),
            abstract_path: abstract_path.into(),
            product_id: product_id.into(),
            value_type: AttributeValueType::FixedValue,
            value: Some(value),
            rule_id: None,
            display_names: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new attribute computed by a rule
    pub fn new_rule_driven(
        path: impl Into<ConcretePath>,
        abstract_path: impl Into<AbstractPath>,
        product_id: impl Into<ProductId>,
        rule_id: RuleId,
    ) -> Self {
        let now = Utc::now();
        Self {
            path: path.into(),
            abstract_path: abstract_path.into(),
            product_id: product_id.into(),
            value_type: AttributeValueType::RuleDriven,
            value: None,
            rule_id: Some(rule_id),
            display_names: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new attribute that is definition only (no value, no rule)
    pub fn new_just_definition(
        path: impl Into<ConcretePath>,
        abstract_path: impl Into<AbstractPath>,
        product_id: impl Into<ProductId>,
    ) -> Self {
        let now = Utc::now();
        Self {
            path: path.into(),
            abstract_path: abstract_path.into(),
            product_id: product_id.into(),
            value_type: AttributeValueType::JustDefinition,
            value: None,
            rule_id: None,
            display_names: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a display name
    pub fn with_display_name(mut self, display_name: AttributeDisplayName) -> Self {
        self.display_names.push(display_name);
        self
    }

    /// Check if attribute has a fixed value
    pub fn is_fixed_value(&self) -> bool {
        matches!(self.value_type, AttributeValueType::FixedValue)
    }

    /// Check if attribute is rule-driven
    pub fn is_rule_driven(&self) -> bool {
        matches!(self.value_type, AttributeValueType::RuleDriven)
    }

    /// Check if attribute is definition only
    pub fn is_just_definition(&self) -> bool {
        matches!(self.value_type, AttributeValueType::JustDefinition)
    }

    /// Get the value (for fixed value attributes)
    pub fn get_value(&self) -> Option<&Value> {
        self.value.as_ref()
    }

    /// Get the rule ID (for rule-driven attributes)
    pub fn get_rule_id(&self) -> Option<&RuleId> {
        self.rule_id.as_ref()
    }

    /// Validate the attribute's type constraints
    pub fn validate(&self) -> CoreResult<()> {
        // Validate path format
        if !validation::is_valid_path(self.path.as_str()) {
            return Err(CoreError::InvalidPath(format!(
                "Invalid path: {}",
                self.path.as_str()
            )));
        }

        // Validate abstract path format
        if !validation::is_valid_abstract_path(self.abstract_path.as_str()) {
            return Err(CoreError::InvalidPath(format!(
                "Invalid abstract path: {}",
                self.abstract_path.as_str()
            )));
        }

        // Validate type constraints
        match self.value_type {
            AttributeValueType::FixedValue => {
                if self.value.is_none() {
                    return Err(CoreError::ValidationFailed {
                        field: "value".to_string(),
                        message: "FIXED_VALUE attribute must have a value".to_string(),
                    });
                }
                if self.rule_id.is_some() {
                    return Err(CoreError::ValidationFailed {
                        field: "rule_id".to_string(),
                        message: "FIXED_VALUE attribute must not have a rule".to_string(),
                    });
                }
            }
            AttributeValueType::RuleDriven => {
                if self.value.is_some() {
                    return Err(CoreError::ValidationFailed {
                        field: "value".to_string(),
                        message: "RULE_DRIVEN attribute must not have a value".to_string(),
                    });
                }
                if self.rule_id.is_none() {
                    return Err(CoreError::ValidationFailed {
                        field: "rule_id".to_string(),
                        message: "RULE_DRIVEN attribute must have a rule".to_string(),
                    });
                }
            }
            AttributeValueType::JustDefinition => {
                if self.value.is_some() {
                    return Err(CoreError::ValidationFailed {
                        field: "value".to_string(),
                        message: "JUST_DEFINITION attribute must not have a value".to_string(),
                    });
                }
                if self.rule_id.is_some() {
                    return Err(CoreError::ValidationFailed {
                        field: "rule_id".to_string(),
                        message: "JUST_DEFINITION attribute must not have a rule".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

// =============================================================================
// LEGACY ALIASES (for backwards compatibility)
// =============================================================================

/// Alias for backwards compatibility
pub type AttributeType = AttributeValueType;

/// Alias for DisplayName (simple string wrapper)
pub type DisplayName = String;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_value_type() {
        assert!(AttributeValueType::FixedValue.requires_value());
        assert!(!AttributeValueType::FixedValue.requires_rule());

        assert!(!AttributeValueType::RuleDriven.requires_value());
        assert!(AttributeValueType::RuleDriven.requires_rule());

        assert!(!AttributeValueType::JustDefinition.requires_value());
        assert!(!AttributeValueType::JustDefinition.requires_rule());
        assert!(AttributeValueType::JustDefinition.is_definition_only());
    }

    #[test]
    fn test_fixed_value_attribute() {
        let attr = Attribute::new_fixed_value(
            "myProduct:cover:basic:premium",
            "myProduct:abstract-path:cover:premium",
            "myProduct",
            Value::Float(100.0),
        );

        assert!(attr.is_fixed_value());
        assert!(!attr.is_rule_driven());
        assert!(!attr.is_just_definition());
        assert_eq!(attr.get_value(), Some(&Value::Float(100.0)));
        assert!(attr.get_rule_id().is_none());
        assert!(attr.validate().is_ok());
    }

    #[test]
    fn test_rule_driven_attribute() {
        let rule_id = RuleId::new();
        let attr = Attribute::new_rule_driven(
            "myProduct:signal:entry:decision",
            "myProduct:abstract-path:signal:decision",
            "myProduct",
            rule_id.clone(),
        );

        assert!(!attr.is_fixed_value());
        assert!(attr.is_rule_driven());
        assert!(!attr.is_just_definition());
        assert!(attr.get_value().is_none());
        assert_eq!(attr.get_rule_id(), Some(&rule_id));
        assert!(attr.validate().is_ok());
    }

    #[test]
    fn test_just_definition_attribute() {
        let attr = Attribute::new_just_definition(
            "myProduct:schema:def:field",
            "myProduct:abstract-path:schema:field",
            "myProduct",
        );

        assert!(!attr.is_fixed_value());
        assert!(!attr.is_rule_driven());
        assert!(attr.is_just_definition());
        assert!(attr.get_value().is_none());
        assert!(attr.get_rule_id().is_none());
        assert!(attr.validate().is_ok());
    }

    #[test]
    fn test_invalid_fixed_value_without_value() {
        let now = Utc::now();
        let attr = Attribute {
            path: ConcretePath::new("myProduct:cover:basic:premium"),
            abstract_path: AbstractPath::new("myProduct:abstract-path:cover:premium"),
            product_id: ProductId::new("myProduct"),
            value_type: AttributeValueType::FixedValue,
            value: None, // Invalid: should have value
            rule_id: None,
            display_names: Vec::new(),
            created_at: now,
            updated_at: now,
        };

        assert!(attr.validate().is_err());
    }

    #[test]
    fn test_invalid_rule_driven_without_rule() {
        let now = Utc::now();
        let attr = Attribute {
            path: ConcretePath::new("myProduct:signal:entry:decision"),
            abstract_path: AbstractPath::new("myProduct:abstract-path:signal:decision"),
            product_id: ProductId::new("myProduct"),
            value_type: AttributeValueType::RuleDriven,
            value: None,
            rule_id: None, // Invalid: should have rule
            display_names: Vec::new(),
            created_at: now,
            updated_at: now,
        };

        assert!(attr.validate().is_err());
    }

    #[test]
    fn test_abstract_attribute_with_tags() {
        let abstract_attr = AbstractAttribute::new(
            "myProduct:abstract-path:cover:premium",
            "myProduct",
            "cover",
            "decimal",
        )
        .with_tag_name("input", 0)
        .with_tag_name("financial", 1)
        .with_description("Premium amount for cover");

        assert!(abstract_attr.has_tag("input"));
        assert!(abstract_attr.has_tag("financial"));
        assert!(!abstract_attr.has_tag("output"));
    }

    #[test]
    fn test_display_name_format() {
        let system = AttributeDisplayName::for_abstract(
            "myProduct",
            "myProduct:abstract-path:cover:premium",
            "cover-premium",
            DisplayNameFormat::System,
            0,
        );

        let human = AttributeDisplayName::for_abstract(
            "myProduct",
            "myProduct:abstract-path:cover:premium",
            "Cover Premium",
            DisplayNameFormat::Human,
            1,
        );

        assert_eq!(system.display_name_format, DisplayNameFormat::System);
        assert_eq!(human.display_name_format, DisplayNameFormat::Human);
    }
}
