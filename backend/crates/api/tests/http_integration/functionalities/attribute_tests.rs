//! Functionality Attribute Tests
//!
//! Tests for required attributes management and querying:
//! - Required attributes on create/update
//! - Listing abstract attributes for a functionality
//! - Listing rules that output to functionality attributes

use crate::fixtures::*;
use serde_json::json;

/// Test creating functionality with required attributes
#[tokio::test]
async fn test_create_functionality_with_required_attributes() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");
    let decimal_datatype_id = ctx.unique_id("decimal");
    let int_datatype_id = ctx.unique_id("int");

    // Create datatypes first
    let decimal_dt_req = json!({
        "id": decimal_datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &decimal_dt_req)
        .await
        .expect("Failed to create decimal datatype");

    let int_dt_req = json!({
        "id": int_datatype_id,
        "name": "Test Int",
        "primitiveType": "INT"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &int_dt_req)
        .await
        .expect("Failed to create int datatype");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes
    for (name, datatype) in [("amount", decimal_datatype_id.as_str()), ("rate", decimal_datatype_id.as_str()), ("term", int_datatype_id.as_str())] {
        let attr_req = json!({
            "componentType": "loan",
            "componentId": "main",
            "attributeName": name,
            "datatypeId": datatype
        });
        ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr_req,
        )
        .await
        .expect("Failed to create abstract attribute");
    }

    // Create functionality with multiple required attributes
    let func_req = json!({
        "name": "loan-calc",
        "description": "Loan calculation",
        "requiredAttributes": [
            {
                "abstractPath": "loan.main.amount",
                "description": "Principal amount"
            },
            {
                "abstractPath": "loan.main.rate",
                "description": "Interest rate"
            },
            {
                "abstractPath": "loan.main.term",
                "description": "Loan term in months"
            }
        ]
    });

    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    let attrs = response["requiredAttributes"].as_array().unwrap();
    assert_eq!(attrs.len(), 3);

    // Verify order indices
    assert_eq!(attrs[0]["orderIndex"], 0);
    assert_eq!(attrs[0]["abstractPath"], "loan.main.amount");
    assert_eq!(attrs[0]["description"], "Principal amount");

    assert_eq!(attrs[1]["orderIndex"], 1);
    assert_eq!(attrs[1]["abstractPath"], "loan.main.rate");

    assert_eq!(attrs[2]["orderIndex"], 2);
    assert_eq!(attrs[2]["abstractPath"], "loan.main.term");
}

/// Test listing abstract attributes for a functionality
#[tokio::test]
async fn test_list_functionality_abstract_attributes() {
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
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes
    for name in ["amount", "rate"] {
        let attr_req = json!({
            "componentType": "loan",
            "componentId": "main",
            "attributeName": name,
            "datatypeId": datatype_id
        });
        ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr_req,
        )
        .await
        .expect("Failed to create abstract attribute");
    }

    // Create functionality
    let func_req = json!({
        "name": "test-func",
        "description": "Test",
        "requiredAttributes": [
            { "abstractPath": "loan.main.amount", "description": "Amount" },
            { "abstractPath": "loan.main.rate", "description": "Rate" }
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // List abstract attributes for functionality
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/test-func/abstract-attributes", product_id),
    )
    .await
    .expect("Failed to list functionality attributes");

    let items = response["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);

    // Verify attribute paths - API returns full path format: {productId}:abstract-path:loan:main:amount
    let paths: Vec<&str> = items.iter()
        .map(|i| i["abstractPath"].as_str().unwrap())
        .collect();
    // Check that paths contain the expected components
    assert!(paths.iter().any(|p| p.contains(":loan:main:amount")), "Expected path containing :loan:main:amount, got {:?}", paths);
    assert!(paths.iter().any(|p| p.contains(":loan:main:rate")), "Expected path containing :loan:main:rate, got {:?}", paths);
}

/// Test listing rules for a functionality
#[tokio::test]
async fn test_list_functionality_rules() {
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
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes
    for name in ["amount", "emi"] {
        let attr_req = json!({
            "componentType": "loan",
            "componentId": "main",
            "attributeName": name,
            "datatypeId": datatype_id
        });
        ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr_req,
        )
        .await
        .expect("Failed to create abstract attribute");
    }

    // Create interest attribute for the second rule
    let interest_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "interest",
        "datatypeId": datatype_id
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &interest_req,
    )
    .await
    .expect("Failed to create interest attribute");

    // Create rule that outputs to emi (use 3-part path with componentId)
    let rule_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "emi = amount * 0.01",
        "expressionJson": r#"{"*": [{"var": "loan/main/amount"}, 0.01]}"#,
        "inputAttributes": ["loan/main/amount"],
        "outputAttributes": ["loan/main/emi"],
        "orderIndex": 0
    });
    let rule_response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/rules", product_id),
        &rule_req,
    )
    .await
    .expect("Failed to create rule");
    let rule_id = rule_response["id"].as_str().unwrap();

    // Create another rule not related to functionality (outputs to interest, not emi)
    let other_rule_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "interest = amount * 0.05",
        "expressionJson": r#"{"*": [{"var": "loan/main/amount"}, 0.05]}"#,
        "inputAttributes": ["loan/main/amount"],
        "outputAttributes": ["loan/main/interest"],
        "orderIndex": 1
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &other_rule_req,
    )
    .await
    .expect("Failed to create other rule");

    // Create functionality requiring emi
    let func_req = json!({
        "name": "emi-func",
        "description": "EMI Functionality",
        "requiredAttributes": [
            { "abstractPath": "loan.main.emi", "description": "EMI output" }
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // List rules for functionality
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/emi-func/rules", product_id),
    )
    .await
    .expect("Failed to list functionality rules");

    let items = response["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "Should only return rule that outputs to emi");
    assert_eq!(items[0]["id"], rule_id);
}

/// Test listing rules for functionality with no matching rules
#[tokio::test]
async fn test_list_functionality_rules_empty() {
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
        "name": "Test Product",
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
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req,
    )
    .await
    .expect("Failed to create abstract attribute");

    // Create functionality (no rules output to this attribute)
    let func_req = json!({
        "name": "orphan-func",
        "description": "Orphan Functionality",
        "requiredAttributes": [
            { "abstractPath": "loan.main.orphan", "description": "Orphan attr" }
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // List rules for functionality
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/orphan-func/rules", product_id),
    )
    .await
    .expect("Failed to list functionality rules");

    let items = response["items"].as_array().unwrap();
    assert!(items.is_empty(), "Should return empty array when no rules match");
    assert_eq!(response["totalCount"], 0);
}

/// Test listing abstract attributes for non-existent functionality returns 404
#[tokio::test]
async fn test_list_abstract_attributes_functionality_not_found() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Try to list attributes for non-existent functionality
    let result = ctx.get::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/non-existent/abstract-attributes", product_id),
    )
    .await;

    assert!(result.is_err());
}

/// Test updating required attributes replaces all
#[tokio::test]
async fn test_update_required_attributes_replaces_all() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create functionality with initial attributes
    let func_req = json!({
        "name": "update-attrs",
        "description": "Update attrs test",
        "requiredAttributes": [
            { "abstractPath": "loan.main.a", "description": "Attr A" },
            { "abstractPath": "loan.main.b", "description": "Attr B" },
            { "abstractPath": "loan.main.c", "description": "Attr C" }
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Update with completely different set
    let update_req = json!({
        "requiredAttributes": [
            { "abstractPath": "loan.main.x", "description": "Attr X" },
            { "abstractPath": "loan.main.y", "description": "Attr Y" }
        ]
    });
    let response: serde_json::Value = ctx.put(
        &format!("/api/products/{}/functionalities/update-attrs", product_id),
        &update_req,
    )
    .await
    .expect("Failed to update functionality");

    let attrs = response["requiredAttributes"].as_array().unwrap();
    assert_eq!(attrs.len(), 2, "Should have exactly 2 attributes now");

    let paths: Vec<&str> = attrs.iter()
        .map(|a| a["abstractPath"].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"loan.main.x"));
    assert!(paths.contains(&"loan.main.y"));
    assert!(!paths.contains(&"loan.main.a"), "Old attributes should be removed");
}

/// Test clearing required attributes with empty array
#[tokio::test]
async fn test_update_clear_required_attributes() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create functionality with attributes
    let func_req = json!({
        "name": "clear-attrs",
        "description": "Clear attrs test",
        "requiredAttributes": [
            { "abstractPath": "loan.main.a", "description": "Attr A" }
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Clear by passing empty array
    let update_req = json!({
        "requiredAttributes": []
    });
    let response: serde_json::Value = ctx.put(
        &format!("/api/products/{}/functionalities/clear-attrs", product_id),
        &update_req,
    )
    .await
    .expect("Failed to update functionality");

    let attrs = response["requiredAttributes"].as_array().unwrap();
    assert!(attrs.is_empty(), "Should have no attributes after clearing");
}

/// Test multiple rules outputting to functionality-required attributes
#[tokio::test]
async fn test_functionality_with_multiple_output_rules() {
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
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes
    for name in ["input", "output-a", "output-b"] {
        let attr_req = json!({
            "componentType": "loan",
            "componentId": "main",
            "attributeName": name,
            "datatypeId": datatype_id
        });
        ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr_req,
        )
        .await
        .expect("Failed to create abstract attribute");
    }

    // Create two rules outputting to different required attributes (use 3-part paths)
    let rule1_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "output-a = input",
        "expressionJson": r#"{"var": "loan/main/input"}"#,
        "inputAttributes": ["loan/main/input"],
        "outputAttributes": ["loan/main/output-a"],
        "orderIndex": 0
    });
    let rule1_resp: serde_json::Value = ctx.post(
        &format!("/api/products/{}/rules", product_id),
        &rule1_req,
    )
    .await
    .expect("Failed to create rule 1");
    let rule1_id = rule1_resp["id"].as_str().unwrap().to_string();

    let rule2_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "output-b = input",
        "expressionJson": r#"{"var": "loan/main/input"}"#,
        "inputAttributes": ["loan/main/input"],
        "outputAttributes": ["loan/main/output-b"],
        "orderIndex": 1
    });
    let rule2_resp: serde_json::Value = ctx.post(
        &format!("/api/products/{}/rules", product_id),
        &rule2_req,
    )
    .await
    .expect("Failed to create rule 2");
    let rule2_id = rule2_resp["id"].as_str().unwrap().to_string();

    // Create functionality requiring both outputs
    let func_req = json!({
        "name": "multi-rule-func",
        "description": "Multi-rule functionality",
        "requiredAttributes": [
            { "abstractPath": "loan.main.output-a", "description": "Output A" },
            { "abstractPath": "loan.main.output-b", "description": "Output B" }
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // List rules for functionality
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/multi-rule-func/rules", product_id),
    )
    .await
    .expect("Failed to list functionality rules");

    let items = response["items"].as_array().unwrap();
    assert_eq!(items.len(), 2, "Should return both rules");

    let rule_ids: Vec<&str> = items.iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert!(rule_ids.contains(&rule1_id.as_str()), "Should contain rule1");
    assert!(rule_ids.contains(&rule2_id.as_str()), "Should contain rule2");
}
