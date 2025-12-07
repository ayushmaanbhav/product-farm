//! Custom Assertion Helpers
//!
//! Provides domain-specific assertions for test readability.

use product_farm_api::rest::types::{
    AbstractAttributeResponse, AttributeResponse, AttributeValueJson, DatatypeResponse,
    EnumerationResponse, EvaluateResponse, FunctionalityResponse, ProductResponse,
    RuleResponse, RuleResultJson, ValidationResponse,
};

// =============================================================================
// Product Assertions
// =============================================================================

/// Assert product response matches expected values
pub fn assert_product_fields(
    product: &ProductResponse,
    expected_id: &str,
    expected_name: &str,
    expected_template: &str,
) {
    assert_eq!(product.id, expected_id, "Product ID mismatch");
    assert_eq!(product.name, expected_name, "Product name mismatch");
    assert_eq!(product.template_type, expected_template, "Product template mismatch");
}

/// Assert product status
pub fn assert_product_status(product: &ProductResponse, expected_status: &str) {
    assert_eq!(
        product.status, expected_status,
        "Product status mismatch: expected '{}', got '{}'",
        expected_status, product.status
    );
}

/// Assert product is in DRAFT status
pub fn assert_product_draft(product: &ProductResponse) {
    assert_product_status(product, "DRAFT");
}

/// Assert product version
pub fn assert_product_version(product: &ProductResponse, expected_version: i64) {
    assert_eq!(
        product.version, expected_version,
        "Product version mismatch: expected {}, got {}",
        expected_version, product.version
    );
}

/// Assert product timestamps are valid
pub fn assert_product_timestamps_valid(product: &ProductResponse) {
    assert!(product.created_at > 0, "created_at should be positive");
    assert!(product.updated_at > 0, "updated_at should be positive");
    assert!(
        product.updated_at >= product.created_at,
        "updated_at should be >= created_at"
    );
}

/// Assert product has parent
pub fn assert_product_has_parent(product: &ProductResponse, expected_parent: &str) {
    assert_eq!(
        product.parent_product_id.as_deref(),
        Some(expected_parent),
        "Product parent mismatch"
    );
}

// =============================================================================
// Attribute Assertions
// =============================================================================

/// Assert abstract attribute path format
pub fn assert_abstract_path_format(attr: &AbstractAttributeResponse) {
    assert!(!attr.abstract_path.is_empty(), "Abstract path should not be empty");
    assert!(
        attr.abstract_path.contains(':'),
        "Abstract path should contain ':' separator"
    );
}

/// Assert abstract attribute path matches expected (string form)
pub fn assert_abstract_attribute_path_str(attr: &AbstractAttributeResponse, expected_path: &str) {
    assert_eq!(
        attr.abstract_path, expected_path,
        "Abstract path mismatch: expected '{}', got '{}'",
        expected_path, attr.abstract_path
    );
}

/// Assert abstract attribute path components match expected
pub fn assert_abstract_attribute_path(attr: &AbstractAttributeResponse, expected_parts: &[&str]) {
    // Build expected path from parts (separated by ':' or '.')
    let expected = expected_parts.join(":");

    // Check if the path contains the expected parts (flexible matching)
    let path_parts: Vec<&str> = attr.abstract_path.split(|c| c == ':' || c == '.').collect();

    for part in expected_parts {
        assert!(
            path_parts.contains(part),
            "Abstract path should contain '{}', got '{}'",
            part, attr.abstract_path
        );
    }
}

/// Assert attribute has specific tag
pub fn assert_attribute_has_tag(attr: &AbstractAttributeResponse, tag_name: &str) {
    let has_tag = attr.tags.iter().any(|t| t.name == tag_name);
    assert!(has_tag, "Attribute should have tag '{}'", tag_name);
}

/// Assert attribute is immutable
pub fn assert_attribute_immutable(attr: &AbstractAttributeResponse) {
    assert!(attr.immutable, "Attribute should be immutable");
}

/// Assert attribute is mutable
pub fn assert_attribute_mutable(attr: &AbstractAttributeResponse) {
    assert!(!attr.immutable, "Attribute should be mutable");
}

/// Assert concrete attribute value type
pub fn assert_attribute_value_type(attr: &AttributeResponse, expected_type: &str) {
    assert_eq!(
        attr.value_type, expected_type,
        "Attribute value type mismatch: expected '{}', got '{}'",
        expected_type, attr.value_type
    );
}

/// Assert concrete attribute has value
pub fn assert_attribute_has_value(attr: &AttributeResponse) {
    assert!(attr.value.is_some(), "Attribute should have a value");
}

/// Assert concrete attribute has rule
pub fn assert_attribute_has_rule(attr: &AttributeResponse, rule_id: &str) {
    assert_eq!(
        attr.rule_id.as_deref(),
        Some(rule_id),
        "Attribute rule_id mismatch"
    );
}

// =============================================================================
// Rule Assertions
// =============================================================================

/// Assert rule inputs match expected
pub fn assert_rule_inputs(rule: &RuleResponse, expected: &[&str]) {
    let actual: Vec<&str> = rule.input_attributes.iter().map(|a| a.attribute_path.as_str()).collect();
    assert_eq!(
        actual.len(),
        expected.len(),
        "Rule input count mismatch: expected {:?}, got {:?}",
        expected,
        actual
    );
    for exp in expected {
        assert!(
            actual.contains(exp),
            "Rule should have input '{}', got {:?}",
            exp,
            actual
        );
    }
}

/// Alias for assert_rule_inputs
pub fn assert_rule_inputs_match(rule: &RuleResponse, expected: &[&str]) {
    assert_rule_inputs(rule, expected);
}

/// Assert rule outputs match expected
pub fn assert_rule_outputs(rule: &RuleResponse, expected: &[&str]) {
    let actual: Vec<&str> = rule.output_attributes.iter().map(|a| a.attribute_path.as_str()).collect();
    assert_eq!(
        actual.len(),
        expected.len(),
        "Rule output count mismatch: expected {:?}, got {:?}",
        expected,
        actual
    );
    for exp in expected {
        assert!(
            actual.contains(exp),
            "Rule should have output '{}', got {:?}",
            exp,
            actual
        );
    }
}

/// Alias for assert_rule_outputs
pub fn assert_rule_outputs_match(rule: &RuleResponse, expected: &[&str]) {
    assert_rule_outputs(rule, expected);
}

/// Assert rule is enabled
pub fn assert_rule_enabled(rule: &RuleResponse) {
    assert!(rule.enabled, "Rule should be enabled");
}

/// Assert rule is disabled
pub fn assert_rule_disabled(rule: &RuleResponse) {
    assert!(!rule.enabled, "Rule should be disabled");
}

/// Assert rule order index
pub fn assert_rule_order(rule: &RuleResponse, expected_order: i32) {
    assert_eq!(
        rule.order_index, expected_order,
        "Rule order_index mismatch: expected {}, got {}",
        expected_order, rule.order_index
    );
}

// =============================================================================
// Datatype Assertions
// =============================================================================

/// Assert datatype primitive type
pub fn assert_datatype_primitive(datatype: &DatatypeResponse, expected_primitive: &str) {
    assert_eq!(
        datatype.primitive_type, expected_primitive,
        "Datatype primitive mismatch: expected '{}', got '{}'",
        expected_primitive, datatype.primitive_type
    );
}

/// Assert datatype has min/max constraints
pub fn assert_datatype_range(datatype: &DatatypeResponse, min: f64, max: f64) {
    assert_eq!(
        datatype.constraints.min,
        Some(min),
        "Datatype min constraint mismatch"
    );
    assert_eq!(
        datatype.constraints.max,
        Some(max),
        "Datatype max constraint mismatch"
    );
}

/// Assert datatype has precision/scale
pub fn assert_datatype_precision(datatype: &DatatypeResponse, precision: i32, scale: i32) {
    assert_eq!(
        datatype.constraints.precision,
        Some(precision),
        "Datatype precision mismatch"
    );
    assert_eq!(
        datatype.constraints.scale,
        Some(scale),
        "Datatype scale mismatch"
    );
}

// =============================================================================
// Enumeration Assertions
// =============================================================================

/// Assert enumeration has values
pub fn assert_enumeration_values(enumeration: &EnumerationResponse, expected: &[&str]) {
    assert_eq!(
        enumeration.values.len(),
        expected.len(),
        "Enumeration value count mismatch"
    );
    for exp in expected {
        assert!(
            enumeration.values.contains(&exp.to_string()),
            "Enumeration should contain value '{}'",
            exp
        );
    }
}

/// Assert enumeration has specific value
pub fn assert_enumeration_contains(enumeration: &EnumerationResponse, value: &str) {
    assert!(
        enumeration.values.contains(&value.to_string()),
        "Enumeration should contain value '{}', got {:?}",
        value,
        enumeration.values
    );
}

/// Assert enumeration value count
pub fn assert_enumeration_value_count(enumeration: &EnumerationResponse, expected_count: usize) {
    assert_eq!(
        enumeration.values.len(),
        expected_count,
        "Enumeration value count mismatch: expected {}, got {}",
        expected_count,
        enumeration.values.len()
    );
}

// =============================================================================
// Functionality Assertions
// =============================================================================

/// Assert functionality status
pub fn assert_functionality_status(functionality: &FunctionalityResponse, expected_status: &str) {
    assert_eq!(
        functionality.status, expected_status,
        "Functionality status mismatch: expected '{}', got '{}'",
        expected_status, functionality.status
    );
}

/// Assert functionality is immutable
pub fn assert_functionality_immutable(functionality: &FunctionalityResponse) {
    assert!(functionality.immutable, "Functionality should be immutable");
}

/// Assert functionality has required attributes count
pub fn assert_functionality_required_count(functionality: &FunctionalityResponse, count: usize) {
    assert_eq!(
        functionality.required_attributes.len(),
        count,
        "Functionality required attribute count mismatch"
    );
}

// =============================================================================
// Evaluation Assertions
// =============================================================================

/// Assert evaluation succeeded
pub fn assert_evaluation_success(result: &EvaluateResponse) {
    assert!(result.success, "Evaluation should succeed");
    assert!(result.errors.is_empty(), "Evaluation should have no errors: {:?}", result.errors);
}

/// Assert evaluation failed
pub fn assert_evaluation_failed(result: &EvaluateResponse) {
    assert!(!result.success, "Evaluation should fail");
}

/// Assert evaluation has output
pub fn assert_evaluation_has_output(result: &EvaluateResponse, key: &str) {
    assert!(
        result.outputs.contains_key(key),
        "Evaluation should have output '{}', got keys: {:?}",
        key,
        result.outputs.keys().collect::<Vec<_>>()
    );
}

/// Assert evaluation output value (float comparison with tolerance)
pub fn assert_output_float(result: &EvaluateResponse, key: &str, expected: f64) {
    let value = result.outputs.get(key)
        .unwrap_or_else(|| panic!("Output '{}' not found", key));

    let actual = match value {
        AttributeValueJson::Float { value } => *value,
        AttributeValueJson::Int { value } => *value as f64,
        _ => panic!("Expected numeric output for '{}', got {:?}", key, value),
    };

    let tolerance = 0.0001;
    assert!(
        (actual - expected).abs() < tolerance,
        "Output '{}' mismatch: expected {}, got {} (tolerance: {})",
        key,
        expected,
        actual,
        tolerance
    );
}

/// Assert evaluation output value (integer)
pub fn assert_output_int(result: &EvaluateResponse, key: &str, expected: i64) {
    let value = result.outputs.get(key)
        .unwrap_or_else(|| panic!("Output '{}' not found", key));

    let actual = match value {
        AttributeValueJson::Int { value } => *value,
        AttributeValueJson::Float { value } => *value as i64,
        _ => panic!("Expected integer output for '{}', got {:?}", key, value),
    };

    assert_eq!(actual, expected, "Output '{}' mismatch", key);
}

/// Assert evaluation output value (string)
pub fn assert_output_string(result: &EvaluateResponse, key: &str, expected: &str) {
    let value = result.outputs.get(key)
        .unwrap_or_else(|| panic!("Output '{}' not found", key));

    match value {
        AttributeValueJson::String { value } => {
            assert_eq!(value, expected, "Output '{}' string mismatch", key);
        }
        _ => panic!("Expected string output for '{}', got {:?}", key, value),
    }
}

/// Assert evaluation output value (boolean)
pub fn assert_output_bool(result: &EvaluateResponse, key: &str, expected: bool) {
    let value = result.outputs.get(key)
        .unwrap_or_else(|| panic!("Output '{}' not found", key));

    match value {
        AttributeValueJson::Bool { value } => {
            assert_eq!(*value, expected, "Output '{}' bool mismatch", key);
        }
        _ => panic!("Expected bool output for '{}', got {:?}", key, value),
    }
}

/// Assert rule result exists and succeeded
pub fn assert_rule_executed(result: &EvaluateResponse, rule_id: &str) {
    let rule_result = result.rule_results.iter()
        .find(|r| r.rule_id == rule_id)
        .unwrap_or_else(|| panic!("Rule '{}' was not executed", rule_id));

    assert!(!rule_result.skipped, "Rule '{}' should not be skipped", rule_id);
    assert!(rule_result.error.is_none(), "Rule '{}' should not have error: {:?}", rule_id, rule_result.error);
}

/// Assert rule was skipped
pub fn assert_rule_skipped(result: &EvaluateResponse, rule_id: &str) {
    let rule_result = result.rule_results.iter()
        .find(|r| r.rule_id == rule_id)
        .unwrap_or_else(|| panic!("Rule '{}' not found in results", rule_id));

    assert!(rule_result.skipped, "Rule '{}' should be skipped", rule_id);
}

/// Assert rule result has specific output value
pub fn assert_rule_output(result: &EvaluateResponse, rule_id: &str, output_path: &str, expected: f64) {
    let rule_result = result.rule_results.iter()
        .find(|r| r.rule_id == rule_id)
        .unwrap_or_else(|| panic!("Rule '{}' not found in results", rule_id));

    let output = rule_result.outputs.iter()
        .find(|o| o.path == output_path)
        .unwrap_or_else(|| panic!("Output '{}' not found for rule '{}'", output_path, rule_id));

    let actual = match &output.value {
        AttributeValueJson::Float { value } => *value,
        AttributeValueJson::Int { value } => *value as f64,
        _ => panic!("Expected numeric output for rule '{}' output '{}'", rule_id, output_path),
    };

    let tolerance = 0.0001;
    assert!(
        (actual - expected).abs() < tolerance,
        "Rule '{}' output '{}' mismatch: expected {}, got {}",
        rule_id,
        output_path,
        expected,
        actual
    );
}

/// Assert execution metrics
pub fn assert_execution_metrics(
    result: &EvaluateResponse,
    expected_executed: i32,
    expected_skipped: i32,
) {
    assert_eq!(
        result.metrics.rules_executed, expected_executed,
        "Rules executed count mismatch"
    );
    assert_eq!(
        result.metrics.rules_skipped, expected_skipped,
        "Rules skipped count mismatch"
    );
}

/// Assert execution time is reasonable (> 0)
pub fn assert_execution_time_valid(result: &EvaluateResponse) {
    assert!(
        result.metrics.total_time_ns > 0,
        "Total execution time should be positive"
    );
}

// =============================================================================
// Validation Assertions
// =============================================================================

/// Assert validation passed
pub fn assert_validation_valid(result: &ValidationResponse) {
    assert!(result.valid, "Validation should pass");
    assert!(
        result.errors.is_empty(),
        "Validation should have no errors: {:?}",
        result.errors
    );
}

/// Assert validation failed
pub fn assert_validation_invalid(result: &ValidationResponse) {
    assert!(!result.valid, "Validation should fail");
    assert!(
        !result.errors.is_empty(),
        "Validation should have errors"
    );
}

/// Assert validation has specific error code
pub fn assert_validation_error_code(result: &ValidationResponse, expected_code: &str) {
    let has_code = result.errors.iter().any(|e| e.code == expected_code);
    assert!(
        has_code,
        "Validation should have error code '{}', got: {:?}",
        expected_code,
        result.errors.iter().map(|e| &e.code).collect::<Vec<_>>()
    );
}

/// Assert validation has warning
pub fn assert_validation_has_warning(result: &ValidationResponse) {
    assert!(
        !result.warnings.is_empty(),
        "Validation should have warnings"
    );
}

// =============================================================================
// Error Response Assertions
// =============================================================================

/// Assert HTTP error response has expected status
pub fn assert_error_status(body: &str, status_code: u16) {
    // Parse error response if possible
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(status) = json.get("status").and_then(|s| s.as_u64()) {
            assert_eq!(
                status as u16,
                status_code,
                "Error status mismatch in body"
            );
        }
    }
}

/// Assert error response contains message
pub fn assert_error_contains(body: &str, expected_text: &str) {
    assert!(
        body.contains(expected_text),
        "Error body should contain '{}', got: {}",
        expected_text,
        body
    );
}
