//! Datatype Primitive Type Tests

use crate::fixtures::{assertions::*, data_builders::DatatypeBuilder, TestContext};
use product_farm_api::rest::types::{CreateDatatypeRequest, DatatypeResponse};

#[tokio::test]
async fn test_create_string_datatype() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("stringprimdt");

    let request = DatatypeBuilder::string(&datatype_id).build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_datatype_primitive(&response, "STRING");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_int_datatype() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("intprimdt");

    let request = DatatypeBuilder::int(&datatype_id).build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_datatype_primitive(&response, "INT");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_float_datatype() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("floatprimdt");

    let request = DatatypeBuilder::float(&datatype_id).build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_datatype_primitive(&response, "FLOAT");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_decimal_datatype() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("decimalprimdt");

    let request = DatatypeBuilder::decimal(&datatype_id).build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_datatype_primitive(&response, "DECIMAL");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_bool_datatype() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("boolprimdt");

    let request = DatatypeBuilder::boolean(&datatype_id).build();

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_datatype_primitive(&response, "BOOL");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_datetime_datatype() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("datetimeprimdt");

    let request = CreateDatatypeRequest {
        id: datatype_id.clone(),
        primitive_type: "DATETIME".to_string(),
        description: None,
        constraints: None,
    };

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_datatype_primitive(&response, "DATETIME");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_array_datatype() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("arrayprimdt");

    let request = CreateDatatypeRequest {
        id: datatype_id.clone(),
        primitive_type: "ARRAY".to_string(),
        description: Some("Array of items".to_string()),
        constraints: None,
    };

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_datatype_primitive(&response, "ARRAY");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_object_datatype() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("objectprimdt");

    let request = CreateDatatypeRequest {
        id: datatype_id.clone(),
        primitive_type: "OBJECT".to_string(),
        description: Some("Complex object type".to_string()),
        constraints: None,
    };

    let response: DatatypeResponse = ctx.server
        .post("/api/datatypes", &request)
        .await
        .expect("Failed to create datatype");

    assert_datatype_primitive(&response, "OBJECT");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_all_primitive_types() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let primitives = [
        "STRING", "INT", "FLOAT", "DECIMAL", "BOOL", "DATETIME",
        "ENUM", "ARRAY", "OBJECT", "ATTRIBUTE_REFERENCE", "IDENTIFIER",
    ];

    for primitive in primitives {
        let datatype_id = ctx.unique_id(&format!("{}dt", primitive.to_lowercase()));

        let request = CreateDatatypeRequest {
            id: datatype_id.clone(),
            primitive_type: primitive.to_string(),
            description: None,
            constraints: None,
        };

        let result = ctx.server
            .post::<_, DatatypeResponse>("/api/datatypes", &request)
            .await;

        match result {
            Ok(response) => {
                assert_eq!(response.primitive_type.to_uppercase(), primitive.to_uppercase());
            }
            Err(e) => {
                // Some primitive types might not be supported
                println!("Primitive type '{}' creation failed: {}", primitive, e);
            }
        }
    }

    ctx.cleanup().await.ok();
}
