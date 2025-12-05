//! Rule definitions - JSON Logic expressions with input/output mappings
//!
//! ## Rule Types (Dynamically Defined)
//!
//! Rule types are dynamically defined strings, not hardcoded enums.
//! Examples: "premium-calculation", "entry-logic", "risk-assessment"
//!
//! ## Display Expression Versioning
//!
//! The `display_expression_version` field tracks the version of the display expression
//! format, allowing for backwards-compatible changes to display formats.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{validation, AbstractPath, CoreError, CoreResult, ProductId, RuleId};

/// Input attribute reference with ordering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleInputAttribute {
    /// Rule ID this belongs to
    pub rule_id: RuleId,
    /// Path to the input attribute
    pub path: AbstractPath,
    /// Order index for this input
    pub order: i32,
}

impl RuleInputAttribute {
    pub fn new(rule_id: RuleId, path: impl Into<AbstractPath>, order: i32) -> Self {
        Self {
            rule_id,
            path: path.into(),
            order,
        }
    }
}

/// Output attribute reference with ordering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleOutputAttribute {
    /// Rule ID this belongs to
    pub rule_id: RuleId,
    /// Path to the output attribute
    pub path: AbstractPath,
    /// Order index for this output
    pub order: i32,
}

impl RuleOutputAttribute {
    pub fn new(rule_id: RuleId, path: impl Into<AbstractPath>, order: i32) -> Self {
        Self {
            rule_id,
            path: path.into(),
            order,
        }
    }
}

/// A rule definition that computes output attributes from input attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique rule identifier (UUID format, no hyphens)
    pub id: RuleId,
    /// Product this rule belongs to
    pub product_id: ProductId,
    /// Rule type/category (e.g., "premium-calculation", "entry-logic")
    /// This is dynamically defined, must match RULE_TYPE_REGEX
    pub rule_type: String,
    /// Input attribute paths (ordered by evaluation order)
    pub input_attributes: Vec<RuleInputAttribute>,
    /// Output attribute paths (ordered)
    pub output_attributes: Vec<RuleOutputAttribute>,
    /// Human-readable expression (for display in UI)
    pub display_expression: String,
    /// Version of the display expression format (for backwards compatibility)
    pub display_expression_version: String,
    /// Compiled JSON Logic expression (stored as JSON string)
    pub compiled_expression: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Whether this rule is enabled (default: true)
    pub enabled: bool,
    /// Order index for rule execution priority (lower = earlier)
    pub order_index: i32,
    /// When the rule was created
    pub created_at: DateTime<Utc>,
    /// When the rule was last updated
    pub updated_at: DateTime<Utc>,
}

impl Rule {
    /// Create a new rule
    pub fn new(
        product_id: impl Into<ProductId>,
        rule_type: impl Into<String>,
        compiled_expression: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: RuleId::new(),
            product_id: product_id.into(),
            rule_type: rule_type.into(),
            input_attributes: Vec::new(),
            output_attributes: Vec::new(),
            display_expression: String::new(),
            display_expression_version: "1.0".to_string(),
            compiled_expression: compiled_expression.into(),
            description: None,
            enabled: true,
            order_index: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create from a JSON Logic value
    pub fn from_json_logic(
        product_id: impl Into<ProductId>,
        rule_type: impl Into<String>,
        expression: serde_json::Value,
    ) -> Self {
        Self::new(
            product_id,
            rule_type,
            serde_json::to_string(&expression).unwrap_or_default(),
        )
    }

    /// Set the rule ID (for loading from storage)
    pub fn with_id(mut self, id: RuleId) -> Self {
        self.id = id;
        self
    }

    /// Add input attributes from paths
    pub fn with_inputs(
        mut self,
        inputs: impl IntoIterator<Item = impl Into<AbstractPath>>,
    ) -> Self {
        let rule_id = self.id.clone();
        self.input_attributes = inputs
            .into_iter()
            .enumerate()
            .map(|(i, path)| {
                // Use saturating conversion to prevent wrap-around
                let order = i.min(i32::MAX as usize) as i32;
                RuleInputAttribute::new(rule_id.clone(), path, order)
            })
            .collect();
        self
    }

    /// Add output attributes from paths
    pub fn with_outputs(
        mut self,
        outputs: impl IntoIterator<Item = impl Into<AbstractPath>>,
    ) -> Self {
        let rule_id = self.id.clone();
        self.output_attributes = outputs
            .into_iter()
            .enumerate()
            .map(|(i, path)| {
                // Use saturating conversion to prevent wrap-around
                let order = i.min(i32::MAX as usize) as i32;
                RuleOutputAttribute::new(rule_id.clone(), path, order)
            })
            .collect();
        self
    }

    /// Set display expression
    pub fn with_display(mut self, display: impl Into<String>) -> Self {
        self.display_expression = display.into();
        self
    }

    /// Set display expression version
    pub fn with_display_version(mut self, version: impl Into<String>) -> Self {
        self.display_expression_version = version.into();
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set order index for execution priority
    pub fn with_order(mut self, order: i32) -> Self {
        self.order_index = order;
        self
    }

    /// Mark rule as disabled
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Mark rule as enabled
    pub fn enabled(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Get the number of input attributes
    pub fn input_count(&self) -> usize {
        self.input_attributes.len()
    }

    /// Get the number of output attributes
    pub fn output_count(&self) -> usize {
        self.output_attributes.len()
    }

    /// Check if rule depends on a specific attribute
    pub fn depends_on(&self, attr: &AbstractPath) -> bool {
        self.input_attributes.iter().any(|a| &a.path == attr)
    }

    /// Check if rule produces a specific attribute
    pub fn produces(&self, attr: &AbstractPath) -> bool {
        self.output_attributes.iter().any(|a| &a.path == attr)
    }

    /// Get the compiled expression as a JSON Value
    pub fn get_expression(&self) -> CoreResult<serde_json::Value> {
        serde_json::from_str(&self.compiled_expression)
            .map_err(|e| CoreError::SerializationError(e.to_string()))
    }

    /// Validate the rule
    pub fn validate(&self) -> CoreResult<()> {
        // Validate rule type
        if !validation::is_valid_rule_type(&self.rule_type) {
            return Err(CoreError::ValidationFailed {
                field: "rule_type".to_string(),
                message: format!(
                    "Rule type '{}' does not match required pattern",
                    self.rule_type
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

        // Validate that compiled expression is valid JSON
        if self.compiled_expression.is_empty() {
            return Err(CoreError::ValidationFailed {
                field: "compiled_expression".to_string(),
                message: "Compiled expression cannot be empty".to_string(),
            });
        }

        self.get_expression().map_err(|_| CoreError::ValidationFailed {
            field: "compiled_expression".to_string(),
            message: "Compiled expression is not valid JSON".to_string(),
        })?;

        // Validate that rule has at least one output
        if self.output_attributes.is_empty() {
            return Err(CoreError::ValidationFailed {
                field: "output_attributes".to_string(),
                message: "Rule must have at least one output attribute".to_string(),
            });
        }

        Ok(())
    }
}

/// Builder for creating rules with a fluent API
pub struct RuleBuilder {
    product_id: ProductId,
    rule_type: String,
    inputs: Vec<AbstractPath>,
    outputs: Vec<AbstractPath>,
    display_expression: String,
    display_expression_version: String,
    expression: Option<serde_json::Value>,
    description: Option<String>,
    enabled: bool,
    order_index: i32,
}

impl RuleBuilder {
    /// Start building a rule
    pub fn new(product_id: impl Into<ProductId>, rule_type: impl Into<String>) -> Self {
        Self {
            product_id: product_id.into(),
            rule_type: rule_type.into(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            display_expression: String::new(),
            display_expression_version: "1.0".to_string(),
            expression: None,
            description: None,
            enabled: true,
            order_index: 0,
        }
    }

    /// Add an input attribute
    pub fn input(mut self, attr: impl Into<AbstractPath>) -> Self {
        self.inputs.push(attr.into());
        self
    }

    /// Add an output attribute
    pub fn output(mut self, attr: impl Into<AbstractPath>) -> Self {
        self.outputs.push(attr.into());
        self
    }

    /// Set display expression
    pub fn display(mut self, expr: impl Into<String>) -> Self {
        self.display_expression = expr.into();
        self
    }

    /// Set display expression version
    pub fn display_version(mut self, version: impl Into<String>) -> Self {
        self.display_expression_version = version.into();
        self
    }

    /// Set JSON Logic expression
    pub fn expression(mut self, expr: serde_json::Value) -> Self {
        self.expression = Some(expr);
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set order index
    pub fn order(mut self, order: i32) -> Self {
        self.order_index = order;
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Build the rule
    pub fn build(self) -> CoreResult<Rule> {
        let expression = self.expression.ok_or_else(|| CoreError::ValidationFailed {
            field: "expression".to_string(),
            message: "Rule expression is required".to_string(),
        })?;
        let rule_id = RuleId::new();

        // Validate attribute counts don't exceed i32::MAX
        if self.inputs.len() > i32::MAX as usize {
            return Err(CoreError::ValidationFailed {
                field: "inputs".to_string(),
                message: format!("Too many input attributes: {}", self.inputs.len()),
            });
        }
        if self.outputs.len() > i32::MAX as usize {
            return Err(CoreError::ValidationFailed {
                field: "outputs".to_string(),
                message: format!("Too many output attributes: {}", self.outputs.len()),
            });
        }

        // Serialize expression
        let compiled_expression = serde_json::to_string(&expression).map_err(|e| {
            CoreError::SerializationError(format!("Failed to serialize rule expression: {}", e))
        })?;

        Ok(Rule {
            id: rule_id.clone(),
            product_id: self.product_id,
            rule_type: self.rule_type,
            input_attributes: self.inputs
                .into_iter()
                .enumerate()
                .map(|(i, path)| {
                    // Safe cast - we validated above that len <= i32::MAX
                    RuleInputAttribute::new(rule_id.clone(), path, i as i32)
                })
                .collect(),
            output_attributes: self.outputs
                .into_iter()
                .enumerate()
                .map(|(i, path)| {
                    // Safe cast - we validated above that len <= i32::MAX
                    RuleOutputAttribute::new(rule_id.clone(), path, i as i32)
                })
                .collect(),
            display_expression: self.display_expression,
            display_expression_version: self.display_expression_version,
            compiled_expression,
            description: self.description,
            enabled: self.enabled,
            order_index: self.order_index,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_rule_creation() {
        let expr = json!({
            "if": [
                {"<": [{"var": "rsi_14"}, 30]},
                "BUY",
                "HOLD"
            ]
        });

        let rule = Rule::from_json_logic(
            "momentumStrategyV1",
            "entry-logic",
            expr,
        )
        .with_inputs(["myProduct:abstract-path:indicator:rsi"])
        .with_outputs(["myProduct:abstract-path:signal:entry"])
        .with_display("BUY when RSI < 30, else HOLD")
        .with_description("RSI oversold entry signal");

        assert_eq!(rule.rule_type, "entry-logic");
        assert_eq!(rule.input_count(), 1);
        assert_eq!(rule.output_count(), 1);
        assert!(rule.depends_on(&AbstractPath::new("myProduct:abstract-path:indicator:rsi")));
        assert!(rule.produces(&AbstractPath::new("myProduct:abstract-path:signal:entry")));
        assert_eq!(rule.display_expression_version, "1.0");
    }

    #[test]
    fn test_rule_builder() {
        let rule = RuleBuilder::new("testProduct", "premium-calculation")
            .input("myProduct:abstract-path:cover:base-rate")
            .input("myProduct:abstract-path:customer:age")
            .output("myProduct:abstract-path:premium:amount")
            .display("base_rate * age_factor")
            .display_version("2.0")
            .expression(json!({
                "*": [
                    {"var": "base_rate"},
                    {"if": [
                        {">": [{"var": "age"}, 60]},
                        1.2,
                        1.0
                    ]}
                ]
            }))
            .description("Premium calculation with age loading")
            .build();

        assert!(rule.is_ok());
        let rule = rule.unwrap();
        assert_eq!(rule.input_count(), 2);
        assert_eq!(rule.output_count(), 1);
        assert_eq!(rule.display_expression_version, "2.0");
    }

    #[test]
    fn test_rule_validation() {
        // Valid rule
        let rule = RuleBuilder::new("testProduct", "calculation")
            .output("myProduct:abstract-path:output:value")
            .expression(json!({"var": "input"}))
            .build()
            .unwrap();

        assert!(rule.validate().is_ok());

        // Invalid rule type (uppercase)
        let rule = Rule::from_json_logic("testProduct", "INVALID_TYPE", json!({}))
            .with_outputs(["myProduct:abstract-path:output:value"]);

        assert!(rule.validate().is_err());
    }

    #[test]
    fn test_rule_get_expression() {
        let original = json!({"+": [1, 2]});
        let rule = Rule::from_json_logic("testProduct", "test", original.clone())
            .with_outputs(["myProduct:abstract-path:output:value"]);

        let retrieved = rule.get_expression().unwrap();
        assert_eq!(retrieved, original);
    }

    #[test]
    fn test_rule_input_output_ordering() {
        let rule = RuleBuilder::new("testProduct", "test")
            .input("myProduct:abstract-path:a:first")
            .input("myProduct:abstract-path:b:second")
            .input("myProduct:abstract-path:c:third")
            .output("myProduct:abstract-path:out:result")
            .expression(json!({}))
            .build()
            .unwrap();

        assert_eq!(rule.input_attributes[0].order, 0);
        assert_eq!(rule.input_attributes[1].order, 1);
        assert_eq!(rule.input_attributes[2].order, 2);
        assert_eq!(rule.output_attributes[0].order, 0);
    }
}
