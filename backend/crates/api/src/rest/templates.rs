//! REST handlers for Template Enumerations
//!
//! Provides HTTP endpoints for template enumeration management

use std::collections::BTreeSet;

use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use product_farm_core::{ProductTemplateEnumeration, TemplateEnumerationId, TemplateType};

use crate::store::SharedStore;

use super::error::{ApiError, ApiResult};
use super::types::*;

/// Create routes for template enumeration endpoints
pub fn routes() -> Router<SharedStore> {
    Router::new()
        .route(
            "/api/template-enumerations",
            get(list_enumerations).post(create_enumeration),
        )
        .route(
            "/api/template-enumerations/{enum_id}",
            get(get_enumeration)
                .put(update_enumeration)
                .delete(delete_enumeration),
        )
        .route(
            "/api/template-enumerations/{enum_id}/values",
            post(add_value),
        )
        .route(
            "/api/template-enumerations/{enum_id}/values/{value}",
            delete(remove_value),
        )
}

/// List all template enumerations
async fn list_enumerations(
    State(store): State<SharedStore>,
) -> ApiResult<Json<ListEnumerationsResponse>> {
    let store = store.read().await;

    let enumerations: Vec<EnumerationResponse> =
        store.enumerations.values().map(|e| e.into()).collect();

    Ok(Json(ListEnumerationsResponse {
        enumerations,
        total: store.enumerations.len(),
    }))
}

/// Create a new template enumeration
async fn create_enumeration(
    State(store): State<SharedStore>,
    Json(req): Json<CreateEnumerationRequest>,
) -> ApiResult<Json<EnumerationResponse>> {
    let mut store = store.write().await;

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
    State(store): State<SharedStore>,
    Path(enum_id): Path<String>,
) -> ApiResult<Json<EnumerationResponse>> {
    let store = store.read().await;

    store
        .enumerations
        .get(&enum_id)
        .map(|e| Json(e.into()))
        .ok_or_else(|| ApiError::NotFound(format!("Enumeration '{}' not found", enum_id)))
}

/// Update an enumeration
async fn update_enumeration(
    State(store): State<SharedStore>,
    Path(enum_id): Path<String>,
    Json(req): Json<UpdateEnumerationRequest>,
) -> ApiResult<Json<EnumerationResponse>> {
    let mut store = store.write().await;

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
    State(store): State<SharedStore>,
    Path(enum_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let mut store = store.write().await;

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
    State(store): State<SharedStore>,
    Path(enum_id): Path<String>,
    Json(req): Json<AddEnumerationValueRequest>,
) -> ApiResult<Json<EnumerationResponse>> {
    let mut store = store.write().await;

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
    State(store): State<SharedStore>,
    Path((enum_id, value)): Path<(String, String)>,
) -> ApiResult<Json<EnumerationResponse>> {
    let mut store = store.write().await;

    let enumeration = store
        .enumerations
        .get_mut(&enum_id)
        .ok_or_else(|| ApiError::NotFound(format!("Enumeration '{}' not found", enum_id)))?;

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
