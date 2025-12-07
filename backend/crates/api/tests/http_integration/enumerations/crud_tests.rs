//! Enumeration CRUD Tests

use crate::fixtures::{
    assertions::*, data_builders::EnumerationBuilder, TestContext,
};
use product_farm_api::rest::types::{
    CreateEnumerationRequest, DeleteResponse, EnumerationResponse, ListEnumerationsResponse,
    UpdateEnumerationRequest,
};
use reqwest::StatusCode;

// =============================================================================
// CREATE Tests
// =============================================================================

#[tokio::test]
async fn test_create_enumeration_basic() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("payment_method");

    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["CASH", "CREDIT", "DEBIT"])
        .build();

    let response: EnumerationResponse = ctx.server
        .post("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    assert_eq!(response.name, enum_name);
    assert_eq!(response.template_type, "insurance");
    assert_enumeration_contains(&response, "CASH");
    assert_enumeration_contains(&response, "CREDIT");
    assert_enumeration_contains(&response, "DEBIT");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_enumeration_with_description() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("status_enum");

    let request = EnumerationBuilder::new(&enum_name)
        .with_template("loan")
        .with_values(vec!["ACTIVE", "INACTIVE", "PENDING"])
        .with_description("Status enumeration for loan products")
        .build();

    let response: EnumerationResponse = ctx.server
        .post("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    assert_eq!(response.description, Some("Status enumeration for loan products".to_string()));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_enumeration_single_value() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("single_val_enum");

    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["ONLY_VALUE"])
        .build();

    let response: EnumerationResponse = ctx.server
        .post("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    assert_enumeration_value_count(&response, 1);
    assert_enumeration_contains(&response, "ONLY_VALUE");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_enumeration_many_values() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("many_vals_enum");

    let values: Vec<&str> = (0..20).map(|i| {
        // Create static strings for values
        match i {
            0 => "VALUE_0", 1 => "VALUE_1", 2 => "VALUE_2", 3 => "VALUE_3", 4 => "VALUE_4",
            5 => "VALUE_5", 6 => "VALUE_6", 7 => "VALUE_7", 8 => "VALUE_8", 9 => "VALUE_9",
            10 => "VALUE_10", 11 => "VALUE_11", 12 => "VALUE_12", 13 => "VALUE_13", 14 => "VALUE_14",
            15 => "VALUE_15", 16 => "VALUE_16", 17 => "VALUE_17", 18 => "VALUE_18", _ => "VALUE_19",
        }
    }).collect();

    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(values.clone())
        .build();

    let response: EnumerationResponse = ctx.server
        .post("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    assert_enumeration_value_count(&response, 20);

    ctx.cleanup().await.ok();
}

// =============================================================================
// READ Tests
// =============================================================================

#[tokio::test]
async fn test_get_enumeration() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("get_enum");

    // Create enumeration
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["A", "B", "C"])
        .build();
    ctx.server.post::<_, EnumerationResponse>("/api/template-enumerations", &request).await
        .expect("Failed to create enumeration");

    // Get enumeration
    let response: EnumerationResponse = ctx.server
        .get(&format!("/api/template-enumerations/{}", enum_name))
        .await
        .expect("Failed to get enumeration");

    assert_eq!(response.name, enum_name);
    assert_enumeration_value_count(&response, 3);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_get_nonexistent_enumeration() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let response = ctx.server
        .get_response("/api/template-enumerations/nonexistent_enum_xyz")
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_list_enumerations() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // Create a few enumerations
    for i in 0..3 {
        let enum_name = ctx.unique_id(&format!("list_enum_{}", i));
        let request = EnumerationBuilder::new(&enum_name)
            .with_template("insurance")
            .with_values(vec!["X", "Y"])
            .build();
        ctx.server.post::<_, EnumerationResponse>("/api/template-enumerations", &request).await
            .expect("Failed to create enumeration");
    }

    // List enumerations
    let response: ListEnumerationsResponse = ctx.server
        .get("/api/template-enumerations?pageSize=100")
        .await
        .expect("Failed to list enumerations");

    assert!(response.items.len() >= 3);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_list_enumerations_with_pagination() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // Create 5 enumerations
    for i in 0..5 {
        let enum_name = ctx.unique_id(&format!("page_enum_{}", i));
        let request = EnumerationBuilder::new(&enum_name)
            .with_template("loan")
            .with_values(vec!["VAL"])
            .build();
        ctx.server.post::<_, EnumerationResponse>("/api/template-enumerations", &request).await
            .expect("Failed to create enumeration");
    }

    // List with page size 2
    let response: ListEnumerationsResponse = ctx.server
        .get("/api/template-enumerations?pageSize=2")
        .await
        .expect("Failed to list enumerations");

    assert!(response.items.len() <= 2);

    ctx.cleanup().await.ok();
}

// =============================================================================
// UPDATE Tests
// =============================================================================

#[tokio::test]
async fn test_update_enumeration_description() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("update_enum");

    // Create enumeration
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["P", "Q"])
        .build();
    ctx.server.post::<_, EnumerationResponse>("/api/template-enumerations", &request).await
        .expect("Failed to create enumeration");

    // Update description
    let update = UpdateEnumerationRequest {
        description: Some("Updated description".to_string()),
    };

    let response: EnumerationResponse = ctx.server
        .put(&format!("/api/template-enumerations/{}", enum_name), &update)
        .await
        .expect("Failed to update enumeration");

    assert_eq!(response.description, Some("Updated description".to_string()));
    // Values should remain unchanged
    assert_enumeration_value_count(&response, 2);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_update_nonexistent_enumeration() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let update = UpdateEnumerationRequest {
        description: Some("Some description".to_string()),
    };

    let response = ctx.server
        .put_response("/api/template-enumerations/nonexistent_enum", &update)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

// =============================================================================
// DELETE Tests
// =============================================================================

#[tokio::test]
async fn test_delete_unused_enumeration() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("delete_enum");

    // Create enumeration
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["DEL1", "DEL2"])
        .build();
    ctx.server.post::<_, EnumerationResponse>("/api/template-enumerations", &request).await
        .expect("Failed to create enumeration");

    // Delete enumeration
    let response: DeleteResponse = ctx.server
        .delete(&format!("/api/template-enumerations/{}", enum_name))
        .await
        .expect("Failed to delete enumeration");

    assert!(response.success);

    // Verify deletion
    let get_response = ctx.server
        .get_response(&format!("/api/template-enumerations/{}", enum_name))
        .await
        .expect("Request should complete");
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_delete_nonexistent_enumeration() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let response = ctx.server
        .delete_response("/api/template-enumerations/nonexistent_enum_xyz")
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Validation Tests
// =============================================================================

#[tokio::test]
async fn test_create_enumeration_empty_values() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("empty_vals_enum");

    let request = CreateEnumerationRequest {
        name: enum_name,
        template_type: "insurance".to_string(),
        values: vec![],
        description: None,
    };

    let response = ctx.server
        .post_response("/api/template-enumerations", &request)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_enumeration_empty_name() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let request = CreateEnumerationRequest {
        name: "".to_string(),
        template_type: "insurance".to_string(),
        values: vec!["A".to_string()],
        description: None,
    };

    let response = ctx.server
        .post_response("/api/template-enumerations", &request)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_enumeration_empty_template() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("no_template_enum");

    let request = CreateEnumerationRequest {
        name: enum_name,
        template_type: "".to_string(),
        values: vec!["A".to_string()],
        description: None,
    };

    let response = ctx.server
        .post_response("/api/template-enumerations", &request)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_duplicate_enumeration() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("dup_enum");

    // Create first enumeration
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["A"])
        .build();
    ctx.server.post::<_, EnumerationResponse>("/api/template-enumerations", &request).await
        .expect("Failed to create first enumeration");

    // Try to create duplicate
    let duplicate = EnumerationBuilder::new(&enum_name)
        .with_template("loan") // Different template, same name
        .with_values(vec!["B"])
        .build();

    let response = ctx.server
        .post_response("/api/template-enumerations", &duplicate)
        .await
        .expect("Request should complete");

    assert!(
        response.status() == StatusCode::CONFLICT || response.status() == StatusCode::BAD_REQUEST,
        "Expected CONFLICT or BAD_REQUEST, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

// =============================================================================
// CRUD Cycle Test
// =============================================================================

#[tokio::test]
async fn test_enumeration_crud_cycle() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("crud_cycle_enum");

    // CREATE
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["INITIAL"])
        .with_description("Initial description")
        .build();
    let created: EnumerationResponse = ctx.server
        .post("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");
    assert_eq!(created.name, enum_name);

    // READ
    let read: EnumerationResponse = ctx.server
        .get(&format!("/api/template-enumerations/{}", enum_name))
        .await
        .expect("Failed to get enumeration");
    assert_eq!(read.name, enum_name);
    assert_eq!(read.description, Some("Initial description".to_string()));

    // UPDATE
    let update = UpdateEnumerationRequest {
        description: Some("Modified description".to_string()),
    };
    let updated: EnumerationResponse = ctx.server
        .put(&format!("/api/template-enumerations/{}", enum_name), &update)
        .await
        .expect("Failed to update enumeration");
    assert_eq!(updated.description, Some("Modified description".to_string()));

    // DELETE
    let deleted: DeleteResponse = ctx.server
        .delete(&format!("/api/template-enumerations/{}", enum_name))
        .await
        .expect("Failed to delete enumeration");
    assert!(deleted.success);

    // VERIFY DELETED
    let verify = ctx.server
        .get_response(&format!("/api/template-enumerations/{}", enum_name))
        .await
        .expect("Request should complete");
    assert_eq!(verify.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}
