//! REST handlers for Rule Evaluation
//!
//! Provides HTTP endpoints for evaluating rules and attributes

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use product_farm_core::ProductId;
use std::collections::HashMap;

use crate::store::SharedStore;

use super::error::{ApiError, ApiResult};
use super::types::*;

/// Create routes for evaluation endpoints
pub fn routes() -> Router<SharedStore> {
    Router::new()
        .route("/api/evaluate", post(evaluate))
        .route("/api/batch-evaluate", post(batch_evaluate))
        .route(
            "/api/products/{product_id}/execution-plan",
            get(get_execution_plan),
        )
        .route("/api/products/{product_id}/validate", get(validate_product))
}

/// Evaluate rules for a product
async fn evaluate(
    State(store): State<SharedStore>,
    Json(req): Json<EvaluateRequest>,
) -> ApiResult<Json<EvaluateResponse>> {
    let store = store.read().await;

    // Verify product exists (store uses String keys)
    if !store.products.contains_key(&req.product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            req.product_id
        )));
    }

    // Rule evaluation is not yet implemented
    // TODO: Implement actual rule evaluation using the json-logic engine:
    // 1. Look up the rules (filtered by rule_ids if provided)
    // 2. Build dependency graph
    // 3. Evaluate rules in order with the provided input_data
    // 4. Return the computed values
    Err(ApiError::not_implemented(
        "Rule evaluation is not yet implemented. Use the gRPC API for rule evaluation.",
    ))
}

/// Batch evaluate multiple requests for a product
async fn batch_evaluate(
    State(store): State<SharedStore>,
    Json(req): Json<BatchEvaluateRequest>,
) -> ApiResult<Json<BatchEvaluateResponse>> {
    let store = store.read().await;

    // Verify product exists (store uses String keys)
    if !store.products.contains_key(&req.product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            req.product_id
        )));
    }

    // Batch evaluation is not yet implemented
    // TODO: Implement batch evaluation that processes multiple input sets efficiently
    Err(ApiError::not_implemented(
        "Batch rule evaluation is not yet implemented. Use the gRPC API for rule evaluation.",
    ))
}

/// Get the execution plan for a product's rules
async fn get_execution_plan(
    State(store): State<SharedStore>,
    Path(product_id): Path<String>,
) -> ApiResult<Json<ExecutionPlanResponse>> {
    let store = store.read().await;

    // Verify product exists (store uses String keys)
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    // Collect rules for this product
    let mut rules: Vec<_> = store
        .rules
        .values()
        .filter(|r| r.product_id == pid && r.enabled)
        .collect();

    // Sort by order_index
    rules.sort_by_key(|r| r.order_index);

    // Build execution levels (group rules by order)
    let mut level_map: HashMap<i32, Vec<String>> = HashMap::new();
    for rule in &rules {
        level_map
            .entry(rule.order_index)
            .or_default()
            .push(rule.id.to_string());
    }
    let mut levels: Vec<ExecutionLevelJson> = level_map
        .into_iter()
        .map(|(level, rule_ids)| ExecutionLevelJson { level, rule_ids })
        .collect();
    levels.sort_by_key(|l| l.level);

    // Build dependencies
    let dependencies: Vec<RuleDependencyJson> = rules
        .iter()
        .map(|rule| RuleDependencyJson {
            rule_id: rule.id.to_string(),
            depends_on: rule
                .input_attributes
                .iter()
                .map(|a| a.path.as_str().to_string())
                .collect(),
            produces: rule
                .output_attributes
                .iter()
                .map(|a| a.path.as_str().to_string())
                .collect(),
        })
        .collect();

    // Find missing inputs (inputs that no rule produces)
    let all_outputs: std::collections::HashSet<String> = rules
        .iter()
        .flat_map(|r| r.output_attributes.iter().map(|a| a.path.as_str().to_string()))
        .collect();

    let missing_inputs: Vec<MissingInputJson> = rules
        .iter()
        .flat_map(|rule| {
            rule.input_attributes.iter().filter_map(|input| {
                let path = input.path.as_str().to_string();
                if !all_outputs.contains(&path) {
                    Some(MissingInputJson {
                        rule_id: rule.id.to_string(),
                        input_path: path,
                    })
                } else {
                    None
                }
            })
        })
        .collect();

    // Generate simple text-based graphs (placeholders)
    let dot_graph = format!(
        "digraph ExecutionPlan {{ {} }}",
        rules
            .iter()
            .map(|r| format!("\"{}\"", r.id))
            .collect::<Vec<_>>()
            .join("; ")
    );

    let mermaid_graph = format!(
        "graph TD\n{}",
        rules
            .iter()
            .map(|r| format!("  {}[{}]", r.id, r.rule_type))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let ascii_graph = format!(
        "Execution Plan:\n{}",
        levels
            .iter()
            .map(|l| format!("  Level {}: {:?}", l.level, l.rule_ids))
            .collect::<Vec<_>>()
            .join("\n")
    );

    Ok(Json(ExecutionPlanResponse {
        levels,
        dependencies,
        missing_inputs,
        dot_graph,
        mermaid_graph,
        ascii_graph,
    }))
}

/// Validate a product's configuration
async fn validate_product(
    State(store): State<SharedStore>,
    Path(product_id): Path<String>,
) -> ApiResult<Json<ValidationResponse>> {
    let store = store.read().await;

    // Verify product exists (store uses String keys)
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    let mut errors: Vec<ValidationError> = Vec::new();
    let mut warnings: Vec<ValidationWarning> = Vec::new();

    // Check for missing required attributes in functionalities
    for functionality in store.functionalities.values().filter(|f| f.product_id == pid) {
        for required in &functionality.required_attributes {
            let has_concrete = store
                .attributes
                .values()
                .any(|a| a.abstract_path == required.abstract_path && a.product_id == pid);

            if !has_concrete {
                errors.push(ValidationError {
                    code: "MISSING_REQUIRED_ATTRIBUTE".to_string(),
                    message: format!(
                        "Functionality '{}' requires attribute '{}' which has no concrete implementation",
                        functionality.name,
                        required.abstract_path.as_str()
                    ),
                    path: Some(required.abstract_path.as_str().to_string()),
                    severity: "error".to_string(),
                });
            }
        }
    }

    // Check for attributes without values (when value_type is Static)
    for attr in store.attributes.values().filter(|a| a.product_id == pid) {
        if attr.value.is_none() && attr.rule_id.is_none() {
            warnings.push(ValidationWarning {
                code: "ATTRIBUTE_NO_VALUE".to_string(),
                message: format!(
                    "Attribute '{}' has no value and no associated rule",
                    attr.path.as_str()
                ),
                path: Some(attr.path.as_str().to_string()),
                suggestion: Some("Add a static value or associate a rule".to_string()),
            });
        }
    }

    // Check for orphaned rules (rules with no outputs)
    for rule in store.rules.values().filter(|r| r.product_id == pid) {
        if rule.output_attributes.is_empty() {
            warnings.push(ValidationWarning {
                code: "RULE_NO_OUTPUTS".to_string(),
                message: format!("Rule '{}' has no output attributes", rule.id),
                path: None,
                suggestion: Some("Add output attributes to make this rule useful".to_string()),
            });
        }
    }

    let valid = errors.is_empty();

    Ok(Json(ValidationResponse {
        product_id: product_id.clone(),
        valid,
        errors,
        warnings,
    }))
}
