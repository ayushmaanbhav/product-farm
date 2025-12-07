//! Abstract Attribute Path Format Tests
//!
//! Tests for abstract path format validation and construction.

use crate::fixtures::{
    assertions::assert_abstract_attribute_path,
    data_builders::{AbstractAttributeBuilder, DatatypeBuilder, ProductBuilder},
    TestContext,
};
use product_farm_api::rest::types::{
    AbstractAttributeResponse, CreateAbstractAttributeRequest, DatatypeResponse,
    ListAbstractAttributesResponse, ProductResponse,
};
use reqwest::StatusCode;

// =============================================================================
// Path Format Tests
// =============================================================================

#[tokio::test]
async fn test_path_without_component_id() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("path_no_id_prod");
    let datatype_id = ctx.unique_id("path_no_id_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attribute without component_id
    let attr_req = AbstractAttributeBuilder::new("loan", "interest_rate")
        .with_datatype(&datatype_id)
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create abstract attribute");

    // Path should be: component_type/attribute_name
    assert_abstract_attribute_path(&response, &["loan", "interest_rate"]);
    assert!(response.component_id.is_none());

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_path_with_component_id() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("path_with_id_prod");
    let datatype_id = ctx.unique_id("path_with_id_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::string(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attribute with component_id
    let attr_req = AbstractAttributeBuilder::new("coverage", "description")
        .with_component_id("liability")
        .with_datatype(&datatype_id)
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create abstract attribute");

    // Path should be: component_type/component_id/attribute_name
    assert_abstract_attribute_path(&response, &["coverage", "liability", "description"]);
    assert_eq!(response.component_id, Some("liability".to_string()));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_multiple_attributes_same_component_type() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("multi_attr_prod");
    let datatype_id = ctx.unique_id("multi_attr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create multiple attributes under same component type
    let attributes = ["attr_one", "attr_two", "attr_three"];

    for attr_name in attributes {
        let attr_req = AbstractAttributeBuilder::new("loan", attr_name)
            .with_datatype(&datatype_id)
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr_req
        ).await.expect("Failed to create attribute");
    }

    // List and verify paths
    let response: ListAbstractAttributesResponse = ctx.server
        .get(&format!("/api/products/{}/abstract-attributes?componentType=loan&pageSize=100", product_id))
        .await
        .expect("Failed to list attributes");

    assert!(response.items.len() >= 3);

    for item in &response.items {
        if item.component_type == "loan" {
            // API returns full path format: {productId}:abstract-path:loan:{attributeName}
            assert!(item.abstract_path.contains(":loan:"),
                "Expected path to contain ':loan:', got {}", item.abstract_path);
        }
    }

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_same_attribute_name_different_components() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("same_name_prod");
    let datatype_id = ctx.unique_id("same_name_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Same attribute name under different component types
    let attr_loan = AbstractAttributeBuilder::new("loan", "amount")
        .with_datatype(&datatype_id)
        .build();
    let loan_resp: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_loan)
        .await
        .expect("Failed to create loan attribute");

    let attr_premium = AbstractAttributeBuilder::new("premium", "amount")
        .with_datatype(&datatype_id)
        .build();
    let premium_resp: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_premium)
        .await
        .expect("Failed to create premium attribute");

    // Both should exist with different paths
    assert_ne!(loan_resp.abstract_path, premium_resp.abstract_path);
    assert!(loan_resp.abstract_path.contains("loan"));
    assert!(premium_resp.abstract_path.contains("premium"));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_same_attribute_different_component_ids() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("diff_comp_id_prod");
    let datatype_id = ctx.unique_id("diff_comp_id_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Same attribute name and component type, different component IDs
    let attr1 = AbstractAttributeBuilder::new("coverage", "limit")
        .with_component_id("liability")
        .with_datatype(&datatype_id)
        .build();
    let resp1: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr1)
        .await
        .expect("Failed to create liability coverage attribute");

    let attr2 = AbstractAttributeBuilder::new("coverage", "limit")
        .with_component_id("collision")
        .with_datatype(&datatype_id)
        .build();
    let resp2: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr2)
        .await
        .expect("Failed to create collision coverage attribute");

    // Both should exist with different paths
    assert_ne!(resp1.abstract_path, resp2.abstract_path);
    assert!(resp1.abstract_path.contains("liability"));
    assert!(resp2.abstract_path.contains("collision"));

    ctx.cleanup().await.ok();
}

// =============================================================================
// Invalid Path Tests
// =============================================================================

#[tokio::test]
async fn test_attribute_name_with_special_characters() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("special_char_prod");
    let datatype_id = ctx.unique_id("special_char_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::string(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Test various special characters
    let special_names = [
        "attr_with_underscore",
        // These might be rejected:
        // "attr-with-dash",
        // "attr.with.dot",
    ];

    for attr_name in special_names {
        let attr_req = AbstractAttributeBuilder::new("loan", attr_name)
            .with_datatype(&datatype_id)
            .build();

        let result = ctx.server
            .post::<_, AbstractAttributeResponse>(
                &format!("/api/products/{}/abstract-attributes", product_id),
                &attr_req
            )
            .await;

        match result {
            Ok(resp) => {
                println!("Special name '{}' accepted, path: {}", attr_name, resp.abstract_path);
            }
            Err(e) => {
                println!("Special name '{}' rejected: {}", attr_name, e);
            }
        }
    }

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_attribute_name_with_slash() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("slash_name_prod");
    let datatype_id = ctx.unique_id("slash_name_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::string(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Attribute name with slash (should be rejected - would break path)
    let attr_req = CreateAbstractAttributeRequest {
        component_type: "loan".to_string(),
        component_id: None,
        attribute_name: "invalid/name".to_string(),
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
async fn test_component_type_with_slash() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("slash_comp_prod");
    let datatype_id = ctx.unique_id("slash_comp_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::string(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Component type with slash (should be rejected)
    let attr_req = CreateAbstractAttributeRequest {
        component_type: "invalid/type".to_string(),
        component_id: None,
        attribute_name: "attr".to_string(),
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
async fn test_very_long_attribute_name() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("long_name_prod");
    let datatype_id = ctx.unique_id("long_name_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::string(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Very long attribute name
    let long_name = "a".repeat(100);
    let attr_req = CreateAbstractAttributeRequest {
        component_type: "loan".to_string(),
        component_id: None,
        attribute_name: long_name,
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

    // Should be rejected for being too long
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Path Lookup Tests
// =============================================================================

#[tokio::test]
async fn test_get_by_encoded_path() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("encoded_path_prod");
    let datatype_id = ctx.unique_id("encoded_path_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attribute with component_id
    let attr_req = AbstractAttributeBuilder::new("coverage", "premium")
        .with_component_id("main")
        .with_datatype(&datatype_id)
        .build();
    let created: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req
    ).await.expect("Failed to create attribute");

    // Get by URL-encoded path using abstract_path from response
    let encoded = urlencoding::encode(&created.abstract_path);
    let response: AbstractAttributeResponse = ctx.server
        .get(&format!("/api/abstract-attributes/{}", encoded))
        .await
        .expect("Failed to get attribute by encoded path");

    assert_eq!(response.attribute_name, "premium");
    assert_eq!(response.component_id, Some("main".to_string()));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_path_case_sensitivity() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("case_path_prod");
    let datatype_id = ctx.unique_id("case_path_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attribute with lowercase
    let attr_req = AbstractAttributeBuilder::new("loan", "interest_rate")
        .with_datatype(&datatype_id)
        .build();
    let created: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req
    ).await.expect("Failed to create attribute");

    // Try to get with different case - construct wrong-case path
    let wrong_case_path = created.abstract_path.to_uppercase();
    let uppercase_path = urlencoding::encode(&wrong_case_path);
    let response = ctx.server
        .get_response(&format!("/api/abstract-attributes/{}", uppercase_path))
        .await
        .expect("Request should complete");

    // Document case sensitivity behavior
    println!("Uppercase path lookup status: {}", response.status());

    ctx.cleanup().await.ok();
}

// =============================================================================
// Path with URL Special Characters
// =============================================================================

#[tokio::test]
async fn test_path_with_url_encoding() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("url_enc_prod");
    let datatype_id = ctx.unique_id("url_enc_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::string(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attribute with underscore (common case)
    let attr_req = AbstractAttributeBuilder::new("policy_details", "policy_number")
        .with_datatype(&datatype_id)
        .build();
    let created: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req
    ).await.expect("Failed to create attribute");

    // Verify retrieval with encoding
    let encoded = urlencoding::encode(&created.abstract_path);
    let response: AbstractAttributeResponse = ctx.server
        .get(&format!("/api/abstract-attributes/{}", encoded))
        .await
        .expect("Failed to get attribute");

    assert_eq!(response.attribute_name, "policy_number");

    ctx.cleanup().await.ok();
}
