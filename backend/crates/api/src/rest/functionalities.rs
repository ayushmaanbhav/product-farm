//! REST handlers for Product Functionalities
//!
//! Provides HTTP endpoints for functionality management

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use product_farm_core::{
    validation, AbstractPath, FunctionalityId, FunctionalityRequiredAttribute,
    ProductFunctionalityStatus, ProductFunctionality, ProductId,
};

use super::error::{ApiError, ApiResult};
use super::types::*;
use super::AppState;

/// Create routes for functionality endpoints
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/products/:product_id/functionalities",
            get(list_functionalities).post(create_functionality),
        )
        .route(
            "/api/products/:product_id/functionalities/:name",
            get(get_functionality)
                .put(update_functionality)
                .delete(delete_functionality),
        )
        .route(
            "/api/products/:product_id/functionalities/:name/activate",
            axum::routing::post(activate_functionality),
        )
        .route(
            "/api/products/:product_id/functionalities/:name/deactivate",
            axum::routing::post(deactivate_functionality),
        )
        .route(
            "/api/products/:product_id/functionalities/:name/abstract-attributes",
            get(list_functionality_abstract_attributes),
        )
        .route(
            "/api/products/:product_id/functionalities/:name/rules",
            get(list_functionality_rules),
        )
        .route(
            "/api/products/:product_id/functionalities/:name/submit",
            axum::routing::post(submit_functionality),
        )
        .route(
            "/api/products/:product_id/functionalities/:name/approve",
            axum::routing::post(approve_functionality),
        )
}

/// List all functionalities for a product
async fn list_functionalities(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> ApiResult<Json<ListFunctionalitiesResponse>> {
    let store = state.store.read().await;

    // Verify product exists (store uses String keys)
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    // Collect functionalities for this product (single iteration)
    let functionalities: Vec<FunctionalityResponse> = store
        .functionalities
        .values()
        .filter(|f| f.product_id == pid)
        .map(|f| f.into())
        .collect();

    let total_count = functionalities.len() as i32;

    Ok(Json(ListFunctionalitiesResponse {
        items: functionalities,
        next_page_token: String::new(),
        total_count,
    }))
}

/// Create a new functionality
async fn create_functionality(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
    Json(req): Json<CreateFunctionalityRequest>,
) -> ApiResult<Json<FunctionalityResponse>> {
    // Validate input
    req.validate_input()?;

    let mut store = state.store.write().await;

    // Verify product exists (store uses String keys)
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    // Validate functionality name before using in key construction
    if !validation::is_valid_functionality_name(&req.name) {
        return Err(ApiError::BadRequest(format!(
            "Invalid functionality name '{}'. Must match pattern: lowercase letters and hyphens only, 1-51 chars",
            req.name
        )));
    }

    // Generate functionality key (store uses String keys)
    // Safe because we validated both product_id (via product lookup) and name (above)
    let func_key = format!("{}:{}", product_id, req.name);
    let func_id = FunctionalityId::new(&func_key);

    // Check for duplicate
    if store.functionalities.contains_key(&func_key) {
        return Err(ApiError::Conflict(format!(
            "Functionality '{}' already exists for product '{}'",
            req.name, product_id
        )));
    }

    // Parse required attributes (description is String, no order_index in input)
    let required_attributes: Vec<FunctionalityRequiredAttribute> = req
        .required_attributes
        .iter()
        .enumerate()
        .map(|(i, attr)| {
            let abstract_path = AbstractPath::new(&attr.abstract_path);
            FunctionalityRequiredAttribute::new(
                func_id.clone(),
                abstract_path,
                attr.description.clone(),
                i as i32,
            )
        })
        .collect();

    // Create the functionality (description is String, immutable is bool)
    let mut functionality = ProductFunctionality::new(
        func_id.clone(),
        req.name,
        pid,
        req.description.clone(),
    );
    functionality.immutable = req.immutable;
    functionality.required_attributes = required_attributes;

    let response = FunctionalityResponse::from(&functionality);
    store.functionalities.insert(func_key, functionality);

    Ok(Json(response))
}

/// Get a specific functionality by name
async fn get_functionality(
    State(state): State<AppState>,
    Path((product_id, name)): Path<(String, String)>,
) -> ApiResult<Json<FunctionalityResponse>> {
    let store = state.store.read().await;

    let func_key = format!("{}:{}", product_id, name);

    store
        .functionalities
        .get(&func_key)
        .map(|f| Json(f.into()))
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "Functionality '{}' not found for product '{}'",
                name, product_id
            ))
        })
}

/// Update a functionality
async fn update_functionality(
    State(state): State<AppState>,
    Path((product_id, name)): Path<(String, String)>,
    Json(req): Json<UpdateFunctionalityRequest>,
) -> ApiResult<Json<FunctionalityResponse>> {
    let mut store = state.store.write().await;

    let func_key = format!("{}:{}", product_id, name);
    let func_id = FunctionalityId::new(&func_key);

    let functionality = store.functionalities.get_mut(&func_key).ok_or_else(|| {
        ApiError::NotFound(format!(
            "Functionality '{}' not found for product '{}'",
            name, product_id
        ))
    })?;

    // Check if immutable
    if functionality.immutable {
        return Err(ApiError::PreconditionFailed(format!(
            "Cannot update immutable functionality '{}'",
            name
        )));
    }

    // Update fields (UpdateFunctionalityRequest has no name field)
    if let Some(description) = req.description {
        if !description.is_empty() {
            functionality.description = description;
        }
    }

    // Update required attributes if provided
    if let Some(required_attrs) = req.required_attributes {
        functionality.required_attributes = required_attrs
            .iter()
            .enumerate()
            .map(|(i, attr)| {
                let abstract_path = AbstractPath::new(&attr.abstract_path);
                FunctionalityRequiredAttribute::new(
                    func_id.clone(),
                    abstract_path,
                    attr.description.clone(),
                    i as i32,
                )
            })
            .collect();
    }

    let response = FunctionalityResponse::from(&*functionality);
    Ok(Json(response))
}

/// Delete a functionality
async fn delete_functionality(
    State(state): State<AppState>,
    Path((product_id, name)): Path<(String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let mut store = state.store.write().await;

    let func_key = format!("{}:{}", product_id, name);

    // Check if immutable
    if let Some(functionality) = store.functionalities.get(&func_key) {
        if functionality.immutable {
            return Err(ApiError::PreconditionFailed(format!(
                "Cannot delete immutable functionality '{}'",
                name
            )));
        }
    }

    if store.functionalities.remove(&func_key).is_some() {
        Ok(Json(serde_json::json!({ "deleted": true })))
    } else {
        Err(ApiError::NotFound(format!(
            "Functionality '{}' not found for product '{}'",
            name, product_id
        )))
    }
}

/// Activate a functionality
async fn activate_functionality(
    State(state): State<AppState>,
    Path((product_id, name)): Path<(String, String)>,
) -> ApiResult<Json<FunctionalityResponse>> {
    let mut store = state.store.write().await;

    let func_key = format!("{}:{}", product_id, name);

    let functionality = store.functionalities.get_mut(&func_key).ok_or_else(|| {
        ApiError::NotFound(format!(
            "Functionality '{}' not found for product '{}'",
            name, product_id
        ))
    })?;

    functionality.status = ProductFunctionalityStatus::Active;

    let response = FunctionalityResponse::from(&*functionality);
    Ok(Json(response))
}

/// Deactivate a functionality
async fn deactivate_functionality(
    State(state): State<AppState>,
    Path((product_id, name)): Path<(String, String)>,
) -> ApiResult<Json<FunctionalityResponse>> {
    let mut store = state.store.write().await;

    let func_key = format!("{}:{}", product_id, name);

    let functionality = store.functionalities.get_mut(&func_key).ok_or_else(|| {
        ApiError::NotFound(format!(
            "Functionality '{}' not found for product '{}'",
            name, product_id
        ))
    })?;

    functionality.status = ProductFunctionalityStatus::Draft;

    let response = FunctionalityResponse::from(&*functionality);
    Ok(Json(response))
}

/// List abstract attributes required by a functionality
async fn list_functionality_abstract_attributes(
    State(state): State<AppState>,
    Path((product_id, name)): Path<(String, String)>,
) -> ApiResult<Json<ListAbstractAttributesResponse>> {
    let store = state.store.read().await;

    let func_key = format!("{}:{}", product_id, name);

    let functionality = store.functionalities.get(&func_key).ok_or_else(|| {
        ApiError::NotFound(format!(
            "Functionality '{}' not found for product '{}'",
            name, product_id
        ))
    })?;

    // Get the required attribute paths and convert short paths to full paths
    let required_paths: std::collections::HashSet<String> = functionality
        .required_attributes
        .iter()
        .map(|ra| {
            let short_path = ra.abstract_path.as_str();
            // Convert short path (loan.main.amount or loan/main/amount) to full path
            parse_short_path_to_full(&product_id, short_path)
        })
        .collect();

    // Fetch the actual abstract attributes
    let attributes: Vec<AbstractAttributeResponse> = store
        .abstract_attributes
        .values()
        .filter(|a| required_paths.contains(a.abstract_path.as_str()))
        .map(|a| a.into())
        .collect();

    let total_count = attributes.len() as i32;

    Ok(Json(ListAbstractAttributesResponse {
        items: attributes,
        next_page_token: String::new(),
        total_count,
    }))
}

/// Convert a short path (loan.main.amount or loan/main/amount) to full abstract path
fn parse_short_path_to_full(product_id: &str, short_path: &str) -> String {
    // Determine separator (dot or slash)
    let separator = if short_path.contains('/') { '/' } else { '.' };
    let parts: Vec<&str> = short_path.split(separator).collect();

    match parts.len() {
        2 => AbstractPath::build(product_id, parts[0], None, parts[1]).as_str().to_string(),
        3 => AbstractPath::build(product_id, parts[0], Some(parts[1]), parts[2]).as_str().to_string(),
        _ => short_path.to_string(), // Return as-is if not in expected format
    }
}

/// List rules that output to functionality-required attributes
async fn list_functionality_rules(
    State(state): State<AppState>,
    Path((product_id, name)): Path<(String, String)>,
) -> ApiResult<Json<ListRulesResponse>> {
    let store = state.store.read().await;

    let func_key = format!("{}:{}", product_id, name);
    let pid = ProductId::new(&product_id);

    let functionality = store.functionalities.get(&func_key).ok_or_else(|| {
        ApiError::NotFound(format!(
            "Functionality '{}' not found for product '{}'",
            name, product_id
        ))
    })?;

    // Get the required attribute paths - convert to full format for comparison
    let required_full_paths: std::collections::HashSet<String> = functionality
        .required_attributes
        .iter()
        .map(|ra| {
            let short_path = ra.abstract_path.as_str();
            parse_short_path_to_full(&product_id, short_path)
        })
        .collect();

    // Find rules that output to any of these attributes
    let rules: Vec<RuleResponse> = store
        .rules
        .values()
        .filter(|r| {
            r.product_id == pid
                && r.output_attributes.iter().any(|oa| {
                    // Convert rule output path to full path for comparison
                    let rule_output_full = parse_short_path_to_full(&product_id, oa.path.as_str());
                    required_full_paths.contains(&rule_output_full)
                })
        })
        .map(|r| r.into())
        .collect();

    let total_count = rules.len() as i32;

    Ok(Json(ListRulesResponse {
        items: rules,
        next_page_token: String::new(),
        total_count,
    }))
}

/// Submit a functionality for approval
async fn submit_functionality(
    State(state): State<AppState>,
    Path((product_id, name)): Path<(String, String)>,
) -> ApiResult<Json<FunctionalityResponse>> {
    let mut store = state.store.write().await;

    let func_key = format!("{}:{}", product_id, name);

    let functionality = store.functionalities.get_mut(&func_key).ok_or_else(|| {
        ApiError::NotFound(format!(
            "Functionality '{}' not found for product '{}'",
            name, product_id
        ))
    })?;

    // Validate current status allows submission
    if functionality.status != ProductFunctionalityStatus::Draft {
        return Err(ApiError::PreconditionFailed(format!(
            "Cannot submit functionality '{}' with status {:?}. Only DRAFT functionalities can be submitted.",
            name, functionality.status
        )));
    }

    // Transition to PendingApproval
    functionality.status = ProductFunctionalityStatus::PendingApproval;

    let response = FunctionalityResponse::from(&*functionality);
    Ok(Json(response))
}

/// Approve or reject a functionality
async fn approve_functionality(
    State(state): State<AppState>,
    Path((product_id, name)): Path<(String, String)>,
    Json(req): Json<ApprovalRequest>,
) -> ApiResult<Json<FunctionalityResponse>> {
    let mut store = state.store.write().await;

    let func_key = format!("{}:{}", product_id, name);

    let functionality = store.functionalities.get_mut(&func_key).ok_or_else(|| {
        ApiError::NotFound(format!(
            "Functionality '{}' not found for product '{}'",
            name, product_id
        ))
    })?;

    // Validate current status allows approval/rejection
    if functionality.status != ProductFunctionalityStatus::PendingApproval {
        let action = if req.approved { "approve" } else { "reject" };
        return Err(ApiError::PreconditionFailed(format!(
            "Cannot {} functionality '{}' with status {:?}. Only PENDING_APPROVAL functionalities can be {}ed.",
            action, name, functionality.status, action
        )));
    }

    if req.approved {
        functionality.status = ProductFunctionalityStatus::Active;
    } else {
        functionality.status = ProductFunctionalityStatus::Draft;
    }

    let response = FunctionalityResponse::from(&*functionality);
    Ok(Json(response))
}
