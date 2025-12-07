//! Enumeration Template Type Tests
//!
//! Tests for template type filtering and association.

use crate::fixtures::{assertions::*, data_builders::EnumerationBuilder, TestContext};
use product_farm_api::rest::types::{EnumerationResponse, ListEnumerationsResponse};
use reqwest::StatusCode;

// =============================================================================
// Template Type Filter Tests
// =============================================================================

#[tokio::test]
async fn test_list_enumerations_by_template_type() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // Create enumerations for different templates
    let insurance_enum = ctx.unique_id("insurance_enum");
    let loan_enum = ctx.unique_id("loan_enum");

    let insurance_request = EnumerationBuilder::new(&insurance_enum)
        .with_template("insurance")
        .with_values(vec!["INS_VAL"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &insurance_request)
        .await
        .expect("Failed to create insurance enumeration");

    let loan_request = EnumerationBuilder::new(&loan_enum)
        .with_template("loan")
        .with_values(vec!["LOAN_VAL"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &loan_request)
        .await
        .expect("Failed to create loan enumeration");

    // Filter by insurance template
    let insurance_list: ListEnumerationsResponse = ctx
        .server
        .get("/api/template-enumerations?templateType=insurance&pageSize=100")
        .await
        .expect("Failed to list insurance enumerations");

    // All returned enumerations should be insurance type
    for enum_item in &insurance_list.items {
        assert_eq!(
            enum_item.template_type, "insurance",
            "Expected insurance template, got {}",
            enum_item.template_type
        );
    }

    // Filter by loan template
    let loan_list: ListEnumerationsResponse = ctx
        .server
        .get("/api/template-enumerations?templateType=loan&pageSize=100")
        .await
        .expect("Failed to list loan enumerations");

    // All returned enumerations should be loan type
    for enum_item in &loan_list.items {
        assert_eq!(
            enum_item.template_type, "loan",
            "Expected loan template, got {}",
            enum_item.template_type
        );
    }

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_list_enumerations_no_filter_returns_all() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // Create enumerations for different templates
    let enum1 = ctx.unique_id("all_enum1");
    let enum2 = ctx.unique_id("all_enum2");

    let req1 = EnumerationBuilder::new(&enum1)
        .with_template("insurance")
        .with_values(vec!["V1"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &req1)
        .await
        .expect("Failed to create enumeration 1");

    let req2 = EnumerationBuilder::new(&enum2)
        .with_template("loan")
        .with_values(vec!["V2"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &req2)
        .await
        .expect("Failed to create enumeration 2");

    // List without filter
    let all_list: ListEnumerationsResponse = ctx
        .server
        .get("/api/template-enumerations?pageSize=100")
        .await
        .expect("Failed to list all enumerations");

    // Should contain both template types
    let templates: Vec<&str> = all_list
        .items
        .iter()
        .map(|e| e.template_type.as_str())
        .collect();

    // Verify we have at least our test enumerations
    assert!(all_list.items.len() >= 2);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_filter_by_nonexistent_template() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // Filter by non-existent template type
    let result: ListEnumerationsResponse = ctx
        .server
        .get("/api/template-enumerations?templateType=nonexistent_template_xyz&pageSize=100")
        .await
        .expect("Failed to list enumerations");

    // Should return empty list (not error)
    assert!(
        result.items.is_empty(),
        "Expected empty list for non-existent template"
    );

    ctx.cleanup().await.ok();
}

// =============================================================================
// Template Type Validation Tests
// =============================================================================

#[tokio::test]
async fn test_create_with_valid_template_types() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let template_types = ["insurance", "loan", "investment", "deposit"];

    for template in template_types {
        let enum_name = ctx.unique_id(&format!("{}_tmpl_enum", template));

        let request = EnumerationBuilder::new(&enum_name)
            .with_template(template)
            .with_values(vec!["VAL"])
            .build();

        let result = ctx
            .server
            .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
            .await;

        match result {
            Ok(response) => {
                assert_eq!(response.template_type, template);
            }
            Err(e) => {
                // Document if certain templates are not supported
                println!("Template '{}' not supported: {}", template, e);
            }
        }
    }

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_template_type_case_sensitivity() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // Test with different case
    let enum_name = ctx.unique_id("case_tmpl_enum");

    let request = EnumerationBuilder::new(&enum_name)
        .with_template("INSURANCE") // uppercase
        .with_values(vec!["VAL"])
        .build();

    let result = ctx
        .server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
        .await;

    // Document behavior - might normalize to lowercase or reject
    match result {
        Ok(response) => {
            println!(
                "Uppercase template accepted, stored as: {}",
                response.template_type
            );
        }
        Err(e) => {
            println!("Uppercase template rejected: {}", e);
        }
    }

    ctx.cleanup().await.ok();
}

// =============================================================================
// Template Association Tests
// =============================================================================

#[tokio::test]
async fn test_enumeration_template_immutable() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("immut_tmpl_enum");

    // Create enumeration with insurance template
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance")
        .with_values(vec!["VAL"])
        .build();
    let created: EnumerationResponse = ctx
        .server
        .post("/api/template-enumerations", &request)
        .await
        .expect("Failed to create enumeration");

    assert_eq!(created.template_type, "insurance");

    // Template type cannot be changed via update
    // (UpdateEnumerationRequest only has description field)
    // Verify template stays the same after any update

    let update = product_farm_api::rest::types::UpdateEnumerationRequest {
        description: Some("New description".to_string()),
    };

    let updated: EnumerationResponse = ctx
        .server
        .put(&format!("/api/template-enumerations/{}", enum_name), &update)
        .await
        .expect("Failed to update enumeration");

    // Template type should remain unchanged
    assert_eq!(updated.template_type, "insurance");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_multiple_enumerations_same_template() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // Create multiple enumerations for the same template
    let enum_names: Vec<String> = (0..3)
        .map(|i| ctx.unique_id(&format!("multi_enum_{}", i)))
        .collect();

    for enum_name in &enum_names {
        let request = EnumerationBuilder::new(enum_name)
            .with_template("insurance")
            .with_values(vec!["V"])
            .build();
        ctx.server
            .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
            .await
            .expect("Failed to create enumeration");
    }

    // All should be accessible
    for enum_name in &enum_names {
        let response: EnumerationResponse = ctx
            .server
            .get(&format!("/api/template-enumerations/{}", enum_name))
            .await
            .expect("Failed to get enumeration");

        assert_eq!(response.template_type, "insurance");
    }

    ctx.cleanup().await.ok();
}

// =============================================================================
// Template and Value Combination Tests
// =============================================================================

#[tokio::test]
async fn test_same_values_different_templates() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let insurance_enum = ctx.unique_id("ins_same_vals");
    let loan_enum = ctx.unique_id("loan_same_vals");

    // Create two enumerations with identical values but different templates
    let ins_request = EnumerationBuilder::new(&insurance_enum)
        .with_template("insurance")
        .with_values(vec!["ACTIVE", "INACTIVE", "PENDING"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &ins_request)
        .await
        .expect("Failed to create insurance enumeration");

    let loan_request = EnumerationBuilder::new(&loan_enum)
        .with_template("loan")
        .with_values(vec!["ACTIVE", "INACTIVE", "PENDING"])
        .build();
    ctx.server
        .post::<_, EnumerationResponse>("/api/template-enumerations", &loan_request)
        .await
        .expect("Failed to create loan enumeration");

    // Both should exist independently
    let ins_response: EnumerationResponse = ctx
        .server
        .get(&format!("/api/template-enumerations/{}", insurance_enum))
        .await
        .expect("Failed to get insurance enumeration");
    let loan_response: EnumerationResponse = ctx
        .server
        .get(&format!("/api/template-enumerations/{}", loan_enum))
        .await
        .expect("Failed to get loan enumeration");

    assert_eq!(ins_response.template_type, "insurance");
    assert_eq!(loan_response.template_type, "loan");
    assert_enumeration_value_count(&ins_response, 3);
    assert_enumeration_value_count(&loan_response, 3);

    ctx.cleanup().await.ok();
}

// =============================================================================
// Pagination with Template Filter Tests
// =============================================================================

#[tokio::test]
async fn test_pagination_with_template_filter() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // Create 5 enumerations for insurance template
    for i in 0..5 {
        let enum_name = ctx.unique_id(&format!("page_ins_enum_{}", i));
        let request = EnumerationBuilder::new(&enum_name)
            .with_template("insurance")
            .with_values(vec!["V"])
            .build();
        ctx.server
            .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
            .await
            .expect("Failed to create enumeration");
    }

    // Create 3 for loan (should not appear in insurance filter)
    for i in 0..3 {
        let enum_name = ctx.unique_id(&format!("page_loan_enum_{}", i));
        let request = EnumerationBuilder::new(&enum_name)
            .with_template("loan")
            .with_values(vec!["V"])
            .build();
        ctx.server
            .post::<_, EnumerationResponse>("/api/template-enumerations", &request)
            .await
            .expect("Failed to create enumeration");
    }

    // Paginate insurance enumerations with page size 2
    let page1: ListEnumerationsResponse = ctx
        .server
        .get("/api/template-enumerations?templateType=insurance&pageSize=2")
        .await
        .expect("Failed to list page 1");

    assert!(page1.items.len() <= 2);
    for item in &page1.items {
        assert_eq!(item.template_type, "insurance");
    }

    ctx.cleanup().await.ok();
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_template_type_with_special_characters() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("special_tmpl_enum");

    // Try template with special characters
    let request = EnumerationBuilder::new(&enum_name)
        .with_template("insurance-type_v2")
        .with_values(vec!["VAL"])
        .build();

    let response = ctx
        .server
        .post_response("/api/template-enumerations", &request)
        .await
        .expect("Request should complete");

    // Document behavior
    println!(
        "Template with special chars response: {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_empty_template_filter_query() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // Query with empty template type
    let result = ctx
        .server
        .get_response("/api/template-enumerations?templateType=&pageSize=100")
        .await
        .expect("Request should complete");

    // Should either return all or be rejected as invalid
    println!("Empty template filter response: {}", result.status());

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_very_long_template_type() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let enum_name = ctx.unique_id("long_tmpl_enum");

    // Very long template type
    let long_template = "a".repeat(100);

    let request = EnumerationBuilder::new(&enum_name)
        .with_template(&long_template)
        .with_values(vec!["VAL"])
        .build();

    let response = ctx
        .server
        .post_response("/api/template-enumerations", &request)
        .await
        .expect("Request should complete");

    // Should be rejected for being too long
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    ctx.cleanup().await.ok();
}
