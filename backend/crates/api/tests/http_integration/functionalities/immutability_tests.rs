//! Functionality Immutability Tests
//!
//! Tests that immutable functionalities cannot be modified or deleted.

use crate::fixtures::*;
use serde_json::json;

/// Test creating an immutable functionality
#[tokio::test]
async fn test_create_immutable_functionality() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create immutable functionality
    let func_req = json!({
        "name": "immutable-func",
        "description": "This is immutable",
        "immutable": true
    });

    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    assert_eq!(response["immutable"], true);
    assert_eq!(response["name"], "immutable-func");
}

/// Test cannot update immutable functionality
#[tokio::test]
async fn test_cannot_update_immutable_functionality() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create immutable functionality
    let func_req = json!({
        "name": "update-block",
        "description": "Original description",
        "immutable": true
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Try to update description
    let update_req = json!({
        "description": "New description"
    });
    let result = ctx.put::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/update-block", product_id),
        &update_req,
    )
    .await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error.contains("412") || error.to_lowercase().contains("precondition") ||
        error.to_lowercase().contains("immutable"),
        "Expected precondition error for immutable update but got: {}",
        error
    );

    // Verify description unchanged
    let get_response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/update-block", product_id),
    )
    .await
    .expect("Failed to get functionality");

    assert_eq!(get_response["description"], "Original description");
}

/// Test cannot delete immutable functionality
#[tokio::test]
async fn test_cannot_delete_immutable_functionality() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create immutable functionality
    let func_req = json!({
        "name": "delete-block",
        "description": "Cannot be deleted",
        "immutable": true
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Try to delete
    let result = ctx.delete::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/delete-block", product_id),
    )
    .await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        error.contains("412") || error.to_lowercase().contains("precondition") ||
        error.to_lowercase().contains("immutable"),
        "Expected precondition error for immutable delete but got: {}",
        error
    );

    // Verify still exists
    let get_response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/delete-block", product_id),
    )
    .await
    .expect("Functionality should still exist");

    assert_eq!(get_response["name"], "delete-block");
}

/// Test cannot update required attributes on immutable functionality
#[tokio::test]
async fn test_cannot_update_required_attributes_on_immutable() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create immutable functionality with required attributes
    let func_req = json!({
        "name": "immutable-attrs",
        "description": "Immutable with attrs",
        "immutable": true,
        "requiredAttributes": [
            { "abstractPath": "loan.main.amount", "description": "Amount" }
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Try to update required attributes
    let update_req = json!({
        "requiredAttributes": [
            { "abstractPath": "loan.main.rate", "description": "Rate" }
        ]
    });
    let result = ctx.put::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/immutable-attrs", product_id),
        &update_req,
    )
    .await;

    assert!(result.is_err());

    // Verify attributes unchanged
    let get_response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/immutable-attrs", product_id),
    )
    .await
    .expect("Failed to get functionality");

    let attrs = get_response["requiredAttributes"].as_array().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0]["abstractPath"], "loan.main.amount");
}

/// Test mutable functionality can be updated and deleted
#[tokio::test]
async fn test_mutable_functionality_can_be_modified() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create mutable functionality (default)
    let func_req = json!({
        "name": "mutable-func",
        "description": "Original"
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Update should work
    let update_req = json!({
        "description": "Updated"
    });
    let update_response: serde_json::Value = ctx.put(
        &format!("/api/products/{}/functionalities/mutable-func", product_id),
        &update_req,
    )
    .await
    .expect("Failed to update functionality");

    assert_eq!(update_response["description"], "Updated");

    // Delete should work
    let delete_response: serde_json::Value = ctx.delete(
        &format!("/api/products/{}/functionalities/mutable-func", product_id),
    )
    .await
    .expect("Failed to delete functionality");

    assert_eq!(delete_response["deleted"], true);
}

/// Test immutable defaults to false when not specified
#[tokio::test]
async fn test_immutable_defaults_to_false() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create functionality without specifying immutable
    let func_req = json!({
        "name": "default-immutable",
        "description": "Test default"
    });

    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    assert_eq!(response["immutable"], false, "immutable should default to false");
}

/// Test explicitly setting immutable to false
#[tokio::test]
async fn test_explicitly_mutable_functionality() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create functionality with immutable explicitly false
    let func_req = json!({
        "name": "explicit-mutable",
        "description": "Explicitly mutable",
        "immutable": false
    });

    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    assert_eq!(response["immutable"], false);

    // Should be updatable
    let update_req = json!({
        "description": "Updated description"
    });
    let update_response: serde_json::Value = ctx.put(
        &format!("/api/products/{}/functionalities/explicit-mutable", product_id),
        &update_req,
    )
    .await
    .expect("Failed to update functionality");

    assert_eq!(update_response["description"], "Updated description");
}

/// Test lifecycle transitions work for immutable functionality
#[tokio::test]
async fn test_immutable_functionality_lifecycle_still_works() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");

    // Create product
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create immutable functionality
    let func_req = json!({
        "name": "immutable-lifecycle",
        "description": "Immutable lifecycle test",
        "immutable": true
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Submit should work
    let submit_response: serde_json::Value = ctx.post_empty(
        &format!("/api/products/{}/functionalities/immutable-lifecycle/submit", product_id),
    )
    .await
    .expect("Failed to submit immutable functionality");

    assert_eq!(submit_response["status"], "PENDING_APPROVAL");

    // Approve should work
    let approve_req = json!({ "approved": true });
    let approve_response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/functionalities/immutable-lifecycle/approve", product_id),
        &approve_req,
    )
    .await
    .expect("Failed to approve immutable functionality");

    assert_eq!(approve_response["status"], "ACTIVE");

    // Activate/deactivate should work
    let deactivate_response: serde_json::Value = ctx.post_empty(
        &format!("/api/products/{}/functionalities/immutable-lifecycle/deactivate", product_id),
    )
    .await
    .expect("Failed to deactivate immutable functionality");

    assert_eq!(deactivate_response["status"], "DRAFT");

    let activate_response: serde_json::Value = ctx.post_empty(
        &format!("/api/products/{}/functionalities/immutable-lifecycle/activate", product_id),
    )
    .await
    .expect("Failed to activate immutable functionality");

    assert_eq!(activate_response["status"], "ACTIVE");
}
