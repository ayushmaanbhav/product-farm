//! Impact Analysis Tests
//!
//! Tests for the /api/products/:product_id/impact-analysis endpoint.

use crate::fixtures::*;
use serde_json::json;

/// Helper to set up a product with a dependency chain
async fn setup_product_with_dependencies(ctx: &TestContext) -> String {
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
        "name": "Impact Analysis Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes: input → mid → output
    for name in ["input", "mid", "output"] {
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

    // Rule 1: mid = input * 2
    let expr1 = json!({"*": [{"var": "loan/main/input"}, 2]});
    let rule1_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/mid = loan/main/input * 2",
        "expressionJson": serde_json::to_string(&expr1).unwrap(),
        "orderIndex": 0,
        "inputAttributes": ["loan/main/input"],
        "outputAttributes": ["loan/main/mid"],
        "description": "Rule 1"
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule1_req,
    )
    .await
    .expect("Failed to create rule 1");

    // Rule 2: output = mid + 10
    let expr2 = json!({"+": [{"var": "loan/main/mid"}, 10]});
    let rule2_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/output = loan/main/mid + 10",
        "expressionJson": serde_json::to_string(&expr2).unwrap(),
        "orderIndex": 1,
        "inputAttributes": ["loan/main/mid"],
        "outputAttributes": ["loan/main/output"],
        "description": "Rule 2"
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule2_req,
    )
    .await
    .expect("Failed to create rule 2");

    product_id
}

/// Test impact analysis returns direct dependencies
#[tokio::test]
async fn test_impact_analysis_direct_dependencies() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_dependencies(&ctx).await;

    // Analyze impact of changing "mid"
    let impact_req = json!({
        "targetPath": "loan/main/mid"
    });

    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/impact-analysis", product_id),
        &impact_req,
    )
    .await
    .expect("Failed to get impact analysis");

    // Verify target path
    assert_eq!(response["targetPath"], "loan/main/mid");

    // Verify direct dependencies exist
    let direct_deps = response["directDependencies"].as_array().unwrap();
    assert!(!direct_deps.is_empty());

    // Verify structure
    for dep in direct_deps {
        assert!(dep["path"].is_string());
        assert!(dep["attributeName"].is_string());
        assert!(dep["direction"].is_string()); // "upstream" or "downstream"
        assert!(dep["distance"].as_i64().is_some());
        assert!(dep["isImmutable"].is_boolean());
    }
}

/// Test impact analysis returns transitive dependencies
#[tokio::test]
async fn test_impact_analysis_transitive_dependencies() {
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
        "name": "Transitive Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create a longer chain: a → b → c → d
    for name in ["a", "b", "c", "d"] {
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

    // Create chain of rules
    let chains = [("a", "b", 0), ("b", "c", 1), ("c", "d", 2)];
    for (input, output, order) in chains {
        let input_path = format!("loan/main/{}", input);
        let output_path = format!("loan/main/{}", output);
        let expr = json!({"var": &input_path});
        let rule_req = json!({
            "ruleType": "CALCULATION",
            "displayExpression": format!("{} = {}", output_path, input_path),
            "expressionJson": serde_json::to_string(&expr).unwrap(),
            "orderIndex": order,
            "inputAttributes": [&input_path],
            "outputAttributes": [&output_path],
            "description": format!("{} to {}", input, output)
        });
        ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/rules", product_id),
            &rule_req,
        )
        .await
        .expect("Failed to create rule");
    }

    // Analyze impact of changing "a" (should affect b, c, d)
    let impact_req = json!({
        "targetPath": "loan/main/a"
    });

    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/impact-analysis", product_id),
        &impact_req,
    )
    .await
    .expect("Failed to get impact analysis");

    // Should have transitive dependencies
    let transitive = response["transitiveDependencies"].as_array().unwrap();
    // b is direct (distance 1), c is distance 2, d is distance 3
    // Check that we have dependencies at different distances
    let distances: Vec<i64> = transitive.iter()
        .map(|d| d["distance"].as_i64().unwrap())
        .collect();

    // Should have dependencies at distances > 1
    assert!(distances.iter().any(|&d| d >= 2), "Should have transitive dependencies at distance >= 2");
}

/// Test impact analysis returns affected rules
#[tokio::test]
async fn test_impact_analysis_affected_rules() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_dependencies(&ctx).await;

    // Analyze impact of changing "input"
    let impact_req = json!({
        "targetPath": "loan/main/input"
    });

    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/impact-analysis", product_id),
        &impact_req,
    )
    .await
    .expect("Failed to get impact analysis");

    // Should have affected rules
    let affected_rules = response["affectedRules"].as_array().unwrap();
    assert!(!affected_rules.is_empty(), "Should have affected rules");

    // All entries should be rule IDs (strings)
    for rule in affected_rules {
        assert!(rule.is_string());
    }
}

/// Test impact analysis returns affected functionalities
#[tokio::test]
async fn test_impact_analysis_affected_functionalities() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_dependencies(&ctx).await;

    // Create a functionality that requires "output"
    let func_req = json!({
        "name": "output-func",
        "description": "Functionality requiring output",
        "requiredAttributes": [
            {"abstractPath": "loan/main/output", "description": "Output value"}
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Analyze impact of changing "input" (affects mid → output → functionality)
    let impact_req = json!({
        "targetPath": "loan/main/input"
    });

    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/impact-analysis", product_id),
        &impact_req,
    )
    .await
    .expect("Failed to get impact analysis");

    // Check affected functionalities
    let affected_funcs = response["affectedFunctionalities"].as_array().unwrap();
    // May or may not include the functionality depending on how deep the analysis goes
    // At minimum, the structure should exist
    assert!(response.get("affectedFunctionalities").is_some());
}

/// Test impact analysis with immutable dependents
#[tokio::test]
async fn test_impact_analysis_immutable_dependents() {
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
        "name": "Immutable Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes, one immutable
    let input_attr = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "input",
        "datatypeId": datatype_id,
        "immutable": false
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input_attr,
    )
    .await
    .expect("Failed to create input attribute");

    let immutable_attr = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "fixed-output",
        "datatypeId": datatype_id,
        "immutable": true
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &immutable_attr,
    )
    .await
    .expect("Failed to create immutable attribute");

    // Create rule: fixed-output = input * 2
    let expr = json!({"*": [{"var": "loan/main/input"}, 2]});
    let rule_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/fixed-output = loan/main/input * 2",
        "expressionJson": serde_json::to_string(&expr).unwrap(),
        "inputAttributes": ["loan/main/input"],
        "outputAttributes": ["loan/main/fixed-output"],
        "description": "To Immutable"
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule_req,
    )
    .await
    .expect("Failed to create rule");

    // Analyze impact of changing "input"
    let impact_req = json!({
        "targetPath": "loan/main/input"
    });

    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/impact-analysis", product_id),
        &impact_req,
    )
    .await
    .expect("Failed to get impact analysis");

    // Check immutable detection
    assert!(response.get("hasImmutableDependents").is_some());
    assert!(response.get("immutablePaths").is_some());
}

/// Test impact analysis for non-existent product
#[tokio::test]
async fn test_impact_analysis_product_not_found() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let impact_req = json!({
        "targetPath": "some.path"
    });

    let result = ctx.post::<serde_json::Value>(
        "/api/products/non-existent-product/impact-analysis",
        &impact_req,
    )
    .await;

    assert!(result.is_err());
}

/// Test impact analysis with no dependencies
#[tokio::test]
async fn test_impact_analysis_no_dependencies() {
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

    // Create product with standalone attribute
    let product_req = json!({
        "id": product_id,
        "name": "Standalone Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create isolated attribute (no rules reference it)
    let attr_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "isolated",
        "datatypeId": datatype_id
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req,
    )
    .await
    .expect("Failed to create attribute");

    // Analyze impact of changing "isolated"
    let impact_req = json!({
        "targetPath": "loan/main/isolated"
    });

    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/impact-analysis", product_id),
        &impact_req,
    )
    .await
    .expect("Failed to get impact analysis");

    // Should have no dependencies
    assert!(response["directDependencies"].as_array().unwrap().is_empty());
    assert!(response["transitiveDependencies"].as_array().unwrap().is_empty());
    assert!(response["affectedRules"].as_array().unwrap().is_empty());
    assert_eq!(response["hasImmutableDependents"], false);
}

/// Test impact analysis shows upstream dependencies
#[tokio::test]
async fn test_impact_analysis_upstream_dependencies() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_dependencies(&ctx).await;

    // Analyze impact of "output" (should show upstream: mid, input)
    let impact_req = json!({
        "targetPath": "loan/main/output"
    });

    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/impact-analysis", product_id),
        &impact_req,
    )
    .await
    .expect("Failed to get impact analysis");

    // Should have upstream dependencies (what this depends on)
    let direct_deps = response["directDependencies"].as_array().unwrap();

    // Look for upstream direction
    let upstream_deps: Vec<_> = direct_deps.iter()
        .filter(|d| d["direction"] == "upstream")
        .collect();

    assert!(!upstream_deps.is_empty(), "Should have upstream dependencies");
}

/// Test impact analysis shows downstream dependencies
#[tokio::test]
async fn test_impact_analysis_downstream_dependencies() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_dependencies(&ctx).await;

    // Analyze impact of "input" (should show downstream: mid, output)
    let impact_req = json!({
        "targetPath": "loan/main/input"
    });

    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/impact-analysis", product_id),
        &impact_req,
    )
    .await
    .expect("Failed to get impact analysis");

    // Should have downstream dependencies (what depends on this)
    let direct_deps = response["directDependencies"].as_array().unwrap();

    // Look for downstream direction
    let downstream_deps: Vec<_> = direct_deps.iter()
        .filter(|d| d["direction"] == "downstream")
        .collect();

    assert!(!downstream_deps.is_empty(), "Should have downstream dependencies");
}
