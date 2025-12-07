//! Rule Validation Tests
//!
//! Tests for rule expression validation and input/output validation.

use crate::fixtures::{
    data_builders::{AbstractAttributeBuilder, DatatypeBuilder, ProductBuilder, RuleBuilder},
    TestContext,
};
use product_farm_api::rest::types::{
    AbstractAttributeResponse, CreateRuleRequest, DatatypeResponse, ProductResponse, RuleResponse,
};
use reqwest::StatusCode;

// =============================================================================
// Expression Validation Tests
// =============================================================================

#[tokio::test]
async fn test_create_rule_empty_expression() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("empty_expr_prod");
    let datatype_id = ctx.unique_id("empty_expr_dt");

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

    // Create rule with empty expression
    let rule_req = CreateRuleRequest {
        rule_type: "calculation".to_string(),
        input_attributes: vec!["loan/val".to_string()],
        output_attributes: vec!["loan/result".to_string()],
        display_expression: "loan/result = loan/val * 2".to_string(),
        expression_json: "".to_string(),
        description: None,
        order_index: 0,
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_rule_invalid_json_expression() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("invalid_json_prod");
    let datatype_id = ctx.unique_id("invalid_json_dt");

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

    // Create rule with invalid JSON expression
    let rule_req = CreateRuleRequest {
        rule_type: "calculation".to_string(),
        input_attributes: vec!["loan/x".to_string()],
        output_attributes: vec!["loan/y".to_string()],
        display_expression: "loan/y = loan/x * 2".to_string(),
        expression_json: "{ invalid json }".to_string(),
        description: None,
        order_index: 0,
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_rule_valid_json_logic() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("valid_json_prod");
    let datatype_id = ctx.unique_id("valid_json_dt");

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

    // Various valid JSON Logic expressions
    let expressions = [
        serde_json::json!({"*": [{"var": "calc/a"}, 2]}),
        serde_json::json!({"+": [{"var": "calc/a"}, 10]}),
        serde_json::json!({"/": [{"var": "calc/a"}, 2]}),
        serde_json::json!({"-": [{"var": "calc/a"}, 5]}),
    ];

    for (i, expr) in expressions.iter().enumerate() {
        // Create unique output for each
        let out_name = format!("out_{}", i);
        let out_attr = AbstractAttributeBuilder::new("calc", &out_name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &out_attr
        ).await.expect("Failed to create output");

        let rule_req = RuleBuilder::new("calculation")
            .with_inputs(vec!["calc/a"])
            .with_outputs(vec![&format!("calc/{}", out_name)])
            .with_expression_json(expr.clone())
            .with_display(&format!("calc/{} = expression {}", out_name, i))
            .build();

        let result = ctx.server
            .post::<_, RuleResponse>(&format!("/api/products/{}/rules", product_id), &rule_req)
            .await;

        assert!(result.is_ok(), "Expression {} should be valid: {:?}", i, expr);
    }

    ctx.cleanup().await.ok();
}

// =============================================================================
// Input/Output Attribute Validation Tests
// =============================================================================

#[tokio::test]
async fn test_create_rule_empty_inputs() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("no_inputs_prod");
    let datatype_id = ctx.unique_id("no_inputs_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let output = AbstractAttributeBuilder::new("loan", "result")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output
    ).await.expect("Failed to create output");

    // Rule with no inputs
    let rule_req = CreateRuleRequest {
        rule_type: "calculation".to_string(),
        input_attributes: vec![],
        output_attributes: vec!["loan/result".to_string()],
        display_expression: "loan/result = 100".to_string(),
        expression_json: serde_json::json!(100).to_string(),
        description: None,
        order_index: 0,
    };

    // Empty inputs might be valid for constant rules - document behavior
    let response = ctx.server
        .post_response(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Request should complete");

    println!("Empty inputs response: {}", response.status());

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_rule_empty_outputs() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("no_outputs_prod");
    let datatype_id = ctx.unique_id("no_outputs_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input = AbstractAttributeBuilder::new("loan", "input")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input
    ).await.expect("Failed to create input");

    // Rule with no outputs should be rejected
    let rule_req = CreateRuleRequest {
        rule_type: "calculation".to_string(),
        input_attributes: vec!["loan/input".to_string()],
        output_attributes: vec![],
        display_expression: "no output".to_string(),
        expression_json: serde_json::json!({"var": "loan/input"}).to_string(),
        description: None,
        order_index: 0,
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_rule_nonexistent_input_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("bad_input_prod");
    let datatype_id = ctx.unique_id("bad_input_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Only create output attribute, not input
    let output = AbstractAttributeBuilder::new("loan", "result")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output
    ).await.expect("Failed to create output");

    // Rule references non-existent input
    let rule_req = CreateRuleRequest {
        rule_type: "calculation".to_string(),
        input_attributes: vec!["loan/nonexistent".to_string()],
        output_attributes: vec!["loan/result".to_string()],
        display_expression: "loan/result = loan/nonexistent * 2".to_string(),
        expression_json: serde_json::json!({"*": [{"var": "loan/nonexistent"}, 2]}).to_string(),
        description: None,
        order_index: 0,
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Request should complete");

    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND,
        "Expected BAD_REQUEST or NOT_FOUND, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_rule_nonexistent_output_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("bad_output_prod");
    let datatype_id = ctx.unique_id("bad_output_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Only create input attribute, not output
    let input = AbstractAttributeBuilder::new("loan", "input")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input
    ).await.expect("Failed to create input");

    // Rule references non-existent output
    let rule_req = CreateRuleRequest {
        rule_type: "calculation".to_string(),
        input_attributes: vec!["loan/input".to_string()],
        output_attributes: vec!["loan/nonexistent_output".to_string()],
        display_expression: "loan/nonexistent_output = loan/input * 2".to_string(),
        expression_json: serde_json::json!({"*": [{"var": "loan/input"}, 2]}).to_string(),
        description: None,
        order_index: 0,
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Request should complete");

    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND,
        "Expected BAD_REQUEST or NOT_FOUND, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

// =============================================================================
// Rule Type Validation Tests
// =============================================================================

#[tokio::test]
async fn test_create_rule_empty_rule_type() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("no_type_prod");
    let datatype_id = ctx.unique_id("no_type_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input = AbstractAttributeBuilder::new("loan", "a")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input
    ).await.expect("Failed to create input");

    let output = AbstractAttributeBuilder::new("loan", "b")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output
    ).await.expect("Failed to create output");

    // Empty rule type
    let rule_req = CreateRuleRequest {
        rule_type: "".to_string(),
        input_attributes: vec!["loan/a".to_string()],
        output_attributes: vec!["loan/b".to_string()],
        display_expression: "loan/b = loan/a * 2".to_string(),
        expression_json: serde_json::json!({"*": [{"var": "loan/a"}, 2]}).to_string(),
        description: None,
        order_index: 0,
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_rule_valid_rule_types() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("valid_types_prod");
    let datatype_id = ctx.unique_id("valid_types_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Test various rule types
    let rule_types = ["calculation", "validation", "transformation", "eligibility"];

    for (i, rule_type) in rule_types.iter().enumerate() {
        let input_name = format!("type_in_{}", i);
        let output_name = format!("type_out_{}", i);

        let input = AbstractAttributeBuilder::new("test", &input_name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &input
        ).await.expect("Failed to create input");

        let output = AbstractAttributeBuilder::new("test", &output_name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &output
        ).await.expect("Failed to create output");

        let rule_req = RuleBuilder::new(rule_type)
            .with_inputs(vec![&format!("test/{}", input_name)])
            .with_outputs(vec![&format!("test/{}", output_name)])
            .multiply(&format!("test/{}", input_name), 2.0, &format!("test/{}", output_name))
            .with_order(i as i32)
            .build();

        let result = ctx.server
            .post::<_, RuleResponse>(&format!("/api/products/{}/rules", product_id), &rule_req)
            .await;

        match result {
            Ok(response) => {
                assert_eq!(response.rule_type, *rule_type);
            }
            Err(e) => {
                println!("Rule type '{}' not supported: {}", rule_type, e);
            }
        }
    }

    ctx.cleanup().await.ok();
}

// =============================================================================
// Display Expression Validation Tests
// =============================================================================

#[tokio::test]
async fn test_create_rule_empty_display_expression() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("no_display_prod");
    let datatype_id = ctx.unique_id("no_display_dt");

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

    // Empty display expression
    let rule_req = CreateRuleRequest {
        rule_type: "calculation".to_string(),
        input_attributes: vec!["loan/x".to_string()],
        output_attributes: vec!["loan/y".to_string()],
        display_expression: "".to_string(),
        expression_json: serde_json::json!({"*": [{"var": "loan/x"}, 2]}).to_string(),
        description: None,
        order_index: 0,
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Request should complete");

    // Document behavior - might be allowed or rejected
    println!("Empty display expression response: {}", response.status());

    ctx.cleanup().await.ok();
}

// =============================================================================
// Order Index Validation Tests
// =============================================================================

#[tokio::test]
async fn test_create_rule_negative_order_index() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("neg_order_prod");
    let datatype_id = ctx.unique_id("neg_order_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input = AbstractAttributeBuilder::new("loan", "in")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input
    ).await.expect("Failed to create input");

    let output = AbstractAttributeBuilder::new("loan", "out")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output
    ).await.expect("Failed to create output");

    // Negative order index
    let rule_req = CreateRuleRequest {
        rule_type: "calculation".to_string(),
        input_attributes: vec!["loan/in".to_string()],
        output_attributes: vec!["loan/out".to_string()],
        display_expression: "loan/out = loan/in * 2".to_string(),
        expression_json: serde_json::json!({"*": [{"var": "loan/in"}, 2]}).to_string(),
        description: None,
        order_index: -1,
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/rules", product_id), &rule_req)
        .await
        .expect("Request should complete");

    // Document behavior
    println!("Negative order index response: {}", response.status());

    ctx.cleanup().await.ok();
}

// =============================================================================
// Product Validation Tests
// =============================================================================

#[tokio::test]
async fn test_create_rule_nonexistent_product() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let rule_req = RuleBuilder::new("calculation")
        .with_inputs(vec!["loan/x"])
        .with_outputs(vec!["loan/y"])
        .multiply("loan/x", 2.0, "loan/y")
        .build();

    let response = ctx.server
        .post_response("/api/products/nonexistent_product/rules", &rule_req)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}
