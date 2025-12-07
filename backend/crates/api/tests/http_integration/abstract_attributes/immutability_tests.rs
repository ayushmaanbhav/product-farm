//! Abstract Attribute Immutability Tests
//!
//! Tests for immutable attribute behavior.

use crate::fixtures::{
    assertions::*,
    data_builders::{AbstractAttributeBuilder, DatatypeBuilder, ProductBuilder},
    TestContext,
};
use product_farm_api::rest::types::{
    AbstractAttributeResponse, DatatypeResponse, DeleteResponse, ProductResponse,
};
use reqwest::StatusCode;

// =============================================================================
// Immutable Flag Tests
// =============================================================================

#[tokio::test]
async fn test_create_immutable_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("immut_attr_prod");
    let datatype_id = ctx.unique_id("immut_attr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let attr_req = AbstractAttributeBuilder::new("core", "product_id")
        .with_datatype(&datatype_id)
        .immutable()
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create immutable attribute");

    assert_attribute_immutable(&response);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_mutable_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("mut_attr_prod");
    let datatype_id = ctx.unique_id("mut_attr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Default is mutable (immutable: false)
    let attr_req = AbstractAttributeBuilder::new("loan", "interest_rate")
        .with_datatype(&datatype_id)
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create mutable attribute");

    assert!(!response.immutable);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Delete Immutable Tests
// =============================================================================

#[tokio::test]
async fn test_cannot_delete_immutable_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("del_immut_prod");
    let datatype_id = ctx.unique_id("del_immut_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create immutable attribute
    let attr_req = AbstractAttributeBuilder::new("core", "locked_field")
        .with_datatype(&datatype_id)
        .immutable()
        .build();
    let created: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req
    ).await.expect("Failed to create immutable attribute");

    // Try to delete
    let encoded_path = urlencoding::encode(&created.abstract_path);
    let response = ctx.server
        .delete_response(&format!("/api/abstract-attributes/{}", encoded_path))
        .await
        .expect("Request should complete");

    // Should be blocked (FORBIDDEN, BAD_REQUEST, CONFLICT, or PRECONDITION_FAILED)
    assert!(
        response.status() == StatusCode::FORBIDDEN ||
        response.status() == StatusCode::BAD_REQUEST ||
        response.status() == StatusCode::CONFLICT ||
        response.status() == StatusCode::PRECONDITION_FAILED,
        "Expected deletion to be blocked, got {}",
        response.status()
    );

    // Verify still exists
    let get_response: AbstractAttributeResponse = ctx.server
        .get(&format!("/api/abstract-attributes/{}", encoded_path))
        .await
        .expect("Immutable attribute should still exist");

    assert_attribute_immutable(&get_response);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_can_delete_mutable_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("del_mut_prod");
    let datatype_id = ctx.unique_id("del_mut_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create mutable attribute
    let attr_req = AbstractAttributeBuilder::new("loan", "deletable_field")
        .with_datatype(&datatype_id)
        .build();
    let created: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req
    ).await.expect("Failed to create mutable attribute");

    // Delete should succeed
    let encoded_path = urlencoding::encode(&created.abstract_path);
    let response: DeleteResponse = ctx.server
        .delete(&format!("/api/abstract-attributes/{}", encoded_path))
        .await
        .expect("Failed to delete mutable attribute");

    assert!(response.success);

    // Verify deleted
    let get_response = ctx.server
        .get_response(&format!("/api/abstract-attributes/{}", encoded_path))
        .await
        .expect("Request should complete");
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Mixed Mutability Tests
// =============================================================================

#[tokio::test]
async fn test_mixed_mutable_immutable_attributes() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("mixed_mut_prod");
    let datatype_id = ctx.unique_id("mixed_mut_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create immutable attribute
    let immut_req = AbstractAttributeBuilder::new("core", "immutable_attr")
        .with_datatype(&datatype_id)
        .immutable()
        .build();
    let immut_resp: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &immut_req)
        .await
        .expect("Failed to create immutable attribute");

    // Create mutable attribute
    let mut_req = AbstractAttributeBuilder::new("loan", "mutable_attr")
        .with_datatype(&datatype_id)
        .build();
    let mut_resp: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &mut_req)
        .await
        .expect("Failed to create mutable attribute");

    assert!(immut_resp.immutable);
    assert!(!mut_resp.immutable);

    // Delete mutable should work
    let mut_path = urlencoding::encode(&mut_resp.abstract_path);
    let del_mut: DeleteResponse = ctx.server
        .delete(&format!("/api/abstract-attributes/{}", mut_path))
        .await
        .expect("Failed to delete mutable attribute");
    assert!(del_mut.success);

    // Delete immutable should fail
    let immut_path = urlencoding::encode(&immut_resp.abstract_path);
    let del_immut = ctx.server
        .delete_response(&format!("/api/abstract-attributes/{}", immut_path))
        .await
        .expect("Request should complete");

    assert!(
        del_immut.status() == StatusCode::FORBIDDEN ||
        del_immut.status() == StatusCode::BAD_REQUEST ||
        del_immut.status() == StatusCode::CONFLICT ||
        del_immut.status() == StatusCode::PRECONDITION_FAILED,
        "Expected immutable deletion to be blocked, got {}",
        del_immut.status()
    );

    ctx.cleanup().await.ok();
}

// =============================================================================
// Immutability Persistence Tests
// =============================================================================

#[tokio::test]
async fn test_immutability_persisted_on_get() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("persist_immut_prod");
    let datatype_id = ctx.unique_id("persist_immut_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create immutable attribute
    let attr_req = AbstractAttributeBuilder::new("core", "persistent_immut")
        .with_datatype(&datatype_id)
        .immutable()
        .build();
    let created: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req
    ).await.expect("Failed to create attribute");

    // Get and verify immutability is preserved
    let encoded_path = urlencoding::encode(&created.abstract_path);
    let response: AbstractAttributeResponse = ctx.server
        .get(&format!("/api/abstract-attributes/{}", encoded_path))
        .await
        .expect("Failed to get attribute");

    assert_attribute_immutable(&response);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_immutability_in_list_response() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("list_immut_prod");
    let datatype_id = ctx.unique_id("list_immut_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create mix of attributes
    let immut_req = AbstractAttributeBuilder::new("core", "immut_listed")
        .with_datatype(&datatype_id)
        .immutable()
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &immut_req
    ).await.expect("Failed to create immutable attribute");

    let mut_req = AbstractAttributeBuilder::new("loan", "mut_listed")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &mut_req
    ).await.expect("Failed to create mutable attribute");

    // List and check immutability flags
    let response: product_farm_api::rest::types::ListAbstractAttributesResponse = ctx.server
        .get(&format!("/api/products/{}/abstract-attributes?pageSize=100", product_id))
        .await
        .expect("Failed to list attributes");

    let immut_attr = response.items.iter()
        .find(|a| a.attribute_name == "immut_listed");
    let mut_attr = response.items.iter()
        .find(|a| a.attribute_name == "mut_listed");

    assert!(immut_attr.is_some(), "Immutable attribute should be in list");
    assert!(mut_attr.is_some(), "Mutable attribute should be in list");

    if let Some(attr) = immut_attr {
        assert!(attr.immutable);
    }
    if let Some(attr) = mut_attr {
        assert!(!attr.immutable);
    }

    ctx.cleanup().await.ok();
}

// =============================================================================
// Immutable with Concrete Attribute Tests
// =============================================================================

#[tokio::test]
async fn test_immutable_abstract_with_concrete_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("immut_concrete_prod");
    let datatype_id = ctx.unique_id("immut_concrete_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create immutable abstract attribute
    let attr_req = AbstractAttributeBuilder::new("core", "base_value")
        .with_datatype(&datatype_id)
        .immutable()
        .build();
    let abstract_attr: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create immutable abstract attribute");

    assert_attribute_immutable(&abstract_attr);

    // The immutability of the abstract attribute should affect its concrete instances
    // (if concrete attributes are created based on it)
    // This test documents the relationship

    ctx.cleanup().await.ok();
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_immutable_attribute_with_tags() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("immut_tag_prod");
    let datatype_id = ctx.unique_id("immut_tag_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create immutable attribute with tags
    let attr_req = AbstractAttributeBuilder::new("core", "tagged_immut")
        .with_datatype(&datatype_id)
        .with_tags(vec!["core", "locked", "system"])
        .immutable()
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create attribute");

    assert_attribute_immutable(&response);
    assert_attribute_has_tag(&response, "core");
    assert_attribute_has_tag(&response, "locked");
    assert_attribute_has_tag(&response, "system");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_immutable_attribute_with_display_names() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("immut_disp_prod");
    let datatype_id = ctx.unique_id("immut_disp_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create immutable attribute with display names
    let attr_req = AbstractAttributeBuilder::new("core", "displayed_immut")
        .with_datatype(&datatype_id)
        .with_display_name("System ID", "en-US", 0)
        .immutable()
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create attribute");

    assert_attribute_immutable(&response);
    assert!(!response.display_names.is_empty());

    ctx.cleanup().await.ok();
}
