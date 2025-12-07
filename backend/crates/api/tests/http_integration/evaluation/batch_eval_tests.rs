//! Batch Evaluation Tests
//!
//! Tests for the /api/batch-evaluate endpoint - batch rule evaluation.

use crate::fixtures::*;
use serde_json::json;

/// Helper to set up a product with a simple calculation rule
async fn setup_product_with_rule(ctx: &TestContext) -> String {
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
        "name": "Batch Eval Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes
    for name in ["amount", "tax"] {
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

    // Create rule: tax = amount * 0.1
    let expr = json!({"*": [{"var": "loan/main/amount"}, 0.1]});
    let rule_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/tax = loan/main/amount * 0.1",
        "expressionJson": serde_json::to_string(&expr).unwrap(),
        "inputAttributes": ["loan/main/amount"],
        "outputAttributes": ["loan/main/tax"]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule_req,
    )
    .await
    .expect("Failed to create rule");

    product_id
}

/// Test batch evaluate with multiple requests
#[tokio::test]
async fn test_batch_evaluate_multiple_requests() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_rule(&ctx).await;

    let batch_req = json!({
        "productId": product_id,
        "requests": [
            {
                "requestId": "req-1",
                "inputData": {
                    "loan/main/amount": {"type": "float", "value": 100.0}
                }
            },
            {
                "requestId": "req-2",
                "inputData": {
                    "loan/main/amount": {"type": "float", "value": 200.0}
                }
            },
            {
                "requestId": "req-3",
                "inputData": {
                    "loan/main/amount": {"type": "float", "value": 500.0}
                }
            }
        ]
    });

    let response: serde_json::Value = ctx.post("/api/batch-evaluate", &batch_req)
        .await
        .expect("Failed to batch evaluate");

    // Verify all results returned
    let results = response["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);

    // Verify request IDs are preserved
    let request_ids: Vec<&str> = results.iter()
        .map(|r| r["requestId"].as_str().unwrap())
        .collect();
    assert!(request_ids.contains(&"req-1"));
    assert!(request_ids.contains(&"req-2"));
    assert!(request_ids.contains(&"req-3"));

    // Verify all succeeded
    for result in results {
        assert_eq!(result["success"], true);
        assert!(result["results"].as_array().unwrap().len() > 0);
    }
}

/// Test batch evaluate with single request
#[tokio::test]
async fn test_batch_evaluate_single_request() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_rule(&ctx).await;

    let batch_req = json!({
        "productId": product_id,
        "requests": [
            {
                "requestId": "single-req",
                "inputData": {
                    "loan/main/amount": {"type": "int", "value": 1000}
                }
            }
        ]
    });

    let response: serde_json::Value = ctx.post("/api/batch-evaluate", &batch_req)
        .await
        .expect("Failed to batch evaluate");

    let results = response["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["requestId"], "single-req");
    assert_eq!(results[0]["success"], true);
}

/// Test batch evaluate with empty requests
#[tokio::test]
async fn test_batch_evaluate_empty_requests() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_rule(&ctx).await;

    let batch_req = json!({
        "productId": product_id,
        "requests": []
    });

    let response: serde_json::Value = ctx.post("/api/batch-evaluate", &batch_req)
        .await
        .expect("Failed to batch evaluate");

    let results = response["results"].as_array().unwrap();
    assert!(results.is_empty());
}

/// Test batch evaluate returns total time
#[tokio::test]
async fn test_batch_evaluate_returns_total_time() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_rule(&ctx).await;

    let batch_req = json!({
        "productId": product_id,
        "requests": [
            {
                "requestId": "req-1",
                "inputData": {
                    "loan/main/amount": {"type": "int", "value": 100}
                }
            },
            {
                "requestId": "req-2",
                "inputData": {
                    "loan/main/amount": {"type": "int", "value": 200}
                }
            }
        ]
    });

    let response: serde_json::Value = ctx.post("/api/batch-evaluate", &batch_req)
        .await
        .expect("Failed to batch evaluate");

    // Verify total_time_ns is present and reasonable
    let total_time_ns = response["totalTimeNs"].as_i64().unwrap();
    assert!(total_time_ns >= 0, "Total time should be non-negative");
}

/// Test batch evaluate for non-existent product
#[tokio::test]
async fn test_batch_evaluate_product_not_found() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let batch_req = json!({
        "productId": "non-existent-product",
        "requests": [
            {
                "requestId": "req-1",
                "inputData": {}
            }
        ]
    });

    let result = ctx.post::<serde_json::Value>("/api/batch-evaluate", &batch_req).await;

    assert!(result.is_err());
}

/// Test batch evaluate with mixed success/failure
#[tokio::test]
async fn test_batch_evaluate_mixed_results() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_rule(&ctx).await;

    // Note: Depending on implementation, missing inputs may not cause failure
    // This test verifies the batch can handle requests independently
    let batch_req = json!({
        "productId": product_id,
        "requests": [
            {
                "requestId": "valid-req",
                "inputData": {
                    "loan/main/amount": {"type": "int", "value": 100}
                }
            },
            {
                "requestId": "missing-input",
                "inputData": {}  // Missing required input
            }
        ]
    });

    let response: serde_json::Value = ctx.post("/api/batch-evaluate", &batch_req)
        .await
        .expect("Failed to batch evaluate");

    let results = response["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);

    // Find the valid request result
    let valid_result = results.iter()
        .find(|r| r["requestId"] == "valid-req")
        .expect("Valid request result not found");
    assert_eq!(valid_result["success"], true);
}

/// Test batch evaluate preserves request order
#[tokio::test]
async fn test_batch_evaluate_preserves_order() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_rule(&ctx).await;

    let batch_req = json!({
        "productId": product_id,
        "requests": [
            {"requestId": "first", "inputData": {"loan/main/amount": {"type": "int", "value": 1}}},
            {"requestId": "second", "inputData": {"loan/main/amount": {"type": "int", "value": 2}}},
            {"requestId": "third", "inputData": {"loan/main/amount": {"type": "int", "value": 3}}},
            {"requestId": "fourth", "inputData": {"loan/main/amount": {"type": "int", "value": 4}}}
        ]
    });

    let response: serde_json::Value = ctx.post("/api/batch-evaluate", &batch_req)
        .await
        .expect("Failed to batch evaluate");

    let results = response["results"].as_array().unwrap();
    assert_eq!(results.len(), 4);

    // Verify order is preserved
    assert_eq!(results[0]["requestId"], "first");
    assert_eq!(results[1]["requestId"], "second");
    assert_eq!(results[2]["requestId"], "third");
    assert_eq!(results[3]["requestId"], "fourth");
}

/// Test batch evaluate with many requests (stress test)
#[tokio::test]
async fn test_batch_evaluate_many_requests() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_rule(&ctx).await;

    // Create 50 requests
    let requests: Vec<serde_json::Value> = (0..50)
        .map(|i| json!({
            "requestId": format!("req-{}", i),
            "inputData": {
                "loan/main/amount": {"type": "int", "value": i * 100}
            }
        }))
        .collect();

    let batch_req = json!({
        "productId": product_id,
        "requests": requests
    });

    let response: serde_json::Value = ctx.post("/api/batch-evaluate", &batch_req)
        .await
        .expect("Failed to batch evaluate");

    let results = response["results"].as_array().unwrap();
    assert_eq!(results.len(), 50);

    // Verify all succeeded
    for result in results {
        assert_eq!(result["success"], true);
    }
}

/// Test batch evaluate with product that has no rules
#[tokio::test]
async fn test_batch_evaluate_no_rules() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product without rules
    let product_req = json!({
        "id": product_id,
        "name": "No Rules Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    let batch_req = json!({
        "productId": product_id,
        "requests": [
            {"requestId": "req-1", "inputData": {}}
        ]
    });

    let response: serde_json::Value = ctx.post("/api/batch-evaluate", &batch_req)
        .await
        .expect("Failed to batch evaluate");

    let results = response["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["success"], true);
    // No computed values since no rules
    assert!(results[0]["results"].as_array().unwrap().is_empty());
}

/// Test batch result includes computed flag
#[tokio::test]
async fn test_batch_evaluate_result_computed_flag() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_rule(&ctx).await;

    let batch_req = json!({
        "productId": product_id,
        "requests": [
            {
                "requestId": "req-1",
                "inputData": {
                    "loan/main/amount": {"type": "int", "value": 100}
                }
            }
        ]
    });

    let response: serde_json::Value = ctx.post("/api/batch-evaluate", &batch_req)
        .await
        .expect("Failed to batch evaluate");

    let results = response["results"].as_array().unwrap();
    let attr_results = results[0]["results"].as_array().unwrap();

    // All results should be marked as computed
    for attr in attr_results {
        assert_eq!(attr["computed"], true);
        assert!(attr["path"].is_string());
        assert!(attr["value"].is_object() || attr["value"].is_string());
    }
}
