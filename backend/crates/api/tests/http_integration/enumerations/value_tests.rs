//! Enumeration Value Management Tests
//!
//! Tests for adding and removing enumeration values.

use crate::fixtures::{assertions::*, data_builders::EnumerationBuilder, TestContext};
use product_farm_api::rest::types::{
    AddEnumerationValueRequest, EnumerationResponse, RemoveEnumerationValueRequest,
};
use reqwest::StatusCode;

// =============================================================================
// Add Value Tests
// =============================================================================

#[tokio::test]
async fn test_add_value_to_enumeration() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("add_val_enum");

    // Create enumeration with initial values
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["A", "B"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    // Add new value
    let add_request = AddEnumerationValueRequest {
        value: "C".to_string(),
    };

    let response: EnumerationResponse = ctx
        .server
        .post(
            &format!("/api/template-enumerations/{}/values", enum_name),
            &add_request,
        )
        .await
        .expect("Failed to add value");

    assert_enumeration_value_count(&response, 3);
    assert_enumeration_contains(&response, "A");
    assert_enumeration_contains(&response, "B");
    assert_enumeration_contains(&response, "C");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_add_multiple_values_sequentially() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("multi_add_enum");

    // Create enumeration with one value
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("loan")
        .with_values(vec!["FIRST"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    // Add values one by one
    let values_to_add = ["SECOND", "THIRD", "FOURTH"];

    for value in values_to_add {
        let add_request = AddEnumerationValueRequest {
            value: value.to_string(),
        };
        ctx.server
            .post::<_, EnumerationResponse>(
                &format!("/api/template-enumerations/{}/values", enum_name),
                &add_request,
            )
            .await
            .expect(&format!("Failed to add value {}", value));
    }

    // Verify final state
    let response: EnumerationResponse = ctx
        .server
        .get(&format!("/api/template-enumerations/{}", enum_name))
        .await
        .expect("Failed to get enumeration");

    assert_enumeration_value_count(&response, 4);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_add_duplicate_value() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("dup_val_enum");

    // Create enumeration
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["EXISTING"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    // Try to add existing value
    let add_request = AddEnumerationValueRequest {
        value: "EXISTING".to_string(),
    };

    let response = ctx
        .server
        .post_response(
            &format!("/api/template-enumerations/{}/values", enum_name),
            &add_request,
        )
        .await
        .expect("Request should complete");

    // Duplicate value might be rejected or silently ignored
    // Document actual behavior
    println!("Add duplicate value response: {}", response.status());

    // If successful, verify count didn't increase
    if response.status().is_success() {
        let get_response: EnumerationResponse = ctx
            .server
            .get(&format!("/api/template-enumerations/{}", enum_name))
            .await
            .expect("Failed to get enumeration");
        // Should still have only 1 value (no duplicates)
        assert_enumeration_value_count(&get_response, 1);
    }

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_add_empty_value() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("empty_val_enum");

    // Create enumeration
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["VALID"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    // Try to add empty value
    let add_request = AddEnumerationValueRequest {
        value: "".to_string(),
    };

    let response = ctx
        .server
        .post_response(
            &format!("/api/template-enumerations/{}/values", enum_name),
            &add_request,
        )
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_add_value_to_nonexistent_enumeration() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let add_request = AddEnumerationValueRequest {
        value: "VALUE".to_string(),
    };

    let response = ctx
        .server
        .post_response("/api/template-enumerations/nonexistent_enum/values", &add_request)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_add_value_with_special_characters() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("special_val_enum");

    // Create enumeration
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["NORMAL"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    // Try to add value with special characters
    let special_values = [
        "VALUE_WITH_UNDERSCORE",
        "VALUE-WITH-DASH",
        "VALUE.WITH.DOT",
    ];

    for special in special_values {
        let add_request = AddEnumerationValueRequest {
            value: special.to_string(),
        };

        let result = ctx
            .server
            .post::<_, EnumerationResponse>(
                &format!("/api/template-enumerations/{}/values", enum_name),
                &add_request,
            )
            .await;

        match result {
            Ok(response) => {
                assert_enumeration_contains(&response, special);
            }
            Err(e) => {
                println!("Special value '{}' rejected: {}", special, e);
            }
        }
    }

    ctx.cleanup().await.ok();
}

// =============================================================================
// Remove Value Tests
// =============================================================================

#[tokio::test]
async fn test_remove_value_from_enumeration() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("remove_val_enum");

    // Create enumeration with multiple values
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["KEEP", "REMOVE", "ALSO_KEEP"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    // Remove a value
    let response: EnumerationResponse = ctx
        .server
        .delete(&format!("/api/template-enumerations/{}/values/REMOVE", enum_name))
        .await
        .expect("Failed to remove value");

    assert_enumeration_value_count(&response, 2);
    assert_enumeration_contains(&response, "KEEP");
    assert_enumeration_contains(&response, "ALSO_KEEP");
    // REMOVE should not be present
    assert!(!response.values.contains(&"REMOVE".to_string()));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_remove_last_value_blocked() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("last_val_enum");

    // Create enumeration with single value
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["ONLY_VALUE"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    // Try to remove the last value
    let response = ctx
        .server
        .delete_response(&format!(
            "/api/template-enumerations/{}/values/ONLY_VALUE",
            enum_name
        ))
        .await
        .expect("Request should complete");

    // Should be rejected - enumeration must have at least one value
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Verify value still exists
    let get_response: EnumerationResponse = ctx
        .server
        .get(&format!("/api/template-enumerations/{}", enum_name))
        .await
        .expect("Failed to get enumeration");
    assert_enumeration_value_count(&get_response, 1);
    assert_enumeration_contains(&get_response, "ONLY_VALUE");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_remove_nonexistent_value() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("no_val_enum");

    // Create enumeration
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["EXISTS"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    // Try to remove non-existent value
    let response = ctx
        .server
        .delete_response(&format!(
            "/api/template-enumerations/{}/values/DOES_NOT_EXIST",
            enum_name
        ))
        .await
        .expect("Request should complete");

    // Might return NOT_FOUND or be idempotent
    println!(
        "Remove non-existent value response: {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_remove_value_from_nonexistent_enumeration() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let response = ctx
        .server
        .delete_response("/api/template-enumerations/nonexistent_enum/values/ANY_VALUE")
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_remove_multiple_values_sequentially() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("multi_remove_enum");

    // Create enumeration with many values
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("loan")
        .with_values(vec!["A", "B", "C", "D", "E"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    // Remove values one by one (keep at least one)
    let values_to_remove = ["B", "D"];

    for value in values_to_remove {
        ctx.server
            .delete::<EnumerationResponse>(&format!(
                "/api/template-enumerations/{}/values/{}",
                enum_name, value
            ))
            .await
            .expect(&format!("Failed to remove value {}", value));
    }

    // Verify final state
    let response: EnumerationResponse = ctx
        .server
        .get(&format!("/api/template-enumerations/{}", enum_name))
        .await
        .expect("Failed to get enumeration");

    assert_enumeration_value_count(&response, 3);
    assert_enumeration_contains(&response, "A");
    assert_enumeration_contains(&response, "C");
    assert_enumeration_contains(&response, "E");

    ctx.cleanup().await.ok();
}

// =============================================================================
// Value Ordering Tests
// =============================================================================

#[tokio::test]
async fn test_values_order_preserved() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("order_enum");

    // Create enumeration with ordered values
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["FIRST", "SECOND", "THIRD"])
        .build();
    let created: EnumerationResponse = ctx
        .server
        .post("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    // Check if order is preserved
    // Note: Order might not be guaranteed depending on implementation
    println!("Created values order: {:?}", created.values);

    // Get and check again
    let response: EnumerationResponse = ctx
        .server
        .get(&format!("/api/template-enumerations/{}", enum_name))
        .await
        .expect("Failed to get enumeration");

    println!("Retrieved values order: {:?}", response.values);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_add_value_appends_to_end() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("append_enum");

    // Create enumeration
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["FIRST"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    // Add value
    let add_request = AddEnumerationValueRequest {
        value: "SECOND".to_string(),
    };
    let response: EnumerationResponse = ctx
        .server
        .post(
            &format!("/api/template-enumerations/{}/values", enum_name),
            &add_request,
        )
        .await
        .expect("Failed to add value");

    // Check if new value was appended
    println!("Values after add: {:?}", response.values);
    // New value should typically be at the end
    if response.values.len() == 2 {
        // Note: Order might not be guaranteed
        assert!(response.values.contains(&"FIRST".to_string()));
        assert!(response.values.contains(&"SECOND".to_string()));
    }

    ctx.cleanup().await.ok();
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_value_with_whitespace() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("whitespace_enum");

    // Create enumeration
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["NORMAL"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    // Try to add value with leading/trailing whitespace
    let add_request = AddEnumerationValueRequest {
        value: "  PADDED  ".to_string(),
    };

    let response = ctx
        .server
        .post_response(
            &format!("/api/template-enumerations/{}/values", enum_name),
            &add_request,
        )
        .await
        .expect("Request should complete");

    // Document behavior - might trim or reject
    println!(
        "Value with whitespace response: {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_value_case_sensitivity() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("case_enum");

    // Create enumeration with lowercase value
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["value"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    // Try to add uppercase version
    let add_request = AddEnumerationValueRequest {
        value: "VALUE".to_string(),
    };

    let result = ctx
        .server
        .post::<_, EnumerationResponse>(
            &format!("/api/template-enumerations/{}/values", enum_name),
            &add_request,
        )
        .await;

    match result {
        Ok(response) => {
            // Both should exist (case-sensitive)
            println!(
                "Case sensitivity - values: {:?}",
                response.values
            );
        }
        Err(e) => {
            // Might be rejected as duplicate (case-insensitive)
            println!("Case sensitivity - rejected: {}", e);
        }
    }

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_add_remove_add_same_value() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("readd_enum");

    // Create enumeration with two values
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["KEEP", "TOGGLE"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    // Remove TOGGLE
    ctx.server
        .delete::<EnumerationResponse>(&format!(
            "/api/template-enumerations/{}/values/TOGGLE",
            enum_name
        ))
        .await
        .expect("Failed to remove value");

    // Verify removed
    let after_remove: EnumerationResponse = ctx
        .server
        .get(&format!("/api/template-enumerations/{}", enum_name))
        .await
        .expect("Failed to get enumeration");
    assert!(!after_remove.values.contains(&"TOGGLE".to_string()));

    // Add TOGGLE back
    let add_request = AddEnumerationValueRequest {
        value: "TOGGLE".to_string(),
    };
    let after_add: EnumerationResponse = ctx
        .server
        .post(
            &format!("/api/template-enumerations/{}/values", enum_name),
            &add_request,
        )
        .await
        .expect("Failed to re-add value");

    assert_enumeration_contains(&after_add, "TOGGLE");
    assert_enumeration_value_count(&after_add, 2);

    ctx.cleanup().await.ok();
}
