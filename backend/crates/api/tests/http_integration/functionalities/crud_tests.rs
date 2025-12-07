//! Functionality CRUD Tests
//!
//! Tests basic Create, Read, Update, Delete operations for functionalities.

use crate::fixtures::*;
use serde_json::json;

/// Test creating a functionality with all fields
#[tokio::test]
async fn test_create_functionality_with_all_fields() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_id("product");
    let datatype_id = ctx.unique_id("decimal");

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // Create product first
    let product_req = json!({
        "id": product_id,
        "name": "Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>(&format!("/api/products"), &product_req)
        .await
        .expect("Failed to create product");

    // Create abstract attributes for required_attributes
    let attr_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "amount",
        "datatypeId": datatype_id,
        "displayNames": [{"name": "Amount", "format": "default", "orderIndex": 0}]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req,
    )
    .await
    .expect("Failed to create abstract attribute");

    // Create functionality with all fields
    let func_req = json!({
        "name": "disbursement",
        "description": "Loan disbursement functionality",
        "immutable": false,
        "requiredAttributes": [
            {
                "abstractPath": "loan.main.amount",
                "description": "The disbursement amount"
            }
        ]
    });

    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Verify response fields
    assert_eq!(response["name"], "disbursement");
    assert_eq!(response["productId"], product_id);
    assert_eq!(response["description"], "Loan disbursement functionality");
    assert_eq!(response["status"], "DRAFT");
    assert_eq!(response["immutable"], false);
    assert_eq!(response["requiredAttributes"].as_array().unwrap().len(), 1);
    assert_eq!(
        response["requiredAttributes"][0]["abstractPath"],
        "loan.main.amount"
    );
    assert!(response["createdAt"].as_i64().unwrap() > 0);
    assert!(response["updatedAt"].as_i64().unwrap() > 0);
}

/// Test creating a functionality with minimal fields
#[tokio::test]
async fn test_create_functionality_minimal_fields() {
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

    // Create functionality with only required fields
    let func_req = json!({
        "name": "simple-func",
        "description": "Simple functionality"
    });

    let response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    assert_eq!(response["name"], "simple-func");
    assert_eq!(response["description"], "Simple functionality");
    assert_eq!(response["status"], "DRAFT");
    assert_eq!(response["immutable"], false);
    assert!(response["requiredAttributes"].as_array().unwrap().is_empty());
}

/// Test getting a functionality by name
#[tokio::test]
async fn test_get_functionality_by_name() {
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

    // Create functionality
    let func_req = json!({
        "name": "get-test",
        "description": "Test get operation"
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Get functionality
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/get-test", product_id),
    )
    .await
    .expect("Failed to get functionality");

    assert_eq!(response["name"], "get-test");
    assert_eq!(response["description"], "Test get operation");
}

/// Test getting a non-existent functionality returns 404
#[tokio::test]
async fn test_get_functionality_not_found() {
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

    // Try to get non-existent functionality
    let result = ctx.get::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/non-existent", product_id),
    )
    .await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("404") || error.to_lowercase().contains("not found"));
}

/// Test listing functionalities for a product
#[tokio::test]
async fn test_list_functionalities() {
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

    // Create multiple functionalities
    for name in ["func-one", "func-two", "func-three"] {
        let func_req = json!({
            "name": name,
            "description": format!("Functionality {}", name)
        });
        ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/functionalities", product_id),
            &func_req,
        )
        .await
        .expect("Failed to create functionality");
    }

    // List functionalities
    let response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities", product_id),
    )
    .await
    .expect("Failed to list functionalities");

    let items = response["items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(response["totalCount"], 3);

    // Verify all are present
    let names: Vec<&str> = items.iter()
        .map(|i| i["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"func-one"));
    assert!(names.contains(&"func-two"));
    assert!(names.contains(&"func-three"));
}

/// Test listing functionalities for non-existent product returns 404
#[tokio::test]
async fn test_list_functionalities_product_not_found() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let result = ctx.get::<serde_json::Value>(
        "/api/products/non-existent-product/functionalities",
    )
    .await;

    assert!(result.is_err());
}

/// Test updating a functionality description
#[tokio::test]
async fn test_update_functionality_description() {
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

    // Create functionality
    let func_req = json!({
        "name": "update-test",
        "description": "Original description"
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Update description
    let update_req = json!({
        "description": "Updated description"
    });
    let response: serde_json::Value = ctx.put(
        &format!("/api/products/{}/functionalities/update-test", product_id),
        &update_req,
    )
    .await
    .expect("Failed to update functionality");

    assert_eq!(response["description"], "Updated description");

    // Verify persistence
    let get_response: serde_json::Value = ctx.get(
        &format!("/api/products/{}/functionalities/update-test", product_id),
    )
    .await
    .expect("Failed to get functionality");

    assert_eq!(get_response["description"], "Updated description");
}

/// Test updating functionality required attributes
#[tokio::test]
async fn test_update_functionality_required_attributes() {
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

    // Create functionality with initial required attributes
    let func_req = json!({
        "name": "update-attrs",
        "description": "Test update attrs",
        "requiredAttributes": [
            {
                "abstractPath": "loan.main.amount",
                "description": "Amount"
            }
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Update required attributes
    let update_req = json!({
        "requiredAttributes": [
            {
                "abstractPath": "loan.main.rate",
                "description": "Interest rate"
            },
            {
                "abstractPath": "loan.main.term",
                "description": "Loan term"
            }
        ]
    });
    let response: serde_json::Value = ctx.put(
        &format!("/api/products/{}/functionalities/update-attrs", product_id),
        &update_req,
    )
    .await
    .expect("Failed to update functionality");

    let attrs = response["requiredAttributes"].as_array().unwrap();
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0]["abstractPath"], "loan.main.rate");
    assert_eq!(attrs[1]["abstractPath"], "loan.main.term");

    // Verify order_index is assigned
    assert_eq!(attrs[0]["orderIndex"], 0);
    assert_eq!(attrs[1]["orderIndex"], 1);
}

/// Test deleting a functionality
#[tokio::test]
async fn test_delete_functionality() {
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

    // Create functionality
    let func_req = json!({
        "name": "to-delete",
        "description": "Will be deleted"
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // Delete functionality
    let delete_response: serde_json::Value = ctx.delete(
        &format!("/api/products/{}/functionalities/to-delete", product_id),
    )
    .await
    .expect("Failed to delete functionality");

    assert_eq!(delete_response["deleted"], true);

    // Verify it's gone
    let get_result = ctx.get::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/to-delete", product_id),
    )
    .await;

    assert!(get_result.is_err());
}

/// Test deleting a non-existent functionality returns 404
#[tokio::test]
async fn test_delete_functionality_not_found() {
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

    // Try to delete non-existent
    let result = ctx.delete::<serde_json::Value>(
        &format!("/api/products/{}/functionalities/non-existent", product_id),
    )
    .await;

    assert!(result.is_err());
}

/// Test duplicate functionality name returns conflict error
#[tokio::test]
async fn test_create_duplicate_functionality_returns_conflict() {
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

    // Create functionality
    let func_req = json!({
        "name": "duplicate",
        "description": "First one"
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create first functionality");

    // Try to create duplicate
    let result = ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("409") || error.to_lowercase().contains("already exists"));
}

/// Test functionality name validation (invalid characters)
#[tokio::test]
async fn test_create_functionality_invalid_name() {
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

    // Invalid names to test
    let invalid_names = [
        "UPPERCASE",           // uppercase not allowed
        "with spaces",         // spaces not allowed
        "with_underscore",     // underscore not allowed
        "with.dots",           // dots not allowed
        "",                    // empty not allowed
    ];

    for invalid_name in invalid_names {
        let func_req = json!({
            "name": invalid_name,
            "description": "Test"
        });

        let result = ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/functionalities", product_id),
            &func_req,
        )
        .await;

        assert!(
            result.is_err(),
            "Expected error for invalid name '{}' but got success",
            invalid_name
        );
    }
}

/// Test functionality with valid name patterns
#[tokio::test]
async fn test_create_functionality_valid_names() {
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

    // Valid names to test
    // Functionality names must be lowercase letters and hyphens only (no numbers)
    let valid_names = [
        "simple",
        "with-hyphen",
        "multi-word-name",
        "a",
        "abcdef",
    ];

    for valid_name in valid_names {
        let func_req = json!({
            "name": valid_name,
            "description": format!("Test {}", valid_name)
        });

        let result = ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/functionalities", product_id),
            &func_req,
        )
        .await;

        assert!(
            result.is_ok(),
            "Expected success for valid name '{}' but got error: {:?}",
            valid_name,
            result.err()
        );
    }
}
