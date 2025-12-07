//! Complete Product Setup Tests
//!
//! End-to-end tests for setting up a complete product from scratch through evaluation.

use crate::fixtures::*;
use serde_json::json;

/// Test complete product setup workflow:
/// 1. Create datatypes
/// 2. Create enumerations
/// 3. Create product
/// 4. Create abstract attributes
/// 5. Create concrete attributes
/// 6. Create rules
/// 7. Create functionalities
/// 8. Evaluate product
/// 9. Submit for approval
/// 10. Approve product
#[tokio::test]
async fn test_complete_product_setup_workflow() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("loan_product");
    let decimal_datatype_id = ctx.unique_id("decimal");
    let int_datatype_id = ctx.unique_id("int");

    // === Step 1: Create datatypes ===
    let decimal_dt_req = json!({
        "id": decimal_datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &decimal_dt_req)
        .await
        .expect("Step 1: Failed to create decimal datatype");

    let int_dt_req = json!({
        "id": int_datatype_id,
        "name": "Test Int",
        "primitiveType": "INT"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &int_dt_req)
        .await
        .expect("Step 1: Failed to create int datatype");

    // === Step 2: Create enumerations ===
    let enum_name = ctx.unique_id("loanstatus").replace('-', "");
    let enum_req = json!({
        "name": enum_name,
        "templateType": "loan",
        "description": "Status of loan",
        "values": ["PENDING", "APPROVED", "DISBURSED", "CLOSED"]
    });
    ctx.post::<serde_json::Value>("/api/template-enumerations", &enum_req)
        .await
        .expect("Step 2: Failed to create enumeration");

    // === Step 3: Create product ===
    let product_req = json!({
        "id": product_id,
        "name": "Personal Loan Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600,
        "expiryAt": 1925097599,
        "description": "A complete personal loan product"
    });
    let product_response: serde_json::Value = ctx.post("/api/products", &product_req)
        .await
        .expect("Step 3: Failed to create product");
    assert_eq!(product_response["status"], "DRAFT");

    // === Step 4: Create abstract attributes ===
    let abstract_attrs = [
        ("principal", decimal_datatype_id.as_str(), "Principal amount"),
        ("interest-rate", decimal_datatype_id.as_str(), "Annual interest rate"),
        ("tenure", int_datatype_id.as_str(), "Loan tenure in months"),
        ("emi", decimal_datatype_id.as_str(), "Monthly EMI"),
        ("total-interest", decimal_datatype_id.as_str(), "Total interest payable"),
        ("total-amount", decimal_datatype_id.as_str(), "Total amount payable"),
    ];

    let mut abstract_paths: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for (name, datatype, desc) in abstract_attrs {
        let attr_req = json!({
            "componentType": "loan",
            "componentId": "main",
            "attributeName": name,
            "datatypeId": datatype,
            "displayNames": [{"name": desc, "format": "default", "orderIndex": 0}]
        });
        let resp: serde_json::Value = ctx.post(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr_req,
        )
        .await
        .expect(&format!("Step 4: Failed to create abstract attribute '{}'", name));
        abstract_paths.insert(name.to_string(), resp["abstractPath"].as_str().unwrap().to_string());
    }

    // === Step 5: Create concrete attributes ===
    // principal, interest-rate, tenure are FIXED_VALUE (inputs)
    let concrete_inputs = [
        ("principal", json!({"type": "decimal", "value": "100000.00"})),
        ("interest-rate", json!({"type": "decimal", "value": "0.12"})),
        ("tenure", json!({"type": "int", "value": 12})),
    ];

    for (name, value) in concrete_inputs {
        let abstract_path = abstract_paths.get(name).expect(&format!("Missing abstract path for {}", name));
        let concrete_req = json!({
            "componentType": "loan",
            "componentId": "main",
            "attributeName": name,
            "abstractPath": abstract_path,
            "valueType": "FIXED_VALUE",
            "value": value
        });
        ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/attributes", product_id),
            &concrete_req,
        )
        .await
        .expect(&format!("Step 5: Failed to create concrete attribute '{}'", name));
    }

    // emi, total-interest, total-amount are RULE_DRIVEN (computed)
    // First create the rules, then create RULE_DRIVEN attributes

    // === Step 6: Create rules ===
    // Rule 1: Calculate monthly rate
    let monthly_rate_attr_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "monthly-rate",
        "datatypeId": decimal_datatype_id
    });
    let monthly_rate_attr_resp: serde_json::Value = ctx.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &monthly_rate_attr_req,
    )
    .await
    .expect("Step 6: Failed to create monthly-rate abstract attribute");
    abstract_paths.insert("monthly-rate".to_string(), monthly_rate_attr_resp["abstractPath"].as_str().unwrap().to_string());

    let monthly_rate_rule = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "interest-rate / 12",
        "expressionJson": r#"{"/": [{"var": "loan/main/interest-rate"}, 12]}"#,
        "inputAttributes": ["loan/main/interest-rate"],
        "outputAttributes": ["loan/main/monthly-rate"],
        "orderIndex": 0
    });
    let monthly_rate_rule_resp: serde_json::Value = ctx.post(
        &format!("/api/products/{}/rules", product_id),
        &monthly_rate_rule,
    )
    .await
    .expect("Step 6: Failed to create monthly rate rule");
    let monthly_rate_rule_id = monthly_rate_rule_resp["id"].as_str().unwrap().to_string();

    // Rule 2: Calculate EMI (simplified: P * r / 12)
    let emi_rule = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "principal * monthly-rate",
        "expressionJson": r#"{"*": [{"var": "loan/main/principal"}, {"var": "loan/main/monthly-rate"}]}"#,
        "inputAttributes": ["loan/main/principal", "loan/main/monthly-rate"],
        "outputAttributes": ["loan/main/emi"],
        "orderIndex": 1
    });
    let emi_rule_resp: serde_json::Value = ctx.post(
        &format!("/api/products/{}/rules", product_id),
        &emi_rule,
    )
    .await
    .expect("Step 6: Failed to create EMI rule");
    let emi_rule_id = emi_rule_resp["id"].as_str().unwrap().to_string();

    // Rule 3: Calculate total interest
    let total_interest_rule = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "(emi * tenure) - principal",
        "expressionJson": r#"{"-": [{"*": [{"var": "loan/main/emi"}, {"var": "loan/main/tenure"}]}, {"var": "loan/main/principal"}]}"#,
        "inputAttributes": ["loan/main/emi", "loan/main/tenure", "loan/main/principal"],
        "outputAttributes": ["loan/main/total-interest"],
        "orderIndex": 2
    });
    let total_interest_rule_resp: serde_json::Value = ctx.post(
        &format!("/api/products/{}/rules", product_id),
        &total_interest_rule,
    )
    .await
    .expect("Step 6: Failed to create total interest rule");
    let total_interest_rule_id = total_interest_rule_resp["id"].as_str().unwrap().to_string();

    // Rule 4: Calculate total amount
    let total_amount_rule = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "principal + total-interest",
        "expressionJson": r#"{"+": [{"var": "loan/main/principal"}, {"var": "loan/main/total-interest"}]}"#,
        "inputAttributes": ["loan/main/principal", "loan/main/total-interest"],
        "outputAttributes": ["loan/main/total-amount"],
        "orderIndex": 3
    });
    let total_amount_rule_resp: serde_json::Value = ctx.post(
        &format!("/api/products/{}/rules", product_id),
        &total_amount_rule,
    )
    .await
    .expect("Step 6: Failed to create total amount rule");
    let total_amount_rule_id = total_amount_rule_resp["id"].as_str().unwrap().to_string();

    // Create RULE_DRIVEN concrete attributes for computed values
    for (name, rule_id) in [
        ("monthly-rate", &monthly_rate_rule_id),
        ("emi", &emi_rule_id),
        ("total-interest", &total_interest_rule_id),
        ("total-amount", &total_amount_rule_id),
    ] {
        let abstract_path = abstract_paths.get(name).expect(&format!("Missing abstract path for {}", name));
        let concrete_req = json!({
            "componentType": "loan",
            "componentId": "main",
            "attributeName": name,
            "abstractPath": abstract_path,
            "valueType": "RULE_DRIVEN",
            "ruleId": rule_id
        });
        ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/attributes", product_id),
            &concrete_req,
        )
        .await
        .expect(&format!("Step 5b: Failed to create RULE_DRIVEN attribute '{}'", name));
    }

    // === Step 7: Create functionalities ===
    let func_req = json!({
        "name": "loan-calculation",
        "description": "Calculate loan EMI and total amounts",
        "requiredAttributes": [
            {"abstractPath": "loan/main/emi", "description": "Monthly EMI"},
            {"abstractPath": "loan/main/total-interest", "description": "Total interest"},
            {"abstractPath": "loan/main/total-amount", "description": "Total payable"}
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Step 7: Failed to create functionality");

    // === Step 8: Evaluate product ===
    let eval_req = json!({
        "productId": product_id,
        "inputData": {
            "loan/main/principal": {"type": "decimal", "value": "100000.00"},
            "loan/main/interest-rate": {"type": "decimal", "value": "0.12"},
            "loan/main/tenure": {"type": "int", "value": 12}
        }
    });
    let eval_response: serde_json::Value = ctx.post("/api/evaluate", &eval_req)
        .await
        .expect("Step 8: Failed to evaluate product");

    assert_eq!(eval_response["success"], true);
    assert!(eval_response["ruleResults"].as_array().unwrap().len() >= 4);

    // === Step 9: Submit for approval ===
    let submit_response: serde_json::Value = ctx.post_empty(
        &format!("/api/products/{}/submit", product_id),
    )
    .await
    .expect("Step 9: Failed to submit product");

    assert_eq!(submit_response["status"], "PENDING_APPROVAL");

    // === Step 10: Approve product ===
    let approve_req = json!({ "approved": true });
    let approve_response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/approve", product_id),
        &approve_req,
    )
    .await
    .expect("Step 10: Failed to approve product");

    assert_eq!(approve_response["status"], "ACTIVE");

    // Final verification: Get product and verify state
    let final_product: serde_json::Value = ctx.get(
        &format!("/api/products/{}", product_id),
    )
    .await
    .expect("Final: Failed to get product");

    assert_eq!(final_product["id"], product_id);
    assert_eq!(final_product["status"], "ACTIVE");
}

/// Test workflow: Clone product and modify
#[tokio::test]
async fn test_clone_and_modify_workflow() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let datatype_id = ctx.unique_id("decimal").replace('-', "");

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // === Create original product ===
    let original_id = ctx.unique_product_id("original");
    let original_req = json!({
        "id": original_id,
        "name": "Original Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>("/api/products", &original_req)
        .await
        .expect("Failed to create original product");

    // Add abstract attribute
    let attr_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "amount",
        "datatypeId": datatype_id
    });
    let abstract_resp: serde_json::Value = ctx.post(
        &format!("/api/products/{}/abstract-attributes", original_id),
        &attr_req,
    )
    .await
    .expect("Failed to create abstract attribute");
    let abstract_path = abstract_resp["abstractPath"].as_str().unwrap();

    // Add concrete attribute
    let concrete_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "amount",
        "abstractPath": abstract_path,
        "valueType": "FIXED_VALUE",
        "value": {"type": "decimal", "value": "5000.00"}
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/attributes", original_id),
        &concrete_req,
    )
    .await
    .expect("Failed to create concrete attribute");

    // === Clone product ===
    let clone_id = ctx.unique_product_id("clone");
    let clone_req = json!({
        "newProductId": clone_id,
        "newProductName": "Cloned Product",
        "cloneAbstractAttributes": true,
        "cloneAttributes": true,
        "cloneRules": true,
        "cloneFunctionalities": true
    });
    let clone_response: serde_json::Value = ctx.post(
        &format!("/api/products/{}/clone", original_id),
        &clone_req,
    )
    .await
    .expect("Failed to clone product");

    assert_eq!(clone_response["product"]["id"], clone_id);
    assert_eq!(clone_response["product"]["status"], "DRAFT");

    // === Modify cloned product ===
    // Update product description
    let update_req = json!({
        "description": "Modified clone with different parameters"
    });
    let update_response: serde_json::Value = ctx.put(
        &format!("/api/products/{}", clone_id),
        &update_req,
    )
    .await
    .expect("Failed to update cloned product");

    assert_eq!(update_response["description"], "Modified clone with different parameters");

    // Verify original is unchanged
    let original: serde_json::Value = ctx.get(
        &format!("/api/products/{}", original_id),
    )
    .await
    .expect("Failed to get original product");

    // Original should not have the modified description
    assert!(original["description"].as_str().unwrap_or("").is_empty() ||
            original["description"] != "Modified clone with different parameters");
}

/// Test workflow: Product with validation errors and fixes
#[tokio::test]
async fn test_product_validation_and_fix_workflow() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("validation_test");
    let datatype_id = ctx.unique_id("decimal").replace('-', "");

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // === Create product ===
    let product_req = json!({
        "id": product_id,
        "name": "Validation Test Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>("/api/products", &product_req)
        .await
        .expect("Failed to create product");

    // === Create functionality with missing required attribute ===
    let attr_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "required-field",
        "datatypeId": datatype_id
    });
    let abstract_resp: serde_json::Value = ctx.post(
        &format!("/api/products/{}/abstract-attributes", product_id),
        &attr_req,
    )
    .await
    .expect("Failed to create abstract attribute");
    let abstract_path = abstract_resp["abstractPath"].as_str().unwrap();

    let func_req = json!({
        "name": "requires-field",
        "description": "Functionality that requires a field",
        "requiredAttributes": [
            {"abstractPath": abstract_path, "description": "Required"}
        ]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/functionalities", product_id),
        &func_req,
    )
    .await
    .expect("Failed to create functionality");

    // === Validate - should have errors ===
    let validation1: serde_json::Value = ctx.get(
        &format!("/api/products/{}/validate", product_id),
    )
    .await
    .expect("Failed to validate product");

    assert_eq!(validation1["valid"], false);
    assert!(!validation1["errors"].as_array().unwrap().is_empty());

    // === Fix: Add concrete attribute ===
    let concrete_req = json!({
        "componentType": "loan",
        "componentId": "main",
        "attributeName": "required-field",
        "abstractPath": abstract_path,
        "valueType": "FIXED_VALUE",
        "value": {"type": "decimal", "value": "100.00"}
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/attributes", product_id),
        &concrete_req,
    )
    .await
    .expect("Failed to create concrete attribute");

    // === Validate again - should be valid ===
    let validation2: serde_json::Value = ctx.get(
        &format!("/api/products/{}/validate", product_id),
    )
    .await
    .expect("Failed to validate product after fix");

    assert_eq!(validation2["valid"], true);
    assert!(validation2["errors"].as_array().unwrap().is_empty());
}

/// Test workflow: Batch evaluation for different scenarios
#[tokio::test]
async fn test_batch_evaluation_scenarios_workflow() {
    let ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("batch_product");
    let datatype_id = ctx.unique_id("decimal").replace('-', "");

    // Create datatype first
    let dt_req = json!({
        "id": datatype_id,
        "name": "Test Decimal",
        "primitiveType": "DECIMAL"
    });
    ctx.post::<serde_json::Value>("/api/datatypes", &dt_req)
        .await
        .expect("Failed to create datatype");

    // === Setup product with simple interest calculation ===
    let product_req = json!({
        "id": product_id,
        "name": "Batch Eval Product",
        "templateType": "loan",
        "effectiveFrom": 1735689600
    });
    ctx.post::<serde_json::Value>("/api/products", &product_req)
        .await
        .expect("Failed to create product");

    // Create attributes
    for name in ["principal", "rate", "interest"] {
        let attr_req = json!({
            "componentType": "loan",
            "componentId": "main",
            "attributeName": name,
            "datatypeId": datatype_id
        });
        ctx.post::<serde_json::Value>(
            &format!("/api/products/{}/abstract-attributes", product_id),
            &attr_req,
        )
        .await
        .expect("Failed to create attribute");
    }

    // Create rule: interest = principal * rate
    let rule_req = json!({
        "ruleType": "CALCULATION",
        "displayExpression": "principal * rate",
        "expressionJson": r#"{"*": [{"var": "loan/main/principal"}, {"var": "loan/main/rate"}]}"#,
        "inputAttributes": ["loan/main/principal", "loan/main/rate"],
        "outputAttributes": ["loan/main/interest"]
    });
    ctx.post::<serde_json::Value>(
        &format!("/api/products/{}/rules", product_id),
        &rule_req,
    )
    .await
    .expect("Failed to create rule");

    // === Batch evaluate multiple scenarios ===
    let batch_req = json!({
        "productId": product_id,
        "requests": [
            {
                "requestId": "scenario-1-low-rate",
                "inputData": {
                    "loan/main/principal": {"type": "float", "value": 10000.0},
                    "loan/main/rate": {"type": "float", "value": 0.05}
                }
            },
            {
                "requestId": "scenario-2-medium-rate",
                "inputData": {
                    "loan/main/principal": {"type": "float", "value": 10000.0},
                    "loan/main/rate": {"type": "float", "value": 0.10}
                }
            },
            {
                "requestId": "scenario-3-high-rate",
                "inputData": {
                    "loan/main/principal": {"type": "float", "value": 10000.0},
                    "loan/main/rate": {"type": "float", "value": 0.15}
                }
            },
            {
                "requestId": "scenario-4-different-principal",
                "inputData": {
                    "loan/main/principal": {"type": "float", "value": 50000.0},
                    "loan/main/rate": {"type": "float", "value": 0.10}
                }
            }
        ]
    });

    let batch_response: serde_json::Value = ctx.post("/api/batch-evaluate", &batch_req)
        .await
        .expect("Failed to batch evaluate");

    // Verify all scenarios completed
    let results = batch_response["results"].as_array().unwrap();
    assert_eq!(results.len(), 4);

    for result in results {
        assert_eq!(result["success"], true);
        assert!(result["results"].as_array().unwrap().len() > 0);
    }

    // Verify request IDs are preserved
    let request_ids: Vec<&str> = results.iter()
        .map(|r| r["requestId"].as_str().unwrap())
        .collect();
    assert!(request_ids.contains(&"scenario-1-low-rate"));
    assert!(request_ids.contains(&"scenario-2-medium-rate"));
    assert!(request_ids.contains(&"scenario-3-high-rate"));
    assert!(request_ids.contains(&"scenario-4-different-principal"));
}
