//! Product Lifecycle Tests
//!
//! Tests for product status transitions: DRAFT -> PENDING_APPROVAL -> ACTIVE -> DISCONTINUED

use crate::fixtures::{
    assertions::*, data_builders::ProductBuilder, TestContext,
};
use product_farm_api::rest::types::{ApprovalRequest, ProductResponse, ProductTemplateResponse, UpdateProductRequest};
use reqwest::StatusCode;

// =============================================================================
// Status Transition Tests
// =============================================================================

#[tokio::test]
async fn test_submit_draft_to_pending() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("submitprod");

    // Create product (starts as DRAFT)
    let request = ProductBuilder::new(&product_id).build();
    let created: ProductResponse = ctx.server
        .post("/api/products", &request)
        .await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());
    assert_product_draft(&created);

    // Submit for approval
    let response: ProductResponse = ctx.server
        .post(&format!("/api/products/{}/submit", product_id), &serde_json::json!({}))
        .await
        .expect("Failed to submit product");

    assert_product_status(&response, "PENDING_APPROVAL");

    // Verify via HTTP API (in-memory backend)
    let get_response: ProductResponse = ctx.server
        .get(&format!("/api/products/{}", product_id))
        .await
        .expect("Failed to get product via API");
    assert_product_status(&get_response, "PENDING_APPROVAL");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_approve_pending_to_active() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("approveprod");

    // Create and submit product
    let request = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    ctx.server.post::<_, ProductResponse>(
        &format!("/api/products/{}/submit", product_id),
        &serde_json::json!({})
    ).await.expect("Failed to submit product");

    // Approve
    let approval = ApprovalRequest {
        approved: true,
        comments: Some("Approved for production".to_string()),
    };
    let response: ProductResponse = ctx.server
        .post(&format!("/api/products/{}/approve", product_id), &approval)
        .await
        .expect("Failed to approve product");

    assert_product_status(&response, "ACTIVE");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_reject_pending_to_draft() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("rejectprod");

    // Create and submit product
    let request = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    ctx.server.post::<_, ProductResponse>(
        &format!("/api/products/{}/submit", product_id),
        &serde_json::json!({})
    ).await.expect("Failed to submit product");

    // Reject
    let response: ProductResponse = ctx.server
        .post(&format!("/api/products/{}/reject", product_id), &serde_json::json!({}))
        .await
        .expect("Failed to reject product");

    assert_product_status(&response, "DRAFT");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_discontinue_active_product() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("discontprod");

    // Create, submit, and approve product
    let request = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    ctx.server.post::<_, ProductResponse>(
        &format!("/api/products/{}/submit", product_id),
        &serde_json::json!({})
    ).await.expect("Failed to submit product");

    let approval = ApprovalRequest { approved: true, comments: None };
    ctx.server.post::<_, ProductResponse>(
        &format!("/api/products/{}/approve", product_id),
        &approval
    ).await.expect("Failed to approve product");

    // Discontinue
    let response: ProductResponse = ctx.server
        .post(&format!("/api/products/{}/discontinue", product_id), &serde_json::json!({}))
        .await
        .expect("Failed to discontinue product");

    assert_product_status(&response, "DISCONTINUED");

    ctx.cleanup().await.ok();
}

// =============================================================================
// Invalid Transition Tests
// =============================================================================

#[tokio::test]
async fn test_cannot_approve_draft_product() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("badapproveprod");

    // Create product (DRAFT status)
    let request = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Try to approve without submitting first
    let approval = ApprovalRequest { approved: true, comments: None };
    let response = ctx.server
        .post_response(&format!("/api/products/{}/approve", product_id), &approval)
        .await
        .expect("Request should complete");

    // Should fail - can't approve DRAFT product
    assert!(
        response.status() == StatusCode::PRECONDITION_FAILED ||
        response.status() == StatusCode::BAD_REQUEST ||
        response.status() == StatusCode::CONFLICT,
        "Expected error status, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_cannot_discontinue_draft_product() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("baddiscontprod");

    // Create product (DRAFT status)
    let request = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Try to discontinue DRAFT product
    let response = ctx.server
        .post_response(&format!("/api/products/{}/discontinue", product_id), &serde_json::json!({}))
        .await
        .expect("Request should complete");

    // Should fail
    assert!(
        !response.status().is_success(),
        "Expected error status, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_cannot_submit_active_product() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("badsubmitprod");

    // Create, submit, approve product
    let request = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    ctx.server.post::<_, ProductResponse>(
        &format!("/api/products/{}/submit", product_id),
        &serde_json::json!({})
    ).await.expect("Failed to submit product");

    let approval = ApprovalRequest { approved: true, comments: None };
    ctx.server.post::<_, ProductResponse>(
        &format!("/api/products/{}/approve", product_id),
        &approval
    ).await.expect("Failed to approve product");

    // Try to submit again (already ACTIVE)
    let response = ctx.server
        .post_response(&format!("/api/products/{}/submit", product_id), &serde_json::json!({}))
        .await
        .expect("Request should complete");

    // Should fail
    assert!(
        !response.status().is_success(),
        "Expected error status, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

// =============================================================================
// Update Restrictions Tests
// =============================================================================

#[tokio::test]
async fn test_cannot_update_non_draft_product() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("noupdateprod");

    // Create and submit product
    let request = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    ctx.server.post::<_, ProductResponse>(
        &format!("/api/products/{}/submit", product_id),
        &serde_json::json!({})
    ).await.expect("Failed to submit product");

    // Try to update PENDING_APPROVAL product
    let update = UpdateProductRequest {
        name: Some("Should Not Work".to_string()),
        description: None,
        effective_from: None,
        expiry_at: None,
    };

    let response = ctx.server
        .put_response(&format!("/api/products/{}", product_id), &update)
        .await
        .expect("Request should complete");

    // Should fail - can't update non-DRAFT product
    assert!(
        response.status() == StatusCode::PRECONDITION_FAILED ||
        response.status() == StatusCode::BAD_REQUEST ||
        response.status() == StatusCode::CONFLICT,
        "Expected error status, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_cannot_delete_non_draft_product() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("nodelprod");

    // Create, submit, approve product
    let request = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    ctx.server.post::<_, ProductResponse>(
        &format!("/api/products/{}/submit", product_id),
        &serde_json::json!({})
    ).await.expect("Failed to submit product");

    let approval = ApprovalRequest { approved: true, comments: None };
    ctx.server.post::<_, ProductResponse>(
        &format!("/api/products/{}/approve", product_id),
        &approval
    ).await.expect("Failed to approve product");

    // Try to delete ACTIVE product
    let response = ctx.server
        .delete_response(&format!("/api/products/{}", product_id))
        .await
        .expect("Request should complete");

    // Should fail
    assert!(
        response.status() == StatusCode::PRECONDITION_FAILED ||
        response.status() == StatusCode::BAD_REQUEST ||
        response.status() == StatusCode::CONFLICT,
        "Expected error status, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

// =============================================================================
// Full Lifecycle Test
// =============================================================================

#[tokio::test]
async fn test_full_product_lifecycle() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("lifecycleprod");

    // Step 1: Create (DRAFT)
    let request = ProductBuilder::new(&product_id)
        .with_name("Lifecycle Test Product")
        .build();
    let created: ProductResponse = ctx.server
        .post("/api/products", &request)
        .await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());
    assert_product_draft(&created);
    assert_product_version(&created, 1);

    // Step 2: Update while DRAFT (should work)
    let update = UpdateProductRequest {
        name: Some("Updated Lifecycle Product".to_string()),
        description: Some("Ready for review".to_string()),
        effective_from: None,
        expiry_at: None,
    };
    let updated: ProductResponse = ctx.server
        .put(&format!("/api/products/{}", product_id), &update)
        .await
        .expect("Failed to update product");
    assert_eq!(updated.name, "Updated Lifecycle Product");

    // Step 3: Submit (DRAFT -> PENDING_APPROVAL)
    let submitted: ProductResponse = ctx.server
        .post(&format!("/api/products/{}/submit", product_id), &serde_json::json!({}))
        .await
        .expect("Failed to submit product");
    assert_product_status(&submitted, "PENDING_APPROVAL");

    // Step 4: Reject (PENDING_APPROVAL -> DRAFT)
    let rejected: ProductResponse = ctx.server
        .post(&format!("/api/products/{}/reject", product_id), &serde_json::json!({}))
        .await
        .expect("Failed to reject product");
    assert_product_draft(&rejected);

    // Step 5: Update after rejection (should work - back to DRAFT)
    let update2 = UpdateProductRequest {
        description: Some("Fixed issues, resubmitting".to_string()),
        name: None,
        effective_from: None,
        expiry_at: None,
    };
    ctx.server.put::<_, ProductResponse>(&format!("/api/products/{}", product_id), &update2)
        .await
        .expect("Should be able to update after rejection");

    // Step 6: Submit again (DRAFT -> PENDING_APPROVAL)
    ctx.server.post::<_, ProductResponse>(
        &format!("/api/products/{}/submit", product_id),
        &serde_json::json!({})
    ).await.expect("Failed to resubmit product");

    // Step 7: Approve (PENDING_APPROVAL -> ACTIVE)
    let approval = ApprovalRequest {
        approved: true,
        comments: Some("Looks good!".to_string()),
    };
    let approved: ProductResponse = ctx.server
        .post(&format!("/api/products/{}/approve", product_id), &approval)
        .await
        .expect("Failed to approve product");
    assert_product_status(&approved, "ACTIVE");

    // Step 8: Discontinue (ACTIVE -> DISCONTINUED)
    let discontinued: ProductResponse = ctx.server
        .post(&format!("/api/products/{}/discontinue", product_id), &serde_json::json!({}))
        .await
        .expect("Failed to discontinue product");
    assert_product_status(&discontinued, "DISCONTINUED");

    ctx.cleanup().await.ok();
}

// =============================================================================
// APPROVE ENDPOINT - REJECTION PATH
// =============================================================================

#[tokio::test]
async fn test_approve_with_rejection_flag() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("approverejprod");

    // Create and submit product
    let request = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    ctx.server.post::<_, ProductResponse>(
        &format!("/api/products/{}/submit", product_id),
        &serde_json::json!({})
    ).await.expect("Failed to submit product");

    // Use approve endpoint with approved: false (rejection via approve)
    let approval = ApprovalRequest {
        approved: false,
        comments: Some("Rejected via approve endpoint".to_string()),
    };
    let response: ProductResponse = ctx.server
        .post(&format!("/api/products/{}/approve", product_id), &approval)
        .await
        .expect("Failed to reject via approve endpoint");

    // Should be back to DRAFT
    assert_product_draft(&response);

    ctx.cleanup().await.ok();
}

// =============================================================================
// PRODUCT TEMPLATES
// =============================================================================

#[tokio::test]
async fn test_list_templates() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // List templates
    let templates: Vec<ProductTemplateResponse> = ctx.server
        .get("/api/product-templates")
        .await
        .expect("Failed to list templates");

    // Should have at least the base templates (insurance, loan, trading)
    assert!(templates.len() >= 3, "Expected at least 3 templates");

    // Check insurance template
    let insurance = templates.iter().find(|t| t.template_type == "insurance");
    assert!(insurance.is_some(), "Insurance template should exist");
    let insurance = insurance.unwrap();
    assert_eq!(insurance.name, "Insurance Product");
    assert!(!insurance.components.is_empty(), "Insurance should have components");

    // Check loan template
    let loan = templates.iter().find(|t| t.template_type == "loan");
    assert!(loan.is_some(), "Loan template should exist");
    let loan = loan.unwrap();
    assert_eq!(loan.name, "Loan Product");

    // Check trading template
    let trading = templates.iter().find(|t| t.template_type == "trading");
    assert!(trading.is_some(), "Trading template should exist");

    ctx.server.shutdown().await;
}
