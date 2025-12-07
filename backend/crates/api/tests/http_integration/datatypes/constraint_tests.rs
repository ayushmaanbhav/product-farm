//! Datatype Constraint Tests

use crate::fixtures::{data_builders::DatatypeBuilder, TestContext};
use product_farm_api::rest::types::DatatypeResponse;

#[tokio::test]
async fn test_numeric_constraints_min_max() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("numconstrdt");

    let request = DatatypeBuilder::decimal(&datatype_id)
        .with_min_max(-1000.0, 1000.0)
        .build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_eq!(response.constraints.min, Some(-1000.0));
    assert_eq!(response.constraints.max, Some(1000.0));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_decimal_precision_scale() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("precisiondt");

    let request = DatatypeBuilder::decimal(&datatype_id)
        .with_precision(18, 4)
        .build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_eq!(response.constraints.precision, Some(18));
    assert_eq!(response.constraints.scale, Some(4));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_string_length_constraints() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("lengthdt");

    let request = DatatypeBuilder::string(&datatype_id)
        .with_length(5, 50)
        .build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_eq!(response.constraints.min_length, Some(5));
    assert_eq!(response.constraints.max_length, Some(50));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_regex_pattern_constraint() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("regexdt");

    // Email pattern
    let request = DatatypeBuilder::string(&datatype_id)
        .with_pattern(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert!(response.constraints.pattern.is_some());

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_combined_constraints() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("combineddt");

    let request = DatatypeBuilder::decimal(&datatype_id)
        .with_min_max(0.0, 999999.99)
        .with_precision(8, 2)
        .with_description("Currency amount")
        .build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_eq!(response.constraints.min, Some(0.0));
    assert_eq!(response.constraints.max, Some(999999.99));
    assert_eq!(response.constraints.precision, Some(8));
    assert_eq!(response.constraints.scale, Some(2));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_zero_constraints() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("zerodt");

    let request = DatatypeBuilder::decimal(&datatype_id)
        .with_min_max(0.0, 0.0) // Zero range
        .build();

    // This might be valid or invalid depending on implementation
    let response = ctx.server
        .post_response("/api/datatypes", &request)
        .await
        .expect("Request should complete");

    // Document actual behavior
    println!("Zero range constraint response: {}", response.status());

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_negative_constraints() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("negativedt");

    let request = DatatypeBuilder::decimal(&datatype_id)
        .with_min_max(-1000000.0, -1.0) // Negative range
        .build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_eq!(response.constraints.min, Some(-1000000.0));
    assert_eq!(response.constraints.max, Some(-1.0));

    ctx.cleanup().await.ok();
}
