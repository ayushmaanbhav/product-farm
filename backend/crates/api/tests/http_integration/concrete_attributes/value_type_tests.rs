//! Concrete Attribute Value Type Tests
//!
//! Tests for FIXED_VALUE, RULE_DRIVEN, and JUST_DEFINITION value types.

use crate::fixtures::{
    assertions::*,
    data_builders::{
        values, AbstractAttributeBuilder, ConcreteAttributeBuilder, DatatypeBuilder,
        ProductBuilder, RuleBuilder,
    },
    TestContext,
};
use product_farm_api::rest::types::{
    AbstractAttributeResponse, AttributeResponse, AttributeValueJson, CreateAttributeRequest,
    DatatypeResponse, ProductResponse, RuleResponse,
};
use reqwest::StatusCode;

// =============================================================================
// FIXED_VALUE Tests
// =============================================================================

#[tokio::test]
async fn test_fixed_value_with_integer() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("fixed_int_prod");
    let datatype_id = ctx.unique_id("fixed_int_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::int(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("loan", "term_months")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    let concrete = ConcreteAttributeBuilder::new("loan", "main", "term_months")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::int(36))
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Failed to create attribute");

    assert_attribute_value_type(&response, "FIXED_VALUE");
    assert!(response.value.is_some());

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_fixed_value_with_decimal() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("fixed_dec_prod");
    let datatype_id = ctx.unique_id("fixed_dec_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id)
        .with_precision(10, 2)
        .build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("loan", "interest_rate")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    let concrete = ConcreteAttributeBuilder::new("loan", "main", "interest_rate")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("5.25"))
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Failed to create attribute");

    assert_attribute_value_type(&response, "FIXED_VALUE");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_fixed_value_with_string() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("fixed_str_prod");
    let datatype_id = ctx.unique_id("fixed_str_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::string(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("loan", "product_code")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    let concrete = ConcreteAttributeBuilder::new("loan", "main", "product_code")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::string("LOAN-001"))
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Failed to create attribute");

    assert_attribute_value_type(&response, "FIXED_VALUE");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_fixed_value_with_boolean() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("fixed_bool_prod");
    let datatype_id = ctx.unique_id("fixed_bool_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::boolean(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("loan", "is_active")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    let concrete = ConcreteAttributeBuilder::new("loan", "main", "is_active")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::boolean(true))
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Failed to create attribute");

    assert_attribute_value_type(&response, "FIXED_VALUE");

    ctx.cleanup().await.ok();
}

// =============================================================================
// RULE_DRIVEN Tests
// =============================================================================

#[tokio::test]
async fn test_rule_driven_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("rule_drv_prod");
    let datatype_id = ctx.unique_id("rule_drv_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Input attribute
    let input_attr = AbstractAttributeBuilder::new("calc", "input")
        .with_datatype(&datatype_id)
        .build();
    let input_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input_attr
    ).await.expect("Failed to create input attribute");

    // Output attribute
    let output_attr = AbstractAttributeBuilder::new("calc", "output")
        .with_datatype(&datatype_id)
        .build();
    let output_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output_attr
    ).await.expect("Failed to create output attribute");

    // Create rule using short path format (componentType/attributeName)
    let rule = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/input"])
        .with_outputs(vec!["calc/output"])
        .with_expression(r#"{"*": [{"var": "calc/input"}, 2]}"#)
        .with_display("calc/output = calc/input * 2")
        .build();
    let rule_resp: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule)
        .await
        .expect("Failed to create rule");

    // Create rule-driven concrete attribute
    let concrete = ConcreteAttributeBuilder::new("calc", "main", "output")
        .with_abstract_path(&output_resp.abstract_path)
        .rule_driven(&rule_resp.id)
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Failed to create attribute");

    assert_attribute_value_type(&response, "RULE_DRIVEN");
    assert_eq!(response.rule_id, Some(rule_resp.id));
    assert!(response.value.is_none());

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_rule_driven_with_nonexistent_rule() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("bad_rule_prod");
    let datatype_id = ctx.unique_id("bad_rule_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("calc", "result")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Try to create with non-existent rule
    let concrete = ConcreteAttributeBuilder::new("calc", "main", "result")
        .with_abstract_path(&abstract_resp.abstract_path)
        .rule_driven("nonexistent_rule_id")
        .build();

    let response = ctx.server
        .post_response(&format!("/api/products/{}/attributes", product_id), &concrete)
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
// JUST_DEFINITION Tests
// =============================================================================

#[tokio::test]
async fn test_just_definition_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("just_def_prod2");
    let datatype_id = ctx.unique_id("just_def_dt2");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("input", "user_value")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Create just_definition attribute (no value, no rule)
    let concrete = ConcreteAttributeBuilder::new("input", "main", "user_value")
        .with_abstract_path(&abstract_resp.abstract_path)
        .just_definition()
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Failed to create attribute");

    assert_attribute_value_type(&response, "JUST_DEFINITION");
    assert!(response.value.is_none());
    assert!(response.rule_id.is_none());

    ctx.cleanup().await.ok();
}

// =============================================================================
// Invalid Combinations Tests
// =============================================================================

#[tokio::test]
async fn test_invalid_value_and_rule_combination() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("invalid_combo_prod");
    let datatype_id = ctx.unique_id("invalid_combo_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create input and output abstracts
    let input_attr = AbstractAttributeBuilder::new("calc", "x")
        .with_datatype(&datatype_id)
        .build();
    let input_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input_attr
    ).await.expect("Failed to create input attribute");

    let output_attr = AbstractAttributeBuilder::new("calc", "y")
        .with_datatype(&datatype_id)
        .build();
    let output_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output_attr
    ).await.expect("Failed to create output attribute");

    // Create a rule using short path format
    let rule = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/x"])
        .with_outputs(vec!["calc/y"])
        .with_expression(r#"{"*": [{"var": "calc/x"}, 2]}"#)
        .with_display("calc/y = calc/x * 2")
        .build();
    let rule_resp: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule)
        .await
        .expect("Failed to create rule");

    // Try to create attribute with BOTH value AND rule_id
    let concrete = CreateAttributeRequest {
        component_type: "calc".to_string(),
        component_id: "main".to_string(),
        attribute_name: "y".to_string(),
        abstract_path: output_resp.abstract_path.clone(),
        value_type: "FIXED_VALUE".to_string(), // Claiming fixed but also has rule
        value: Some(values::decimal("100.00")),
        rule_id: Some(rule_resp.id),
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Request should complete");

    // Should be rejected - can't have both
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_fixed_value_without_value() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("no_val_prod");
    let datatype_id = ctx.unique_id("no_val_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("calc", "missing_val")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Try to create FIXED_VALUE without providing a value
    let concrete = CreateAttributeRequest {
        component_type: "calc".to_string(),
        component_id: "main".to_string(),
        attribute_name: "missing_val".to_string(),
        abstract_path: abstract_resp.abstract_path.clone(),
        value_type: "FIXED_VALUE".to_string(),
        value: None, // Missing value for FIXED_VALUE
        rule_id: None,
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Request should complete");

    // Should be rejected - FIXED_VALUE requires value
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_rule_driven_without_rule() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("no_rule_prod");
    let datatype_id = ctx.unique_id("no_rule_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("calc", "missing_rule")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Try to create RULE_DRIVEN without providing a rule_id
    let concrete = CreateAttributeRequest {
        component_type: "calc".to_string(),
        component_id: "main".to_string(),
        attribute_name: "missing_rule".to_string(),
        abstract_path: abstract_resp.abstract_path.clone(),
        value_type: "RULE_DRIVEN".to_string(),
        value: None,
        rule_id: None, // Missing rule for RULE_DRIVEN
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Request should complete");

    // Should be rejected - RULE_DRIVEN requires rule_id
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Value Type Transitions Tests
// =============================================================================

#[tokio::test]
async fn test_update_fixed_value_to_rule_driven() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("trans_prod");
    let datatype_id = ctx.unique_id("trans_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let input_attr = AbstractAttributeBuilder::new("calc", "src")
        .with_datatype(&datatype_id)
        .build();
    let input_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input_attr
    ).await.expect("Failed to create input attribute");

    let output_attr = AbstractAttributeBuilder::new("calc", "target")
        .with_datatype(&datatype_id)
        .build();
    let output_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output_attr
    ).await.expect("Failed to create output attribute");

    // Create rule using short path format
    let rule = RuleBuilder::new("calculation")
        .with_inputs(vec!["calc/src"])
        .with_outputs(vec!["calc/target"])
        .with_expression(r#"{"*": [{"var": "calc/src"}, 2]}"#)
        .with_display("calc/target = calc/src * 2")
        .build();
    let rule_resp: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule)
        .await
        .expect("Failed to create rule");

    // Create as FIXED_VALUE first
    let concrete = ConcreteAttributeBuilder::new("calc", "main", "target")
        .with_abstract_path(&output_resp.abstract_path)
        .fixed_value(values::decimal("100.00"))
        .build();
    let created: AttributeResponse = ctx.server.post(
        &format!("/api/products/{}/attributes", product_id),
        &concrete
    ).await.expect("Failed to create attribute");

    // Try to update to RULE_DRIVEN
    let update = product_farm_api::rest::types::UpdateAttributeRequest {
        value: None,
        rule_id: Some(rule_resp.id),
    };

    let encoded = urlencoding::encode(&created.path);
    let response = ctx.server
        .put_response(&format!("/api/attributes/{}", encoded), &update)
        .await
        .expect("Request should complete");

    // Document behavior - might be allowed or rejected
    println!("Value type transition response: {}", response.status());

    ctx.cleanup().await.ok();
}
