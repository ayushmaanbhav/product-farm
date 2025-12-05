//! Rule validation service
//!
//! Provides comprehensive validation for rules before execution:
//! - JSON Logic expression syntax validation
//! - Dependency cycle detection
//! - Input/output attribute validation
//! - Missing dependency detection

use product_farm_core::Rule;
use product_farm_json_logic::parse;
use product_farm_rule_engine::RuleDag;
use std::collections::{HashMap, HashSet};

/// Validation result for a set of rules
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether all validations passed
    pub valid: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// List of validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Execution plan if valid
    pub execution_levels: Option<Vec<Vec<String>>>,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            execution_levels: None,
        }
    }

    fn add_error(&mut self, error: ValidationError) {
        self.valid = false;
        self.errors.push(error);
    }

    fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }
}

/// A validation error that prevents rule execution
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Rule ID that caused the error (if applicable)
    pub rule_id: Option<String>,
    /// Error code for categorization
    pub code: ValidationErrorCode,
    /// Human-readable error message
    pub message: String,
}

/// Validation error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationErrorCode {
    /// Invalid JSON Logic syntax
    InvalidSyntax,
    /// Circular dependency detected
    CyclicDependency,
    /// Missing required input
    MissingInput,
    /// Duplicate output attribute
    DuplicateOutput,
    /// Empty rule set
    EmptyRuleSet,
    /// Invalid rule configuration
    InvalidConfig,
}

/// A validation warning that doesn't prevent execution
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Rule ID that caused the warning (if applicable)
    pub rule_id: Option<String>,
    /// Warning code for categorization
    pub code: ValidationWarningCode,
    /// Human-readable warning message
    pub message: String,
}

/// Validation warning codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationWarningCode {
    /// Rule has no outputs defined
    NoOutputs,
    /// Rule has no inputs defined
    NoInputs,
    /// Unused rule output
    UnusedOutput,
    /// Disabled rule in chain
    DisabledRule,
}

/// Rule validator service
pub struct RuleValidator;

impl RuleValidator {
    /// Validate a set of rules
    pub fn validate(rules: &[Rule]) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Check for empty rule set
        if rules.is_empty() {
            result.add_error(ValidationError {
                rule_id: None,
                code: ValidationErrorCode::EmptyRuleSet,
                message: "No rules provided for validation".to_string(),
            });
            return result;
        }

        // Validate each rule's JSON Logic syntax
        Self::validate_syntax(rules, &mut result);

        // Check for duplicate outputs
        Self::check_duplicate_outputs(rules, &mut result);

        // Check for cycles and build execution plan
        Self::check_dependencies(rules, &mut result);

        // Check for warnings
        Self::check_warnings(rules, &mut result);

        result
    }

    /// Validate a single rule's JSON Logic expression
    pub fn validate_expression(rule: &Rule) -> Result<(), String> {
        let expr = rule.get_expression().map_err(|e| e.to_string())?;
        parse(&expr).map(|_| ()).map_err(|e| e.to_string())
    }

    fn validate_syntax(rules: &[Rule], result: &mut ValidationResult) {
        for rule in rules {
            match rule.get_expression() {
                Ok(expr) => {
                    if let Err(e) = parse(&expr) {
                        result.add_error(ValidationError {
                            rule_id: Some(rule.id.to_string()),
                            code: ValidationErrorCode::InvalidSyntax,
                            message: format!("Invalid JSON Logic syntax: {}", e),
                        });
                    }
                }
                Err(e) => {
                    result.add_error(ValidationError {
                        rule_id: Some(rule.id.to_string()),
                        code: ValidationErrorCode::InvalidSyntax,
                        message: format!("Invalid expression JSON: {}", e),
                    });
                }
            }
        }
    }

    fn check_duplicate_outputs(rules: &[Rule], result: &mut ValidationResult) {
        let mut output_to_rule: HashMap<String, String> = HashMap::new();

        for rule in rules {
            for output in &rule.output_attributes {
                let output_str = output.path.as_str().to_string();
                if let Some(existing_rule) = output_to_rule.get(&output_str) {
                    result.add_error(ValidationError {
                        rule_id: Some(rule.id.to_string()),
                        code: ValidationErrorCode::DuplicateOutput,
                        message: format!(
                            "Output '{}' is already produced by rule '{}'",
                            output_str, existing_rule
                        ),
                    });
                } else {
                    output_to_rule.insert(output_str, rule.id.to_string());
                }
            }
        }
    }

    fn check_dependencies(rules: &[Rule], result: &mut ValidationResult) {
        // Only check enabled rules
        let enabled_rules: Vec<_> = rules.iter().filter(|r| r.enabled).cloned().collect();

        if enabled_rules.is_empty() {
            return;
        }

        // Build DAG to check for cycles
        match RuleDag::from_rules(&enabled_rules) {
            Ok(dag) => {
                // Store execution levels
                if let Ok(levels) = dag.execution_levels() {
                    result.execution_levels = Some(
                        levels
                            .iter()
                            .map(|level| level.iter().map(|r| r.to_string()).collect())
                            .collect(),
                    );
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("cycle") || error_msg.contains("Cycle") {
                    result.add_error(ValidationError {
                        rule_id: None,
                        code: ValidationErrorCode::CyclicDependency,
                        message: error_msg,
                    });
                } else {
                    result.add_error(ValidationError {
                        rule_id: None,
                        code: ValidationErrorCode::InvalidConfig,
                        message: error_msg,
                    });
                }
            }
        }
    }

    fn check_warnings(rules: &[Rule], result: &mut ValidationResult) {
        // Collect all outputs for unused output detection
        let mut all_inputs: HashSet<&str> = HashSet::new();
        let mut all_outputs: HashSet<&str> = HashSet::new();

        for rule in rules {
            for input in &rule.input_attributes {
                all_inputs.insert(input.path.as_str());
            }
            for output in &rule.output_attributes {
                all_outputs.insert(output.path.as_str());
            }
        }

        for rule in rules {
            // Check for no outputs
            if rule.output_attributes.is_empty() {
                result.add_warning(ValidationWarning {
                    rule_id: Some(rule.id.to_string()),
                    code: ValidationWarningCode::NoOutputs,
                    message: "Rule has no output attributes defined".to_string(),
                });
            }

            // Check for no inputs
            if rule.input_attributes.is_empty() {
                result.add_warning(ValidationWarning {
                    rule_id: Some(rule.id.to_string()),
                    code: ValidationWarningCode::NoInputs,
                    message: "Rule has no input attributes defined".to_string(),
                });
            }

            // Check for disabled rules
            if !rule.enabled {
                result.add_warning(ValidationWarning {
                    rule_id: Some(rule.id.to_string()),
                    code: ValidationWarningCode::DisabledRule,
                    message: "Rule is disabled and will not be executed".to_string(),
                });
            }

            // Check for unused outputs (outputs not used by any other rule)
            for output in &rule.output_attributes {
                if !all_inputs.contains(output.path.as_str()) {
                    // This is a terminal output, which is fine but worth noting
                    // Don't add warning for now as terminal outputs are expected
                }
            }
        }
    }
}

/// Find missing inputs that are not provided by any rule output
pub fn find_missing_inputs(rules: &[Rule], provided_inputs: &[&str]) -> Vec<String> {
    let mut all_outputs: HashSet<&str> = HashSet::new();
    let mut all_inputs: HashSet<&str> = HashSet::new();

    // Add provided inputs
    for input in provided_inputs {
        all_outputs.insert(input);
    }

    // Collect all rule outputs and inputs
    for rule in rules {
        if !rule.enabled {
            continue;
        }
        for output in &rule.output_attributes {
            all_outputs.insert(output.path.as_str());
        }
        for input in &rule.input_attributes {
            all_inputs.insert(input.path.as_str());
        }
    }

    // Find inputs not satisfied by outputs or provided inputs
    all_inputs
        .difference(&all_outputs)
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_rules() {
        let rules = vec![
            Rule::from_json_logic("product1", "calc", json!({"*": [{"var": "x"}, 2]}))
                .with_inputs(["x"])
                .with_outputs(["doubled"]),
            Rule::from_json_logic("product1", "calc", json!({"+": [{"var": "doubled"}, 10]}))
                .with_inputs(["doubled"])
                .with_outputs(["result"]),
        ];

        let result = RuleValidator::validate(&rules);
        assert!(result.valid, "Errors: {:?}", result.errors);
        assert!(result.execution_levels.is_some());
        assert_eq!(result.execution_levels.unwrap().len(), 2);
    }

    #[test]
    fn test_invalid_syntax() {
        let rules = vec![Rule::from_json_logic(
            "product1",
            "calc",
            json!({"invalid_op": [1, 2]}),
        )
        .with_inputs(["x"])
        .with_outputs(["result"])];

        let result = RuleValidator::validate(&rules);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == ValidationErrorCode::InvalidSyntax));
    }

    #[test]
    fn test_duplicate_outputs() {
        let rules = vec![
            Rule::from_json_logic("product1", "calc", json!({"var": "x"}))
                .with_inputs(["x"])
                .with_outputs(["result"]),
            Rule::from_json_logic("product1", "calc", json!({"var": "y"}))
                .with_inputs(["y"])
                .with_outputs(["result"]), // Duplicate!
        ];

        let result = RuleValidator::validate(&rules);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == ValidationErrorCode::DuplicateOutput));
    }

    #[test]
    fn test_cyclic_dependency() {
        let rules = vec![
            Rule::from_json_logic("product1", "calc", json!({"var": "b"}))
                .with_inputs(["b"])
                .with_outputs(["a"]),
            Rule::from_json_logic("product1", "calc", json!({"var": "a"}))
                .with_inputs(["a"])
                .with_outputs(["b"]),
        ];

        let result = RuleValidator::validate(&rules);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == ValidationErrorCode::CyclicDependency));
    }

    #[test]
    fn test_disabled_rule_warning() {
        let rules = vec![Rule::from_json_logic("product1", "calc", json!({"var": "x"}))
            .with_inputs(["x"])
            .with_outputs(["result"])
            .disabled()];

        let result = RuleValidator::validate(&rules);
        assert!(result.valid); // Disabled rules don't cause errors
        assert!(result
            .warnings
            .iter()
            .any(|w| w.code == ValidationWarningCode::DisabledRule));
    }

    #[test]
    fn test_find_missing_inputs() {
        let rules = vec![
            Rule::from_json_logic("product1", "calc", json!({"var": "a"}))
                .with_inputs(["a"])
                .with_outputs(["b"]),
            Rule::from_json_logic("product1", "calc", json!({"var": "b"}))
                .with_inputs(["b", "c"]) // c is not provided
                .with_outputs(["d"]),
        ];

        let missing = find_missing_inputs(&rules, &["a"]);
        assert_eq!(missing.len(), 1);
        assert!(missing.contains(&"c".to_string()));
    }

    #[test]
    fn test_empty_rules() {
        let result = RuleValidator::validate(&[]);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == ValidationErrorCode::EmptyRuleSet));
    }
}
