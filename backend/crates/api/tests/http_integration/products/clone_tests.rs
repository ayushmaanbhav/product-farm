//! Product Clone Tests
//!
//! Tests for product cloning functionality.

use crate::fixtures::{
    assertions::*, data_builders::{ProductBuilder, CloneProductBuilder}, TestContext,
};
use product_farm_api::rest::types::{CloneProductResponse, ProductResponse};
use reqwest::StatusCode;

// =============================================================================
// Basic Clone Tests
// =============================================================================

#[tokio::test]
async fn test_clone_product_basic() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let source_id = ctx.unique_product_id("sourceprod");
    let clone_id = ctx.unique_product_id("cloneprod");

    // Create source product
    let request = ProductBuilder::new(&source_id)
        .with_name("Source Product")
        .with_template("insurance")
        .with_description("Original product")
        .build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create source product");
    ctx.dgraph.track_product(source_id.clone());

    // Clone product
    let clone_request = CloneProductBuilder::new(&clone_id, "Cloned Product")
        .with_description("A clone of the source")
        .build();

    let response: CloneProductResponse = ctx.server
        .post(&format!("/api/products/{}/clone", source_id), &clone_request)
        .await
        .expect("Failed to clone product");

    // Assert cloned product
    assert_eq!(response.product.id, clone_id);
    assert_eq!(response.product.name, "Cloned Product");
    assert_eq!(response.product.template_type, "insurance"); // Inherits template
    assert_product_draft(&response.product);
    assert_product_has_parent(&response.product, &source_id);

    // Verify clone exists via HTTP API (in-memory backend)
    let get_response: ProductResponse = ctx.server
        .get(&format!("/api/products/{}", clone_id))
        .await
        .expect("Failed to get cloned product via API");
    assert_eq!(get_response.parent_product_id, Some(source_id.clone()));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_clone_inherits_template_type() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    let templates = ["insurance", "trading", "loan"];

    for template in templates {
        let source_id = ctx.unique_product_id(&format!("{}source", template));
        let clone_id = ctx.unique_product_id(&format!("{}clone", template));

        // Create source with specific template
        let request = ProductBuilder::new(&source_id)
            .with_template(template)
            .build();
        ctx.server.post::<_, ProductResponse>("/api/products", &request).await
            .expect("Failed to create source");
        ctx.dgraph.track_product(source_id.clone());

        // Clone
        let clone_request = CloneProductBuilder::new(&clone_id, "Clone").build();
        let response: CloneProductResponse = ctx.server
            .post(&format!("/api/products/{}/clone", source_id), &clone_request)
            .await
            .expect("Failed to clone");

        // Verify inherited template
        assert_eq!(response.product.template_type, template);
        ctx.dgraph.track_product(clone_id);
    }

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_clone_with_selective_components() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let source_id = ctx.unique_product_id("selectsource");
    let clone_id = ctx.unique_product_id("selectclone");

    // Create source product
    let request = ProductBuilder::new(&source_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create source");
    ctx.dgraph.track_product(source_id.clone());

    // Clone with selective components (empty selection)
    let clone_request = CloneProductBuilder::new(&clone_id, "Selective Clone")
        .with_components(vec![])
        .with_concrete_attributes(false)
        .build();

    let response: CloneProductResponse = ctx.server
        .post(&format!("/api/products/{}/clone", source_id), &clone_request)
        .await
        .expect("Failed to clone");

    assert_eq!(response.product.id, clone_id);
    ctx.dgraph.track_product(clone_id);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Clone Error Cases
// =============================================================================

#[tokio::test]
async fn test_clone_nonexistent_product() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let clone_id = ctx.unique_product_id("badclone");

    let clone_request = CloneProductBuilder::new(&clone_id, "Bad Clone").build();

    let response = ctx.server
        .post_response("/api/products/nonexistent_source_xyz/clone", &clone_request)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_clone_with_duplicate_id() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let source_id = ctx.unique_product_id("dupsource");
    let existing_id = ctx.unique_product_id("existingprod");

    // Create source product
    let request = ProductBuilder::new(&source_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create source");
    ctx.dgraph.track_product(source_id.clone());

    // Create existing product with the ID we want to clone to
    let existing_request = ProductBuilder::new(&existing_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &existing_request).await
        .expect("Failed to create existing product");
    ctx.dgraph.track_product(existing_id.clone());

    // Try to clone with duplicate ID
    let clone_request = CloneProductBuilder::new(&existing_id, "Duplicate Clone").build();
    let response = ctx.server
        .post_response(&format!("/api/products/{}/clone", source_id), &clone_request)
        .await
        .expect("Request should complete");

    // Should fail due to duplicate ID
    assert!(
        response.status() == StatusCode::CONFLICT ||
        response.status() == StatusCode::BAD_REQUEST,
        "Expected CONFLICT or BAD_REQUEST, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_clone_with_invalid_new_id() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let source_id = ctx.unique_product_id("invalidsource");

    // Create source product
    let request = ProductBuilder::new(&source_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create source");
    ctx.dgraph.track_product(source_id.clone());

    // Try to clone with invalid ID (contains spaces)
    let clone_request = CloneProductBuilder::new("invalid clone id", "Bad Clone").build();
    let response = ctx.server
        .post_response(&format!("/api/products/{}/clone", source_id), &clone_request)
        .await
        .expect("Request should complete");

    assert!(
        response.status() == StatusCode::BAD_REQUEST,
        "Expected BAD_REQUEST, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

// =============================================================================
// Clone State Tests
// =============================================================================

#[tokio::test]
async fn test_clone_always_creates_draft() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let source_id = ctx.unique_product_id("activesource");
    let clone_id = ctx.unique_product_id("draftclone");

    // Create, submit, and approve source product
    let request = ProductBuilder::new(&source_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create source");
    ctx.dgraph.track_product(source_id.clone());

    ctx.server.post::<_, ProductResponse>(
        &format!("/api/products/{}/submit", source_id),
        &serde_json::json!({})
    ).await.expect("Failed to submit");

    ctx.server.post::<_, ProductResponse>(
        &format!("/api/products/{}/approve", source_id),
        &serde_json::json!({"approved": true})
    ).await.expect("Failed to approve");

    // Clone active product
    let clone_request = CloneProductBuilder::new(&clone_id, "Draft Clone").build();
    let response: CloneProductResponse = ctx.server
        .post(&format!("/api/products/{}/clone", source_id), &clone_request)
        .await
        .expect("Failed to clone");

    // Clone should be DRAFT regardless of source status
    assert_product_draft(&response.product);
    ctx.dgraph.track_product(clone_id);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_clone_resets_version() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let source_id = ctx.unique_product_id("versionsource");
    let clone_id = ctx.unique_product_id("versionclone");

    // Create source product
    let request = ProductBuilder::new(&source_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create source");
    ctx.dgraph.track_product(source_id.clone());

    // Clone
    let clone_request = CloneProductBuilder::new(&clone_id, "Version Clone").build();
    let response: CloneProductResponse = ctx.server
        .post(&format!("/api/products/{}/clone", source_id), &clone_request)
        .await
        .expect("Failed to clone");

    // Clone should have version 1
    assert_product_version(&response.product, 1);
    ctx.dgraph.track_product(clone_id);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_clone_sets_parent_reference() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let source_id = ctx.unique_product_id("parentsource");
    let clone_id = ctx.unique_product_id("childclone");

    // Create source product
    let request = ProductBuilder::new(&source_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create source");
    ctx.dgraph.track_product(source_id.clone());

    // Clone
    let clone_request = CloneProductBuilder::new(&clone_id, "Child Clone").build();
    let response: CloneProductResponse = ctx.server
        .post(&format!("/api/products/{}/clone", source_id), &clone_request)
        .await
        .expect("Failed to clone");

    // Verify parent reference
    assert_product_has_parent(&response.product, &source_id);

    // Get clone and verify again
    let clone: ProductResponse = ctx.server
        .get(&format!("/api/products/{}", clone_id))
        .await
        .expect("Failed to get clone");
    assert_product_has_parent(&clone, &source_id);

    ctx.dgraph.track_product(clone_id);
    ctx.cleanup().await.ok();
}
