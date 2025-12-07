//! Rule Chain Evaluation Tests
//!
//! Tests for multi-rule chains where output of one rule becomes input of another.

use crate::fixtures::*;
use serde_json::json;

/// Test two-rule chain: A → B
#[tokio::test]
async fn test_two_rule_chain() {
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
        "name": "Chain Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes
    for name in ["input", "doubled", "quadrupled"] {
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

    // Rule 1: doubled = input * 2
    let expr1 = json!({"*": [{"var": "loan/main/input"}, 2]});
    let rule1_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/doubled = loan/main/input * 2",
        "expressionJson": serde_json::to_string(&expr1).unwrap(),
        "inputAttributes": ["loan/main/input"],
        "outputAttributes": ["loan/main/doubled"],
        "orderIndex": 0
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule1_req,
    )
    .await
    .expect("Failed to create rule 1");

    // Rule 2: quadrupled = doubled * 2 (depends on rule 1 output)
    let expr2 = json!({"*": [{"var": "loan/main/doubled"}, 2]});
    let rule2_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/quadrupled = loan/main/doubled * 2",
        "expressionJson": serde_json::to_string(&expr2).unwrap(),
        "inputAttributes": ["loan/main/doubled"],
        "outputAttributes": ["loan/main/quadrupled"],
        "orderIndex": 1
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule2_req,
    )
    .await
    .expect("Failed to create rule 2");

    // Evaluate with input = 5
    let eval_req = json!({
        "productId": product_id,
        "inputData": {
            "loan/main/input": {"type": "int", "value": 5}
        }
    });

    let response: serde_json::Value = ctx.post("/api/evaluate", &eval_req)
        .await
        .expect("Failed to evaluate");

    assert_eq!(response["success"], true);

    // Verify rule results show both rules executed
    let rule_results = response["ruleResults"].as_array().unwrap();
    assert_eq!(rule_results.len(), 2, "Both rules should execute");

    // Verify outputs exist
    let outputs = &response["outputs"];
    assert!(outputs["loan/main/doubled"].is_object());
    assert!(outputs["loan/main/quadrupled"].is_object());
}

/// Test three-rule chain: A → B → C
#[tokio::test]
async fn test_three_rule_chain() {
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
        "name": "Three Chain Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes
    for name in ["base", "step-one", "step-two", "final"] {
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

    // Rule 1: step-one = base + 10
    let expr1 = json!({"+": [{"var": "loan/main/base"}, 10]});
    let rule1_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/step-one = loan/main/base + 10",
        "expressionJson": serde_json::to_string(&expr1).unwrap(),
        "inputAttributes": ["loan/main/base"],
        "outputAttributes": ["loan/main/step-one"],
        "orderIndex": 0
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule1_req,
    )
    .await
    .expect("Failed to create rule 1");

    // Rule 2: step-two = step-one * 2
    let expr2 = json!({"*": [{"var": "loan/main/step-one"}, 2]});
    let rule2_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/step-two = loan/main/step-one * 2",
        "expressionJson": serde_json::to_string(&expr2).unwrap(),
        "inputAttributes": ["loan/main/step-one"],
        "outputAttributes": ["loan/main/step-two"],
        "orderIndex": 1
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule2_req,
    )
    .await
    .expect("Failed to create rule 2");

    // Rule 3: final = step-two - 5
    let expr3 = json!({"-": [{"var": "loan/main/step-two"}, 5]});
    let rule3_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/final = loan/main/step-two - 5",
        "expressionJson": serde_json::to_string(&expr3).unwrap(),
        "inputAttributes": ["loan/main/step-two"],
        "outputAttributes": ["loan/main/final"],
        "orderIndex": 2
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule3_req,
    )
    .await
    .expect("Failed to create rule 3");

    // Evaluate with base = 5
    // Expected: step-one = 15, step-two = 30, final = 25
    let eval_req = json!({
        "productId": product_id,
        "inputData": {
            "loan/main/base": {"type": "int", "value": 5}
        }
    });

    let response: serde_json::Value = ctx.post("/api/evaluate", &eval_req)
        .await
        .expect("Failed to evaluate");

    assert_eq!(response["success"], true);

    // Verify all three rules executed
    let rule_results = response["ruleResults"].as_array().unwrap();
    assert_eq!(rule_results.len(), 3);

    // Verify all intermediate outputs
    let outputs = &response["outputs"];
    assert!(outputs["loan/main/step-one"].is_object());
    assert!(outputs["loan/main/step-two"].is_object());
    assert!(outputs["loan/main/final"].is_object());
}

/// Test parallel rules at same level
#[tokio::test]
async fn test_parallel_rules_same_level() {
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
        "name": "Parallel Rules Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes
    for name in ["input", "output-a", "output-b", "output-c"] {
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

    // Three rules at same order_index, all reading from same input
    for (suffix, multiplier) in [("a", 2), ("b", 3), ("c", 4)] {
        let expr = json!({"*": [{"var": "loan/main/input"}, multiplier]});
        let rule_req = json!({
            "ruleType": "CALCULATION",
            "displayExpression": format!("loan/main/output-{} = loan/main/input * {}", suffix, multiplier),
            "expressionJson": serde_json::to_string(&expr).unwrap(),
            "inputAttributes": ["loan/main/input"],
            "outputAttributes": [format!("loan/main/output-{}", suffix)],
            "orderIndex": 0
        });
        ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/rules", product_id),
            &rule_req,
        )
        .await
        .expect("Failed to create rule");
    }

    // Evaluate with input = 10
    let eval_req = json!({
        "productId": product_id,
        "inputData": {
            "loan/main/input": {"type": "int", "value": 10}
        }
    });

    let response: serde_json::Value = ctx.post("/api/evaluate", &eval_req)
        .await
        .expect("Failed to evaluate");

    assert_eq!(response["success"], true);

    // Verify all three rules executed
    let rule_results = response["ruleResults"].as_array().unwrap();
    assert_eq!(rule_results.len(), 3);

    // Verify all outputs
    let outputs = &response["outputs"];
    assert!(outputs["loan/main/output-a"].is_object());
    assert!(outputs["loan/main/output-b"].is_object());
    assert!(outputs["loan/main/output-c"].is_object());
}

/// Test diamond dependency pattern: A → B, A → C, B + C → D
#[tokio::test]
async fn test_diamond_pattern() {
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
        "name": "Diamond Pattern Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes
    for name in ["input", "branch-a", "branch-b", "merged"] {
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

    // Rule A1: branch-a = input * 2
    let expr_a1 = json!({"*": [{"var": "loan/main/input"}, 2]});
    let rule_a1 = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/branch-a = loan/main/input * 2",
        "expressionJson": serde_json::to_string(&expr_a1).unwrap(),
        "inputAttributes": ["loan/main/input"],
        "outputAttributes": ["loan/main/branch-a"],
        "orderIndex": 0
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule_a1,
    )
    .await
    .expect("Failed to create rule A1");

    // Rule A2: branch-b = input * 3
    let expr_a2 = json!({"*": [{"var": "loan/main/input"}, 3]});
    let rule_a2 = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/branch-b = loan/main/input * 3",
        "expressionJson": serde_json::to_string(&expr_a2).unwrap(),
        "inputAttributes": ["loan/main/input"],
        "outputAttributes": ["loan/main/branch-b"],
        "orderIndex": 0
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule_a2,
    )
    .await
    .expect("Failed to create rule A2");

    // Rule D: merged = branch-a + branch-b
    let expr_d = json!({"+": [{"var": "loan/main/branch-a"}, {"var": "loan/main/branch-b"}]});
    let rule_d = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/merged = loan/main/branch-a + loan/main/branch-b",
        "expressionJson": serde_json::to_string(&expr_d).unwrap(),
        "inputAttributes": ["loan/main/branch-a", "loan/main/branch-b"],
        "outputAttributes": ["loan/main/merged"],
        "orderIndex": 1
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule_d,
    )
    .await
    .expect("Failed to create rule D");

    // Evaluate with input = 10
    // Expected: branch-a = 20, branch-b = 30, merged = 50
    let eval_req = json!({
        "productId": product_id,
        "inputData": {
            "loan/main/input": {"type": "int", "value": 10}
        }
    });

    let response: serde_json::Value = ctx.post("/api/evaluate", &eval_req)
        .await
        .expect("Failed to evaluate");

    assert_eq!(response["success"], true);

    // Verify all rules executed
    let rule_results = response["ruleResults"].as_array().unwrap();
    assert_eq!(rule_results.len(), 3);

    // Verify outputs
    let outputs = &response["outputs"];
    assert!(outputs["loan/main/branch-a"].is_object());
    assert!(outputs["loan/main/branch-b"].is_object());
    assert!(outputs["loan/main/merged"].is_object());
}

/// Test rule with multiple inputs
#[tokio::test]
async fn test_rule_with_multiple_inputs() {
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
        "name": "Multi Input Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes
    for name in ["principal", "rate", "term", "emi"] {
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

    // EMI = P * r * (1 + r)^n / ((1 + r)^n - 1)
    // Simplified: emi = principal * rate / 12 (just for test)
    let expr = json!({"/": [{"*": [{"var": "loan/main/principal"}, {"var": "loan/main/rate"}]}, 12]});
    let rule_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/emi = loan/main/principal * loan/main/rate / 12",
        "expressionJson": serde_json::to_string(&expr).unwrap(),
        "inputAttributes": ["loan/main/principal", "loan/main/rate", "loan/main/term"],
        "outputAttributes": ["loan/main/emi"]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule_req,
    )
    .await
    .expect("Failed to create rule");

    // Evaluate
    let eval_req = json!({
        "productId": product_id,
        "inputData": {
            "loan/main/principal": {"type": "float", "value": 100000.0},
            "loan/main/rate": {"type": "float", "value": 0.12},
            "loan/main/term": {"type": "int", "value": 12}
        }
    });

    let response: serde_json::Value = ctx.post("/api/evaluate", &eval_req)
        .await
        .expect("Failed to evaluate");

    assert_eq!(response["success"], true);
    assert!(response["outputs"]["loan/main/emi"].is_object());
}

/// Test rule with multiple outputs
#[tokio::test]
async fn test_rule_with_multiple_outputs() {
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
        "name": "Multi Output Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes
    for name in ["input", "doubled", "tripled"] {
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

    // Note: JSON Logic typically outputs a single value. For multiple outputs,
    // the API would need to handle object/array outputs. Testing API handles the structure.
    let expr = json!({"*": [{"var": "loan/main/input"}, 2]});
    let rule_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/doubled = loan/main/input * 2",
        "expressionJson": serde_json::to_string(&expr).unwrap(),
        "inputAttributes": ["loan/main/input"],
        "outputAttributes": ["loan/main/doubled", "loan/main/tripled"]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule_req,
    )
    .await
    .expect("Failed to create rule");

    let eval_req = json!({
        "productId": product_id,
        "inputData": {
            "loan/main/input": {"type": "int", "value": 5}
        }
    });

    let response: serde_json::Value = ctx.post("/api/evaluate", &eval_req)
        .await
        .expect("Failed to evaluate");

    assert_eq!(response["success"], true);
}

/// Test execution levels are tracked correctly
#[tokio::test]
async fn test_execution_levels_in_metrics() {
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
        "name": "Levels Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create attributes
    for name in ["input", "level1", "level2"] {
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

    // Rule at level 0
    let expr1 = json!({"*": [{"var": "loan/main/input"}, 2]});
    let rule1 = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/level1 = loan/main/input * 2",
        "expressionJson": serde_json::to_string(&expr1).unwrap(),
        "inputAttributes": ["loan/main/input"],
        "outputAttributes": ["loan/main/level1"],
        "orderIndex": 0
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule1,
    )
    .await
    .expect("Failed to create rule 1");

    // Rule at level 1
    let expr2 = json!({"*": [{"var": "loan/main/level1"}, 2]});
    let rule2 = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/level2 = loan/main/level1 * 2",
        "expressionJson": serde_json::to_string(&expr2).unwrap(),
        "inputAttributes": ["loan/main/level1"],
        "outputAttributes": ["loan/main/level2"],
        "orderIndex": 1
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule2,
    )
    .await
    .expect("Failed to create rule 2");

    let eval_req = json!({
        "productId": product_id,
        "inputData": {
            "loan/main/input": {"type": "int", "value": 5}
        }
    });

    let response: serde_json::Value = ctx.post("/api/evaluate", &eval_req)
        .await
        .expect("Failed to evaluate");

    assert_eq!(response["success"], true);

    // Check metrics.levels
    let levels = response["metrics"]["levels"].as_array().unwrap();
    assert!(!levels.is_empty());

    // Each level should have level number and rule count
    for level in levels {
        assert!(level["level"].as_i64().is_some());
        assert!(level["rulesCount"].as_i64().is_some());
    }
}
