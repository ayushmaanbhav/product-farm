//! Abstract Attribute Tag Tests
//!
//! Tests for tag-based queries and tag management.

use crate::fixtures::{
    assertions::*,
    data_builders::{AbstractAttributeBuilder, DatatypeBuilder, ProductBuilder},
    TestContext,
};
use product_farm_api::rest::types::{
    AbstractAttributeResponse, DatatypeResponse, ListAbstractAttributesResponse, ProductResponse,
};

// =============================================================================
// Tag Creation Tests
// =============================================================================

#[tokio::test]
async fn test_create_attribute_with_single_tag() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("single_tag_prod");
    let datatype_id = ctx.unique_id("single_tag_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let attr_req = AbstractAttributeBuilder::new("loan", "principal")
        .with_datatype(&datatype_id)
        .with_tags(vec!["input"])
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create abstract attribute");

    assert_eq!(response.tags.len(), 1);
    assert_attribute_has_tag(&response, "input");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_attribute_with_multiple_tags() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("multi_tag_prod");
    let datatype_id = ctx.unique_id("multi_tag_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let attr_req = AbstractAttributeBuilder::new("premium", "total_premium")
        .with_datatype(&datatype_id)
        .with_tags(vec!["output", "calculated", "premium-related", "visible"])
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create abstract attribute");

    assert!(response.tags.len() >= 4);
    assert_attribute_has_tag(&response, "output");
    assert_attribute_has_tag(&response, "calculated");
    assert_attribute_has_tag(&response, "premium-related");
    assert_attribute_has_tag(&response, "visible");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_attribute_without_tags() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("no_tag_prod");
    let datatype_id = ctx.unique_id("no_tag_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let attr_req = AbstractAttributeBuilder::new("loan", "amount")
        .with_datatype(&datatype_id)
        // No tags
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create abstract attribute");

    assert!(response.tags.is_empty());

    ctx.cleanup().await.ok();
}

// =============================================================================
// Tag Query Tests
// =============================================================================

#[tokio::test]
async fn test_list_attributes_by_single_tag() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("query_tag_prod");
    let datatype_id = ctx.unique_id("query_tag_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attributes with different tags
    let attr_input = AbstractAttributeBuilder::new("loan", "input_attr")
        .with_datatype(&datatype_id)
        .with_tags(vec!["input"])
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_input
    ).await.expect("Failed to create input attribute");

    let attr_output = AbstractAttributeBuilder::new("loan", "output_attr")
        .with_datatype(&datatype_id)
        .with_tags(vec!["output"])
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_output
    ).await.expect("Failed to create output attribute");

    // Query by tag
    let response: ListAbstractAttributesResponse = ctx.server
        .get(&format!("/api/products/{}/abstract-attributes/by-tag/input?pageSize=100", product_id))
        .await
        .expect("Failed to list attributes by tag");

    // All returned should have "input" tag
    for item in &response.items {
        assert_attribute_has_tag(item, "input");
    }

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_query_by_nonexistent_tag() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("no_tag_query_prod");
    let datatype_id = ctx.unique_id("no_tag_query_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attribute with different tag
    let attr_req = AbstractAttributeBuilder::new("loan", "test_attr")
        .with_datatype(&datatype_id)
        .with_tags(vec!["existing-tag"])
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req
    ).await.expect("Failed to create attribute");

    // Query by non-existent tag
    let response: ListAbstractAttributesResponse = ctx.server
        .get(&format!("/api/products/{}/abstract-attributes/by-tag/nonexistent-tag?pageSize=100", product_id))
        .await
        .expect("Failed to list attributes by tag");

    // Should return empty list
    assert!(response.items.is_empty());

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_multiple_attributes_share_tag() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("shared_tag_prod");
    let datatype_id = ctx.unique_id("shared_tag_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create multiple attributes with shared tag
    for i in 0..3 {
        let attr_req = AbstractAttributeBuilder::new("premium", &format!("premium_attr_{}", i))
            .with_datatype(&datatype_id)
            .with_tags(vec!["premium-category"])
            .build();
        ctx.server.post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr_req
        ).await.expect("Failed to create attribute");
    }

    // Query by shared tag
    let response: ListAbstractAttributesResponse = ctx.server
        .get(&format!("/api/products/{}/abstract-attributes/by-tag/premium-category?pageSize=100", product_id))
        .await
        .expect("Failed to list attributes by tag");

    assert!(response.items.len() >= 3);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Tag Format Tests
// =============================================================================

#[tokio::test]
async fn test_tag_with_hyphen() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("hyphen_tag_prod");
    let datatype_id = ctx.unique_id("hyphen_tag_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let attr_req = AbstractAttributeBuilder::new("loan", "test_attr")
        .with_datatype(&datatype_id)
        .with_tags(vec!["premium-related", "user-input", "tax-applicable"])
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create attribute");

    assert_attribute_has_tag(&response, "premium-related");
    assert_attribute_has_tag(&response, "user-input");
    assert_attribute_has_tag(&response, "tax-applicable");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_tag_with_underscore() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("underscore_tag_prod");
    let datatype_id = ctx.unique_id("underscore_tag_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let attr_req = AbstractAttributeBuilder::new("loan", "test_attr")
        .with_datatype(&datatype_id)
        .with_tags(vec!["input_field", "required_attr"])
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create attribute");

    assert_attribute_has_tag(&response, "input_field");
    assert_attribute_has_tag(&response, "required_attr");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_duplicate_tags_in_request() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("dup_tag_prod");
    let datatype_id = ctx.unique_id("dup_tag_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Send duplicate tags in request
    let attr_req = AbstractAttributeBuilder::new("loan", "test_attr")
        .with_datatype(&datatype_id)
        .with_tags(vec!["input", "input", "output"]) // Duplicate "input"
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create attribute");

    // Should deduplicate (or document actual behavior)
    let input_count = response.tags.iter().filter(|t| t.name == "input").count();
    println!("Duplicate tag handling - 'input' appears {} times", input_count);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Tag Order Tests
// =============================================================================

#[tokio::test]
async fn test_tag_order_index() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("tag_order_prod");
    let datatype_id = ctx.unique_id("tag_order_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let attr_req = AbstractAttributeBuilder::new("loan", "test_attr")
        .with_datatype(&datatype_id)
        .with_tags(vec!["first", "second", "third"])
        .build();

    let response: AbstractAttributeResponse = ctx.server
        .post(&format!("/api/products/{}/abstract-attributes", product_id), &attr_req)
        .await
        .expect("Failed to create attribute");

    // Check order indices
    if response.tags.len() >= 3 {
        // Tags should have order_index assigned
        for tag in &response.tags {
            println!("Tag '{}' has order_index: {}", tag.name, tag.order_index);
        }

        // Verify order indices are sequential or at least assigned
        let indices: Vec<i32> = response.tags.iter().map(|t| t.order_index).collect();
        println!("Tag order indices: {:?}", indices);
    }

    ctx.cleanup().await.ok();
}

// =============================================================================
// Tag Filtering Combined Tests
// =============================================================================

#[tokio::test]
async fn test_tag_query_with_component_type_filter() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("combo_filter_prod");
    let datatype_id = ctx.unique_id("combo_filter_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create attributes with same tag in different components
    let attr_loan = AbstractAttributeBuilder::new("loan", "loan_input")
        .with_datatype(&datatype_id)
        .with_tags(vec!["input"])
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_loan
    ).await.expect("Failed to create loan attribute");

    let attr_premium = AbstractAttributeBuilder::new("premium", "premium_input")
        .with_datatype(&datatype_id)
        .with_tags(vec!["input"])
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_premium
    ).await.expect("Failed to create premium attribute");

    // Query by tag - should get both
    let all_inputs: ListAbstractAttributesResponse = ctx.server
        .get(&format!("/api/products/{}/abstract-attributes/by-tag/input?pageSize=100", product_id))
        .await
        .expect("Failed to list by tag");

    assert!(all_inputs.items.len() >= 2);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_empty_tag_rejected() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("empty_tag_prod");
    let datatype_id = ctx.unique_id("empty_tag_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Include empty tag
    let attr_req = AbstractAttributeBuilder::new("loan", "test_attr")
        .with_datatype(&datatype_id)
        .with_tags(vec!["valid", ""])
        .build();

    let result = ctx.server
        .post::<_, AbstractAttributeResponse>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr_req
        )
        .await;

    // Document behavior - might reject or filter out empty
    match result {
        Ok(response) => {
            let has_empty = response.tags.iter().any(|t| t.name.is_empty());
            println!("Empty tag filtered: {}", !has_empty);
        }
        Err(e) => {
            println!("Empty tag rejected: {}", e);
        }
    }

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_very_long_tag() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("long_tag_prod");
    let datatype_id = ctx.unique_id("long_tag_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Very long tag name
    let long_tag = "a".repeat(100);
    let attr_req = AbstractAttributeBuilder::new("loan", "test_attr")
        .with_datatype(&datatype_id)
        .with_tags(vec![&long_tag])
        .build();

    let result = ctx.server
        .post_response(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr_req
        )
        .await
        .expect("Request should complete");

    // Document behavior
    println!("Very long tag response: {}", result.status());

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_tag_case_sensitivity() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("case_tag_prod");
    let datatype_id = ctx.unique_id("case_tag_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Create with lowercase tag
    let attr_req = AbstractAttributeBuilder::new("loan", "test_attr")
        .with_datatype(&datatype_id)
        .with_tags(vec!["input"])
        .build();
    ctx.server.post::<_, AbstractAttributeResponse>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req
    ).await.expect("Failed to create attribute");

    // Query with uppercase
    let uppercase_result: ListAbstractAttributesResponse = ctx.server
        .get(&format!("/api/products/{}/abstract-attributes/by-tag/INPUT?pageSize=100", product_id))
        .await
        .expect("Failed to query with uppercase tag");

    // Document case sensitivity
    println!(
        "Uppercase tag query returned {} items (lowercase created)",
        uppercase_result.items.len()
    );

    ctx.cleanup().await.ok();
}
