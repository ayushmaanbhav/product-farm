//! Abstract Attribute CRUD Tests

use crate::fixtures::{
    assertions::*,
    data_builders::{AbstractAttributeBuilder, DatatypeBuilder, ProductBuilder},
    TestContext,
};
use product_farm_api::rest::types::{
    AbstractAttributeResponse, CreateAbstractAttributeRequest, DatatypeResponse,
    DeleteResponse, ListAbstractAttributesResponse, ProductResponse,
};
use reqwest::StatusCode;

// =============================================================================
// CREATE Tests
// =============================================================================

#[tokio::test]
async fn test_create_abstract_attribute_basic() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup: Create product and datatype
    let product_id = ctx.unique_product_id("attr_prod");
    let datatype_id = ctx.unique_id("attr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create abstract attribute
    let attr_req = AbstractAttributeBuilder::new("loan", "interest_rate")
        .with_datatype(&datatype_id)
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create abstract attribute");

    assert_eq!(response.component_type, "loan");
    assert_eq!(response.attribute_name, "interest_rate");
    assert_eq!(response.datatype_id, datatype_id);
    assert_eq!(response.product_id, product_id);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_abstract_attribute_with_component_id() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("comp_attr_prod");
    let datatype_id = ctx.unique_id("comp_attr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::string(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create with component_id
    let attr_req = AbstractAttributeBuilder::new("coverage", "coverage_name")
        .with_component_id("liability")
        .with_datatype(&datatype_id)
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create abstract attribute");

    assert_eq!(response.component_type, "coverage");
    assert_eq!(response.component_id, Some("liability".to_string()));
    assert_eq!(response.attribute_name, "coverage_name");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_abstract_attribute_with_description() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("desc_attr_prod");
    let datatype_id = ctx.unique_id("desc_attr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let attr_req = AbstractAttributeBuilder::new("pricing", "base_premium")
        .with_datatype(&datatype_id)
        .with_description("Base premium amount before adjustments")
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create abstract attribute");

    assert_eq!(response.description, "Base premium amount before adjustments");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_abstract_attribute_with_display_names() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("disp_attr_prod");
    let datatype_id = ctx.unique_id("disp_attr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let attr_req = AbstractAttributeBuilder::new("policy", "sum_assured")
        .with_datatype(&datatype_id)
        .with_display_name("Sum Assured", "en-US", 0)
        .with_display_name("Montant Assuré", "fr-FR", 1)
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create abstract attribute");

    assert_eq!(response.display_names.len(), 2);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_abstract_attribute_with_tags() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("tag_attr_prod");
    let datatype_id = ctx.unique_id("tag_attr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let attr_req = AbstractAttributeBuilder::new("premium", "total_premium")
        .with_datatype(&datatype_id)
        .with_tags(vec!["output", "calculated", "premium-related"])
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create abstract attribute");

    assert_attribute_has_tag(&response, "output");
    assert_attribute_has_tag(&response, "calculated");
    assert_attribute_has_tag(&response, "premium-related");

    ctx.cleanup().await.ok();
}

// =============================================================================
// READ Tests
// =============================================================================

#[tokio::test]
async fn test_get_abstract_attribute_by_path() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("get_attr_prod");
    let datatype_id = ctx.unique_id("get_attr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::int(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attribute
    let attr_req = AbstractAttributeBuilder::new("policy", "term_months")
        .with_datatype(&datatype_id)
        .build();
    let created: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req
    ).await.expect("Failed to create abstract attribute");

    // Get by path - use the abstract_path from the response
    let encoded_path = urlencoding::encode(&created.abstract_path);
    let response: AbstractAttributeResponse = ctx.server
        .get(&format!("/api/abstract-attributes/{}", encoded_path))
        .await
        .expect("Failed to get abstract attribute");

    assert_eq!(response.attribute_name, "term_months");
    assert_eq!(response.component_type, "policy");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_get_nonexistent_abstract_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup: Create product
    let product_id = ctx.unique_product_id("no_attr_prod");
    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Use a fake abstract path format that won't exist
    let fake_path = format!("{}:abstract-path:nonexistent:attribute", product_id);
    let encoded_path = urlencoding::encode(&fake_path);
    let response = ctx.server
        .get_response(&format!("/api/abstract-attributes/{}", encoded_path))
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_list_abstract_attributes() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("list_attr_prod");
    let datatype_id = ctx.unique_id("list_attr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create multiple attributes
    for i in 0..3 {
        let attr_req = AbstractAttributeBuilder::new("loan", &format!("attr_{}", i))
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr_req
        ).await.expect("Failed to create abstract attribute");
    }

    // List attributes
    let response: ListAbstractAttributesResponse = ctx.server
        .get(&format!("/api/products/{}/abstract-attributes?pageSize=100", product_id))
        .await
        .expect("Failed to list abstract attributes");

    assert!(response.items.len() >= 3);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_list_abstract_attributes_by_component_type() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("filter_attr_prod");
    let datatype_id = ctx.unique_id("filter_attr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attributes with different component types
    let attr1 = AbstractAttributeBuilder::new("loan", "loan_attr")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr1
    ).await.expect("Failed to create loan attribute");

    let attr2 = AbstractAttributeBuilder::new("premium", "premium_attr")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr2
    ).await.expect("Failed to create premium attribute");

    // Filter by component type
    let response: ListAbstractAttributesResponse = ctx.server
        .get(&format!("/api/products/{}/abstract-attributes?componentType=loan&pageSize=100", product_id))
        .await
        .expect("Failed to list abstract attributes");

    for item in &response.items {
        assert_eq!(item.component_type, "loan");
    }

    ctx.cleanup().await.ok();
}

// =============================================================================
// DELETE Tests
// =============================================================================

#[tokio::test]
async fn test_delete_abstract_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("del_attr_prod");
    let datatype_id = ctx.unique_id("del_attr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attribute
    let attr_req = AbstractAttributeBuilder::new("loan", "to_delete")
        .with_datatype(&datatype_id)
        .build();
    let created: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req
    ).await.expect("Failed to create abstract attribute");

    // Delete attribute using the abstract_path from response
    let encoded_path = urlencoding::encode(&created.abstract_path);
    let response: DeleteResponse = ctx.server
        .delete(&format!("/api/abstract-attributes/{}", encoded_path))
        .await
        .expect("Failed to delete abstract attribute");

    assert!(response.success);

    // Verify deletion
    let get_response = ctx.server
        .get_response(&format!("/api/abstract-attributes/{}", encoded_path))
        .await
        .expect("Request should complete");
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_delete_nonexistent_abstract_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup: Create product
    let product_id = ctx.unique_product_id("del_no_attr_prod");
    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Use correct abstract path format for 404 test
    let fake_path = format!("{}:abstract-path:nonexistent:attribute", product_id);
    let encoded_path = urlencoding::encode(&fake_path);
    let response = ctx.server
        .delete_response(&format!("/api/abstract-attributes/{}", encoded_path))
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Validation Tests
// =============================================================================

#[tokio::test]
async fn test_create_attribute_missing_datatype() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup: Create product only (no datatype)
    let product_id = ctx.unique_product_id("no_dt_attr_prod");
    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Try to create attribute with non-existent datatype
    let attr_req = CreateAbstractAttributeRequest {
        component_type: "loan".to_string(),
        component_id: None,
        attribute_name: "test_attr".to_string(),
        datatype_id: "nonexistent_datatype".to_string(),
        enum_name: None,
        constraint_expression: None,
        immutable: false,
        description: None,
        display_names: vec![],
        tags: vec![],
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
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
async fn test_create_attribute_empty_name() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("empty_name_prod");
    let datatype_id = ctx.unique_id("empty_name_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::string(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let attr_req = CreateAbstractAttributeRequest {
        component_type: "loan".to_string(),
        component_id: None,
        attribute_name: "".to_string(), // Empty name
        datatype_id: datatype_id.clone(),
        enum_name: None,
        constraint_expression: None,
        immutable: false,
        description: None,
        display_names: vec![],
        tags: vec![],
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_attribute_empty_component_type() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("empty_comp_prod");
    let datatype_id = ctx.unique_id("empty_comp_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::string(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let attr_req = CreateAbstractAttributeRequest {
        component_type: "".to_string(), // Empty
        component_id: None,
        attribute_name: "test_attr".to_string(),
        datatype_id: datatype_id.clone(),
        enum_name: None,
        constraint_expression: None,
        immutable: false,
        description: None,
        display_names: vec![],
        tags: vec![],
    };

    let response = ctx.server
        .post_response(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_duplicate_abstract_attribute() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("dup_attr_prod");
    let datatype_id = ctx.unique_id("dup_attr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create first attribute
    let attr_req = AbstractAttributeBuilder::new("loan", "duplicate_attr")
        .with_datatype(&datatype_id)
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req
    ).await.expect("Failed to create first attribute");

    // Try to create duplicate
    let dup_req = AbstractAttributeBuilder::new("loan", "duplicate_attr")
        .with_datatype(&datatype_id)
        .build();

    let response = ctx.server
        .post_response(&format!("/api/products/{}/abstract-attributes", product_id), &dup_req)
        .await
        .expect("Request should complete");

    assert!(
        response.status() == StatusCode::CONFLICT || response.status() == StatusCode::BAD_REQUEST,
        "Expected CONFLICT or BAD_REQUEST, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_attribute_for_nonexistent_product() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let attr_req = AbstractAttributeBuilder::new("loan", "test_attr")
        .with_datatype("some_datatype")
        .build();

    let response = ctx.server
        .post_response("/api/products/nonexistent_product/abstract-attributes", &attr_req)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

// =============================================================================
// CRUD Cycle Test
// =============================================================================

#[tokio::test]
async fn test_abstract_attribute_crud_cycle() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("crud_cycle_prod");
    let datatype_id = ctx.unique_id("crud_cycle_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // CREATE
    let attr_req = AbstractAttributeBuilder::new("policy", "coverage_amount")
        .with_datatype(&datatype_id)
        .with_description("Coverage amount for policy")
        .with_tags(vec!["coverage", "amount"])
        .build();

    let created: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create abstract attribute");

    assert_eq!(created.attribute_name, "coverage_amount");
    assert_eq!(created.component_type, "policy");

    // READ - use the abstract_path from the response
    let encoded_path = urlencoding::encode(&created.abstract_path);
    let read: AbstractAttributeResponse = ctx.server
        .get(&format!("/api/abstract-attributes/{}", encoded_path))
        .await
        .expect("Failed to get abstract attribute");

    assert_eq!(read.attribute_name, "coverage_amount");
    assert_eq!(read.description, "Coverage amount for policy");

    // DELETE
    let deleted: DeleteResponse = ctx.server
        .delete(&format!("/api/abstract-attributes/{}", encoded_path))
        .await
        .expect("Failed to delete abstract attribute");

    assert!(deleted.success);

    // VERIFY DELETED
    let verify = ctx.server
        .get_response(&format!("/api/abstract-attributes/{}", encoded_path))
        .await
        .expect("Request should complete");
    assert_eq!(verify.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}
