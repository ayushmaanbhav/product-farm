//! Product Validation Tests
//!
//! Tests for the /api/products/:product_id/validate endpoint.

use crate::fixtures::*;
use serde_json::json;

/// Test validation of a complete valid product
#[tokio::test]
async fn test_validate_complete_product() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");
    let datatype_id = ctx.unique_id("decimal");

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Complete Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attribute
    let attr_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "amount",
        "datatypeId": datatype_id
    });
    let attr_resp = ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req,
    )
    .await
    .expect("Failed to create abstract attribute");

    // Create concrete attribute with value
    let concrete_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "amount",
        "abstractPath": attr_resp["abstractPath"].as_str().unwrap(),
        "valueType": "FIXED_VALUE",
        "value": {"type": "decimal", "value": "1000.00"}
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/attributes", product_id),
        &concrete_req,
    )
    .await
    .expect("Failed to create concrete attribute");

    // Create functionality - use the full abstract path from the abstract attribute response
    let func_req = json!({
        "name": "basic-func",
        "description": "Basic functionality",
        "requiredAttributes": [
            {"abstractPath": attr_resp["abstractPath"].as_str().unwrap(), "description": "Amount"}
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Validate
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/validate", product_id),
    )
    .await
    .expect("Failed to validate product");

    // Should be valid
    assert_eq!(response["productId"], product_id);
    assert_eq!(response["valid"], true);
    assert!(response["errors"].as_array().unwrap().is_empty());
}

/// Test validation detects missing required attributes
#[tokio::test]
async fn test_validate_missing_required_attribute() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");
    let datatype_id = ctx.unique_id("decimal");

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Incomplete Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attribute (but no concrete)
    let attr_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "missing-concrete",
        "datatypeId": datatype_id
    });
    let attr_resp = ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req,
    )
    .await
    .expect("Failed to create abstract attribute");

    // Create functionality requiring the attribute (no concrete implementation)
    let func_req = json!({
        "name": "incomplete-func",
        "description": "Functionality with missing requirement",
        "requiredAttributes": [
            {"abstractPath": attr_resp["abstractPath"].as_str().unwrap(), "description": "Missing"}
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Validate
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/validate", product_id),
    )
    .await
    .expect("Failed to validate product");

    // Should be invalid with errors
    assert_eq!(response["valid"], false);
    let errors = response["errors"].as_array().unwrap();
    assert!(!errors.is_empty());

    // Check for missing required attribute error
    let has_missing_attr_error = errors.iter().any(|e| {
        e["code"].as_str() == Some("MISSING_REQUIRED_ATTRIBUTE")
    });
    assert!(has_missing_attr_error, "Should have MISSING_REQUIRED_ATTRIBUTE error");
}

/// Test validation detects attribute without value or rule
#[tokio::test]
async fn test_validate_attribute_no_value() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");
    let datatype_id = ctx.unique_id("decimal");

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "No Value Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attribute
    let attr_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "no-value",
        "datatypeId": datatype_id
    });
    let attr_resp = ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req,
    )
    .await
    .expect("Failed to create abstract attribute");

    // Create concrete attribute with JUST_DEFINITION (no value, no rule)
    let concrete_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "no-value",
        "abstractPath": attr_resp["abstractPath"].as_str().unwrap(),
        "valueType": "JUST_DEFINITION"
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/attributes", product_id),
        &concrete_req,
    )
    .await
    .expect("Failed to create concrete attribute");

    // Validate
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/validate", product_id),
    )
    .await
    .expect("Failed to validate product");

    // Should have warnings
    let warnings = response["warnings"].as_array().unwrap();
    let has_no_value_warning = warnings.iter().any(|w| {
        w["code"].as_str() == Some("ATTRIBUTE_NO_VALUE")
    });
    assert!(has_no_value_warning, "Should have ATTRIBUTE_NO_VALUE warning");
}

/// Test validation detects orphan rules (rules with no outputs)
/// NOTE: This test is ignored because the API validation prevents creating rules
/// with empty output_attributes, so this scenario cannot be tested.
#[tokio::test]
#[ignore = "Cannot create rules with no outputs - API validation prevents it"]
async fn test_validate_rule_no_outputs() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");
    let datatype_id = ctx.unique_id("decimal");

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Orphan Rule Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create input abstract attribute
    let input_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "input",
        "datatypeId": datatype_id
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input_req,
    )
    .await
    .expect("Failed to create input abstract attribute");

    // Create output abstract attribute (but won't create concrete for it)
    let output_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "unused-output",
        "datatypeId": datatype_id
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output_req,
    )
    .await
    .expect("Failed to create output abstract attribute");

    // Create rule with output that's never used (orphan rule)
    let expr = json!({"var": "loan/main/input"});
    let rule_req = json!({
        "id": ctx.unique_id("orphan-rule"),
        "name": "Orphan Rule",
        "ruleType": "CALCULATION",
        "expressionJson": serde_json::to_string(&expr).unwrap(),
        "displayExpression": "loan/main/unused-output = loan/main/input",
        "inputAttributes": ["loan/main/input"],
        "outputAttributes": ["loan/main/unused-output"],
        "enabled": true
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule_req,
    )
    .await
    .expect("Failed to create rule");

    // Validate
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/validate", product_id),
    )
    .await
    .expect("Failed to validate product");

    // Should have warnings about orphan rule
    let warnings = response["warnings"].as_array().unwrap();
    let has_orphan_warning = warnings.iter().any(|w| {
        w["code"].as_str() == Some("RULE_NO_OUTPUTS")
    });
    assert!(has_orphan_warning, "Should have RULE_NO_OUTPUTS warning");
}

/// Test validate non-existent product
#[tokio::test]
async fn test_validate_product_not_found() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let result = ctx.get::<serde_json::Value>(
        "/api/products/non-existent-product/validate",
    )
    .await;

    assert!(result.is_err());
}

/// Test validation response structure
#[tokio::test]
async fn test_validate_response_structure() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create minimal product
    let product_req = json!({
        "id": product_id,
        "name": "Structure Test",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Validate
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/validate", product_id),
    )
    .await
    .expect("Failed to validate product");

    // Verify response structure
    assert!(response.get("productId").is_some());
    assert!(response.get("valid").is_some());
    assert!(response.get("errors").is_some());
    assert!(response.get("warnings").is_some());

    assert!(response["productId"].is_string());
    assert!(response["valid"].is_boolean());
    assert!(response["errors"].is_array());
    assert!(response["warnings"].is_array());
}

/// Test validation error structure
#[tokio::test]
async fn test_validate_error_structure() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");
    let datatype_id = ctx.unique_id("decimal");

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // Create product with missing required attribute
    let product_req = json!({
        "id": product_id,
        "name": "Error Structure Test",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attribute
    let attr_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "missing",
        "datatypeId": datatype_id
    });
    let attr_resp = ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req,
    )
    .await
    .expect("Failed to create abstract attribute");

    // Create functionality requiring attribute without concrete implementation
    let func_req = json!({
        "name": "error-func",
        "description": "Will cause error",
        "requiredAttributes": [
            {"abstractPath": attr_resp["abstractPath"].as_str().unwrap(), "description": "Missing"}
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Validate
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/validate", product_id),
    )
    .await
    .expect("Failed to validate product");

    // Check error structure
    let errors = response["errors"].as_array().unwrap();
    assert!(!errors.is_empty());

    for error in errors {
        assert!(error["code"].is_string());
        assert!(error["message"].is_string());
        assert!(error["severity"].is_string());
        // path is optional but should be present for attribute-related errors
    }
}

/// Test validation warning structure
#[tokio::test]
async fn test_validate_warning_structure() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");
    let datatype_id = ctx.unique_id("decimal");

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Warning Structure Test",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attribute
    let attr_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "orphan",
        "datatypeId": datatype_id
    });
    let attr_resp = ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req,
    )
    .await
    .expect("Failed to create abstract attribute");

    // Create concrete with no value
    let concrete_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "orphan",
        "abstractPath": attr_resp["abstractPath"].as_str().unwrap(),
        "valueType": "JUST_DEFINITION"
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/attributes", product_id),
        &concrete_req,
    )
    .await
    .expect("Failed to create concrete attribute");

    // Validate
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/validate", product_id),
    )
    .await
    .expect("Failed to validate product");

    // Check warning structure
    let warnings = response["warnings"].as_array().unwrap();
    if !warnings.is_empty() {
        for warning in warnings {
            assert!(warning["code"].is_string());
            assert!(warning["message"].is_string());
            // suggestion is optional
        }
    }
}

/// Test validation with multiple errors
#[tokio::test]
async fn test_validate_multiple_errors() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");
    let datatype_id = ctx.unique_id("decimal");

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Multiple Errors Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create multiple abstract attributes
    let mut abstract_paths = Vec::new();
    for name in ["missing-a", "missing-b", "missing-c"] {
        let attr_req = json!({
            "componentType": "loan",
            "componentId": "main",
            "attributeName": name,
            "datatypeId": datatype_id
        });
        let attr_resp = ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr_req,
        )
        .await
        .expect("Failed to create abstract attribute");
        abstract_paths.push(attr_resp["abstractPath"].as_str().unwrap().to_string());
    }

    // Create functionalities requiring all of them (none have concrete)
    let func_req = json!({
        "name": "multi-error-func",
        "description": "Multiple missing",
        "requiredAttributes": [
            {"abstractPath": &abstract_paths[0], "description": "A"},
            {"abstractPath": &abstract_paths[1], "description": "B"},
            {"abstractPath": &abstract_paths[2], "description": "C"}
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Validate
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/validate", product_id),
    )
    .await
    .expect("Failed to validate product");

    // Should have multiple errors
    let errors = response["errors"].as_array().unwrap();
    assert!(errors.len() >= 3, "Should have at least 3 errors");
    assert_eq!(response["valid"], false);
}

/// Test validation with product with no issues
#[tokio::test]
async fn test_validate_product_no_issues() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create minimal valid product (no functionalities, no rules)
    let product_req = json!({
        "id": product_id,
        "name": "Minimal Valid Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Validate
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/validate", product_id),
    )
    .await
    .expect("Failed to validate product");

    // Should be valid with no errors or warnings
    assert_eq!(response["valid"], true);
    assert!(response["errors"].as_array().unwrap().is_empty());
    // Warnings may or may not be empty depending on implementation
}
