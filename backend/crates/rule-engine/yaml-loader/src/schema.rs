//! Flexible YAML schema types.
//!
//! These types represent the intermediate parsed YAML structure before
//! transformation into core Product-FARM types.

use product_farm_core::{
    AbstractAttribute, DataType, Product, ProductFunctionality, Rule,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root YAML document - all fields optional for flexibility.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct YamlDocument {
    /// Schema version.
    pub version: Option<String>,

    /// Product metadata.
    pub product: Option<YamlProductMeta>,

    /// Data types / enums.
    #[serde(alias = "types", alias = "datatypes", alias = "data-types")]
    pub types: Option<HashMap<String, YamlType>>,

    /// Entity definitions.
    #[serde(alias = "entities", alias = "schema", alias = "models")]
    pub entities: Option<HashMap<String, YamlEntity>>,

    /// Function/rule definitions.
    #[serde(alias = "functions", alias = "rules", alias = "computations")]
    pub functions: Option<HashMap<String, YamlFunction>>,

    /// Functionality definitions.
    #[serde(alias = "functionalities", alias = "features", alias = "capabilities")]
    pub functionalities: Option<HashMap<String, YamlFunctionality>>,

    /// Constraint definitions.
    #[serde(alias = "constraints", alias = "validations")]
    pub constraints: Option<HashMap<String, YamlConstraint>>,

    /// Layer visibility configuration.
    pub layers: Option<HashMap<String, YamlLayerConfig>>,
}

/// Product metadata.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct YamlProductMeta {
    /// Product ID (can be inferred from folder name).
    pub id: Option<String>,

    /// Product name.
    pub name: Option<String>,

    /// Product description.
    pub description: Option<String>,

    /// Version string.
    pub version: Option<String>,

    /// Tags for categorization.
    pub tags: Option<Vec<String>>,
}

/// Type/enum definition.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum YamlType {
    /// Simple type alias: "string", "decimal", "boolean"
    Simple(String),

    /// Enum definition with values.
    Enum {
        #[serde(alias = "type")]
        kind: Option<String>,
        values: Vec<String>,
        #[serde(default)]
        default: Option<String>,
        #[serde(default)]
        description: Option<String>,
    },

    /// Full type definition.
    Full {
        #[serde(alias = "type", alias = "kind")]
        base_type: String,
        #[serde(default)]
        min: Option<serde_json::Value>,
        #[serde(default)]
        max: Option<serde_json::Value>,
        #[serde(default)]
        pattern: Option<String>,
        #[serde(default)]
        description: Option<String>,
    },
}

/// Entity definition with flexible field parsing.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct YamlEntity {
    /// Entity description.
    pub description: Option<String>,

    /// Explicit attributes section.
    pub attributes: Option<HashMap<String, YamlFieldDefinition>>,

    /// Explicit relationships section.
    pub relationships: Option<HashMap<String, YamlRelationship>>,

    /// Flattened fields (catch-all for inline definitions).
    #[serde(flatten)]
    pub fields: HashMap<String, serde_yaml::Value>,
}

/// Field definition - can be expressed in many ways.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum YamlFieldDefinition {
    /// Simple type string: "string", "decimal", "enum[a,b,c]"
    Simple(String),

    /// Full typed definition.
    Typed {
        #[serde(alias = "type", alias = "kind")]
        field_type: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        default: Option<serde_json::Value>,
        #[serde(default)]
        required: Option<bool>,
        #[serde(default)]
        min: Option<serde_json::Value>,
        #[serde(default)]
        max: Option<serde_json::Value>,
        #[serde(default)]
        pattern: Option<String>,
        #[serde(default)]
        values: Option<Vec<String>>,
        #[serde(alias = "computed", alias = "formula", alias = "expression")]
        computed: Option<String>,
        #[serde(default)]
        static_: Option<bool>,
        #[serde(default)]
        instance: Option<bool>,
    },

    /// Array value (list of options).
    Array(Vec<String>),

    /// Nested object (for complex types).
    Nested(HashMap<String, serde_yaml::Value>),
}

/// Relationship definition.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YamlRelationship {
    /// Target entity name.
    pub target: String,

    /// Cardinality: "one-to-one", "one-to-many", "many-to-one", "many-to-many"
    #[serde(default = "default_cardinality")]
    pub cardinality: String,

    /// Whether the relationship is optional.
    #[serde(default)]
    pub optional: bool,

    /// Inverse relationship name.
    pub inverse: Option<String>,

    /// Description.
    pub description: Option<String>,
}

fn default_cardinality() -> String {
    "many-to-one".to_string()
}

/// Function/rule definition.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct YamlFunction {
    /// Function description.
    pub description: Option<String>,

    /// Input attribute paths.
    pub inputs: Option<Vec<String>>,

    /// Output attribute paths.
    pub outputs: Option<Vec<String>>,

    /// JSON Logic expression.
    #[serde(alias = "logic", alias = "rule")]
    pub expression: Option<serde_json::Value>,

    /// Evaluator type: "json-logic", "llm", or custom.
    #[serde(alias = "type")]
    pub evaluator: Option<String>,

    /// Evaluator configuration (model name, prompts, etc.).
    #[serde(alias = "config")]
    pub evaluator_config: Option<HashMap<String, serde_json::Value>>,

    /// Order index for execution priority.
    pub order: Option<i32>,

    /// Whether the function is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Functionality definition.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct YamlFunctionality {
    /// Functionality description.
    pub description: Option<String>,

    /// Required attributes for this functionality.
    #[serde(alias = "requires")]
    pub required_attributes: Option<Vec<String>>,

    /// Functions that implement this functionality.
    pub functions: Option<Vec<String>>,

    /// Tags.
    pub tags: Option<Vec<String>>,
}

/// Constraint definition.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YamlConstraint {
    /// Constraint expression (JSON Logic or condition string).
    pub expression: serde_json::Value,

    /// Error message when constraint fails.
    pub message: Option<String>,

    /// Severity: "error", "warning", "info"
    #[serde(default = "default_severity")]
    pub severity: String,

    /// Attribute paths this constraint applies to.
    pub applies_to: Option<Vec<String>>,
}

fn default_severity() -> String {
    "error".to_string()
}

/// Layer visibility configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct YamlLayerConfig {
    /// Layer name.
    pub name: Option<String>,

    /// Layer description.
    pub description: Option<String>,

    /// Visible entity names.
    pub entities: Option<Vec<String>>,

    /// Visible attribute paths.
    pub attributes: Option<Vec<String>>,

    /// Visible function names.
    pub functions: Option<Vec<String>>,
}

// =============================================================================
// Master Schema - Transformed Output
// =============================================================================

/// The unified master schema derived from all YAML sources.
#[derive(Debug, Clone)]
pub struct MasterSchema {
    /// The product entity.
    pub product: Product,

    /// All data types.
    pub data_types: Vec<DataType>,

    /// All abstract attributes.
    pub attributes: Vec<AbstractAttribute>,

    /// All rules.
    pub rules: Vec<Rule>,

    /// All functionalities.
    pub functionalities: Vec<ProductFunctionality>,

    /// Layer visibility configuration.
    pub layer_config: LayerVisibilityConfig,
}

impl MasterSchema {
    /// Create a new empty master schema.
    pub fn new(product: Product) -> Self {
        Self {
            product,
            data_types: Vec::new(),
            attributes: Vec::new(),
            rules: Vec::new(),
            functionalities: Vec::new(),
            layer_config: LayerVisibilityConfig::default(),
        }
    }

    /// Find a rule by name.
    pub fn find_rule(&self, name: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.rule_type == name)
    }

    /// Find an attribute by path.
    pub fn find_attribute(&self, path: &str) -> Option<&AbstractAttribute> {
        self.attributes.iter().find(|a| a.abstract_path.as_str() == path)
    }
}

/// Layer visibility configuration for interface-specific views.
#[derive(Debug, Clone, Default)]
pub struct LayerVisibilityConfig {
    /// Named layers with their visibility rules.
    pub layers: HashMap<String, LayerDefinition>,
}

/// Definition of a visibility layer.
#[derive(Debug, Clone, Default)]
pub struct LayerDefinition {
    /// Layer name.
    pub name: String,

    /// Description.
    pub description: Option<String>,

    /// Entity names visible in this layer.
    pub visible_entities: std::collections::HashSet<String>,

    /// Attribute paths visible in this layer.
    pub visible_attributes: std::collections::HashSet<String>,

    /// Function names visible in this layer.
    pub visible_functions: std::collections::HashSet<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_document_deserialize() {
        let yaml = r#"
version: "1.0"
product:
  id: test-product
  name: Test Product

entities:
  Scenario:
    difficulty: string
    max_score: decimal

functions:
  calculate-score:
    inputs: [difficulty, max_score]
    outputs: [score]
    expression: { "*": [{ "var": "max_score" }, 0.8] }
"#;
        let doc: YamlDocument = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(doc.product.unwrap().id.unwrap(), "test-product");
        assert!(doc.entities.is_some());
        assert!(doc.functions.is_some());
    }

    #[test]
    fn test_yaml_field_definition_variants() {
        // Simple string
        let yaml = r#""string""#;
        let field: YamlFieldDefinition = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(field, YamlFieldDefinition::Simple(_)));

        // Typed
        let yaml = r#"
type: decimal
min: 0
max: 100
"#;
        let field: YamlFieldDefinition = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(field, YamlFieldDefinition::Typed { .. }));

        // Array
        let yaml = r#"[easy, medium, hard]"#;
        let field: YamlFieldDefinition = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(field, YamlFieldDefinition::Array(_)));
    }
}
