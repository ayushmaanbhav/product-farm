//! DataType definitions - dynamically configured types

use serde::{Deserialize, Serialize};

use crate::DataTypeId;

/// Primitive type underlying a DataType
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrimitiveType {
    /// String type
    String,
    /// Integer type (64-bit)
    Int,
    /// Floating point type
    Float,
    /// Decimal type (precise arithmetic)
    Decimal,
    /// Boolean type
    Bool,
    /// Date/time type
    Datetime,
    /// Enumeration type (must reference an enum definition)
    Enum,
    /// Array type
    Array,
    /// Object/map type
    Object,
    /// Reference to another attribute
    AttributeReference,
    /// Identifier type (for referencing entities)
    Identifier,
}

impl PrimitiveType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrimitiveType::String => "string",
            PrimitiveType::Int => "int",
            PrimitiveType::Float => "float",
            PrimitiveType::Decimal => "decimal",
            PrimitiveType::Bool => "bool",
            PrimitiveType::Datetime => "datetime",
            PrimitiveType::Enum => "enum",
            PrimitiveType::Array => "array",
            PrimitiveType::Object => "object",
            PrimitiveType::AttributeReference => "attribute_reference",
            PrimitiveType::Identifier => "identifier",
        }
    }
}

/// A data type definition (dynamically created via API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataType {
    /// Unique name for this data type
    pub id: DataTypeId,
    /// The underlying primitive type
    pub primitive_type: PrimitiveType,
    /// Human-readable description
    pub description: Option<String>,
    /// Validation constraints (JSON schema subset)
    pub constraints: Option<DataTypeConstraints>,
}

impl DataType {
    /// Create a new data type
    pub fn new(id: impl Into<DataTypeId>, primitive_type: PrimitiveType) -> Self {
        Self {
            id: id.into(),
            primitive_type,
            description: None,
            constraints: None,
        }
    }

    /// Add description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add constraints
    pub fn with_constraints(mut self, constraints: DataTypeConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }
}

/// Constraints for data type validation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataTypeConstraints {
    /// Minimum value (for numeric types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Maximum value (for numeric types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Minimum length (for strings/arrays)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    /// Maximum length (for strings/arrays)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    /// Regex pattern (for strings)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    /// Decimal precision (for decimal types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<u8>,
    /// Decimal scale (for decimal types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<u8>,
    /// JSON Logic expression for complex constraint validation
    /// Expression must return true for valid values
    /// Variables: $value (the value being validated), other attribute paths for cross-field validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint_rule_expression: Option<String>,
    /// Custom error message when constraint rule validation fails
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint_error_message: Option<String>,
}

/// An enumeration definition (for enum data types)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDefinition {
    /// Unique name for this enum
    pub name: String,
    /// Product template type this enum belongs to
    pub template_type: String,
    /// Allowed values
    pub values: Vec<String>,
    /// Human-readable description
    pub description: Option<String>,
}

impl EnumDefinition {
    /// Create a new enum definition
    pub fn new(
        name: impl Into<String>,
        template_type: impl Into<String>,
        values: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            template_type: template_type.into(),
            values,
            description: None,
        }
    }

    /// Check if a value is valid for this enum
    pub fn is_valid(&self, value: &str) -> bool {
        self.values.iter().any(|v| v == value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datatype_creation() {
        let dt = DataType::new("price", PrimitiveType::Decimal)
            .with_description("Price with 4 decimal precision")
            .with_constraints(DataTypeConstraints {
                min: Some(0.0),
                precision: Some(18),
                scale: Some(4),
                ..Default::default()
            });

        assert_eq!(dt.id.as_str(), "price");
        assert_eq!(dt.primitive_type, PrimitiveType::Decimal);
        assert!(dt.constraints.is_some());
    }

    #[test]
    fn test_enum_definition() {
        let signal_enum = EnumDefinition::new(
            "SignalType",
            "TRADING",
            vec!["BUY".to_string(), "SELL".to_string(), "HOLD".to_string()],
        );

        assert!(signal_enum.is_valid("BUY"));
        assert!(signal_enum.is_valid("SELL"));
        assert!(!signal_enum.is_valid("INVALID"));
    }
}
