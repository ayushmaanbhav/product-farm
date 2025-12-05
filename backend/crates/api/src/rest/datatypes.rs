//! REST handlers for Datatypes
//!
//! Provides HTTP endpoints for datatype management

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use product_farm_core::{DataType, DataTypeConstraints, DataTypeId, PrimitiveType};

use crate::store::SharedStore;

use super::error::{ApiError, ApiResult};
use super::types::*;

/// Create routes for datatype endpoints
pub fn routes() -> Router<SharedStore> {
    Router::new()
        .route("/api/datatypes", get(list_datatypes).post(create_datatype))
        .route(
            "/api/datatypes/:datatype_id",
            get(get_datatype).put(update_datatype).delete(delete_datatype),
        )
}

/// List all datatypes
async fn list_datatypes(State(store): State<SharedStore>) -> ApiResult<Json<ListDatatypesResponse>> {
    let store = store.read().await;

    let datatypes: Vec<DatatypeResponse> = store.datatypes.values().map(|d| d.into()).collect();

    Ok(Json(ListDatatypesResponse {
        items: datatypes,
        next_page_token: String::new(),
        total_count: store.datatypes.len() as i32,
    }))
}

/// Create a new datatype
async fn create_datatype(
    State(store): State<SharedStore>,
    Json(req): Json<CreateDatatypeRequest>,
) -> ApiResult<Json<DatatypeResponse>> {
    let mut store = store.write().await;

    // Check for duplicate (store uses String keys)
    if store.datatypes.contains_key(&req.id) {
        return Err(ApiError::Conflict(format!(
            "Datatype '{}' already exists",
            req.id
        )));
    }

    // Parse primitive type
    let primitive_type = parse_primitive_type(&req.primitive_type);

    // Parse constraints with safe type conversions
    let constraints = req.constraints.as_ref().map(|c| DataTypeConstraints {
        min: c.min,
        max: c.max,
        min_length: c.min_length.and_then(|v| usize::try_from(v).ok()),
        max_length: c.max_length.and_then(|v| usize::try_from(v).ok()),
        pattern: c.pattern.clone(),
        precision: c.precision.and_then(|v| u8::try_from(v).ok()),
        scale: c.scale.and_then(|v| u8::try_from(v).ok()),
        constraint_rule_expression: c.constraint_rule_expression.clone(),
        constraint_error_message: c.constraint_error_message.clone(),
    });

    // Create the datatype (id field expects DataTypeId)
    let datatype_id: DataTypeId = req.id.clone().into();
    let datatype = DataType {
        id: datatype_id,
        primitive_type,
        constraints,
        description: req.description,
    };

    let response = DatatypeResponse::from(&datatype);
    store.datatypes.insert(req.id, datatype);

    Ok(Json(response))
}

/// Get a specific datatype by ID
async fn get_datatype(
    State(store): State<SharedStore>,
    Path(datatype_id): Path<String>,
) -> ApiResult<Json<DatatypeResponse>> {
    let store = store.read().await;

    store
        .datatypes
        .get(&datatype_id)
        .map(|d| Json(d.into()))
        .ok_or_else(|| ApiError::NotFound(format!("Datatype '{}' not found", datatype_id)))
}

/// Update a datatype
async fn update_datatype(
    State(store): State<SharedStore>,
    Path(datatype_id): Path<String>,
    Json(req): Json<UpdateDatatypeRequest>,
) -> ApiResult<Json<DatatypeResponse>> {
    let mut store = store.write().await;

    let datatype = store
        .datatypes
        .get_mut(&datatype_id)
        .ok_or_else(|| ApiError::NotFound(format!("Datatype '{}' not found", datatype_id)))?;

    // Update fields with safe type conversions
    if let Some(constraints) = &req.constraints {
        datatype.constraints = Some(DataTypeConstraints {
            min: constraints.min,
            max: constraints.max,
            min_length: constraints.min_length.and_then(|v| usize::try_from(v).ok()),
            max_length: constraints.max_length.and_then(|v| usize::try_from(v).ok()),
            pattern: constraints.pattern.clone(),
            precision: constraints.precision.and_then(|v| u8::try_from(v).ok()),
            scale: constraints.scale.and_then(|v| u8::try_from(v).ok()),
            constraint_rule_expression: constraints.constraint_rule_expression.clone(),
            constraint_error_message: constraints.constraint_error_message.clone(),
        });
    }

    if let Some(description) = &req.description {
        datatype.description = Some(description.clone()).filter(|s| !s.is_empty());
    }

    let response = DatatypeResponse::from(&*datatype);
    Ok(Json(response))
}

/// Delete a datatype
async fn delete_datatype(
    State(store): State<SharedStore>,
    Path(datatype_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let mut store = store.write().await;

    // Check if any abstract attributes use this datatype
    let datatype_id_obj: DataTypeId = datatype_id.clone().into();
    let in_use = store
        .abstract_attributes
        .values()
        .any(|a| a.datatype_id == datatype_id_obj);

    if in_use {
        return Err(ApiError::PreconditionFailed(format!(
            "Cannot delete datatype '{}' - it is in use by abstract attributes",
            datatype_id
        )));
    }

    if store.datatypes.remove(&datatype_id).is_some() {
        Ok(Json(serde_json::json!({ "deleted": true })))
    } else {
        Err(ApiError::NotFound(format!(
            "Datatype '{}' not found",
            datatype_id
        )))
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn parse_primitive_type(primitive_type: &str) -> PrimitiveType {
    match primitive_type.to_uppercase().as_str() {
        "STRING" => PrimitiveType::String,
        "INTEGER" | "INT" => PrimitiveType::Int,
        "FLOAT" | "DOUBLE" => PrimitiveType::Float,
        "BOOLEAN" | "BOOL" => PrimitiveType::Bool,
        "DECIMAL" => PrimitiveType::Decimal,
        "DATE" | "DATETIME" => PrimitiveType::Datetime,
        "ARRAY" => PrimitiveType::Array,
        "OBJECT" => PrimitiveType::Object,
        "ENUM" => PrimitiveType::Enum,
        _ => PrimitiveType::String,
    }
}
