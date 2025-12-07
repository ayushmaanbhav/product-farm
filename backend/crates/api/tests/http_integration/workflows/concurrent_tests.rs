//! Concurrent Access Tests
//!
//! Tests for concurrent access patterns and data consistency under load.

use crate::fixtures::*;
use serde_json::json;
use std::sync::Arc;
use tokio::task::JoinSet;

/// Test: Concurrent product creation
#[tokio::test]
async fn test_concurrent_product_creation() {
    let ctx = Arc::new(TestContext::new().await.expect("Failed to create test context"));
    let mut tasks = JoinSet::new();

    // Create 10 products concurrently
    for i in 0..10 {
        let ctx = ctx.clone();
        tasks.spawn(async move {
            // Product IDs must start with letter and use underscores, not hyphens
            let product_id = format!("concurrentProd{}_{}", i, uuid::Uuid::new_v4().to_string().replace('-', "").chars().take(8).collect::<String>());
            let product_req = json!({
                "id": product_id,
                "name": format!("Concurrent Product {}", i),
                "templateType": "loan",
                "effectiveFrom": 1735689600
            });

            let result = ctx.post::<serde_json::Value>("/api/products", &product_req).await;
            (product_id, result.is_ok())
        });
    }

    // Collect results
    let mut successes = 0;
    while let Some(result) = tasks.join_next().await {
        if let Ok((_product_id, success)) = result {
            if success {
                successes += 1;
            }
        }
    }

    // All should succeed
    assert_eq!(successes, 10, "All concurrent product creations should succeed");
}

/// Test: Concurrent reads don't block each other
#[tokio::test]
async fn test_concurrent_reads() {
    let ctx = Arc::new(TestContext::new().await.expect("Failed to create test context"));
    // Product IDs: start with letter, use underscores (not hyphens)
    let product_id = format!("readTest_{}", uuid::Uuid::new_v4().to_string().replace('-', "").chars().take(8).collect::<String>());
    // Datatype IDs: lowercase, hyphens OK
    let datatype_id = format!("decimal{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // Create product first
    let product_req = json!({
        "id": product_id,
        "name": "Concurrent Read Test",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>("/api/products", &product_req)
        .await
        .expect("Failed to create product");

    // Add some attributes
    for i in 0..5 {
        let attr_req = json!({
            "componentType": "loan",
            "componentId": "main",
            "attributeName": format!("attr-{}", i),
            "datatypeId": datatype_id
        });
        ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr_req,
        )
        .await
        .expect("Failed to create attribute");
    }

    let product_id = Arc::new(product_id);
    let mut tasks = JoinSet::new();

    // Perform 20 concurrent reads
    for _ in 0..20 {
        let ctx = ctx.clone();
        let product_id = product_id.clone();
        tasks.spawn(async move {
            let result = ctx.get::<serde_json::Value>(
                &format!("/api/products/{}", product_id),
            ).await;
            result.is_ok()
        });
    }

    // Collect results
    let mut successes = 0;
    while let Some(result) = tasks.join_next().await {
        if let Ok(success) = result {
            if success {
                successes += 1;
            }
        }
    }

    assert_eq!(successes, 20, "All concurrent reads should succeed");
}

/// Test: Concurrent evaluation requests
#[tokio::test]
async fn test_concurrent_evaluation() {
    let ctx = Arc::new(TestContext::new().await.expect("Failed to create test context"));
    let product_id = format!("evalConcurrent_{}", uuid::Uuid::new_v4().to_string().replace('-', "").chars().take(8).collect::<String>());
    let datatype_id = format!("decimal{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // Create product with rule
    let product_req = json!({
        "id": product_id,
        "name": "Concurrent Eval Test",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>("/api/products", &product_req)
        .await
        .expect("Failed to create product");

    // Create attributes
    for name in ["input", "output"] {
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
        .expect("Failed to create attribute");
    }

    // Create rule - API expects:
    // ruleType (lowercase), inputAttributes as string paths, outputAttributes as string paths
    // displayExpression and expressionJson as strings
    // NOTE: expressionJson must use full paths as variable names
    let rule_req = json!({
        "ruleType": "calculation",
        "displayExpression": "loan/main/input * 2",
        "expressionJson": r#"{"*": [{"var": "loan/main/input"}, 2]}"#,
        "inputAttributes": ["loan/main/input"],
        "outputAttributes": ["loan/main/output"]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule_req,
    )
    .await
    .expect("Failed to create rule");

    let product_id = Arc::new(product_id);
    let mut tasks = JoinSet::new();

    // Perform 15 concurrent evaluations with different inputs
    for i in 0..15 {
        let ctx = ctx.clone();
        let product_id = product_id.clone();
        tasks.spawn(async move {
            let eval_req = json!({
                "productId": product_id.as_str(),
                "inputData": {
                    "loan/main/input": {"type": "int", "value": i * 10}
                }
            });
            let result = ctx.post::<serde_json::Value>("/api/evaluate", &eval_req).await;
            match result {
                Ok(resp) => resp["success"] == true,
                Err(_) => false,
            }
        });
    }

    // Collect results
    let mut successes = 0;
    while let Some(result) = tasks.join_next().await {
        if let Ok(success) = result {
            if success {
                successes += 1;
            }
        }
    }

    assert_eq!(successes, 15, "All concurrent evaluations should succeed");
}

/// Test: Concurrent updates to same product (write conflicts)
#[tokio::test]
async fn test_concurrent_updates() {
    let ctx = Arc::new(TestContext::new().await.expect("Failed to create test context"));
    let product_id = format!("updateConcurrent_{}", uuid::Uuid::new_v4().to_string().replace('-', "").chars().take(8).collect::<String>());

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Concurrent Update Test",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>("/api/products", &product_req)
        .await
        .expect("Failed to create product");

    let product_id = Arc::new(product_id);
    let mut tasks = JoinSet::new();

    // Attempt 5 concurrent updates
    for i in 0..5 {
        let ctx = ctx.clone();
        let product_id = product_id.clone();
        tasks.spawn(async move {
            let update_req = json!({
                "description": format!("Updated by task {}", i)
            });
            let result = ctx.put::<serde_json::Value>(
                &format!("/api/products/{}", product_id),
                &update_req,
            ).await;
            result.is_ok()
        });
    }

    // Collect results - some may fail due to conflicts, that's okay
    let mut successes = 0;
    let mut failures = 0;
    while let Some(result) = tasks.join_next().await {
        if let Ok(success) = result {
            if success {
                successes += 1;
            } else {
                failures += 1;
            }
        }
    }

    // At least one should succeed
    assert!(successes >= 1, "At least one update should succeed");

    // Final state should be consistent (one of the updates won)
    let final_product: serde_json::Value = ctx.get(
        &format!("/api/products/{}", product_id),
    )
    .await
    .expect("Failed to get final product state");

    // Description should be set to one of the update values
    let desc = final_product["description"].as_str().unwrap_or("");
    assert!(desc.starts_with("Updated by task"), "Description should be set");
}

/// Test: Concurrent attribute creation on same product
#[tokio::test]
async fn test_concurrent_attribute_creation() {
    let ctx = Arc::new(TestContext::new().await.expect("Failed to create test context"));
    let product_id = format!("attrConcurrent_{}", uuid::Uuid::new_v4().to_string().replace('-', "").chars().take(8).collect::<String>());
    let datatype_id = format!("decimal{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());

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
        "name": "Concurrent Attr Test",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>("/api/products", &product_req)
        .await
        .expect("Failed to create product");

    let product_id = Arc::new(product_id);
    let datatype_id = Arc::new(datatype_id);
    let mut tasks = JoinSet::new();

    // Create 10 attributes concurrently (different names, so no conflicts)
    for i in 0..10 {
        let ctx = ctx.clone();
        let product_id = product_id.clone();
        let datatype_id = datatype_id.clone();
        tasks.spawn(async move {
            // Attribute names: lowercase with hyphens OK
            let attr_req = json!({
                "componentType": "loan",
                "componentId": "main",
                "attributeName": format!("concurrent-attr{}", i),
                "datatypeId": datatype_id.as_str()
            });
            let result = ctx.post::<serde_json::Value>(
                &format!("/api/products/{}/abstract-attributes", product_id),
                &attr_req,
            ).await;
            result.is_ok()
        });
    }

    // Collect results
    let mut successes = 0;
    while let Some(result) = tasks.join_next().await {
        if let Ok(success) = result {
            if success {
                successes += 1;
            }
        }
    }

    assert_eq!(successes, 10, "All attribute creations should succeed");

    // Verify all attributes exist
    let attrs: serde_json::Value = ctx.get(
        &format!("/api/products/{}/abstract-attributes", product_id),
    )
    .await
    .expect("Failed to list attributes");

    let items = attrs["items"].as_array().unwrap();
    assert_eq!(items.len(), 10, "All 10 attributes should exist");
}

/// Test: Mixed read/write concurrent operations
#[tokio::test]
async fn test_mixed_concurrent_operations() {
    let ctx = Arc::new(TestContext::new().await.expect("Failed to create test context"));
    let product_id = format!("mixedConcurrent_{}", uuid::Uuid::new_v4().to_string().replace('-', "").chars().take(8).collect::<String>());
    let datatype_id = format!("decimal{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // Create product with initial setup
    let product_req = json!({
        "id": product_id,
        "name": "Mixed Concurrent Test",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>("/api/products", &product_req)
        .await
        .expect("Failed to create product");

    // Create initial attribute
    let attr_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "initial-attr0",
        "datatypeId": datatype_id
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req,
    )
    .await
    .expect("Failed to create initial attribute");

    let product_id = Arc::new(product_id);
    let datatype_id = Arc::new(datatype_id);
    let mut tasks = JoinSet::new();

    // Mix of operations: reads, writes, lists
    for i in 0..20 {
        let ctx = ctx.clone();
        let product_id = product_id.clone();
        let datatype_id = datatype_id.clone();
        match i % 4 {
            0 => {
                // Read product
                tasks.spawn(async move {
                    ctx.get::<serde_json::Value>(
                        &format!("/api/products/{}", product_id),
                    ).await.is_ok()
                });
            }
            1 => {
                // List attributes
                tasks.spawn(async move {
                    ctx.get::<serde_json::Value>(
                        &format!("/api/products/{}/abstract-attributes", product_id),
                    ).await.is_ok()
                });
            }
            2 => {
                // Create new attribute
                tasks.spawn(async move {
                    let attr_req = json!({
                        "componentType": "loan",
                        "componentId": "main",
                        "attributeName": format!("new-attr{}", i),
                        "datatypeId": datatype_id.as_str()
                    });
                    ctx.post::<serde_json::Value>(
                        &format!("/api/products/{}/abstract-attributes", product_id),
                        &attr_req,
                    ).await.is_ok()
                });
            }
            _ => {
                // Validate product
                tasks.spawn(async move {
                    ctx.get::<serde_json::Value>(
                        &format!("/api/products/{}/validate", product_id),
                    ).await.is_ok()
                });
            }
        }
    }

    // Collect results
    let mut successes = 0;
    while let Some(result) = tasks.join_next().await {
        if let Ok(success) = result {
            if success {
                successes += 1;
            }
        }
    }

    // Most operations should succeed
    assert!(successes >= 15, "Most operations should succeed: {} out of 20", successes);
}

/// Test: Concurrent batch evaluation
#[tokio::test]
async fn test_concurrent_batch_evaluation() {
    let ctx = Arc::new(TestContext::new().await.expect("Failed to create test context"));
    let product_id = format!("batchConcurrent_{}", uuid::Uuid::new_v4().to_string().replace('-', "").chars().take(8).collect::<String>());
    let datatype_id = format!("decimal{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // Create product with rule
    let product_req = json!({
        "id": product_id,
        "name": "Batch Concurrent Test",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>("/api/products", &product_req)
        .await
        .expect("Failed to create product");

    // Create attributes
    for name in ["x", "y"] {
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
        .expect("Failed to create attribute");
    }

    // Create rule - API expects correct format with full paths
    let rule_req = json!({
        "ruleType": "calculation",
        "displayExpression": "loan/main/x + 1",
        "expressionJson": r#"{"+": [{"var": "loan/main/x"}, 1]}"#,
        "inputAttributes": ["loan/main/x"],
        "outputAttributes": ["loan/main/y"]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule_req,
    )
    .await
    .expect("Failed to create rule");

    let product_id = Arc::new(product_id);
    let mut tasks = JoinSet::new();

    // Submit multiple batch evaluations concurrently
    for batch_idx in 0..5 {
        let ctx = ctx.clone();
        let product_id = product_id.clone();
        tasks.spawn(async move {
            let requests: Vec<serde_json::Value> = (0..10)
                .map(|i| json!({
                    "requestId": format!("batch-{}-req-{}", batch_idx, i),
                    "inputData": {
                        "loan/main/x": {"type": "int", "value": i * 10}
                    }
                }))
                .collect();

            let batch_req = json!({
                "productId": product_id.as_str(),
                "requests": requests
            });

            let result = ctx.post::<serde_json::Value>("/api/batch-evaluate", &batch_req).await;
            match result {
                Ok(resp) => {
                    let results = resp["results"].as_array().unwrap();
                    results.len() == 10 && results.iter().all(|r| r["success"] == true)
                }
                Err(_) => false,
            }
        });
    }

    // Collect results
    let mut successes = 0;
    while let Some(result) = tasks.join_next().await {
        if let Ok(success) = result {
            if success {
                successes += 1;
            }
        }
    }

    assert_eq!(successes, 5, "All batch evaluations should succeed");
}
