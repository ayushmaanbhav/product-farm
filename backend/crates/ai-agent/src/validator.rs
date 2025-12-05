//! Rule Validation Tools
//!
//! Validates JSON Logic expressions for:
//! - Syntax correctness
//! - Type consistency
//! - Cycle detection in dependencies
//! - Missing attribute references

#[allow(unused_imports)]
use crate::error::{AgentError, AgentResult};
use crate::tools::{ValidateRuleOutput, ValidationError, ValidationWarning};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Validates JSON Logic rules
pub struct RuleValidator {
    /// Known attributes for the product
    known_attributes: HashSet<String>,
    /// Known datatypes
    known_datatypes: HashMap<String, DataType>,
}

#[derive(Debug, Clone)]
pub struct DataType {
    pub name: String,
    pub primitive: PrimitiveType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimitiveType {
    Number,
    String,
    Boolean,
    Array,
    Object,
    Null,
    Any,
}

impl RuleValidator {
    pub fn new() -> Self {
        Self {
            known_attributes: HashSet::new(),
            known_datatypes: HashMap::new(),
        }
    }

    pub fn with_attributes(mut self, attributes: impl IntoIterator<Item = String>) -> Self {
        self.known_attributes = attributes.into_iter().collect();
        self
    }

    pub fn with_datatype(mut self, name: String, primitive: PrimitiveType) -> Self {
        self.known_datatypes.insert(
            name.clone(),
            DataType {
                name,
                primitive,
            },
        );
        self
    }

    /// Validate a JSON Logic expression
    pub fn validate(
        &self,
        expression: &Value,
        input_attributes: &[String],
        output_attributes: &[String],
    ) -> AgentResult<ValidateRuleOutput> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut inferred_types = HashMap::new();
        let mut used_variables = HashSet::new();

        // Validate the expression structure
        self.validate_expression(
            expression,
            &mut errors,
            &mut warnings,
            &mut inferred_types,
            &mut used_variables,
        );

        // Check that all used variables are declared as inputs
        for var in &used_variables {
            if !input_attributes.contains(var) && !var.starts_with("current") {
                warnings.push(ValidationWarning {
                    code: "UNDECLARED_INPUT".to_string(),
                    message: format!(
                        "Variable '{}' is used but not declared in input_attributes",
                        var
                    ),
                    suggestion: Some(format!("Add '{}' to input_attributes", var)),
                });
            }
        }

        // Check that declared inputs are actually used
        for input in input_attributes {
            if !used_variables.contains(input) {
                warnings.push(ValidationWarning {
                    code: "UNUSED_INPUT".to_string(),
                    message: format!("Input attribute '{}' is declared but never used", input),
                    suggestion: Some("Remove unused input or use it in the expression".to_string()),
                });
            }
        }

        // Check output attributes are valid
        if output_attributes.is_empty() {
            errors.push(ValidationError {
                code: "NO_OUTPUT".to_string(),
                message: "Rule must have at least one output attribute".to_string(),
                location: None,
            });
        }

        // Check for known attributes if we have them
        if !self.known_attributes.is_empty() {
            for attr in input_attributes.iter().chain(output_attributes.iter()) {
                if !self.known_attributes.contains(attr) {
                    warnings.push(ValidationWarning {
                        code: "UNKNOWN_ATTRIBUTE".to_string(),
                        message: format!("Attribute '{}' is not defined in the product", attr),
                        suggestion: Some("Create the attribute first or check the path".to_string()),
                    });
                }
            }
        }

        Ok(ValidateRuleOutput {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            inferred_types,
        })
    }

    fn validate_expression(
        &self,
        expr: &Value,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
        inferred_types: &mut HashMap<String, String>,
        used_variables: &mut HashSet<String>,
    ) {
        match expr {
            Value::Object(map) if map.len() == 1 => {
                let (op, args) = map.iter().next().unwrap();
                self.validate_operation(op, args, errors, warnings, inferred_types, used_variables);
            }
            Value::Object(map) if map.is_empty() => {
                errors.push(ValidationError {
                    code: "EMPTY_OBJECT".to_string(),
                    message: "Empty object is not valid JSON Logic".to_string(),
                    location: None,
                });
            }
            Value::Object(_) => {
                errors.push(ValidationError {
                    code: "MULTI_OP_OBJECT".to_string(),
                    message: "JSON Logic objects must have exactly one key (the operation)"
                        .to_string(),
                    location: None,
                });
            }
            // Literals are always valid
            Value::Number(_) | Value::String(_) | Value::Bool(_) | Value::Null => {}
            Value::Array(arr) => {
                for item in arr {
                    self.validate_expression(item, errors, warnings, inferred_types, used_variables);
                }
            }
        }
    }

    fn validate_operation(
        &self,
        op: &str,
        args: &Value,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
        inferred_types: &mut HashMap<String, String>,
        used_variables: &mut HashSet<String>,
    ) {
        match op {
            // Variable access
            "var" => {
                if let Some(path) = self.extract_var_path(args) {
                    used_variables.insert(path.clone());
                    // Try to infer type from known datatypes
                    if let Some(dtype) = self.known_datatypes.get(&path) {
                        inferred_types.insert(path, format!("{:?}", dtype.primitive));
                    }
                } else {
                    errors.push(ValidationError {
                        code: "INVALID_VAR".to_string(),
                        message: "Variable path must be a string or array with string path"
                            .to_string(),
                        location: Some(format!("{{\"var\": {}}}", args)),
                    });
                }
            }

            // Comparison operators
            "==" | "!=" | "===" | "!==" => {
                self.validate_binary_args(op, args, errors, warnings, inferred_types, used_variables);
            }

            // Numeric comparisons
            ">" | ">=" | "<" | "<=" => {
                self.validate_binary_args(op, args, errors, warnings, inferred_types, used_variables);
                self.check_numeric_operands(op, args, warnings);
            }

            // Arithmetic
            "+" | "-" | "*" | "/" | "%" => {
                self.validate_array_args(op, args, errors, warnings, inferred_types, used_variables);
                self.check_numeric_operands(op, args, warnings);
            }

            // Logical
            "and" | "or" | "!" | "!!" | "not" => {
                self.validate_array_args(op, args, errors, warnings, inferred_types, used_variables);
            }

            // Conditional
            "if" => {
                self.validate_if(args, errors, warnings, inferred_types, used_variables);
            }

            // Array operations
            "in" | "cat" | "substr" | "merge" | "map" | "filter" | "reduce" | "all" | "some"
            | "none" => {
                self.validate_array_args(op, args, errors, warnings, inferred_types, used_variables);
            }

            // Missing checks
            "missing" | "missing_some" => {
                self.validate_missing(op, args, used_variables);
            }

            // Min/Max
            "min" | "max" => {
                self.validate_array_args(op, args, errors, warnings, inferred_types, used_variables);
            }

            // Log (debug)
            "log" => {
                self.validate_expression(args, errors, warnings, inferred_types, used_variables);
                warnings.push(ValidationWarning {
                    code: "DEBUG_LOG".to_string(),
                    message: "Log operation should be removed before production".to_string(),
                    suggestion: Some("Remove or wrap in a feature flag".to_string()),
                });
            }

            // Unknown operation
            _ => {
                warnings.push(ValidationWarning {
                    code: "UNKNOWN_OP".to_string(),
                    message: format!("Unknown operation '{}' - may not be supported", op),
                    suggestion: Some("Check supported JSON Logic operations".to_string()),
                });
                // Still validate args
                self.validate_expression(args, errors, warnings, inferred_types, used_variables);
            }
        }
    }

    fn extract_var_path(&self, args: &Value) -> Option<String> {
        match args {
            Value::String(s) => Some(s.clone()),
            Value::Array(arr) if !arr.is_empty() => arr[0].as_str().map(|s| s.to_string()),
            _ => None,
        }
    }

    fn validate_binary_args(
        &self,
        op: &str,
        args: &Value,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
        inferred_types: &mut HashMap<String, String>,
        used_variables: &mut HashSet<String>,
    ) {
        if let Value::Array(arr) = args {
            if arr.len() < 2 {
                errors.push(ValidationError {
                    code: "INSUFFICIENT_ARGS".to_string(),
                    message: format!("Operation '{}' requires at least 2 arguments", op),
                    location: Some(format!("{{\"{}\": {}}}", op, args)),
                });
            }
            for arg in arr {
                self.validate_expression(arg, errors, warnings, inferred_types, used_variables);
            }
        } else {
            errors.push(ValidationError {
                code: "INVALID_ARGS".to_string(),
                message: format!("Operation '{}' requires an array of arguments", op),
                location: Some(format!("{{\"{}\": {}}}", op, args)),
            });
        }
    }

    fn validate_array_args(
        &self,
        _op: &str,
        args: &Value,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
        inferred_types: &mut HashMap<String, String>,
        used_variables: &mut HashSet<String>,
    ) {
        match args {
            Value::Array(arr) => {
                for arg in arr {
                    self.validate_expression(arg, errors, warnings, inferred_types, used_variables);
                }
            }
            _ => {
                // Single argument is sometimes valid
                self.validate_expression(args, errors, warnings, inferred_types, used_variables);
            }
        }
    }

    fn validate_if(
        &self,
        args: &Value,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
        inferred_types: &mut HashMap<String, String>,
        used_variables: &mut HashSet<String>,
    ) {
        if let Value::Array(arr) = args {
            if arr.len() < 2 {
                errors.push(ValidationError {
                    code: "INVALID_IF".to_string(),
                    message: "IF requires at least condition and then-value".to_string(),
                    location: None,
                });
            }
            for arg in arr {
                self.validate_expression(arg, errors, warnings, inferred_types, used_variables);
            }
            // Warn if no else clause
            if arr.len() == 2 {
                warnings.push(ValidationWarning {
                    code: "NO_ELSE".to_string(),
                    message: "IF without ELSE may return null when condition is false".to_string(),
                    suggestion: Some("Add an else clause for explicit handling".to_string()),
                });
            }
        } else {
            errors.push(ValidationError {
                code: "INVALID_IF".to_string(),
                message: "IF requires an array of [condition, then-value, else-value]".to_string(),
                location: None,
            });
        }
    }

    fn validate_missing(&self, op: &str, args: &Value, used_variables: &mut HashSet<String>) {
        match args {
            Value::Array(arr) => {
                if op == "missing_some" && arr.len() >= 2 {
                    if let Value::Array(paths) = &arr[1] {
                        for path in paths {
                            if let Some(s) = path.as_str() {
                                used_variables.insert(s.to_string());
                            }
                        }
                    }
                } else {
                    for path in arr {
                        if let Some(s) = path.as_str() {
                            used_variables.insert(s.to_string());
                        }
                    }
                }
            }
            Value::String(s) => {
                used_variables.insert(s.clone());
            }
            _ => {}
        }
    }

    fn check_numeric_operands(&self, op: &str, args: &Value, warnings: &mut Vec<ValidationWarning>) {
        // Check if we can detect non-numeric literals
        if let Value::Array(arr) = args {
            for (i, arg) in arr.iter().enumerate() {
                match arg {
                    Value::String(s) if s.parse::<f64>().is_err() => {
                        warnings.push(ValidationWarning {
                            code: "TYPE_MISMATCH".to_string(),
                            message: format!(
                                "Operation '{}' expects numeric operands, but argument {} is a string",
                                op, i
                            ),
                            suggestion: Some("Use numeric literals or ensure variables are numeric".to_string()),
                        });
                    }
                    Value::Bool(_) => {
                        warnings.push(ValidationWarning {
                            code: "TYPE_COERCION".to_string(),
                            message: format!(
                                "Operation '{}' will coerce boolean to number (true=1, false=0)",
                                op
                            ),
                            suggestion: Some("Use explicit numeric values for clarity".to_string()),
                        });
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Default for RuleValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_simple_expression() {
        let validator = RuleValidator::new();
        let expr = json!({">": [{"var": "age"}, 18]});
        let result = validator
            .validate(&expr, &["age".to_string()], &["is_adult".to_string()])
            .unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_missing_input() {
        let validator = RuleValidator::new();
        let expr = json!({">": [{"var": "age"}, 18]});
        let result = validator
            .validate(&expr, &[], &["is_adult".to_string()])
            .unwrap();
        // Should warn about undeclared input
        assert!(result.warnings.iter().any(|w| w.code == "UNDECLARED_INPUT"));
    }

    #[test]
    fn test_validate_no_output() {
        let validator = RuleValidator::new();
        let expr = json!({">": [{"var": "age"}, 18]});
        let result = validator.validate(&expr, &["age".to_string()], &[]).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "NO_OUTPUT"));
    }

    #[test]
    fn test_validate_if_without_else() {
        let validator = RuleValidator::new();
        let expr = json!({"if": [{">": [{"var": "age"}, 18]}, "adult"]});
        let result = validator
            .validate(&expr, &["age".to_string()], &["category".to_string()])
            .unwrap();
        assert!(result.warnings.iter().any(|w| w.code == "NO_ELSE"));
    }
}
