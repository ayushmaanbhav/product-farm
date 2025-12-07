//! Rule Ordering Tests
//!
//! Tests for rule order_index behavior and sorting.

use crate::fixtures::{
    assertions::*,
    data_builders::{AbstractAttributeBuilder, DatatypeBuilder, ProductBuilder, RuleBuilder},
    TestContext,
};
use product_farm_api::rest::types::{
    AbstractAttributeResponse, DatatypeResponse, ListRulesResponse, ProductResponse, RuleResponse,
    UpdateRuleRequest,
};

// =============================================================================
// Order Index Basic Tests
// =============================================================================

#[tokio::test]
async fn test_rules_sorted_by_order_index() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("sorted_rules_prod");
    let datatype_id = ctx.unique_id("sorted_rules_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attributes for 3 rules
    for i in 0..3 {
        let input = AbstractAttributeBuilder::new("calc", &format!("in{}", i))
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &input
        ).await.expect("Failed to create input");

        let output = AbstractAttributeBuilder::new("calc", &format!("out{}", i))
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &output
        ).await.expect("Failed to create output");
    }

    // Create rules in reverse order (2, 1, 0)
    for i in (0..3).rev() {
        let rule = RuleBuilder::new("calculation")
            .with_inputs(vec![&format!("calc/in{}", i)])
            .with_outputs(vec![&format!("calc/out{}", i)])
            .multiply(&format!("calc/in{}", i), 2.0, &format!("calc/out{}", i))
            .with_order(i)
            .build();
        ctx.server.post::<_, RuleResponse>(
            &format!("/api/products/{}/rules", product_id),
            &rule
        ).await.expect("Failed to create rule");
    }

    // List rules - should be sorted by order_index
    let response: ListRulesResponse = ctx.server
        .get(&format!("/api/products/{}/rules?pageSize=100", product_id))
        .await
        .expect("Failed to list rules");

    // Verify order
    let mut prev_order = -1;
    for rule in &response.items {
        assert!(
            rule.order_index >= prev_order,
            "Rules not sorted: {} should come after {}",
            rule.order_index,
            prev_order
        );
        prev_order = rule.order_index;
    }

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_default_order_index() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("default_order_prod");
    let datatype_id = ctx.unique_id("default_order_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input = AbstractAttributeBuilder::new("calc", "in")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input
    ).await.expect("Failed to create input");

    let output = AbstractAttributeBuilder::new("calc", "out")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output
    ).await.expect("Failed to create output");

    // Create rule without explicit order_index
    let rule = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/in"])
        .with_outputs(vec!["calc/out"])
        .multiply("calc/in", 2.0, "calc/out")
        // No with_order() call - use default
        .build();

    let response: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule)
        .await
        .expect("Failed to create rule");

    // Default order_index should be 0
    assert_eq!(response.order_index, 0);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_same_order_index_multiple_rules() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("same_order_prod");
    let datatype_id = ctx.unique_id("same_order_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create 3 rules with same order_index
    for i in 0..3 {
        let input = AbstractAttributeBuilder::new("calc", &format!("same_in{}", i))
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &input
        ).await.expect("Failed to create input");

        let output = AbstractAttributeBuilder::new("calc", &format!("same_out{}", i))
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &output
        ).await.expect("Failed to create output");

        let rule = RuleBuilder::new("calculation")
            .with_inputs(vec![&format!("calc/same_in{}", i)])
            .with_outputs(vec![&format!("calc/same_out{}", i)])
            .multiply(&format!("calc/same_in{}", i), 2.0, &format!("calc/same_out{}", i))
            .with_order(5) // Same order for all
            .build();
        ctx.server.post::<_, RuleResponse>(
            &format!("/api/products/{}/rules", product_id),
            &rule
        ).await.expect("Failed to create rule");
    }

    // All rules should have order_index 5
    let response: ListRulesResponse = ctx.server
        .get(&format!("/api/products/{}/rules?pageSize=100", product_id))
        .await
        .expect("Failed to list rules");

    for rule in &response.items {
        assert_eq!(rule.order_index, 5);
    }

    ctx.cleanup().await.ok();
}

// =============================================================================
// Update Order Index Tests
// =============================================================================

#[tokio::test]
async fn test_update_rule_order_index() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("upd_order_prod");
    let datatype_id = ctx.unique_id("upd_order_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input = AbstractAttributeBuilder::new("calc", "x")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input
    ).await.expect("Failed to create input");

    let output = AbstractAttributeBuilder::new("calc", "y")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output
    ).await.expect("Failed to create output");

    // Create rule with order_index 0
    let rule = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/x"])
        .with_outputs(vec!["calc/y"])
        .multiply("calc/x", 2.0, "calc/y")
        .with_order(0)
        .build();

    let created: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule)
        .await
        .expect("Failed to create rule");

    assert_rule_order(&created, 0);

    // Update order_index
    let update = UpdateRuleRequest {
        rule_type: None,
        input_attributes: None,
        output_attributes: None,
        display_expression: None,
        expression_json: None,
        description: None,
        enabled: None,
        order_index: Some(10),
    };

    let updated: RuleResponse = ctx.server
        .put(&format!("/api/rules/{}", created.id), &update)
        .await
        .expect("Failed to update rule");

    assert_rule_order(&updated, 10);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_reorder_rules() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("reorder_prod");
    let datatype_id = ctx.unique_id("reorder_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create 3 rules
    let mut rule_ids = Vec::new();
    for i in 0..3 {
        let input = AbstractAttributeBuilder::new("calc", &format!("reord_in{}", i))
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &input
        ).await.expect("Failed to create input");

        let output = AbstractAttributeBuilder::new("calc", &format!("reord_out{}", i))
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &output
        ).await.expect("Failed to create output");

        let rule = RuleBuilder::new("calculation")
            .with_inputs(vec![&format!("calc/reord_in{}", i)])
            .with_outputs(vec![&format!("calc/reord_out{}", i)])
            .multiply(&format!("calc/reord_in{}", i), 2.0, &format!("calc/reord_out{}", i))
            .with_order(i)
            .build();
        let created: RuleResponse = ctx.server
            .post(&format!("/api/products/{}/rules", product_id), &rule)
            .await
            .expect("Failed to create rule");
        rule_ids.push(created.id);
    }

    // Reorder: move rule 0 to position 2
    let update = UpdateRuleRequest {
        rule_type: None,
        input_attributes: None,
        output_attributes: None,
        display_expression: None,
        expression_json: None,
        description: None,
        enabled: None,
        order_index: Some(2),
    };

    ctx.server.put::<_, RuleResponse>(
        &format!("/api/rules/{}", rule_ids[0]),
        &update
    ).await.expect("Failed to update rule order");

    // Verify new order
    let list: ListRulesResponse = ctx.server
        .get(&format!("/api/products/{}/rules?pageSize=100", product_id))
        .await
        .expect("Failed to list rules");

    println!("Rules after reorder:");
    for rule in &list.items {
        println!("  {} -> order {}", rule.id, rule.order_index);
    }

    ctx.cleanup().await.ok();
}

// =============================================================================
// Large Order Index Tests
// =============================================================================

#[tokio::test]
async fn test_large_order_index_values() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("large_order_prod");
    let datatype_id = ctx.unique_id("large_order_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input = AbstractAttributeBuilder::new("calc", "large_in")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input
    ).await.expect("Failed to create input");

    let output = AbstractAttributeBuilder::new("calc", "large_out")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output
    ).await.expect("Failed to create output");

    // Create rule with large order_index
    let rule = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/large_in"])
        .with_outputs(vec!["calc/large_out"])
        .multiply("calc/large_in", 2.0, "calc/large_out")
        .with_order(1000000)
        .build();

    let response: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule)
        .await
        .expect("Failed to create rule");

    assert_eq!(response.order_index, 1000000);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Order Index Gap Tests
// =============================================================================

#[tokio::test]
async fn test_order_index_with_gaps() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("gap_order_prod");
    let datatype_id = ctx.unique_id("gap_order_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create rules with gaps: 0, 10, 20
    let order_indices = [0, 10, 20];
    for (idx, order) in order_indices.iter().enumerate() {
        let input = AbstractAttributeBuilder::new("calc", &format!("gap_in{}", idx))
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &input
        ).await.expect("Failed to create input");

        let output = AbstractAttributeBuilder::new("calc", &format!("gap_out{}", idx))
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &output
        ).await.expect("Failed to create output");

        let rule = RuleBuilder::new("calculation")
            .with_inputs(vec![&format!("calc/gap_in{}", idx)])
            .with_outputs(vec![&format!("calc/gap_out{}", idx)])
            .multiply(&format!("calc/gap_in{}", idx), 2.0, &format!("calc/gap_out{}", idx))
            .with_order(*order)
            .build();
        ctx.server.post::<_, RuleResponse>(
            &format!("/api/products/{}/rules", product_id),
            &rule
        ).await.expect("Failed to create rule");
    }

    // List and verify order is preserved
    let response: ListRulesResponse = ctx.server
        .get(&format!("/api/products/{}/rules?pageSize=100", product_id))
        .await
        .expect("Failed to list rules");

    let orders: Vec<i32> = response.items.iter().map(|r| r.order_index).collect();
    assert!(orders.windows(2).all(|w| w[0] <= w[1]), "Rules should be sorted even with gaps");

    ctx.cleanup().await.ok();
}

// =============================================================================
// Insert Between Tests
// =============================================================================

#[tokio::test]
async fn test_insert_rule_between_existing() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("insert_order_prod");
    let datatype_id = ctx.unique_id("insert_order_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create rules at positions 0 and 10
    for (idx, order) in [(0, 0), (1, 10)].iter() {
        let input = AbstractAttributeBuilder::new("calc", &format!("ins_in{}", idx))
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &input
        ).await.expect("Failed to create input");

        let output = AbstractAttributeBuilder::new("calc", &format!("ins_out{}", idx))
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &output
        ).await.expect("Failed to create output");

        let rule = RuleBuilder::new("calculation")
            .with_inputs(vec![&format!("calc/ins_in{}", idx)])
            .with_outputs(vec![&format!("calc/ins_out{}", idx)])
            .multiply(&format!("calc/ins_in{}", idx), 2.0, &format!("calc/ins_out{}", idx))
            .with_order(*order)
            .build();
        ctx.server.post::<_, RuleResponse>(
            &format!("/api/products/{}/rules", product_id),
            &rule
        ).await.expect("Failed to create rule");
    }

    // Insert new rule between them at position 5
    let input = AbstractAttributeBuilder::new("calc", "middle_in")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input
    ).await.expect("Failed to create input");

    let output = AbstractAttributeBuilder::new("calc", "middle_out")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output
    ).await.expect("Failed to create output");

    let middle_rule = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/middle_in"])
        .with_outputs(vec!["calc/middle_out"])
        .multiply("calc/middle_in", 2.0, "calc/middle_out")
        .with_order(5) // Between 0 and 10
        .build();

    let inserted: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &middle_rule)
        .await
        .expect("Failed to create middle rule");

    assert_eq!(inserted.order_index, 5);

    // Verify order: 0, 5, 10
    let response: ListRulesResponse = ctx.server
        .get(&format!("/api/products/{}/rules?pageSize=100", product_id))
        .await
        .expect("Failed to list rules");

    let orders: Vec<i32> = response.items.iter().map(|r| r.order_index).collect();
    assert!(orders.contains(&0));
    assert!(orders.contains(&5));
    assert!(orders.contains(&10));

    ctx.cleanup().await.ok();
}
