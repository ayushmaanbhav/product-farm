//! Concrete Attribute Dependency Tests
//!
//! Tests for dependencies between concrete and abstract attributes.

use crate::fixtures::{
    data_builders::{
        values, AbstractAttributeBuilder, ConcreteAttributeBuilder, DatatypeBuilder,
        ProductBuilder,
    },
    TestContext,
};
use product_farm_api::rest::types::{
    AbstractAttributeResponse, AttributeResponse, CreateAttributeRequest, DatatypeResponse,
    ProductResponse,
};
use reqwest::StatusCode;

// =============================================================================
// Abstract Attribute Dependency Tests
// =============================================================================

#[tokio::test]
async fn test_concrete_requires_abstract_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup: Create product and datatype, but NOT abstract attribute
    let product_id = ctx.unique_product_id("no_abstract_prod");
    let datatype_id = ctx.unique_id("no_abstract_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Try to create concrete attribute without abstract attribute
    let concrete = CreateAttributeRequest {
        component_type: "loan".to_string(),
        component_id: "main".to_string(),
        attribute_name: "no_abstract".to_string(),
        abstract_path: "loan/no_abstract".to_string(), // This abstract doesn't exist
        value_type: "FIXED_VALUE".to_string(),
        value: Some(values::decimal("100.00")),
        rule_id: None,
    };

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

#[tokio::test]
async fn test_concrete_with_existing_abstract() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("with_abstract_prod");
    let datatype_id = ctx.unique_id("with_abstract_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create abstract attribute first
    let abstract_attr = AbstractAttributeBuilder::new("loan", "amount")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Create concrete attribute
    let concrete = ConcreteAttributeBuilder::new("loan", "main", "amount")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("5000.00"))
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Should create concrete with existing abstract");

    assert_eq!(response.abstract_path, abstract_resp.abstract_path);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_multiple_concrete_for_same_abstract() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("multi_concrete_prod");
    let datatype_id = ctx.unique_id("multi_concrete_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create one abstract attribute
    let abstract_attr = AbstractAttributeBuilder::new("coverage", "limit")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Create multiple concrete attributes with different component_ids
    let component_ids = ["liability", "collision", "comprehensive"];

    for comp_id in component_ids {
        let concrete = ConcreteAttributeBuilder::new("coverage", comp_id, "limit")
            .with_abstract_path(&abstract_resp.abstract_path)
            .fixed_value(values::decimal("10000.00"))
            .build();

        let response: AttributeResponse = ctx.server
            .post(&format!("/api/products/{}/attributes", product_id), &concrete)
            .await
            .expect(&format!("Failed to create concrete for {}", comp_id));

        assert_eq!(response.component_id, comp_id);
    }

    // List all and verify count
    let list: product_farm_api::rest::types::ListAttributesResponse = ctx.server
        .get(&format!("/api/products/{}/attributes?pageSize=100", product_id))
        .await
        .expect("Failed to list attributes");

    assert!(list.items.len() >= 3);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Datatype Dependency Tests
// =============================================================================

#[tokio::test]
async fn test_concrete_inherits_datatype_from_abstract() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("inherit_dt_prod");
    let datatype_id = ctx.unique_id("inherit_dt_type");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Datatype with constraints
    let dt_req = DatatypeBuilder::decimal(&datatype_id)
        .with_min_max(0.0, 1000.0)
        .build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("loan", "constrained")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Create concrete attribute - should inherit constraints from abstract's datatype
    // Value within range should succeed
    let concrete_valid = ConcreteAttributeBuilder::new("loan", "main", "constrained")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("500.00"))
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete_valid)
        .await
        .expect("Valid value should be accepted");

    assert!(response.value.is_some());

    ctx.cleanup().await.ok();
}

// =============================================================================
// Abstract Path Format Tests
// =============================================================================

#[tokio::test]
async fn test_abstract_path_must_match_component_structure() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("path_match_prod");
    let datatype_id = ctx.unique_id("path_match_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create abstract attribute with specific structure
    let abstract_attr = AbstractAttributeBuilder::new("loan", "rate")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Try to create concrete with mismatched component_type
    let concrete = CreateAttributeRequest {
        component_type: "premium".to_string(), // Different from abstract's "loan"
        component_id: "main".to_string(),
        attribute_name: "rate".to_string(),
        abstract_path: abstract_resp.abstract_path.clone(),
        value_type: "FIXED_VALUE".to_string(),
        value: Some(values::decimal("5.0")),
        rule_id: None,
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Request should complete");

    // Should be rejected - component_type mismatch
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_attribute_name_must_match_abstract() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("name_match_prod");
    let datatype_id = ctx.unique_id("name_match_dt");

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

    // Try to create concrete with mismatched attribute_name
    let concrete = CreateAttributeRequest {
        component_type: "loan".to_string(),
        component_id: "main".to_string(),
        attribute_name: "different_name".to_string(), // Different from "amount"
        abstract_path: abstract_resp.abstract_path.clone(),
        value_type: "FIXED_VALUE".to_string(),
        value: Some(values::decimal("100.00")),
        rule_id: None,
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Request should complete");

    // Should be rejected - attribute_name mismatch
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Delete Abstract with Concrete Tests
// =============================================================================

#[tokio::test]
async fn test_delete_abstract_blocks_if_concrete_exists() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("del_abstract_prod");
    let datatype_id = ctx.unique_id("del_abstract_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create abstract attribute
    let abstract_attr = AbstractAttributeBuilder::new("loan", "protected")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Create concrete attribute
    let concrete = ConcreteAttributeBuilder::new("loan", "main", "protected")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("100.00"))
        .build();
    ctx.server.post::<_, AttributeResponse>(
        &format!("/api/products/{}/attributes", product_id),
        &concrete
    ).await.expect("Failed to create concrete attribute");

    // Try to delete abstract - should be blocked
    let abstract_path_encoded = urlencoding::encode(&abstract_resp.abstract_path);
    let response = ctx.server
        .delete_response(&format!("/api/abstract-attributes/{}", abstract_path_encoded))
        .await
        .expect("Request should complete");

    // Should be blocked because concrete exists
    assert!(
        response.status() == StatusCode::CONFLICT ||
        response.status() == StatusCode::BAD_REQUEST,
        "Expected CONFLICT or BAD_REQUEST, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_delete_abstract_after_concrete_deleted() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("del_order_prod");
    let datatype_id = ctx.unique_id("del_order_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create abstract and concrete
    let abstract_attr = AbstractAttributeBuilder::new("loan", "deletable")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    let concrete = ConcreteAttributeBuilder::new("loan", "main", "deletable")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("100.00"))
        .build();
    let concrete_resp: AttributeResponse = ctx.server.post(
        &format!("/api/products/{}/attributes", product_id),
        &concrete
    ).await.expect("Failed to create concrete attribute");

    // Delete concrete first
    let concrete_path_encoded = urlencoding::encode(&concrete_resp.path);
    ctx.server.delete::<product_farm_api::rest::types::DeleteResponse>(
        &format!("/api/attributes/{}", concrete_path_encoded)
    ).await.expect("Failed to delete concrete attribute");

    // Now delete abstract should succeed
    let abstract_path_encoded = urlencoding::encode(&abstract_resp.abstract_path);
    let response: product_farm_api::rest::types::DeleteResponse = ctx.server
        .delete(&format!("/api/abstract-attributes/{}", abstract_path_encoded))
        .await
        .expect("Should be able to delete abstract after concrete is deleted");

    assert!(response.success);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Cross-Product Dependency Tests
// =============================================================================

#[tokio::test]
async fn test_concrete_cannot_reference_other_product_abstract() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup two products
    let product_id_1 = ctx.unique_product_id("prod1");
    let product_id_2 = ctx.unique_product_id("prod2");
    let datatype_id = ctx.unique_id("cross_prod_dt");

    let prod1_req = ProductBuilder::new(&product_id_1).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &prod1_req).await
        .expect("Failed to create product 1");
    ctx.dgraph.track_product(product_id_1.clone());

    let prod2_req = ProductBuilder::new(&product_id_2).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &prod2_req).await
        .expect("Failed to create product 2");
    ctx.dgraph.track_product(product_id_2.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create abstract attribute in product 1
    let abstract_attr = AbstractAttributeBuilder::new("loan", "amount")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id_1),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Try to create concrete in product 2 referencing product 1's abstract
    let concrete = ConcreteAttributeBuilder::new("loan", "main", "amount")
        .with_abstract_path(&abstract_resp.abstract_path) // This exists in product 1, not product 2
        .fixed_value(values::decimal("100.00"))
        .build();

    let response = ctx.server
        .post_response(&format!("/api/products/{}/attributes", product_id_2), &concrete)
        .await
        .expect("Request should complete");

    // Should be rejected - abstract doesn't exist in product 2
    assert!(
        response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND,
        "Expected BAD_REQUEST or NOT_FOUND, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}
