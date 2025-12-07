//! REST handlers for Template Enumerations
//!
//! Provides HTTP endpoints for template enumeration management

use std::collections::BTreeSet;

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use product_farm_core::{ProductTemplateEnumeration, TemplateEnumerationId, TemplateType};
use serde::Deserialize;

/// Query parameters for listing enumerations
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListEnumerationsQuery {
    pub template_type: Option<String>,
    pub page_size: Option<i32>,
    pub page_token: Option<String>,
}

use super::error::{ApiError, ApiResult};
use super::types::*;
use super::AppState;

/// Create routes for template enumeration endpoints
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/template-enumerations",
            get(list_enumerations).post(create_enumeration),
        )
        .route(
            "/api/template-enumerations/:enum_id",
            get(get_enumeration)
                .put(update_enumeration)
                .delete(delete_enumeration),
        )
        .route(
            "/api/template-enumerations/:enum_id/values",
            post(add_value),
        )
        .route(
            "/api/template-enumerations/:enum_id/values/:value",
            delete(remove_value),
        )
}

/// List all template enumerations
async fn list_enumerations(
    State(state): State<AppState>,
    Query(query): Query<ListEnumerationsQuery>,
) -> ApiResult<Json<ListEnumerationsResponse>> {
    let store = state.store.read().await;

    // Collect filtered enumerations
    let mut enumerations: Vec<EnumerationResponse> = store
        .enumerations
        .values()
        .filter(|e| {
            query.template_type.as_ref().map_or(true, |tt| e.template_type.as_str() == tt)
        })
        .map(|e| e.into())
        .collect();

    // Sort by id for consistent ordering
    enumerations.sort_by(|a, b| a.id.cmp(&b.id));

    let total_count = enumerations.len() as i32;

    // Apply pagination
    let page_size = query.page_size.unwrap_or(100) as usize;
    let offset = query.page_token.as_ref()
        .and_then(|t| t.parse::<usize>().ok())
        .unwrap_or(0);

    let paginated: Vec<EnumerationResponse> = enumerations
        .into_iter()
        .skip(offset)
        .take(page_size)
        .collect();

    let next_page_token = if offset + page_size < total_count as usize {
        (offset + page_size).to_string()
    } else {
        String::new()
    };

    Ok(Json(ListEnumerationsResponse {
        items: paginated,
        next_page_token,
        total_count,
    }))
}

/// Create a new template enumeration
async fn create_enumeration(
    State(state): State<AppState>,
    Json(req): Json<CreateEnumerationRequest>,
) -> ApiResult<Json<EnumerationResponse>> {
    // Validate input
    req.validate_input()?;

    // Validate template type length
    const MAX_TEMPLATE_TYPE_LENGTH: usize = 64;
    if req.template_type.len() > MAX_TEMPLATE_TYPE_LENGTH {
        return Err(ApiError::BadRequest(format!(
            "template_type exceeds maximum length of {} characters",
            MAX_TEMPLATE_TYPE_LENGTH
        )));
    }

    let mut store = state.store.write().await;

    // Generate key from name (store uses String keys)
    let enum_key = req.name.clone();

    // Check for duplicate
    if store.enumerations.contains_key(&enum_key) {
        return Err(ApiError::Conflict(format!(
            "Enumeration '{}' already exists",
            req.name
        )));
    }

    // Convert Vec<String> to BTreeSet<String>
    let values: BTreeSet<String> = req.values.into_iter().collect();

    // Create the enumeration
    let enumeration = ProductTemplateEnumeration {
        id: TemplateEnumerationId::new(&req.name),
        name: req.name,
        template_type: TemplateType::new(&req.template_type),
        values,
        description: req.description,
    };

    let response = EnumerationResponse::from(&enumeration);
    store.enumerations.insert(enum_key, enumeration);

    Ok(Json(response))
}

/// Get a specific enumeration by ID
async fn get_enumeration(
    State(state): State<AppState>,
    Path(enum_id): Path<String>,
) -> ApiResult<Json<EnumerationResponse>> {
    let store = state.store.read().await;

    store
        .enumerations
        .get(&enum_id)
        .map(|e| Json(e.into()))
        .ok_or_else(|| ApiError::NotFound(format!("Enumeration '{}' not found", enum_id)))
}

/// Update an enumeration
async fn update_enumeration(
    State(state): State<AppState>,
    Path(enum_id): Path<String>,
    Json(req): Json<UpdateEnumerationRequest>,
) -> ApiResult<Json<EnumerationResponse>> {
    let mut store = state.store.write().await;

    let enumeration = store
        .enumerations
        .get_mut(&enum_id)
        .ok_or_else(|| ApiError::NotFound(format!("Enumeration '{}' not found", enum_id)))?;

    // Update fields (UpdateEnumerationRequest only has description)
    if let Some(description) = req.description {
        enumeration.description = Some(description).filter(|s| !s.is_empty());
    }

    let response = EnumerationResponse::from(&*enumeration);
    Ok(Json(response))
}

/// Delete an enumeration
async fn delete_enumeration(
    State(state): State<AppState>,
    Path(enum_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let mut store = state.store.write().await;

    // Check if any abstract attributes use this enumeration
    let in_use = store
        .abstract_attributes
        .values()
        .any(|a| a.enum_name.as_ref() == Some(&enum_id));

    if in_use {
        return Err(ApiError::PreconditionFailed(format!(
            "Cannot delete enumeration '{}' - it is in use by abstract attributes",
            enum_id
        )));
    }

    if store.enumerations.remove(&enum_id).is_some() {
        Ok(Json(serde_json::json!({ "deleted": true })))
    } else {
        Err(ApiError::NotFound(format!(
            "Enumeration '{}' not found",
            enum_id
        )))
    }
}

/// Add a value to an enumeration
async fn add_value(
    State(state): State<AppState>,
    Path(enum_id): Path<String>,
    Json(req): Json<AddEnumerationValueRequest>,
) -> ApiResult<Json<EnumerationResponse>> {
    // Validate that value is not empty
    if req.value.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "Enumeration value cannot be empty".to_string()
        ));
    }

    let mut store = state.store.write().await;

    let enumeration = store
        .enumerations
        .get_mut(&enum_id)
        .ok_or_else(|| ApiError::NotFound(format!("Enumeration '{}' not found", enum_id)))?;

    // Check if value already exists
    if enumeration.values.contains(&req.value) {
        return Err(ApiError::Conflict(format!(
            "Value '{}' already exists in enumeration '{}'",
            req.value, enum_id
        )));
    }

    enumeration.values.insert(req.value);

    let response = EnumerationResponse::from(&*enumeration);
    Ok(Json(response))
}

/// Remove a value from an enumeration
async fn remove_value(
    State(state): State<AppState>,
    Path((enum_id, value)): Path<(String, String)>,
) -> ApiResult<Json<EnumerationResponse>> {
    let mut store = state.store.write().await;

    let enumeration = store
        .enumerations
        .get_mut(&enum_id)
        .ok_or_else(|| ApiError::NotFound(format!("Enumeration '{}' not found", enum_id)))?;

    // Check if this is the last value
    if enumeration.values.len() == 1 {
        return Err(ApiError::BadRequest(
            "Cannot remove the last value from an enumeration".to_string()
        ));
    }

    // Try to remove the value
    if !enumeration.values.remove(&value) {
        return Err(ApiError::NotFound(format!(
            "Value '{}' not found in enumeration '{}'",
            value, enum_id
        )));
    }

    let response = EnumerationResponse::from(&*enumeration);
    Ok(Json(response))
}
