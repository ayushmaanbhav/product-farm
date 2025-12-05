//! Product Template Enumeration - dynamic enumeration values for product templates
//!
//! ProductTemplateEnumeration defines valid enumeration values for a given product template type.
//! Unlike hardcoded enums, these are stored in the database and can be managed dynamically.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{validation, CoreError, CoreResult};
use crate::product::TemplateType;

/// Unique identifier for a product template enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateEnumerationId(pub String);

impl TemplateEnumerationId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for TemplateEnumerationId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for TemplateEnumerationId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for TemplateEnumerationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A product template enumeration definition
///
/// Defines a set of valid enumeration values for a specific product template type.
/// For example, for an "insurance" template type, you might have enumerations for:
/// - "coverage_type" with values ["BASIC", "STANDARD", "PREMIUM"]
/// - "risk_category" with values ["LOW", "MEDIUM", "HIGH"]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductTemplateEnumeration {
    /// Unique enumeration identifier
    pub id: TemplateEnumerationId,
    /// Name of the enumeration (e.g., "coverage_type", "risk_category")
    pub name: String,
    /// Template type this enumeration belongs to (e.g., "insurance", "trading")
    pub template_type: TemplateType,
    /// Valid values for this enumeration (ordered set)
    pub values: BTreeSet<String>,
    /// Optional description
    pub description: Option<String>,
}

impl ProductTemplateEnumeration {
    /// Create a new template enumeration
    pub fn new(
        id: impl Into<TemplateEnumerationId>,
        name: impl Into<String>,
        template_type: impl Into<TemplateType>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            template_type: template_type.into(),
            values: BTreeSet::new(),
            description: None,
        }
    }

    /// Add values to the enumeration
    pub fn with_values(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.values = values.into_iter().map(|v| v.into()).collect();
        self
    }

    /// Add a single value
    pub fn add_value(&mut self, value: impl Into<String>) {
        self.values.insert(value.into());
    }

    /// Remove a value
    pub fn remove_value(&mut self, value: &str) -> bool {
        self.values.remove(value)
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Validate the enumeration
    pub fn validate(&self) -> CoreResult<()> {
        // Validate enumeration name
        if !validation::is_valid_enumeration_name(&self.name) {
            return Err(CoreError::ValidationFailed {
                field: "name".to_string(),
                message: format!(
                    "Enumeration name '{}' does not match required pattern",
                    self.name
                ),
            });
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

        // Validate each value
        for value in &self.values {
            if !validation::is_valid_enumeration_value(value) {
                return Err(CoreError::ValidationFailed {
                    field: "values".to_string(),
                    message: format!("Enumeration value '{}' does not match required pattern", value),
                });
            }
        }

        Ok(())
    }

    /// Check if a value is valid for this enumeration
    pub fn contains(&self, value: &str) -> bool {
        self.values.contains(value)
    }

    /// Get all values as a slice
    pub fn values_iter(&self) -> impl Iterator<Item = &String> {
        self.values.iter()
    }

    /// Get the number of values
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the enumeration is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Validate that a value is in this enumeration
    pub fn validate_value(&self, value: &str) -> CoreResult<()> {
        if self.contains(value) {
            Ok(())
        } else {
            Err(CoreError::InvalidEnumValue {
                enum_name: self.name.clone(),
                value: value.to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_enumeration_creation() {
        let enum_def = ProductTemplateEnumeration::new(
            "coverage-type-enum",
            "coverage-type",
            "insurance",
        )
        .with_values(["basic", "standard", "premium"])
        .with_description("Types of coverage available");

        assert_eq!(enum_def.name, "coverage-type");
        assert_eq!(enum_def.template_type.as_str(), "insurance");
        assert_eq!(enum_def.len(), 3);
        assert!(enum_def.contains("basic"));
        assert!(enum_def.contains("premium"));
        assert!(!enum_def.contains("invalid"));
        assert!(enum_def.validate().is_ok());
    }

    #[test]
    fn test_add_remove_values() {
        let mut enum_def = ProductTemplateEnumeration::new(
            "risk-enum",
            "risk-category",
            "insurance",
        )
        .with_values(["low", "medium"]);

        assert_eq!(enum_def.len(), 2);

        enum_def.add_value("high");
        assert_eq!(enum_def.len(), 3);
        assert!(enum_def.contains("high"));

        let removed = enum_def.remove_value("low");
        assert!(removed);
        assert_eq!(enum_def.len(), 2);
        assert!(!enum_def.contains("low"));
    }

    #[test]
    fn test_validate_value() {
        let enum_def = ProductTemplateEnumeration::new(
            "status-enum",
            "policy-status",
            "insurance",
        )
        .with_values(["active", "suspended", "cancelled"]);

        assert!(enum_def.validate_value("active").is_ok());
        assert!(enum_def.validate_value("invalid").is_err());
    }

    #[test]
    fn test_ordered_values() {
        let enum_def = ProductTemplateEnumeration::new(
            "test-enum",
            "test-values",
            "trading",
        )
        .with_values(["charlie", "alpha", "bravo"]);

        // BTreeSet maintains sorted order
        let values: Vec<_> = enum_def.values_iter().collect();
        assert_eq!(values, vec!["alpha", "bravo", "charlie"]);
    }

    #[test]
    fn test_different_template_types() {
        let insurance_enum = ProductTemplateEnumeration::new(
            "ins-enum",
            "coverage-type",
            "insurance",
        );
        let trading_enum = ProductTemplateEnumeration::new(
            "trade-enum",
            "signal-type",
            "trading",
        );

        assert_eq!(insurance_enum.template_type.as_str(), "insurance");
        assert_eq!(trading_enum.template_type.as_str(), "trading");
    }

    #[test]
    fn test_validation_fails_invalid_name() {
        let enum_def = ProductTemplateEnumeration::new(
            "test-enum",
            "INVALID_NAME", // Uppercase + underscore not allowed
            "trading",
        );
        assert!(enum_def.validate().is_err());
    }

    #[test]
    fn test_validation_fails_invalid_value() {
        let enum_def = ProductTemplateEnumeration::new(
            "test-enum",
            "valid-name",
            "trading",
        )
        .with_values(["INVALID_VALUE"]); // Uppercase not allowed

        assert!(enum_def.validate().is_err());
    }
}
