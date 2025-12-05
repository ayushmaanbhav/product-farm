//! REST handlers for Abstract and Concrete Attributes
//!
//! Provides HTTP endpoints for attribute management

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use product_farm_core::{
    AbstractAttribute, AbstractAttributeRelatedAttribute, AbstractAttributeTag,
    AbstractPath, Attribute, AttributeDisplayName,
    AttributeValueType, ConcretePath, DataTypeId, DisplayNameFormat, ProductId, Tag, Value,
};

use crate::store::SharedStore;

use super::error::{ApiError, ApiResult};
use super::types::*;

/// Create routes for attribute endpoints
pub fn routes() -> Router<SharedStore> {
    Router::new()
        // Abstract attribute routes
        .route(
            "/api/products/{product_id}/abstract-attributes",
            get(list_abstract_attributes).post(create_abstract_attribute),
        )
        .route(
            "/api/abstract-attributes/*path",
            get(get_abstract_attribute).delete(delete_abstract_attribute),
        )
        // Concrete attribute routes
        .route(
            "/api/products/{product_id}/attributes",
            get(list_attributes).post(create_attribute),
        )
        .route(
            "/api/attributes/*path",
            get(get_attribute).put(update_attribute).delete(delete_attribute),
        )
}

// =============================================================================
// ABSTRACT ATTRIBUTE HANDLERS
// =============================================================================

/// List all abstract attributes for a product
async fn list_abstract_attributes(
    State(store): State<SharedStore>,
    Path(product_id): Path<String>,
) -> ApiResult<Json<ListAbstractAttributesResponse>> {
    let store = store.read().await;

    // Verify product exists
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    // Collect abstract attributes for this product (single iteration)
    let attributes: Vec<AbstractAttributeResponse> = store
        .abstract_attributes
        .values()
        .filter(|a| a.product_id == pid)
        .map(|a| a.into())
        .collect();

    let total = attributes.len();

    Ok(Json(ListAbstractAttributesResponse {
        attributes,
        total,
    }))
}

/// Create a new abstract attribute
async fn create_abstract_attribute(
    State(store): State<SharedStore>,
    Path(product_id): Path<String>,
    Json(req): Json<CreateAbstractAttributeRequest>,
) -> ApiResult<Json<AbstractAttributeResponse>> {
    let mut store = store.write().await;

    // Verify product exists
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    // Build the abstract path using the product ID
    let abstract_path = AbstractPath::build(
        &product_id,
        &req.component_type,
        req.component_id.as_deref(),
        &req.attribute_name,
    );
    let path_key = abstract_path.as_str().to_string();

    // Check for duplicate
    if store.abstract_attributes.contains_key(&path_key) {
        return Err(ApiError::Conflict(format!(
            "Abstract attribute '{}' already exists",
            path_key
        )));
    }

    // Validate datatype exists
    if !store.datatypes.contains_key(&req.datatype_id) {
        return Err(ApiError::BadRequest(format!(
            "Datatype '{}' not found",
            req.datatype_id
        )));
    }

    let datatype_id: DataTypeId = req.datatype_id.clone().into();

    // Parse display names
    let display_names: Vec<AttributeDisplayName> = req
        .display_names
        .iter()
        .map(|dn| AttributeDisplayName::for_abstract(
            pid.clone(),
            abstract_path.clone(),
            dn.name.clone(),
            parse_display_format(&dn.format),
            dn.order_index,
        ))
        .collect();

    // Parse tags (tags is Vec<String>)
    let tags: Vec<AbstractAttributeTag> = req
        .tags
        .iter()
        .enumerate()
        .map(|(i, tag_name)| AbstractAttributeTag::new(
            abstract_path.clone(),
            Tag::new(tag_name),
            pid.clone(),
            i as i32,
        ))
        .collect();

    // No related attributes in the request type for now
    let related_attributes: Vec<AbstractAttributeRelatedAttribute> = Vec::new();

    // Create the abstract attribute
    let attr = AbstractAttribute::new(
        abstract_path.clone(),
        pid.clone(),
        req.component_type.clone(),
        datatype_id,
    );

    // Apply optional fields via builder pattern
    let mut attr = attr;
    if let Some(ref cid) = req.component_id {
        attr = attr.with_component_id(cid.clone());
    }
    if let Some(ref enum_name) = req.enum_name {
        attr = attr.with_enum(enum_name.clone());
    }
    if let Some(ref desc) = req.description {
        if !desc.is_empty() {
            attr = attr.with_description(desc.clone());
        }
    }
    if let Some(ref constraint) = req.constraint_expression {
        // Parse constraint as JSON - return error if invalid
        let constraint_value: serde_json::Value = serde_json::from_str(constraint)
            .map_err(|e| ApiError::bad_request(format!(
                "Invalid constraint expression JSON: {}",
                e
            )))?;
        attr = attr.with_constraint(constraint_value);
    }
    if req.immutable {
        attr = attr.immutable();
    }
    // Add display names, tags, and related attributes
    attr.display_names = display_names;
    attr.tags = tags;
    attr.related_attributes = related_attributes;

    let response = AbstractAttributeResponse::from(&attr);
    store.abstract_attributes.insert(path_key, attr);

    Ok(Json(response))
}

/// Get a specific abstract attribute by path
async fn get_abstract_attribute(
    State(store): State<SharedStore>,
    Path(path): Path<String>,
) -> ApiResult<Json<AbstractAttributeResponse>> {
    // Validate path format
    validate_path(&path)?;

    let store = store.read().await;

    store
        .abstract_attributes
        .get(&path)
        .map(|a| Json(a.into()))
        .ok_or_else(|| ApiError::NotFound(format!("Abstract attribute '{}' not found", path)))
}

/// Delete an abstract attribute
async fn delete_abstract_attribute(
    State(store): State<SharedStore>,
    Path(path): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // Validate path format
    validate_path(&path)?;

    let mut store = store.write().await;

    let abstract_path = AbstractPath::new(&path);

    // Check if attribute is immutable
    if let Some(attr) = store.abstract_attributes.get(&path) {
        if attr.immutable {
            return Err(ApiError::PreconditionFailed(format!(
                "Cannot delete immutable abstract attribute '{}'",
                path
            )));
        }
    }

    if store.abstract_attributes.remove(&path).is_some() {
        // Also remove any concrete attributes that reference this abstract attribute
        store
            .attributes
            .retain(|_, a| a.abstract_path != abstract_path);

        Ok(Json(serde_json::json!({ "deleted": true })))
    } else {
        Err(ApiError::NotFound(format!(
            "Abstract attribute '{}' not found",
            path
        )))
    }
}

// =============================================================================
// CONCRETE ATTRIBUTE HANDLERS
// =============================================================================

/// List all concrete attributes for a product
async fn list_attributes(
    State(store): State<SharedStore>,
    Path(product_id): Path<String>,
) -> ApiResult<Json<ListAttributesResponse>> {
    let store = store.read().await;

    // Verify product exists
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    // Collect attributes for this product (single iteration)
    let attributes: Vec<AttributeResponse> = store
        .attributes
        .values()
        .filter(|a| a.product_id == pid)
        .map(|a| a.into())
        .collect();

    let total = attributes.len();

    Ok(Json(ListAttributesResponse {
        attributes,
        total,
    }))
}

/// Create a new concrete attribute
async fn create_attribute(
    State(store): State<SharedStore>,
    Path(product_id): Path<String>,
    Json(req): Json<CreateAttributeRequest>,
) -> ApiResult<Json<AttributeResponse>> {
    let mut store = store.write().await;

    // Verify product exists
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    // Validate abstract attribute exists
    let abstract_path = AbstractPath::new(&req.abstract_path);

    if !store.abstract_attributes.contains_key(&req.abstract_path) {
        return Err(ApiError::BadRequest(format!(
            "Abstract attribute '{}' not found",
            req.abstract_path
        )));
    }

    // Build the concrete path
    let path = ConcretePath::build(
        &product_id,
        &req.component_type,
        &req.component_id,
        &req.attribute_name,
    );
    let path_key = path.as_str().to_string();

    // Check for duplicate
    if store.attributes.contains_key(&path_key) {
        return Err(ApiError::Conflict(format!(
            "Attribute '{}' already exists",
            path_key
        )));
    }

    // Parse value type and create appropriate attribute
    let value_type = parse_value_type(&req.value_type);
    let value = req.value.as_ref().map(Value::from);

    // Create the attribute based on value type
    let attr = match value_type {
        AttributeValueType::FixedValue => {
            if let Some(val) = value {
                Attribute::new_fixed_value(path.clone(), abstract_path, pid, val)
            } else {
                Attribute::new_just_definition(path.clone(), abstract_path, pid)
            }
        }
        AttributeValueType::RuleDriven => {
            // For rule-driven, we'll need a rule_id which should come from elsewhere
            // For now, create as just_definition
            Attribute::new_just_definition(path.clone(), abstract_path, pid)
        }
        AttributeValueType::JustDefinition => {
            Attribute::new_just_definition(path.clone(), abstract_path, pid)
        }
    };

    let response = AttributeResponse::from(&attr);
    store.attributes.insert(path_key, attr);

    Ok(Json(response))
}

/// Get a specific concrete attribute by path
async fn get_attribute(
    State(store): State<SharedStore>,
    Path(path): Path<String>,
) -> ApiResult<Json<AttributeResponse>> {
    // Validate path format
    validate_path(&path)?;

    let store = store.read().await;

    store
        .attributes
        .get(&path)
        .map(|a| Json(a.into()))
        .ok_or_else(|| ApiError::NotFound(format!("Attribute '{}' not found", path)))
}

/// Update a concrete attribute
async fn update_attribute(
    State(store): State<SharedStore>,
    Path(path): Path<String>,
    Json(req): Json<UpdateAttributeRequest>,
) -> ApiResult<Json<AttributeResponse>> {
    // Validate path format
    validate_path(&path)?;

    let mut store = store.write().await;

    // Check if the abstract attribute is immutable first
    if let Some(attr) = store.attributes.get(&path) {
        let abstract_key = attr.abstract_path.as_str().to_string();
        if let Some(abstract_attr) = store.abstract_attributes.get(&abstract_key) {
            if abstract_attr.immutable {
                return Err(ApiError::PreconditionFailed(format!(
                    "Cannot update attribute '{}' - abstract attribute is immutable",
                    path
                )));
            }
        }
    }

    let attr = store
        .attributes
        .get_mut(&path)
        .ok_or_else(|| ApiError::NotFound(format!("Attribute '{}' not found", path)))?;

    // Update fields
    if let Some(value) = &req.value {
        attr.value = Some(Value::from(value));
    }

    let response = AttributeResponse::from(&*attr);
    Ok(Json(response))
}

/// Delete a concrete attribute
async fn delete_attribute(
    State(store): State<SharedStore>,
    Path(path): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // Validate path format
    validate_path(&path)?;

    let mut store = store.write().await;

    // Check if the abstract attribute is immutable
    if let Some(attr) = store.attributes.get(&path) {
        let abstract_key = attr.abstract_path.as_str().to_string();
        if let Some(abstract_attr) = store.abstract_attributes.get(&abstract_key) {
            if abstract_attr.immutable {
                return Err(ApiError::PreconditionFailed(format!(
                    "Cannot delete attribute '{}' - abstract attribute is immutable",
                    path
                )));
            }
        }
    }

    if store.attributes.remove(&path).is_some() {
        Ok(Json(serde_json::json!({ "deleted": true })))
    } else {
        Err(ApiError::NotFound(format!(
            "Attribute '{}' not found",
            path
        )))
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Validate path format to prevent path traversal and DoS attacks
fn validate_path(path: &str) -> ApiResult<()> {
    // Check length (from types.rs constants)
    if path.len() > MAX_PATH_LENGTH {
        return Err(ApiError::bad_request(format!(
            "Path too long ({} > {} characters)",
            path.len(),
            MAX_PATH_LENGTH
        )));
    }

    // Check for empty path
    if path.is_empty() {
        return Err(ApiError::bad_request("Path cannot be empty"));
    }

    // Check for path traversal attempts
    if path.contains("..") {
        return Err(ApiError::bad_request("Invalid path: contains '..'"));
    }

    // Check for null bytes (security risk)
    if path.contains('\0') {
        return Err(ApiError::bad_request("Invalid path: contains null byte"));
    }

    // Validate characters: allow alphanumeric, slashes, colons, dashes, underscores, dots (single)
    let valid = path.chars().all(|c| {
        c.is_alphanumeric() || matches!(c, '/' | ':' | '-' | '_' | '.')
    });
    if !valid {
        return Err(ApiError::bad_request(
            "Invalid path: contains invalid characters",
        ));
    }

    Ok(())
}

fn parse_display_format(format: &str) -> DisplayNameFormat {
    match format.to_uppercase().as_str() {
        "SYSTEM" => DisplayNameFormat::System,
        "ORIGINAL" => DisplayNameFormat::Original,
        "HUMAN" => DisplayNameFormat::Human,
        _ => DisplayNameFormat::System,
    }
}

fn parse_value_type(value_type: &str) -> AttributeValueType {
    match value_type.to_uppercase().as_str() {
        "FIXED_VALUE" | "STATIC" => AttributeValueType::FixedValue,
        "RULE_DRIVEN" | "DERIVED" => AttributeValueType::RuleDriven,
        "JUST_DEFINITION" | "INPUT" => AttributeValueType::JustDefinition,
        _ => AttributeValueType::FixedValue,
    }
}
