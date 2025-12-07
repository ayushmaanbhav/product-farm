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

use crate::config::limits::MAX_COMPONENT_LENGTH;
use super::error::{ApiError, ApiResult};
use super::types::*;
use super::AppState;

/// Create routes for rule endpoints
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/products/:product_id/rules",
            get(list_rules).post(create_rule),
        )
        .route(
            "/api/rules/:rule_id",
            get(get_rule).put(update_rule).delete(delete_rule),
        )
}

/// List all rules for a product
async fn list_rules(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> ApiResult<Json<ListRulesResponse>> {
    let store = state.store.read().await;

    // Verify product exists
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    // Collect rules for this product, sorted by order_index
    let mut rules: Vec<&Rule> = store
        .rules
        .values()
        .filter(|r| r.product_id == pid)
        .collect();

    // Sort by order_index ascending
    rules.sort_by_key(|r| r.order_index);

    let rules: Vec<RuleResponse> = rules.iter().map(|r| (*r).into()).collect();

    let total_count = rules.len() as i32;

    Ok(Json(ListRulesResponse {
        items: rules,
        next_page_token: String::new(),
        total_count,
    }))
}

/// Create a new rule
async fn create_rule(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
    Json(req): Json<CreateRuleRequest>,
) -> ApiResult<Json<RuleResponse>> {
    // Validate input
    req.validate_input()?;

    let mut store = state.store.write().await;

    // Verify product exists
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    // Validate path format for all input attributes
    for input_path in &req.input_attributes {
        validate_short_path_format(input_path, "input")?;
        validate_attribute_exists(&store, &product_id, input_path, "input")?;
    }

    // Validate path format for all output attributes
    for output_path in &req.output_attributes {
        validate_short_path_format(output_path, "output")?;
        validate_attribute_exists(&store, &product_id, output_path, "output")?;
    }

    // Validate no self-referential rules (input attribute = output attribute)
    for input_path in &req.input_attributes {
        for output_path in &req.output_attributes {
            // Normalize paths for comparison (both could be dot or slash separated)
            let input_normalized = input_path.replace('.', "/");
            let output_normalized = output_path.replace('.', "/");
            if input_normalized == output_normalized {
                return Err(ApiError::BadRequest(format!(
                    "Self-referential rule not allowed: '{}' is both input and output",
                    input_path
                )));
            }
        }
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

    // Parse and set the actual expression from the request
    let expression: serde_json::Value = serde_json::from_str(&req.expression_json)
        .map_err(|e| ApiError::BadRequest(format!("Invalid expression JSON: {}", e)))?;
    builder = builder.expression(expression);

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
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> ApiResult<Json<RuleResponse>> {
    let store = state.store.read().await;

    store
        .rules
        .get(&rule_id)
        .map(|r| Json(r.into()))
        .ok_or_else(|| ApiError::NotFound(format!("Rule '{}' not found", rule_id)))
}

/// Update a rule
async fn update_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
    Json(req): Json<UpdateRuleRequest>,
) -> ApiResult<Json<RuleResponse>> {
    let mut store = state.store.write().await;

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

        // Update expression_json if provided
        if let Some(expression_json) = &req.expression_json {
            // Validate it's valid JSON
            let _: serde_json::Value = serde_json::from_str(expression_json)
                .map_err(|e| ApiError::BadRequest(format!("Invalid expression JSON: {}", e)))?;
            rule.compiled_expression = expression_json.clone();
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
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let mut store = state.store.write().await;

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

/// Validate that a short-form attribute path has valid format.
///
/// Valid formats:
/// - componentType/attributeName (2 parts with slash)
/// - componentType/componentId/attributeName (3 parts with slash)
/// - componentType.attributeName (2 parts with dot) - also accepted
/// - componentType.componentId.attributeName (3 parts with dot) - also accepted
///
/// Components must:
/// - Not be empty
/// - Contain only alphanumeric characters, hyphens, underscores, and dots (for attribute names)
/// - Not exceed maximum length (64 chars per component)
fn validate_short_path_format(short_path: &str, attr_type: &str) -> ApiResult<()> {
    // Check for empty path
    if short_path.is_empty() {
        return Err(ApiError::BadRequest(format!(
            "{} attribute path cannot be empty",
            attr_type
        )));
    }

    // Check for path traversal attempts
    if short_path.contains("..") {
        return Err(ApiError::BadRequest(format!(
            "{} attribute path '{}' contains invalid sequence '..'",
            attr_type, short_path
        )));
    }

    // Determine separator (slash preferred, dot also accepted)
    let separator = if short_path.contains('/') { '/' } else { '.' };
    let parts: Vec<&str> = short_path.split(separator).collect();

    // Validate number of parts
    if parts.len() < 2 || parts.len() > 3 {
        return Err(ApiError::BadRequest(format!(
            "Invalid {} attribute path '{}': expected format 'componentType/attributeName' or 'componentType/componentId/attributeName'",
            attr_type, short_path
        )));
    }

    // Validate each component
    for (i, part) in parts.iter().enumerate() {
        let part_name = match (parts.len(), i) {
            (2, 0) => "componentType",
            (2, 1) => "attributeName",
            (3, 0) => "componentType",
            (3, 1) => "componentId",
            (3, 2) => "attributeName",
            _ => "component",
        };

        // Check for empty parts
        if part.is_empty() {
            return Err(ApiError::BadRequest(format!(
                "Invalid {} attribute path '{}': {} cannot be empty",
                attr_type, short_path, part_name
            )));
        }

        // Check length
        if part.len() > MAX_COMPONENT_LENGTH {
            return Err(ApiError::BadRequest(format!(
                "Invalid {} attribute path '{}': {} exceeds maximum length of {} characters",
                attr_type, short_path, part_name, MAX_COMPONENT_LENGTH
            )));
        }

        // Validate characters - allow alphanumeric, hyphens, underscores
        // For attribute names, also allow dots (for nested names like "loan.amount")
        let valid_chars = if i == parts.len() - 1 {
            // Attribute name - allow dots for nested names
            |c: char| c.is_alphanumeric() || c == '-' || c == '_' || c == '.'
        } else {
            // Component type/id - no dots allowed
            |c: char| c.is_alphanumeric() || c == '-' || c == '_'
        };

        if !part.chars().all(valid_chars) {
            return Err(ApiError::BadRequest(format!(
                "Invalid {} attribute path '{}': {} contains invalid characters",
                attr_type, short_path, part_name
            )));
        }
    }

    Ok(())
}

/// Validate that a short-form attribute path references an existing abstract attribute.
///
/// Supports both slash notation (loan/amount, loan/main/amount) and
/// dot notation (loan.amount, loan.main.amount).
///
/// # Arguments
/// * `store` - The current store state
/// * `product_id` - The product the attribute should belong to
/// * `short_path` - The short-form attribute path
/// * `attr_type` - Either "input" or "output" for error messages
fn validate_attribute_exists(
    store: &crate::store::EntityStore,
    product_id: &str,
    short_path: &str,
    attr_type: &str,
) -> ApiResult<()> {
    // Determine separator (dot or slash)
    let separator = if short_path.contains('/') { '/' } else { '.' };
    let parts: Vec<&str> = short_path.split(separator).collect();

    let (component_type, component_id, attribute_name) = match parts.len() {
        2 => (parts[0], None, parts[1]),
        3 => (parts[0], Some(parts[1]), parts[2]),
        _ => {
            return Err(ApiError::BadRequest(format!(
                "Invalid {} attribute path '{}': expected format 'componentType/attributeName' or 'componentType/componentId/attributeName'",
                attr_type, short_path
            )));
        }
    };

    // Build the full abstract path key
    let full_path = AbstractPath::build(product_id, component_type, component_id, attribute_name);
    let path_key = full_path.as_str().to_string();

    // Check if abstract attribute exists
    if !store.abstract_attributes.contains_key(&path_key) {
        return Err(ApiError::BadRequest(format!(
            "{} attribute '{}' not found in product '{}'",
            attr_type, short_path, product_id
        )));
    }

    Ok(())
}
