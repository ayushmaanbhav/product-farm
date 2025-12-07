//! Rule CRUD Tests

use crate::fixtures::{
    assertions::*,
    data_builders::{AbstractAttributeBuilder, DatatypeBuilder, ProductBuilder, RuleBuilder},
    TestContext,
};
use product_farm_api::rest::types::{
    AbstractAttributeResponse, CreateRuleRequest, DeleteResponse, DatatypeResponse,
    ListRulesResponse, ProductResponse, RuleResponse, UpdateRuleRequest,
};
use reqwest::StatusCode;

// =============================================================================
// CREATE Tests
// =============================================================================

#[tokio::test]
async fn test_create_rule_basic() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup: Create product, datatype, and attributes
    let product_id = ctx.unique_product_id("rule_prod");
    let datatype_id = ctx.unique_id("rule_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create input and output attributes
    let input_attr = AbstractAttributeBuilder::new("loan", "principal")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input_attr
    ).await.expect("Failed to create input attribute");

    let output_attr = AbstractAttributeBuilder::new("loan", "doubled_principal")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output_attr
    ).await.expect("Failed to create output attribute");

    // Create rule
    let rule_req = RuleBuilder::new("calculation")
        .with_inputs(vec!["loan/principal"])
        .with_outputs(vec!["loan/doubled_principal"])
        .multiply("loan/principal", 2.0, "loan/doubled_principal")
        .build();

    let response: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Failed to create rule");

    assert_eq!(response.rule_type, "calculation");
    assert_eq!(response.product_id, product_id);
    assert!(response.enabled);
    assert_rule_inputs_match(&response, &["loan/principal"]);
    assert_rule_outputs_match(&response, &["loan/doubled_principal"]);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_rule_with_description() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("desc_rule_prod");
    let datatype_id = ctx.unique_id("desc_rule_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input_attr = AbstractAttributeBuilder::new("premium", "base")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input_attr
    ).await.expect("Failed to create input attribute");

    let output_attr = AbstractAttributeBuilder::new("premium", "adjusted")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output_attr
    ).await.expect("Failed to create output attribute");

    let rule_req = RuleBuilder::new("calculation")
        .with_inputs(vec!["premium/base"])
        .with_outputs(vec!["premium/adjusted"])
        .with_description("Applies 10% markup to base premium")
        .multiply("premium/base", 1.1, "premium/adjusted")
        .build();

    let response: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Failed to create rule");

    assert_eq!(response.description, Some("Applies 10% markup to base premium".to_string()));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_rule_with_multiple_inputs() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("multi_input_prod");
    let datatype_id = ctx.unique_id("multi_input_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create multiple input attributes
    for name in &["input_a", "input_b", "input_c"] {
        let attr = AbstractAttributeBuilder::new("calc", name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr
        ).await.expect("Failed to create attribute");
    }

    let output = AbstractAttributeBuilder::new("calc", "result")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output
    ).await.expect("Failed to create output attribute");

    // Create rule with multiple inputs
    let expression = serde_json::json!({
        "+": [
            {"var": "calc/input_a"},
            {"var": "calc/input_b"},
            {"var": "calc/input_c"}
        ]
    });

    let rule_req = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/input_a", "calc/input_b", "calc/input_c"])
        .with_outputs(vec!["calc/result"])
        .with_expression_json(expression)
        .with_display("calc/result = calc/input_a + calc/input_b + calc/input_c")
        .build();

    let response: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Failed to create rule");

    assert_rule_inputs_match(&response, &["calc/input_a", "calc/input_b", "calc/input_c"]);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_rule_with_multiple_outputs() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("multi_output_prod");
    let datatype_id = ctx.unique_id("multi_output_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input = AbstractAttributeBuilder::new("calc", "source")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input
    ).await.expect("Failed to create input attribute");

    // Create multiple output attributes
    for name in &["output_a", "output_b"] {
        let attr = AbstractAttributeBuilder::new("calc", name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr
        ).await.expect("Failed to create attribute");
    }

    // Rule that produces multiple outputs
    let expression = serde_json::json!({
        "map": [
            {"*": [{"var": "calc/source"}, 2]},
            {"*": [{"var": "calc/source"}, 3]}
        ]
    });

    let rule_req = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/source"])
        .with_outputs(vec!["calc/output_a", "calc/output_b"])
        .with_expression_json(expression)
        .with_display("calc/output_a = calc/source * 2; calc/output_b = calc/source * 3")
        .build();

    let response: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Failed to create rule");

    assert_rule_outputs_match(&response, &["calc/output_a", "calc/output_b"]);

    ctx.cleanup().await.ok();
}

// =============================================================================
// READ Tests
// =============================================================================

#[tokio::test]
async fn test_get_rule_by_id() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("get_rule_prod");
    let datatype_id = ctx.unique_id("get_rule_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input_attr = AbstractAttributeBuilder::new("loan", "amount")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input_attr
    ).await.expect("Failed to create input attribute");

    let output_attr = AbstractAttributeBuilder::new("loan", "doubled")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output_attr
    ).await.expect("Failed to create output attribute");

    // Create rule
    let rule_req = RuleBuilder::new("calculation")
        .with_inputs(vec!["loan/amount"])
        .with_outputs(vec!["loan/doubled"])
        .multiply("loan/amount", 2.0, "loan/doubled")
        .build();

    let created: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Failed to create rule");

    // Get rule by ID (note: individual rule operations use /api/rules/:rule_id)
    let response: RuleResponse = ctx.server
        .get(&format!("/api/rules/{}", created.id))
        .await
        .expect("Failed to get rule");

    assert_eq!(response.id, created.id);
    assert_eq!(response.rule_type, "calculation");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_get_nonexistent_rule() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    let product_id = ctx.unique_product_id("no_rule_prod");
    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let response = ctx.server
        .get_response("/api/rules/nonexistent_rule_id")
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_list_rules() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("list_rules_prod");
    let datatype_id = ctx.unique_id("list_rules_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attributes and multiple rules
    for i in 0..3 {
        let input_name = format!("input_{}", i);
        let output_name = format!("output_{}", i);

        let input = AbstractAttributeBuilder::new("calc", &input_name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &input
        ).await.expect("Failed to create input");

        let output = AbstractAttributeBuilder::new("calc", &output_name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &output
        ).await.expect("Failed to create output");

        let rule = RuleBuilder::new("calculation")
            .with_inputs(vec![&format!("calc/{}", input_name)])
            .with_outputs(vec![&format!("calc/{}", output_name)])
            .multiply(&format!("calc/{}", input_name), 2.0, &format!("calc/{}", output_name))
            .with_order(i)
            .build();
        ctx.server.post::<_, RuleResponse>(
            &format!("/api/products/{}/rules", product_id),
            &rule
        ).await.expect("Failed to create rule");
    }

    // List rules
    let response: ListRulesResponse = ctx.server
        .get(&format!("/api/products/{}/rules?pageSize=100", product_id))
        .await
        .expect("Failed to list rules");

    assert!(response.items.len() >= 3);

    ctx.cleanup().await.ok();
}

// =============================================================================
// UPDATE Tests
// =============================================================================

#[tokio::test]
async fn test_update_rule_description() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("upd_rule_prod");
    let datatype_id = ctx.unique_id("upd_rule_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input = AbstractAttributeBuilder::new("loan", "val")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input
    ).await.expect("Failed to create input");

    let output = AbstractAttributeBuilder::new("loan", "result")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output
    ).await.expect("Failed to create output");

    // Create rule
    let rule_req = RuleBuilder::new("calculation")
        .with_inputs(vec!["loan/val"])
        .with_outputs(vec!["loan/result"])
        .multiply("loan/val", 2.0, "loan/result")
        .build();

    let created: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Failed to create rule");

    // Update description
    let update = UpdateRuleRequest {
        rule_type: None,
        input_attributes: None,
        output_attributes: None,
        display_expression: None,
        expression_json: None,
        description: Some("Updated description".to_string()),
        enabled: None,
        order_index: None,
    };

    let response: RuleResponse = ctx.server
        .put(&format!("/api/rules/{}", created.id), &update)
        .await
        .expect("Failed to update rule");

    assert_eq!(response.description, Some("Updated description".to_string()));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_disable_rule() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("disable_rule_prod");
    let datatype_id = ctx.unique_id("disable_rule_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input = AbstractAttributeBuilder::new("loan", "x")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input
    ).await.expect("Failed to create input");

    let output = AbstractAttributeBuilder::new("loan", "y")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output
    ).await.expect("Failed to create output");

    // Create rule (enabled by default)
    let rule_req = RuleBuilder::new("calculation")
        .with_inputs(vec!["loan/x"])
        .with_outputs(vec!["loan/y"])
        .multiply("loan/x", 2.0, "loan/y")
        .build();

    let created: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Failed to create rule");

    assert!(created.enabled);

    // Disable rule
    let update = UpdateRuleRequest {
        rule_type: None,
        input_attributes: None,
        output_attributes: None,
        display_expression: None,
        expression_json: None,
        description: None,
        enabled: Some(false),
        order_index: None,
    };

    let response: RuleResponse = ctx.server
        .put(&format!("/api/rules/{}", created.id), &update)
        .await
        .expect("Failed to update rule");

    assert_rule_disabled(&response);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_enable_disabled_rule() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("enable_rule_prod");
    let datatype_id = ctx.unique_id("enable_rule_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input = AbstractAttributeBuilder::new("calc", "a")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input
    ).await.expect("Failed to create input");

    let output = AbstractAttributeBuilder::new("calc", "b")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output
    ).await.expect("Failed to create output");

    // Create and disable rule
    let rule_req = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/a"])
        .with_outputs(vec!["calc/b"])
        .multiply("calc/a", 2.0, "calc/b")
        .build();

    let created: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Failed to create rule");

    // Disable
    let disable = UpdateRuleRequest {
        rule_type: None,
        input_attributes: None,
        output_attributes: None,
        display_expression: None,
        expression_json: None,
        description: None,
        enabled: Some(false),
        order_index: None,
    };
    ctx.server.put::<_, RuleResponse>(
        &format!("/api/rules/{}", created.id),
        &disable
    ).await.expect("Failed to disable rule");

    // Re-enable
    let enable = UpdateRuleRequest {
        rule_type: None,
        input_attributes: None,
        output_attributes: None,
        display_expression: None,
        expression_json: None,
        description: None,
        enabled: Some(true),
        order_index: None,
    };
    let response: RuleResponse = ctx.server
        .put(&format!("/api/rules/{}", created.id), &enable)
        .await
        .expect("Failed to enable rule");

    assert_rule_enabled(&response);

    ctx.cleanup().await.ok();
}

// =============================================================================
// DELETE Tests
// =============================================================================

#[tokio::test]
async fn test_delete_rule() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("del_rule_prod");
    let datatype_id = ctx.unique_id("del_rule_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input = AbstractAttributeBuilder::new("loan", "del_in")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input
    ).await.expect("Failed to create input");

    let output = AbstractAttributeBuilder::new("loan", "del_out")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output
    ).await.expect("Failed to create output");

    // Create rule
    let rule_req = RuleBuilder::new("calculation")
        .with_inputs(vec!["loan/del_in"])
        .with_outputs(vec!["loan/del_out"])
        .multiply("loan/del_in", 2.0, "loan/del_out")
        .build();

    let created: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Failed to create rule");

    // Delete rule (individual rule ops use /api/rules/:rule_id)
    let response: DeleteResponse = ctx.server
        .delete(&format!("/api/rules/{}", created.id))
        .await
        .expect("Failed to delete rule");

    assert!(response.success);

    // Verify deleted
    let get_response = ctx.server
        .get_response(&format!("/api/rules/{}", created.id))
        .await
        .expect("Request should complete");
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_delete_nonexistent_rule() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    let product_id = ctx.unique_product_id("del_no_rule_prod");
    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let response = ctx.server
        .delete_response("/api/rules/nonexistent_rule")
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

// =============================================================================
// CRUD Cycle Test
// =============================================================================

#[tokio::test]
async fn test_rule_crud_cycle() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("crud_rule_prod");
    let datatype_id = ctx.unique_id("crud_rule_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input = AbstractAttributeBuilder::new("calc", "input")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input
    ).await.expect("Failed to create input");

    let output = AbstractAttributeBuilder::new("calc", "output")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output
    ).await.expect("Failed to create output");

    // CREATE
    let rule_req = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/input"])
        .with_outputs(vec!["calc/output"])
        .with_description("Initial rule")
        .multiply("calc/input", 2.0, "calc/output")
        .build();

    let created: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Failed to create rule");

    assert!(created.enabled);

    // READ (individual rule ops use /api/rules/:rule_id)
    let read: RuleResponse = ctx.server
        .get(&format!("/api/rules/{}", created.id))
        .await
        .expect("Failed to get rule");

    assert_eq!(read.id, created.id);
    assert_eq!(read.description, Some("Initial rule".to_string()));

    // UPDATE
    let update = UpdateRuleRequest {
        rule_type: None,
        input_attributes: None,
        output_attributes: None,
        display_expression: None,
        expression_json: None,
        description: Some("Updated rule".to_string()),
        enabled: Some(false),
        order_index: None,
    };

    let updated: RuleResponse = ctx.server
        .put(&format!("/api/rules/{}", created.id), &update)
        .await
        .expect("Failed to update rule");

    assert_eq!(updated.description, Some("Updated rule".to_string()));
    assert!(!updated.enabled);

    // DELETE
    let deleted: DeleteResponse = ctx.server
        .delete(&format!("/api/rules/{}", created.id))
        .await
        .expect("Failed to delete rule");

    assert!(deleted.success);

    // VERIFY DELETED
    let verify = ctx.server
        .get_response(&format!("/api/rules/{}", created.id))
        .await
        .expect("Request should complete");
    assert_eq!(verify.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}
