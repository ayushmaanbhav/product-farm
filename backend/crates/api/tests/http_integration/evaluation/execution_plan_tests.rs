//! Execution Plan Tests
//!
//! Tests for the /api/products/:product_id/execution-plan endpoint.

use crate::fixtures::*;
use serde_json::json;

/// Helper to set up a product with multiple rules
async fn setup_product_with_chain(ctx: &TestContext) -> String {
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
        "name": "Execution Plan Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes
    for name in ["input", "step1", "step2", "output"] {
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
    let rules = [
        ("Rule 1", 0, "input", "step1"),
        ("Rule 2", 1, "step1", "step2"),
        ("Rule 3", 2, "step2", "output"),
    ];

    for (name, order, input, output) in rules {
        let input_path = format!("loan/main/{}", input);
        let output_path = format!("loan/main/{}", output);
        let expr = json!({"*": [{"var": &input_path}, 2]});
        let rule_req = json!({
            "ruleType": "CALCULATION",
            "displayExpression": format!("{} = {} * 2", output_path, input_path),
            "expressionJson": serde_json::to_string(&expr).unwrap(),
            "orderIndex": order,
            "inputAttributes": [&input_path],
            "outputAttributes": [&output_path],
            "description": name
        });
        ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/rules", product_id),
            &rule_req,
        )
        .await
        .expect("Failed to create rule");
    }

    product_id
}

/// Test get execution plan returns levels
#[tokio::test]
async fn test_get_execution_plan_levels() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_chain(&ctx).await;

    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/execution-plan", product_id),
    )
    .await
    .expect("Failed to get execution plan");

    // Verify levels exist
    let levels = response["levels"].as_array().unwrap();
    assert!(!levels.is_empty());

    // Verify each level has proper structure
    for level in levels {
        assert!(level["level"].as_i64().is_some());
        assert!(level["ruleIds"].as_array().is_some());
    }
}

/// Test execution plan includes dependencies
#[tokio::test]
async fn test_get_execution_plan_dependencies() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_chain(&ctx).await;

    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/execution-plan", product_id),
    )
    .await
    .expect("Failed to get execution plan");

    // Verify dependencies exist
    let dependencies = response["dependencies"].as_array().unwrap();
    assert_eq!(dependencies.len(), 3, "Should have 3 rules");

    // Verify each dependency has proper structure
    for dep in dependencies {
        assert!(dep["ruleId"].is_string());
        assert!(dep["dependsOn"].as_array().is_some());
        assert!(dep["produces"].as_array().is_some());
    }
}

/// Test execution plan identifies missing inputs
#[tokio::test]
async fn test_get_execution_plan_missing_inputs() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_chain(&ctx).await;

    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/execution-plan", product_id),
    )
    .await
    .expect("Failed to get execution plan");

    // The first rule has "input" which is not produced by any rule
    let missing_inputs = response["missingInputs"].as_array().unwrap();

    // Should have at least one missing input (the initial "input" attribute)
    assert!(!missing_inputs.is_empty());

    // Verify structure
    for missing in missing_inputs {
        assert!(missing["ruleId"].is_string());
        assert!(missing["inputPath"].is_string());
    }
}

/// Test execution plan includes DOT graph
#[tokio::test]
async fn test_get_execution_plan_dot_graph() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_chain(&ctx).await;

    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/execution-plan", product_id),
    )
    .await
    .expect("Failed to get execution plan");

    let dot_graph = response["dotGraph"].as_str().unwrap();
    assert!(dot_graph.starts_with("digraph"), "DOT graph should start with 'digraph'");
    assert!(dot_graph.contains("{"), "DOT graph should contain graph body");
    assert!(dot_graph.contains("}"), "DOT graph should close properly");
}

/// Test execution plan includes Mermaid graph
#[tokio::test]
async fn test_get_execution_plan_mermaid_graph() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_chain(&ctx).await;

    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/execution-plan", product_id),
    )
    .await
    .expect("Failed to get execution plan");

    let mermaid_graph = response["mermaidGraph"].as_str().unwrap();
    assert!(mermaid_graph.starts_with("graph"), "Mermaid graph should start with 'graph'");
}

/// Test execution plan includes ASCII graph
#[tokio::test]
async fn test_get_execution_plan_ascii_graph() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = setup_product_with_chain(&ctx).await;

    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/execution-plan", product_id),
    )
    .await
    .expect("Failed to get execution plan");

    let ascii_graph = response["asciiGraph"].as_str().unwrap();
    assert!(ascii_graph.contains("Level"), "ASCII graph should show levels");
}

/// Test execution plan for product with no rules
#[tokio::test]
async fn test_get_execution_plan_no_rules() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product without rules
    let product_req = json!({
        "id": product_id,
        "name": "Empty Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/execution-plan", product_id),
    )
    .await
    .expect("Failed to get execution plan");

    // Should return empty levels and dependencies
    assert!(response["levels"].as_array().unwrap().is_empty());
    assert!(response["dependencies"].as_array().unwrap().is_empty());
    assert!(response["missingInputs"].as_array().unwrap().is_empty());
}

/// Test execution plan for non-existent product
#[tokio::test]
async fn test_get_execution_plan_product_not_found() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let result = ctx.get::<serde_json::Value>(
        "/api/products/non-existent-product/execution-plan",
    )
    .await;

    assert!(result.is_err());
}

/// Test execution plan with disabled rules excludes them
#[tokio::test]
async fn test_get_execution_plan_excludes_disabled_rules() {
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
        "name": "Disabled Rule Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
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
        .expect("Failed to create abstract attribute");
    }

    // Create enabled rule
    let expr = json!({"var": "loan/main/input"});
    let enabled_rule_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/output = loan/main/input",
        "expressionJson": serde_json::to_string(&expr).unwrap(),
        "inputAttributes": ["loan/main/input"],
        "outputAttributes": ["loan/main/output"],
        "description": "Enabled Rule"
    });
    let enabled_rule: serde_json::Value = ctx.post(
        &format!("/api/products/{}/rules", product_id),
        &enabled_rule_req,
    )
    .await
    .expect("Failed to create enabled rule");
    let enabled_rule_id = enabled_rule["id"].as_str().unwrap().to_string();

    // Create a second attribute for the disabled rule (different output to avoid conflicts)
    let attr_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "output2",
        "datatypeId": datatype_id
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req,
    )
    .await
    .expect("Failed to create abstract attribute");

    // Create disabled rule (create first, then disable via PUT)
    let expr2 = json!({"var": "loan/main/input"});
    let disabled_rule_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "loan/main/output2 = loan/main/input",
        "expressionJson": serde_json::to_string(&expr2).unwrap(),
        "inputAttributes": ["loan/main/input"],
        "outputAttributes": ["loan/main/output2"],
        "description": "Disabled Rule"
    });
    let disabled_rule: serde_json::Value = ctx.post(
        &format!("/api/products/{}/rules", product_id),
        &disabled_rule_req,
    )
    .await
    .expect("Failed to create disabled rule");
    let disabled_rule_id = disabled_rule["id"].as_str().unwrap().to_string();

    // Disable the rule via PUT
    let disable_req = json!({
        "enabled": false
    });
    ctx.put::<serde_json::Value>(
        &format!("/api/rules/{}", disabled_rule_id),
        &disable_req,
    )
    .await
    .expect("Failed to disable rule");

    // Get execution plan
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/execution-plan", product_id),
    )
    .await
    .expect("Failed to get execution plan");

    // Should only include the enabled rule
    let dependencies = response["dependencies"].as_array().unwrap();
    assert_eq!(dependencies.len(), 1, "Should only have 1 enabled rule");
    assert_eq!(dependencies[0]["ruleId"], enabled_rule_id);
}

/// Test execution plan groups rules by order_index
#[tokio::test]
async fn test_get_execution_plan_groups_by_order() {
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
        "name": "Ordered Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create attributes
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

    // Create 4 rules: 2 at order 0, 2 at order 1
    // Each rule takes one input and produces a different output
    let rule_configs = [
        (0, "a", "a_out"),  // input a -> output a_out
        (0, "b", "b_out"),  // input b -> output b_out
        (1, "c", "c_out"),  // input c -> output c_out
        (1, "d", "d_out"),  // input d -> output d_out
    ];

    // Create output attributes too
    for output in ["a_out", "b_out", "c_out", "d_out"] {
        let attr_req = json!({
            "componentType": "loan",
            "componentId": "main",
            "attributeName": output,
            "datatypeId": datatype_id
        });
        ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr_req,
        )
        .await
        .expect("Failed to create output abstract attribute");
    }

    for (i, (order, input_attr, output_attr)) in rule_configs.iter().enumerate() {
        let input_path = format!("loan/main/{}", input_attr);
        let output_path = format!("loan/main/{}", output_attr);
        let expr = json!({"var": &input_path});
        let rule_req = json!({
            "ruleType": "CALCULATION",
            "displayExpression": format!("{} = {}", output_path, input_path),
            "expressionJson": serde_json::to_string(&expr).unwrap(),
            "orderIndex": order,
            "inputAttributes": [&input_path],
            "outputAttributes": [&output_path],
            "description": format!("Rule {}", i)
        });
        ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/rules", product_id),
            &rule_req,
        )
        .await
        .expect("Failed to create rule");
    }

    // Get execution plan
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/execution-plan", product_id),
    )
    .await
    .expect("Failed to get execution plan");

    let levels = response["levels"].as_array().unwrap();
    assert_eq!(levels.len(), 2, "Should have 2 levels (order 0 and 1)");

    // Level 0 should have 2 rules
    let level_0 = levels.iter().find(|l| l["level"] == 0).unwrap();
    assert_eq!(level_0["ruleIds"].as_array().unwrap().len(), 2);

    // Level 1 should have 2 rules
    let level_1 = levels.iter().find(|l| l["level"] == 1).unwrap();
    assert_eq!(level_1["ruleIds"].as_array().unwrap().len(), 2);
}
