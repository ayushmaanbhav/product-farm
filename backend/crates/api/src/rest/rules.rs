//! REST handlers for Rules
//!
//! Provides HTTP endpoints for rule management

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use product_farm_core::{AbstractPath, ProductId, Rule, RuleBuilder};
use product_farm_rule_engine::RuleDag;

use crate::store::SharedStore;

use super::error::{ApiError, ApiResult};
use super::types::*;

/// Create routes for rule endpoints
pub fn routes() -> Router<SharedStore> {
    Router::new()
        .route(
            "/api/products/{product_id}/rules",
            get(list_rules).post(create_rule),
        )
        .route(
            "/api/rules/{rule_id}",
            get(get_rule).put(update_rule).delete(delete_rule),
        )
}

/// List all rules for a product
async fn list_rules(
    State(store): State<SharedStore>,
    Path(product_id): Path<String>,
) -> ApiResult<Json<ListRulesResponse>> {
    let store = store.read().await;

    // Verify product exists
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    // Collect rules for this product (single iteration)
    let rules: Vec<RuleResponse> = store
        .rules
        .values()
        .filter(|r| r.product_id == pid)
        .map(|r| r.into())
        .collect();

    let total = rules.len();

    Ok(Json(ListRulesResponse { rules, total }))
}

/// Create a new rule
async fn create_rule(
    State(store): State<SharedStore>,
    Path(product_id): Path<String>,
    Json(req): Json<CreateRuleRequest>,
) -> ApiResult<Json<RuleResponse>> {
    let mut store = store.write().await;

    // Verify product exists
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    // Build the rule using RuleBuilder
    let mut builder = RuleBuilder::new(product_id.as_str(), &req.rule_type);

    // Add input attributes
    for input_path in &req.input_attributes {
        builder = builder.input(AbstractPath::new(input_path));
    }

    // Add output attributes
    for output_path in &req.output_attributes {
        builder = builder.output(AbstractPath::new(output_path));
    }

    // Set display expression
    builder = builder.display(&req.display_expression);

    // Set a default expression (required to build)
    builder = builder.expression(serde_json::json!({"var": "input"}));

    // Set description if provided
    if let Some(desc) = &req.description {
        builder = builder.description(desc);
    }

    // Set order index (defaults to 0 via serde default)
    if req.order_index != 0 {
        builder = builder.order(req.order_index);
    }

    // Build the rule (returns Result<Rule, CoreError>)
    let rule = builder.build()
        .map_err(|e| ApiError::BadRequest(format!("Failed to build rule: {}", e)))?;
    let rule_key = rule.id.to_string();

    // Validate that adding this rule doesn't create a cycle in the dependency graph
    let pid = ProductId::new(&product_id);
    validate_no_cycles(&store, &pid, &rule, None)?;

    let response = RuleResponse::from(&rule);
    store.rules.insert(rule_key, rule);

    Ok(Json(response))
}

/// Get a specific rule by ID
async fn get_rule(
    State(store): State<SharedStore>,
    Path(rule_id): Path<String>,
) -> ApiResult<Json<RuleResponse>> {
    let store = store.read().await;

    store
        .rules
        .get(&rule_id)
        .map(|r| Json(r.into()))
        .ok_or_else(|| ApiError::NotFound(format!("Rule '{}' not found", rule_id)))
}

/// Update a rule
async fn update_rule(
    State(store): State<SharedStore>,
    Path(rule_id): Path<String>,
    Json(req): Json<UpdateRuleRequest>,
) -> ApiResult<Json<RuleResponse>> {
    let mut store = store.write().await;

    // Check if we're modifying attributes that could affect the dependency graph
    let modifies_dependencies = req.input_attributes.is_some() || req.output_attributes.is_some();

    // Get the original rule (clone if we need to validate/rollback)
    let original_rule = store
        .rules
        .get(&rule_id)
        .cloned()
        .ok_or_else(|| ApiError::NotFound(format!("Rule '{}' not found", rule_id)))?;

    let product_id = original_rule.product_id.clone();

    // Apply updates in a scope so mutable borrow ends before validation
    let updated_rule = {
        let rule = store
            .rules
            .get_mut(&rule_id)
            .ok_or_else(|| ApiError::NotFound(format!("Rule '{}' not found", rule_id)))?;

        // Update fields
        if let Some(rule_type) = req.rule_type {
            rule.rule_type = rule_type;
        }
        if let Some(display_expression) = req.display_expression {
            rule.display_expression = display_expression;
        }
        if let Some(description) = req.description {
            rule.description = Some(description).filter(|s| !s.is_empty());
        }
        if let Some(enabled) = req.enabled {
            rule.enabled = enabled;
        }
        if let Some(order_index) = req.order_index {
            rule.order_index = order_index;
        }

        // Update input attributes if provided
        if let Some(input_attrs) = req.input_attributes {
            let rule_id_clone = rule.id.clone();
            rule.input_attributes = input_attrs
                .iter()
                .enumerate()
                .map(|(i, attr_path)| {
                    product_farm_core::RuleInputAttribute::new(
                        rule_id_clone.clone(),
                        AbstractPath::new(attr_path),
                        i as i32,
                    )
                })
                .collect();
        }

        // Update output attributes if provided
        if let Some(output_attrs) = req.output_attributes {
            let rule_id_clone = rule.id.clone();
            rule.output_attributes = output_attrs
                .iter()
                .enumerate()
                .map(|(i, attr_path)| {
                    product_farm_core::RuleOutputAttribute::new(
                        rule_id_clone.clone(),
                        AbstractPath::new(attr_path),
                        i as i32,
                    )
                })
                .collect();
        }

        // Update timestamp
        rule.updated_at = chrono::Utc::now();

        // Clone updated rule for validation and response
        rule.clone()
    }; // Mutable borrow of store.rules ends here

    // If we modified dependencies, validate no cycles were introduced
    if modifies_dependencies {
        if let Err(e) = validate_no_cycles(&store, &product_id, &updated_rule, Some(&rule_id)) {
            // Restore original rule on validation failure
            if let Some(r) = store.rules.get_mut(&rule_id) {
                *r = original_rule;
            }
            return Err(e);
        }
    }

    let response = RuleResponse::from(&updated_rule);
    Ok(Json(response))
}

/// Delete a rule
async fn delete_rule(
    State(store): State<SharedStore>,
    Path(rule_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let mut store = store.write().await;

    if let Some(removed_rule) = store.rules.remove(&rule_id) {
        let removed_rule_id = removed_rule.id;
        // Also clear rule_id from any attributes that reference this rule
        for attr in store.attributes.values_mut() {
            if attr.rule_id == Some(removed_rule_id.clone()) {
                attr.rule_id = None;
            }
        }

        Ok(Json(serde_json::json!({ "deleted": true })))
    } else {
        Err(ApiError::NotFound(format!("Rule '{}' not found", rule_id)))
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Validate that adding or updating a rule doesn't create a cycle in the dependency graph.
///
/// # Arguments
/// * `store` - The current store state
/// * `product_id` - The product the rule belongs to
/// * `new_rule` - The new or updated rule to validate
/// * `exclude_rule_key` - Optional rule key to exclude (for updates - the original rule)
fn validate_no_cycles(
    store: &crate::store::EntityStore,
    product_id: &ProductId,
    new_rule: &Rule,
    exclude_rule_key: Option<&str>,
) -> ApiResult<()> {
    // Collect all existing rules for this product
    let mut rules_for_product: Vec<Rule> = store
        .rules
        .iter()
        .filter(|(key, rule)| {
            rule.product_id == *product_id
                && exclude_rule_key.map(|k| k != key.as_str()).unwrap_or(true)
        })
        .map(|(_, rule)| rule.clone())
        .collect();

    // Add the new/updated rule
    rules_for_product.push(new_rule.clone());

    // Build the DAG - this will detect cycles
    match RuleDag::from_rules(&rules_for_product) {
        Ok(_) => Ok(()),
        Err(e) => Err(ApiError::bad_request(format!(
            "Rule would create a cyclic dependency: {}",
            e
        ))),
    }
}
