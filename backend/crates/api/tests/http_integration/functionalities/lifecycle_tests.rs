//! Functionality Lifecycle Tests
//!
//! Tests status transitions for functionalities:
//! - DRAFT → PENDING_APPROVAL (submit)
//! - PENDING_APPROVAL → ACTIVE (approve)
//! - PENDING_APPROVAL → DRAFT (reject)
//! - activate/deactivate transitions

use crate::fixtures::*;
use serde_json::json;

/// Helper to create a product and functionality for lifecycle tests
async fn setup_product_with_functionality(ctx: &TestContext, func_name: &str) -> (String, String) {
    let product_id = ctx.unique_id("product");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Lifecycle Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create functionality
    let func_req = json!({
        "name": func_name,
        "description": "Lifecycle test functionality"
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    (product_id, func_name.to_string())
}

/// Test submitting a DRAFT functionality for approval
#[tokio::test]
async fn test_submit_functionality_draft_to_pending() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let (product_id, func_name) = setup_product_with_functionality(&ctx, "submit-test").await;

    // Verify initial status is DRAFT
    let initial: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/{}", product_id, func_name),
    )
    .await
    .expect("Failed to get functionality");
    assert_eq!(initial["status"], "DRAFT");

    // Submit for approval
    let response: serde_json::Value = ctx.post_empty(
        &format!("/api/products/{}/functionalities/{}/submit", product_id, func_name),
    )
    .await
    .expect("Failed to submit functionality");

    assert_eq!(response["status"], "PENDING_APPROVAL");
    assert_eq!(response["name"], func_name);

    // Verify persistence
    let after_submit: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/{}", product_id, func_name),
    )
    .await
    .expect("Failed to get functionality after submit");
    assert_eq!(after_submit["status"], "PENDING_APPROVAL");
}

/// Test approving a PENDING_APPROVAL functionality
#[tokio::test]
async fn test_approve_functionality_pending_to_active() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let (product_id, func_name) = setup_product_with_functionality(&ctx, "approve-test").await;

    // Submit first
    ctx.post_empty::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/{}/submit", product_id, func_name),
    )
    .await
    .expect("Failed to submit functionality");

    // Approve
    let approve_req = json!({ "approved": true });
    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/functionalities/{}/approve", product_id, func_name),
        &approve_req,
    )
    .await
    .expect("Failed to approve functionality");

    assert_eq!(response["status"], "ACTIVE");

    // Verify persistence
    let after_approve: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/{}", product_id, func_name),
    )
    .await
    .expect("Failed to get functionality after approve");
    assert_eq!(after_approve["status"], "ACTIVE");
}

/// Test rejecting a PENDING_APPROVAL functionality
#[tokio::test]
async fn test_reject_functionality_pending_to_draft() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let (product_id, func_name) = setup_product_with_functionality(&ctx, "reject-test").await;

    // Submit first
    ctx.post_empty::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/{}/submit", product_id, func_name),
    )
    .await
    .expect("Failed to submit functionality");

    // Reject
    let reject_req = json!({ "approved": false });
    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/functionalities/{}/approve", product_id, func_name),
        &reject_req,
    )
    .await
    .expect("Failed to reject functionality");

    assert_eq!(response["status"], "DRAFT");

    // Verify persistence
    let after_reject: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/{}", product_id, func_name),
    )
    .await
    .expect("Failed to get functionality after reject");
    assert_eq!(after_reject["status"], "DRAFT");
}

/// Test activating a functionality directly
#[tokio::test]
async fn test_activate_functionality() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let (product_id, func_name) = setup_product_with_functionality(&ctx, "activate-test").await;

    // Activate directly
    let response: serde_json::Value = ctx.post_empty(
        &format!("/api/products/{}/functionalities/{}/activate", product_id, func_name),
    )
    .await
    .expect("Failed to activate functionality");

    assert_eq!(response["status"], "ACTIVE");
}

/// Test deactivating a functionality
#[tokio::test]
async fn test_deactivate_functionality() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let (product_id, func_name) = setup_product_with_functionality(&ctx, "deactivate-test").await;

    // Activate first
    ctx.post_empty::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/{}/activate", product_id, func_name),
    )
    .await
    .expect("Failed to activate functionality");

    // Deactivate
    let response: serde_json::Value = ctx.post_empty(
        &format!("/api/products/{}/functionalities/{}/deactivate", product_id, func_name),
    )
    .await
    .expect("Failed to deactivate functionality");

    assert_eq!(response["status"], "DRAFT");
}

/// Test cannot submit non-DRAFT functionality
#[tokio::test]
async fn test_submit_non_draft_fails() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let (product_id, func_name) = setup_product_with_functionality(&ctx, "submit-fail-test").await;

    // Submit once (DRAFT → PENDING_APPROVAL)
    ctx.post_empty::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/{}/submit", product_id, func_name),
    )
    .await
    .expect("Failed to submit functionality");

    // Try to submit again (should fail - already PENDING_APPROVAL)
    let result = ctx.post_empty::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/{}/submit", product_id, func_name),
    )
    .await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error.contains("412") || error.to_lowercase().contains("precondition") ||
        error.to_lowercase().contains("only draft"),
        "Expected precondition error but got: {}",
        error
    );
}

/// Test cannot approve DRAFT functionality directly
#[tokio::test]
async fn test_approve_draft_fails() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let (product_id, func_name) = setup_product_with_functionality(&ctx, "approve-draft-test").await;

    // Try to approve without submitting first
    let approve_req = json!({ "approved": true });
    let result = ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/{}/approve", product_id, func_name),
        &approve_req,
    )
    .await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error.contains("412") || error.to_lowercase().contains("precondition") ||
        error.to_lowercase().contains("only pending"),
        "Expected precondition error but got: {}",
        error
    );
}

/// Test cannot approve ACTIVE functionality
#[tokio::test]
async fn test_approve_active_fails() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let (product_id, func_name) = setup_product_with_functionality(&ctx, "approve-active-test").await;

    // Submit and approve first
    ctx.post_empty::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/{}/submit", product_id, func_name),
    )
    .await
    .expect("Failed to submit functionality");

    let approve_req = json!({ "approved": true });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/{}/approve", product_id, func_name),
        &approve_req,
    )
    .await
    .expect("Failed to approve functionality");

    // Try to approve again (should fail - already ACTIVE)
    let result = ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/{}/approve", product_id, func_name),
        &approve_req,
    )
    .await;

    assert!(result.is_err());
}

/// Test lifecycle: full cycle from DRAFT to ACTIVE and back
#[tokio::test]
async fn test_full_lifecycle_cycle() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let (product_id, func_name) = setup_product_with_functionality(&ctx, "full-cycle").await;

    // 1. Initial state: DRAFT
    let initial: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/{}", product_id, func_name),
    )
    .await
    .unwrap();
    assert_eq!(initial["status"], "DRAFT", "Step 1: Initial should be DRAFT");

    // 2. Submit: DRAFT → PENDING_APPROVAL
    ctx.post_empty::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/{}/submit", product_id, func_name),
    )
    .await
    .unwrap();
    let after_submit: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/{}", product_id, func_name),
    )
    .await
    .unwrap();
    assert_eq!(after_submit["status"], "PENDING_APPROVAL", "Step 2: After submit should be PENDING_APPROVAL");

    // 3. Reject: PENDING_APPROVAL → DRAFT
    let reject_req = json!({ "approved": false });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/{}/approve", product_id, func_name),
        &reject_req,
    )
    .await
    .unwrap();
    let after_reject: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/{}", product_id, func_name),
    )
    .await
    .unwrap();
    assert_eq!(after_reject["status"], "DRAFT", "Step 3: After reject should be DRAFT");

    // 4. Submit again: DRAFT → PENDING_APPROVAL
    ctx.post_empty::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/{}/submit", product_id, func_name),
    )
    .await
    .unwrap();

    // 5. Approve: PENDING_APPROVAL → ACTIVE
    let approve_req = json!({ "approved": true });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/{}/approve", product_id, func_name),
        &approve_req,
    )
    .await
    .unwrap();
    let after_approve: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/{}", product_id, func_name),
    )
    .await
    .unwrap();
    assert_eq!(after_approve["status"], "ACTIVE", "Step 5: After approve should be ACTIVE");

    // 6. Deactivate: ACTIVE → DRAFT
    ctx.post_empty::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/{}/deactivate", product_id, func_name),
    )
    .await
    .unwrap();
    let after_deactivate: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/{}", product_id, func_name),
    )
    .await
    .unwrap();
    assert_eq!(after_deactivate["status"], "DRAFT", "Step 6: After deactivate should be DRAFT");
}

/// Test submit non-existent functionality returns 404
#[tokio::test]
async fn test_submit_functionality_not_found() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product only
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Try to submit non-existent
    let result = ctx.post_empty::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/non-existent/submit", product_id),
    )
    .await;

    assert!(result.is_err());
}

/// Test activate non-existent functionality returns 404
#[tokio::test]
async fn test_activate_functionality_not_found() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product only
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Try to activate non-existent
    let result = ctx.post_empty::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/non-existent/activate", product_id),
    )
    .await;

    assert!(result.is_err());
}
