//! Concrete Attribute Constraint Tests
//!
//! Tests for datatype constraint validation on concrete attribute values.

use crate::fixtures::{
    data_builders::{
        values, AbstractAttributeBuilder, ConcreteAttributeBuilder, DatatypeBuilder,
        ProductBuilder,
    },
    TestContext,
};
use product_farm_api::rest::types::{
    AbstractAttributeResponse, AttributeResponse, DatatypeResponse, ProductResponse,
};
use reqwest::StatusCode;

// =============================================================================
// Numeric Range Constraint Tests
// =============================================================================

#[tokio::test]
async fn test_value_within_min_max_range() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("range_ok_prod");
    let datatype_id = ctx.unique_id("range_ok_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Datatype with min/max constraints
    let dt_req = DatatypeBuilder::decimal(&datatype_id)
        .with_min_max(0.0, 100.0)
        .build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("calc", "percentage")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Value within range
    let concrete = ConcreteAttributeBuilder::new("calc", "main", "percentage")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("50.0"))
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Value within range should be accepted");

    assert!(response.value.is_some());

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_value_below_min_rejected() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("below_min_prod");
    let datatype_id = ctx.unique_id("below_min_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id)
        .with_min_max(10.0, 100.0)
        .build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("calc", "value")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Value below minimum
    let concrete = ConcreteAttributeBuilder::new("calc", "main", "value")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("5.0")) // Below min of 10
        .build();

    let response = ctx.server
        .post_response(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_value_above_max_rejected() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("above_max_prod");
    let datatype_id = ctx.unique_id("above_max_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id)
        .with_min_max(0.0, 100.0)
        .build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("calc", "value")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Value above maximum
    let concrete = ConcreteAttributeBuilder::new("calc", "main", "value")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("150.0")) // Above max of 100
        .build();

    let response = ctx.server
        .post_response(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_value_at_boundary() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("boundary_prod");
    let datatype_id = ctx.unique_id("boundary_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id)
        .with_min_max(0.0, 100.0)
        .build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    // Test at minimum boundary
    let abstract_min = AbstractAttributeBuilder::new("calc", "at_min")
        .with_datatype(&datatype_id)
        .build();
    let min_abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_min
    ).await.expect("Failed to create abstract attribute");

    let concrete_min = ConcreteAttributeBuilder::new("calc", "main", "at_min")
        .with_abstract_path(&min_abstract_resp.abstract_path)
        .fixed_value(values::decimal("0.0")) // At min
        .build();

    let min_response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete_min)
        .await
        .expect("Value at min boundary should be accepted");
    assert!(min_response.value.is_some());

    // Test at maximum boundary
    let abstract_max = AbstractAttributeBuilder::new("calc", "at_max")
        .with_datatype(&datatype_id)
        .build();
    let max_abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_max
    ).await.expect("Failed to create abstract attribute");

    let concrete_max = ConcreteAttributeBuilder::new("calc", "main", "at_max")
        .with_abstract_path(&max_abstract_resp.abstract_path)
        .fixed_value(values::decimal("100.0")) // At max
        .build();

    let max_response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete_max)
        .await
        .expect("Value at max boundary should be accepted");
    assert!(max_response.value.is_some());

    ctx.cleanup().await.ok();
}

// =============================================================================
// String Length Constraint Tests
// =============================================================================

#[tokio::test]
async fn test_string_within_length_limits() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("str_len_ok_prod");
    let datatype_id = ctx.unique_id("str_len_ok_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::string(&datatype_id)
        .with_length(5, 20)
        .build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("data", "code")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // String within limits
    let concrete = ConcreteAttributeBuilder::new("data", "main", "code")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::string("VALID123")) // 8 chars, within 5-20
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("String within limits should be accepted");

    assert!(response.value.is_some());

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_string_too_short_rejected() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("str_short_prod");
    let datatype_id = ctx.unique_id("str_short_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::string(&datatype_id)
        .with_length(5, 20)
        .build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("data", "code")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // String too short
    let concrete = ConcreteAttributeBuilder::new("data", "main", "code")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::string("AB")) // 2 chars, below min of 5
        .build();

    let response = ctx.server
        .post_response(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_string_too_long_rejected() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("str_long_prod");
    let datatype_id = ctx.unique_id("str_long_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::string(&datatype_id)
        .with_length(1, 10)
        .build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("data", "short_code")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // String too long
    let concrete = ConcreteAttributeBuilder::new("data", "main", "short_code")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::string("THIS_IS_WAY_TOO_LONG")) // 20 chars, above max of 10
        .build();

    let response = ctx.server
        .post_response(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Pattern Constraint Tests
// =============================================================================

#[tokio::test]
async fn test_string_matches_pattern() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("pattern_ok_prod");
    let datatype_id = ctx.unique_id("pattern_ok_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Pattern for product code: 2 letters followed by 4 digits
    let dt_req = DatatypeBuilder::string(&datatype_id)
        .with_pattern("^[A-Z]{2}[0-9]{4}$")
        .build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("product", "code")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Value matching pattern
    let concrete = ConcreteAttributeBuilder::new("product", "main", "code")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::string("AB1234"))
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Matching pattern should be accepted");

    assert!(response.value.is_some());

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_string_not_matching_pattern_rejected() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("pattern_bad_prod");
    let datatype_id = ctx.unique_id("pattern_bad_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::string(&datatype_id)
        .with_pattern("^[A-Z]{2}[0-9]{4}$")
        .build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("product", "code")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Value NOT matching pattern
    let concrete = ConcreteAttributeBuilder::new("product", "main", "code")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::string("invalid"))
        .build();

    let response = ctx.server
        .post_response(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Precision/Scale Constraint Tests
// =============================================================================

#[tokio::test]
async fn test_decimal_within_precision_scale() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("prec_ok_prod");
    let datatype_id = ctx.unique_id("prec_ok_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id)
        .with_precision(8, 2) // 8 total digits, 2 after decimal
        .build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("finance", "amount")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Value within precision/scale
    let concrete = ConcreteAttributeBuilder::new("finance", "main", "amount")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("123456.78")) // 8 digits, 2 after decimal
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Value within precision should be accepted");

    assert!(response.value.is_some());

    ctx.cleanup().await.ok();
}

// =============================================================================
// Update with Constraint Validation Tests
// =============================================================================

#[tokio::test]
async fn test_update_value_violates_constraint() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("upd_constr_prod");
    let datatype_id = ctx.unique_id("upd_constr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    let dt_req = DatatypeBuilder::decimal(&datatype_id)
        .with_min_max(0.0, 100.0)
        .build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("calc", "bounded")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Create with valid value
    let concrete = ConcreteAttributeBuilder::new("calc", "main", "bounded")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("50.0"))
        .build();
    let created: AttributeResponse = ctx.server.post(
        &format!("/api/products/{}/attributes", product_id),
        &concrete
    ).await.expect("Failed to create attribute");

    // Try to update with invalid value
    let update = product_farm_api::rest::types::UpdateAttributeRequest {
        value: Some(values::decimal("200.0")), // Above max of 100
        rule_id: None,
    };

    let encoded = urlencoding::encode(&created.path);
    let response = ctx.server
        .put_response(&format!("/api/attributes/{}", encoded), &update)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

// =============================================================================
// No Constraint Tests
// =============================================================================

#[tokio::test]
async fn test_attribute_without_constraints() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Setup
    let product_id = ctx.unique_product_id("no_constr_prod");
    let datatype_id = ctx.unique_id("no_constr_dt");

    let product_req = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &product_req).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Datatype without constraints
    let dt_req = DatatypeBuilder::decimal(&datatype_id).build();
    ctx.server.post::<_, DatatypeResponse>("/api/datatypes", &dt_req).await
        .expect("Failed to create datatype");

    let abstract_attr = AbstractAttributeBuilder::new("calc", "unconstrained")
        .with_datatype(&datatype_id)
        .build();
    let abstract_resp: AbstractAttributeResponse = ctx.server.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &abstract_attr
    ).await.expect("Failed to create abstract attribute");

    // Any value should be accepted
    let concrete = ConcreteAttributeBuilder::new("calc", "main", "unconstrained")
        .with_abstract_path(&abstract_resp.abstract_path)
        .fixed_value(values::decimal("999999999.999999"))
        .build();

    let response: AttributeResponse = ctx.server
        .post(&format!("/api/products/{}/attributes", product_id), &concrete)
        .await
        .expect("Unconstrained value should be accepted");

    assert!(response.value.is_some());

    ctx.cleanup().await.ok();
}
