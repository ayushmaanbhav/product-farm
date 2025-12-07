//! Rule Dependency Tests
//!
//! Tests for rule DAG construction and cyclic dependency detection.

use crate::fixtures::{
    data_builders::{AbstractAttributeBuilder, DatatypeBuilder, ProductBuilder, RuleBuilder},
    TestContext,
};
use product_farm_api::rest::types::{
    AbstractAttributeResponse, DatatypeResponse, ProductResponse, RuleResponse,
};
use reqwest::StatusCode;

// =============================================================================
// Valid Rule Chain Tests
// =============================================================================

#[tokio::test]
async fn test_create_two_rule_chain() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("chain2_prod");
    let datatype_id = ctx.unique_id("chain2_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attributes: A -> B -> C
    for name in &["a", "b", "c"] {
        let attr = AbstractAttributeBuilder::new("calc", name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr
        ).await.expect("Failed to create attribute");
    }

    // Rule 1: A -> B
    let rule1 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/a"])
        .with_outputs(vec!["calc/b"])
        .multiply("calc/a", 2.0, "calc/b")
        .with_order(0)
        .build();
    let r1: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule1)
        .await
        .expect("Failed to create rule 1");

    // Rule 2: B -> C
    let rule2 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/b"])
        .with_outputs(vec!["calc/c"])
        .multiply("calc/b", 3.0, "calc/c")
        .with_order(1)
        .build();
    let r2: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule2)
        .await
        .expect("Failed to create rule 2");

    // Both rules should be created
    assert!(r1.enabled);
    assert!(r2.enabled);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_three_rule_chain() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("chain3_prod");
    let datatype_id = ctx.unique_id("chain3_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attributes: A -> B -> C -> D
    for name in &["a", "b", "c", "d"] {
        let attr = AbstractAttributeBuilder::new("calc", name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr
        ).await.expect("Failed to create attribute");
    }

    // Rule 1: A -> B
    let rule1 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/a"])
        .with_outputs(vec!["calc/b"])
        .multiply("calc/a", 2.0, "calc/b")
        .with_order(0)
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule1
    ).await.expect("Failed to create rule 1");

    // Rule 2: B -> C
    let rule2 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/b"])
        .with_outputs(vec!["calc/c"])
        .multiply("calc/b", 3.0, "calc/c")
        .with_order(1)
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule2
    ).await.expect("Failed to create rule 2");

    // Rule 3: C -> D
    let rule3 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/c"])
        .with_outputs(vec!["calc/d"])
        .multiply("calc/c", 4.0, "calc/d")
        .with_order(2)
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule3
    ).await.expect("Failed to create rule 3");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_parallel_rules() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("parallel_prod");
    let datatype_id = ctx.unique_id("parallel_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attributes: A -> (B, C) both independent
    for name in &["a", "b", "c"] {
        let attr = AbstractAttributeBuilder::new("calc", name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr
        ).await.expect("Failed to create attribute");
    }

    // Rule 1: A -> B
    let rule1 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/a"])
        .with_outputs(vec!["calc/b"])
        .multiply("calc/a", 2.0, "calc/b")
        .with_order(0)
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule1
    ).await.expect("Failed to create rule 1");

    // Rule 2: A -> C (parallel to rule 1)
    let rule2 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/a"])
        .with_outputs(vec!["calc/c"])
        .multiply("calc/a", 3.0, "calc/c")
        .with_order(0) // Same order level
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule2
    ).await.expect("Failed to create rule 2");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_diamond_dependency() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("diamond_prod");
    let datatype_id = ctx.unique_id("diamond_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Diamond pattern: A -> B, A -> C, B+C -> D
    for name in &["a", "b", "c", "d"] {
        let attr = AbstractAttributeBuilder::new("calc", name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr
        ).await.expect("Failed to create attribute");
    }

    // Rule 1: A -> B
    let rule1 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/a"])
        .with_outputs(vec!["calc/b"])
        .multiply("calc/a", 2.0, "calc/b")
        .with_order(0)
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule1
    ).await.expect("Failed to create rule 1");

    // Rule 2: A -> C
    let rule2 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/a"])
        .with_outputs(vec!["calc/c"])
        .multiply("calc/a", 3.0, "calc/c")
        .with_order(0)
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule2
    ).await.expect("Failed to create rule 2");

    // Rule 3: B + C -> D
    let expression = serde_json::json!({"+": [{"var": "calc/b"}, {"var": "calc/c"}]});
    let rule3 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/b", "calc/c"])
        .with_outputs(vec!["calc/d"])
        .with_expression_json(expression)
        .with_display("calc/d = calc/b + calc/c")
        .with_order(1)
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule3
    ).await.expect("Failed to create rule 3");

    ctx.cleanup().await.ok();
}

// =============================================================================
// Cyclic Dependency Detection Tests
// =============================================================================

#[tokio::test]
async fn test_detect_direct_cycle() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("direct_cycle_prod");
    let datatype_id = ctx.unique_id("direct_cycle_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attribute A that references itself
    let attr = AbstractAttributeBuilder::new("calc", "a")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr
    ).await.expect("Failed to create attribute");

    // Try to create rule: A -> A (direct cycle)
    let rule = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/a"])
        .with_outputs(vec!["calc/a"])
        .multiply("calc/a", 2.0, "calc/a")
        .build();

    let response = ctx.server
        .post_response(&format!("/api/products/{}/rules", product_id), &rule)
        .await
        .expect("Request should complete");

    // Direct cycle should be rejected
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_detect_indirect_cycle_ab() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("cycle_ab_prod");
    let datatype_id = ctx.unique_id("cycle_ab_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attributes A and B
    for name in &["a", "b"] {
        let attr = AbstractAttributeBuilder::new("calc", name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr
        ).await.expect("Failed to create attribute");
    }

    // Rule 1: A -> B
    let rule1 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/a"])
        .with_outputs(vec!["calc/b"])
        .multiply("calc/a", 2.0, "calc/b")
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule1
    ).await.expect("Failed to create rule 1");

    // Rule 2: B -> A (creates cycle A -> B -> A)
    let rule2 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/b"])
        .with_outputs(vec!["calc/a"])
        .multiply("calc/b", 3.0, "calc/a")
        .build();

    let response = ctx.server
        .post_response(&format!("/api/products/{}/rules", product_id), &rule2)
        .await
        .expect("Request should complete");

    // Cycle should be detected and rejected
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_detect_indirect_cycle_abc() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("cycle_abc_prod");
    let datatype_id = ctx.unique_id("cycle_abc_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attributes A, B, C
    for name in &["a", "b", "c"] {
        let attr = AbstractAttributeBuilder::new("calc", name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr
        ).await.expect("Failed to create attribute");
    }

    // Rule 1: A -> B
    let rule1 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/a"])
        .with_outputs(vec!["calc/b"])
        .multiply("calc/a", 2.0, "calc/b")
        .with_order(0)
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule1
    ).await.expect("Failed to create rule 1");

    // Rule 2: B -> C
    let rule2 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/b"])
        .with_outputs(vec!["calc/c"])
        .multiply("calc/b", 3.0, "calc/c")
        .with_order(1)
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule2
    ).await.expect("Failed to create rule 2");

    // Rule 3: C -> A (creates cycle A -> B -> C -> A)
    let rule3 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/c"])
        .with_outputs(vec!["calc/a"])
        .multiply("calc/c", 4.0, "calc/a")
        .with_order(2)
        .build();

    let response = ctx.server
        .post_response(&format!("/api/products/{}/rules", product_id), &rule3)
        .await
        .expect("Request should complete");

    // Cycle should be detected and rejected
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Multiple Output Dependency Tests
// =============================================================================

#[tokio::test]
async fn test_rule_with_shared_output() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("shared_out_prod");
    let datatype_id = ctx.unique_id("shared_out_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attributes
    for name in &["a", "b", "shared_out"] {
        let attr = AbstractAttributeBuilder::new("calc", name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr
        ).await.expect("Failed to create attribute");
    }

    // Rule 1: A -> shared_out
    let rule1 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/a"])
        .with_outputs(vec!["calc/shared_out"])
        .multiply("calc/a", 2.0, "calc/shared_out")
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule1
    ).await.expect("Failed to create rule 1");

    // Rule 2: B -> shared_out (same output as rule 1)
    let rule2 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/b"])
        .with_outputs(vec!["calc/shared_out"])
        .multiply("calc/b", 3.0, "calc/shared_out")
        .build();

    let response = ctx.server
        .post_response(&format!("/api/products/{}/rules", product_id), &rule2)
        .await
        .expect("Request should complete");

    // Document behavior - might be allowed (conflict resolution) or rejected
    println!("Shared output response: {}", response.status());

    ctx.cleanup().await.ok();
}

// =============================================================================
// Rule Update Dependency Tests
// =============================================================================

#[tokio::test]
async fn test_update_rule_creates_cycle() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("upd_cycle_prod");
    let datatype_id = ctx.unique_id("upd_cycle_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attributes A, B, C
    for name in &["a", "b", "c"] {
        let attr = AbstractAttributeBuilder::new("calc", name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr
        ).await.expect("Failed to create attribute");
    }

    // Rule 1: A -> B
    let rule1 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/a"])
        .with_outputs(vec!["calc/b"])
        .multiply("calc/a", 2.0, "calc/b")
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule1
    ).await.expect("Failed to create rule 1");

    // Rule 2: B -> C (initially valid chain)
    let rule2 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/b"])
        .with_outputs(vec!["calc/c"])
        .multiply("calc/b", 3.0, "calc/c")
        .build();
    let created: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule2)
        .await
        .expect("Failed to create rule 2");

    // Try to update rule 2 to: B -> A (would create cycle: A -> B -> A)
    // Rule 1: A -> B, Rule 2 (updated): B -> A creates cycle
    let update = product_farm_api::rest::types::UpdateRuleRequest {
        rule_type: None,
        input_attributes: None, // Keep input as B
        output_attributes: Some(vec!["calc/a".to_string()]), // Change output from C to A
        display_expression: Some("calc/a = calc/b * 4".to_string()),
        expression_json: Some(serde_json::json!({"*": [{"var": "calc/b"}, 4]}).to_string()),
        description: None,
        enabled: None,
        order_index: None,
    };

    let response = ctx.server
        .put_response(&format!("/api/rules/{}", created.id), &update)
        .await
        .expect("Request should complete");

    // Cycle should be detected
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Independent Rules Tests
// =============================================================================

#[tokio::test]
async fn test_independent_rules_no_dependency() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("indep_prod");
    let datatype_id = ctx.unique_id("indep_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create independent attribute pairs
    for name in &["a1", "b1", "a2", "b2"] {
        let attr = AbstractAttributeBuilder::new("calc", name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr
        ).await.expect("Failed to create attribute");
    }

    // Rule 1: A1 -> B1
    let rule1 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/a1"])
        .with_outputs(vec!["calc/b1"])
        .multiply("calc/a1", 2.0, "calc/b1")
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule1
    ).await.expect("Failed to create rule 1");

    // Rule 2: A2 -> B2 (completely independent)
    let rule2 = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/a2"])
        .with_outputs(vec!["calc/b2"])
        .multiply("calc/a2", 3.0, "calc/b2")
        .build();
    ctx.server.post::<_, RuleResponse>(
        &format!("/api/products/{}/rules", product_id),
        &rule2
    ).await.expect("Failed to create rule 2");

    // Both should succeed - no dependency issues
    let rules: product_farm_api::rest::types::ListRulesResponse = ctx.server
        .get(&format!("/api/products/{}/rules?pageSize=100", product_id))
        .await
        .expect("Failed to list rules");

    assert!(rules.items.len() >= 2);

    ctx.cleanup().await.ok();
}
