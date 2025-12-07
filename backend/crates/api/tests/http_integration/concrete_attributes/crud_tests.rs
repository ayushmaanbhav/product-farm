//! Concrete Attribute CRUD Tests

use crate::fixtures::{
    assertions::*,
    data_builders::{
        values, AbstractAttributeBuilder, ConcreteAttributeBuilder, DatatypeBuilder,
        ProductBuilder, RuleBuilder,
    },
    TestContext,
};
use product_farm_api::rest::types::{
    AbstractAttributeResponse, AttributeResponse, CreateAttributeRequest, DeleteResponse,
    DatatypeResponse, ListAttributesResponse, ProductResponse, RuleResponse, UpdateAttributeRequest,
};
use reqwest::StatusCode;

// =============================================================================
// CREATE Tests
// =============================================================================

#[tokio::test]
async fn test_create_concrete_attribute_fixed_value() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup: Create product, datatype, abstract attribute
    let product_id = ctx.unique_product_id("concrete_prod");
    let datatype_id = ctx.unique_id("concrete_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("loan", "principal")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Create concrete attribute - use the abstract_path from the response
    let concrete = ConcreteAttributeBuilder::new("loan", "main", "principal")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("10000.00"))
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Failed to create concrete attribute");

    assert_eq!(response.component_type, "loan");
    assert_eq!(response.component_id, "main");
    assert_eq!(response.attribute_name, "principal");
    assert_attribute_value_type(&response, "FIXED_VALUE");
    assert!(response.value.is_some());

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_concrete_attribute_rule_driven() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("rule_driven_prod");
    let datatype_id = ctx.unique_id("rule_driven_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create input and output abstract attributes
    let input_attr = AbstractAttributeBuilder::new("loan", "base_amount")
        .with_datatype(&datatype_id)
        .build();
    let input_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &input_attr
    ).await.expect("Failed to create input abstract");

    let output_attr = AbstractAttributeBuilder::new("loan", "calculated_amount")
        .with_datatype(&datatype_id)
        .build();
    let output_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &output_attr
    ).await.expect("Failed to create output abstract");

    // Create rule using short path format (componentType/attributeName)
    let rule = RuleBuilder::new("calculation")
        .with_inputs(vec!["loan/base_amount"])
        .with_outputs(vec!["loan/calculated_amount"])
        .with_expression(r#"{"*": [{"var": "loan/base_amount"}, 2]}"#)
        .with_display("loan/calculated_amount = loan/base_amount * 2")
        .build();
    let rule_resp: RuleResponse = ctx.server
        .post(&format!("/api/products/{}/rules", product_id), &rule)
        .await
        .expect("Failed to create rule");

    // Create concrete attribute driven by rule - use full abstract path
    let concrete = ConcreteAttributeBuilder::new("loan", "main", "calculated_amount")
        .with_abstract_path(&output_resp.abstract_path)
        .rule_driven(&rule_resp.id)
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Failed to create concrete attribute");

    assert_attribute_value_type(&response, "RULE_DRIVEN");
    assert_eq!(response.rule_id, Some(rule_resp.id));
    assert!(response.value.is_none());

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_concrete_attribute_just_definition() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("just_def_prod");
    let datatype_id = ctx.unique_id("just_def_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("loan", "input_field")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Create concrete attribute without value or rule - use full abstract path
    let concrete = ConcreteAttributeBuilder::new("loan", "main", "input_field")
        .with_abstract_path(&abstract_resp.abstract_path)
        .just_definition()
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Failed to create concrete attribute");

    assert_attribute_value_type(&response, "JUST_DEFINITION");
    assert!(response.value.is_none());
    assert!(response.rule_id.is_none());

    ctx.cleanup().await.ok();
}

// =============================================================================
// READ Tests
// =============================================================================

#[tokio::test]
async fn test_get_concrete_attribute_by_path() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("get_concrete_prod");
    let datatype_id = ctx.unique_id("get_concrete_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("loan", "amount")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    let concrete = ConcreteAttributeBuilder::new("loan", "primary", "amount")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("5000.00"))
        .build();
    let created: AttributeResponse = ctx.server.post(
        &format!("/api/products/{}/attributes", product_id),
        &concrete
    ).await.expect("Failed to create concrete attribute");

    // Get by path - use the path from the response
    let encoded = urlencoding::encode(&created.path);
    let response: AttributeResponse = ctx.server
        .get(&format!("/api/attributes/{}", encoded))
        .await
        .expect("Failed to get concrete attribute");

    assert_eq!(response.attribute_name, "amount");
    assert_eq!(response.component_id, "primary");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_get_nonexistent_concrete_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    let product_id = ctx.unique_product_id("no_concrete_prod");
    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Use correct concrete path format that won't exist
    let fake_path = format!("{}:nonexistent:path:attr", product_id);
    let encoded = urlencoding::encode(&fake_path);
    let response = ctx.server
        .get_response(&format!("/api/attributes/{}", encoded))
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_list_concrete_attributes() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("list_concrete_prod");
    let datatype_id = ctx.unique_id("list_concrete_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create multiple concrete attributes
    for i in 0..3 {
        let abstract_attr = AbstractAttributeBuilder::new("loan", &format!("attr_{}", i))
            .with_datatype(&datatype_id)
            .build();
        let abstract_resp: AbstractAttributeResponse = ctx.server.post(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &abstract_attr
        ).await.expect("Failed to create abstract attribute");

        let concrete = ConcreteAttributeBuilder::new("loan", "main", &format!("attr_{}", i))
            .with_abstract_path(&abstract_resp.abstract_path)
            .fixed_value(values::int(i as i64))
            .build();
        ctx.server.post::<_, AttributeResponse>(
            &format!("/api/products/{}/attributes", product_id),
            &concrete
        ).await.expect("Failed to create concrete attribute");
    }

    // List attributes
    let response: ListAttributesResponse = ctx.server
        .get(&format!("/api/products/{}/attributes?pageSize=100", product_id))
        .await
        .expect("Failed to list attributes");

    assert!(response.items.len() >= 3);

    ctx.cleanup().await.ok();
}

// =============================================================================
// UPDATE Tests
// =============================================================================

#[tokio::test]
async fn test_update_concrete_attribute_value() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("upd_concrete_prod");
    let datatype_id = ctx.unique_id("upd_concrete_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("loan", "rate")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    let concrete = ConcreteAttributeBuilder::new("loan", "main", "rate")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("5.5"))
        .build();
    let created: AttributeResponse = ctx.server.post(
        &format!("/api/products/{}/attributes", product_id),
        &concrete
    ).await.expect("Failed to create concrete attribute");

    // Update value using the path from response
    let update = UpdateAttributeRequest {
        value: Some(values::decimal("6.5")),
        rule_id: None,
    };

    let encoded = urlencoding::encode(&created.path);
    let response: AttributeResponse = ctx.server
        .put(&format!("/api/attributes/{}", encoded), &update)
        .await
        .expect("Failed to update attribute");

    assert!(response.value.is_some());

    ctx.cleanup().await.ok();
}

// =============================================================================
// DELETE Tests
// =============================================================================

#[tokio::test]
async fn test_delete_concrete_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("del_concrete_prod");
    let datatype_id = ctx.unique_id("del_concrete_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("loan", "deletable")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    let concrete = ConcreteAttributeBuilder::new("loan", "main", "deletable")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::int(100))
        .build();
    let created: AttributeResponse = ctx.server.post(
        &format!("/api/products/{}/attributes", product_id),
        &concrete
    ).await.expect("Failed to create concrete attribute");

    // Delete - use the path from the created response
    let encoded = urlencoding::encode(&created.path);
    let response: DeleteResponse = ctx.server
        .delete(&format!("/api/attributes/{}", encoded))
        .await
        .expect("Failed to delete attribute");

    assert!(response.success);

    // Verify deleted
    let get_response = ctx.server
        .get_response(&format!("/api/attributes/{}", encoded))
        .await
        .expect("Request should complete");
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

// =============================================================================
// CRUD Cycle Test
// =============================================================================

#[tokio::test]
async fn test_concrete_attribute_crud_cycle() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("crud_concrete_prod");
    let datatype_id = ctx.unique_id("crud_concrete_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("calc", "value")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // CREATE
    let concrete = ConcreteAttributeBuilder::new("calc", "inst1", "value")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("100.00"))
        .build();

    let created: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Failed to create concrete attribute");

    // READ - use the path from the created response
    let encoded = urlencoding::encode(&created.path);
    let read: AttributeResponse = ctx.server
        .get(&format!("/api/attributes/{}", encoded))
        .await
        .expect("Failed to get attribute");

    assert_eq!(read.path, created.path);

    // UPDATE
    let update = UpdateAttributeRequest {
        value: Some(values::decimal("200.00")),
        rule_id: None,
    };
    let updated: AttributeResponse = ctx.server
        .put(&format!("/api/attributes/{}", encoded), &update)
        .await
        .expect("Failed to update attribute");

    assert!(updated.value.is_some());

    // DELETE
    let deleted: DeleteResponse = ctx.server
        .delete(&format!("/api/attributes/{}", encoded))
        .await
        .expect("Failed to delete attribute");

    assert!(deleted.success);

    // VERIFY DELETED
    let verify = ctx.server
        .get_response(&format!("/api/attributes/{}", encoded))
        .await
        .expect("Request should complete");
    assert_eq!(verify.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}
