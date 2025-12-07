//! Datatype CRUD Tests

use crate::fixtures::{
    assertions::*, data_builders::DatatypeBuilder, TestContext,
};
use product_farm_api::rest::types::{
    CreateDatatypeRequest, DeleteResponse, DatatypeResponse, ListDatatypesResponse,
    UpdateDatatypeRequest,
};
use reqwest::StatusCode;

// =============================================================================
// CREATE Tests
// =============================================================================

#[tokio::test]
async fn test_create_datatype_basic() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("testdt");

    let request = DatatypeBuilder::decimal(&datatype_id)
        .with_description("Test decimal type")
        .build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_eq!(response.id, datatype_id);
    assert_eq!(response.name, datatype_id);
    assert_datatype_primitive(&response, "DECIMAL");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_datatype_with_constraints() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("constraineddt");

    let request = DatatypeBuilder::decimal(&datatype_id)
        .with_min_max(0.0, 100.0)
        .with_precision(10, 2)
        .build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_datatype_range(&response, 0.0, 100.0);
    assert_datatype_precision(&response, 10, 2);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_string_datatype_with_length() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("stringdt");

    let request = DatatypeBuilder::string(&datatype_id)
        .with_length(1, 100)
        .build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_datatype_primitive(&response, "STRING");
    assert_eq!(response.constraints.min_length, Some(1));
    assert_eq!(response.constraints.max_length, Some(100));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_string_datatype_with_pattern() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("patterndt");

    let request = DatatypeBuilder::string(&datatype_id)
        .with_pattern("^[A-Z]{2}[0-9]{4}$")
        .build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_eq!(response.constraints.pattern, Some("^[A-Z]{2}[0-9]{4}$".to_string()));

    ctx.cleanup().await.ok();
}

// =============================================================================
// READ Tests
// =============================================================================

#[tokio::test]
async fn test_get_datatype() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("getdt");

    // Create datatype
    let request = DatatypeBuilder::int(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &request).await
        .expect("Failed to create datatype");

    // Get datatype
    let response: DatatypeResponse = ctx.server
        .get(&format!("/api/datatypes/{}", datatype_id))
        .await
        .expect("Failed to get datatype");

    assert_eq!(response.id, datatype_id);
    assert_datatype_primitive(&response, "INT");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_get_nonexistent_datatype() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let response = ctx.server
        .get_response("/api/datatypes/nonexistent_dt_xyz")
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_list_datatypes() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // Create a few datatypes
    for i in 0..3 {
        let datatype_id = ctx.unique_id(&format!("listdt{}", i));
        let request = DatatypeBuilder::string(&datatype_id).build();
        ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &request).await
            .expect("Failed to create datatype");
    }

    // List datatypes
    let response: ListDatatypesResponse = ctx.server
        .get("/api/datatypes?pageSize=100")
        .await
        .expect("Failed to list datatypes");

    assert!(response.items.len() >= 3);

    ctx.cleanup().await.ok();
}

// =============================================================================
// UPDATE Tests
// =============================================================================

#[tokio::test]
async fn test_update_datatype_description() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("updatedt");

    // Create datatype
    let request = DatatypeBuilder::float(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &request).await
        .expect("Failed to create datatype");

    // Update description
    let update = UpdateDatatypeRequest {
        description: Some("Updated description".to_string()),
        constraints: None,
    };

    let response: DatatypeResponse = ctx.server
        .put(&format!("/api/datatypes/{}", datatype_id), &update)
        .await
        .expect("Failed to update datatype");

    assert_eq!(response.description, Some("Updated description".to_string()));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_update_datatype_constraints() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("updateconstrdt");

    // Create datatype with initial constraints
    let request = DatatypeBuilder::decimal(&datatype_id)
        .with_min_max(0.0, 100.0)
        .build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &request).await
        .expect("Failed to create datatype");

    // Update constraints
    let update = UpdateDatatypeRequest {
        description: None,
        constraints: Some(product_farm_api::rest::types::DatatypeConstraintsJson {
            min: Some(10.0),
            max: Some(200.0),
            min_length: None,
            max_length: None,
            pattern: None,
            precision: None,
            scale: None,
            constraint_rule_expression: None,
            constraint_error_message: None,
        }),
    };

    let response: DatatypeResponse = ctx.server
        .put(&format!("/api/datatypes/{}", datatype_id), &update)
        .await
        .expect("Failed to update datatype");

    assert_datatype_range(&response, 10.0, 200.0);

    ctx.cleanup().await.ok();
}

// =============================================================================
// DELETE Tests
// =============================================================================

#[tokio::test]
async fn test_delete_unused_datatype() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("deletedt");

    // Create datatype
    let request = DatatypeBuilder::boolean(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &request).await
        .expect("Failed to create datatype");

    // Delete datatype
    let response: DeleteResponse = ctx.server
        .delete(&format!("/api/datatypes/{}", datatype_id))
        .await
        .expect("Failed to delete datatype");

    assert!(response.success);

    // Verify deletion
    let get_response = ctx.server
        .get_response(&format!("/api/datatypes/{}", datatype_id))
        .await
        .expect("Request should complete");
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Validation Tests
// =============================================================================

#[tokio::test]
async fn test_create_datatype_invalid_min_max() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("badrangedt");

    // min > max should fail
    let request = CreateDatatypeRequest {
        id: datatype_id,
        primitive_type: "DECIMAL".to_string(),
        description: None,
        constraints: Some(product_farm_api::rest::types::DatatypeConstraintsJson {
            min: Some(100.0),
            max: Some(50.0), // max < min
            min_length: None,
            max_length: None,
            pattern: None,
            precision: None,
            scale: None,
            constraint_rule_expression: None,
            constraint_error_message: None,
        }),
    };

    let response = ctx.server
        .post_response("/api/datatypes", &request)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_datatype_invalid_scale_precision() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("badprecisiondt");

    // scale > precision should fail
    let request = CreateDatatypeRequest {
        id: datatype_id,
        primitive_type: "DECIMAL".to_string(),
        description: None,
        constraints: Some(product_farm_api::rest::types::DatatypeConstraintsJson {
            min: None,
            max: None,
            min_length: None,
            max_length: None,
            pattern: None,
            precision: Some(5),
            scale: Some(10), // scale > precision
            constraint_rule_expression: None,
            constraint_error_message: None,
        }),
    };

    let response = ctx.server
        .post_response("/api/datatypes", &request)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_duplicate_datatype() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("dupdt");

    // Create first datatype
    let request = DatatypeBuilder::string(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &request).await
        .expect("Failed to create first datatype");

    // Try to create duplicate
    let duplicate = DatatypeBuilder::int(&datatype_id).build();
    let response = ctx.server
        .post_response("/api/datatypes", &duplicate)
        .await
        .expect("Request should complete");

    assert!(
        response.status() == StatusCode::CONFLICT || response.status() == StatusCode::BAD_REQUEST,
        "Expected CONFLICT or BAD_REQUEST, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}
