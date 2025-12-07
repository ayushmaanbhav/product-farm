//! REST handlers for Abstract and Concrete Attributes
//!
//! Provides HTTP endpoints for attribute management

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

/// Query parameters for listing abstract attributes
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListAbstractAttributesQuery {
    pub component_type: Option<String>,
    pub page_size: Option<i32>,
    pub page_token: Option<String>,
}
use product_farm_core::{
    AbstractAttribute, AbstractAttributeRelatedAttribute, AbstractAttributeTag,
    AbstractPath, Attribute, AttributeDisplayName,
    AttributeValueType, ConcretePath, DataType, DataTypeId,
    DisplayNameFormat, PrimitiveType, ProductId, RuleId, Tag, Value,
};

use crate::config::limits::{MAX_PATH_LENGTH, MAX_PATTERN_LENGTH, MAX_REGEX_SIZE};
use super::error::{ApiError, ApiResult};
use super::types::*;
use super::AppState;

/// Create routes for attribute endpoints
pub fn routes() -> Router<AppState> {
    Router::new()
        // Abstract attribute routes
        .route(
            "/api/products/:product_id/abstract-attributes",
            get(list_abstract_attributes).post(create_abstract_attribute),
        )
        .route(
            "/api/products/:product_id/abstract-attributes/by-component/:component_type",
            get(list_abstract_attributes_by_component),
        )
        .route(
            "/api/products/:product_id/abstract-attributes/by-tag/:tag",
            get(list_abstract_attributes_by_tag),
        )
        .route(
            "/api/abstract-attributes/*path",
            get(get_abstract_attribute).delete(delete_abstract_attribute),
        )
        // Concrete attribute routes
        .route(
            "/api/products/:product_id/attributes",
            get(list_attributes).post(create_attribute),
        )
        .route(
            "/api/products/:product_id/attributes/by-tag/:tag",
            get(list_attributes_by_tag),
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
    State(state): State<AppState>,
    Path(product_id): Path<String>,
    Query(query): Query<ListAbstractAttributesQuery>,
) -> ApiResult<Json<ListAbstractAttributesResponse>> {
    let store = state.store.read().await;

    // Verify product exists
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    // Collect abstract attributes for this product with optional component type filter
    let attributes: Vec<AbstractAttributeResponse> = store
        .abstract_attributes
        .values()
        .filter(|a| {
            a.product_id == pid &&
            query.component_type.as_ref().map_or(true, |ct| &a.component_type == ct)
        })
        .map(|a| a.into())
        .collect();

    let total_count = attributes.len() as i32;

    Ok(Json(ListAbstractAttributesResponse {
        items: attributes,
        next_page_token: String::new(),
        total_count,
    }))
}

/// List abstract attributes by component type
async fn list_abstract_attributes_by_component(
    State(state): State<AppState>,
    Path((product_id, component_type)): Path<(String, String)>,
) -> ApiResult<Json<ListAbstractAttributesResponse>> {
    let store = state.store.read().await;

    // Verify product exists
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    let attributes: Vec<AbstractAttributeResponse> = store
        .abstract_attributes
        .values()
        .filter(|a| a.product_id == pid && a.component_type == component_type)
        .map(|a| a.into())
        .collect();

    let total_count = attributes.len() as i32;

    Ok(Json(ListAbstractAttributesResponse {
        items: attributes,
        next_page_token: String::new(),
        total_count,
    }))
}

/// List abstract attributes by tag
async fn list_abstract_attributes_by_tag(
    State(state): State<AppState>,
    Path((product_id, tag)): Path<(String, String)>,
) -> ApiResult<Json<ListAbstractAttributesResponse>> {
    let store = state.store.read().await;

    // Verify product exists
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    let attributes: Vec<AbstractAttributeResponse> = store
        .abstract_attributes
        .values()
        .filter(|a| {
            a.product_id == pid && a.tags.iter().any(|t| t.tag.as_str() == tag)
        })
        .map(|a| a.into())
        .collect();

    let total_count = attributes.len() as i32;

    Ok(Json(ListAbstractAttributesResponse {
        items: attributes,
        next_page_token: String::new(),
        total_count,
    }))
}

/// Create a new abstract attribute
async fn create_abstract_attribute(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
    Json(req): Json<CreateAbstractAttributeRequest>,
) -> ApiResult<Json<AbstractAttributeResponse>> {
    // Validate input
    req.validate_input()?;

    // Validate no invalid characters in path components
    if req.component_type.contains('/') || req.component_type.contains(':') {
        return Err(ApiError::BadRequest(
            "component_type cannot contain '/' or ':'".to_string()
        ));
    }
    if req.attribute_name.contains('/') || req.attribute_name.contains(':') {
        return Err(ApiError::BadRequest(
            "attribute_name cannot contain '/' or ':'".to_string()
        ));
    }
    if let Some(ref comp_id) = req.component_id {
        if comp_id.contains('/') || comp_id.contains(':') {
            return Err(ApiError::BadRequest(
                "component_id cannot contain '/' or ':'".to_string()
            ));
        }
    }

    // Validate length constraints
    const MAX_ATTRIBUTE_NAME_LENGTH: usize = 64;
    if req.attribute_name.len() > MAX_ATTRIBUTE_NAME_LENGTH {
        return Err(ApiError::BadRequest(format!(
            "attribute_name exceeds maximum length of {} characters",
            MAX_ATTRIBUTE_NAME_LENGTH
        )));
    }
    if req.component_type.len() > MAX_ATTRIBUTE_NAME_LENGTH {
        return Err(ApiError::BadRequest(format!(
            "component_type exceeds maximum length of {} characters",
            MAX_ATTRIBUTE_NAME_LENGTH
        )));
    }

    let mut store = state.store.write().await;

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
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> ApiResult<Json<AbstractAttributeResponse>> {
    // Validate path format
    validate_path(&path)?;

    let store = state.store.read().await;

    store
        .abstract_attributes
        .get(&path)
        .map(|a| Json(a.into()))
        .ok_or_else(|| ApiError::NotFound(format!("Abstract attribute '{}' not found", path)))
}

/// Delete an abstract attribute
async fn delete_abstract_attribute(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // Validate path format
    validate_path(&path)?;

    let mut store = state.store.write().await;

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

    // Check if any concrete attributes reference this abstract attribute
    let has_concrete_refs = store.attributes.values()
        .any(|a| a.abstract_path == abstract_path);
    if has_concrete_refs {
        return Err(ApiError::Conflict(format!(
            "Cannot delete abstract attribute '{}' - concrete attributes still reference it",
            path
        )));
    }

    // Check if any rules reference this abstract attribute (input or output)
    // Rules store paths in short format (componentType/componentId/attributeName)
    // We need to check if the abstract path matches any rule's input/output
    let has_rule_refs = store.rules.values().any(|rule| {
        let path_suffix = extract_path_suffix(&path);
        rule.input_attributes.iter().any(|ia| {
            let input_path = ia.path.as_str();
            paths_match(input_path, &path_suffix)
        }) || rule.output_attributes.iter().any(|oa| {
            let output_path = oa.path.as_str();
            paths_match(output_path, &path_suffix)
        })
    });
    if has_rule_refs {
        return Err(ApiError::Conflict(format!(
            "Cannot delete abstract attribute '{}' - rules still reference it",
            path
        )));
    }

    if store.abstract_attributes.remove(&path).is_some() {
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
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> ApiResult<Json<ListAttributesResponse>> {
    let store = state.store.read().await;

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

    let total_count = attributes.len() as i32;

    Ok(Json(ListAttributesResponse {
        items: attributes,
        next_page_token: String::new(),
        total_count,
    }))
}

/// List concrete attributes by tag (filters via abstract attribute tags)
async fn list_attributes_by_tag(
    State(state): State<AppState>,
    Path((product_id, tag)): Path<(String, String)>,
) -> ApiResult<Json<ListAttributesResponse>> {
    let store = state.store.read().await;

    // Verify product exists
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    // First, collect abstract paths that have the tag
    let tagged_abstract_paths: std::collections::HashSet<String> = store
        .abstract_attributes
        .values()
        .filter(|a| a.product_id == pid && a.tags.iter().any(|t| t.tag.as_str() == tag))
        .map(|a| a.abstract_path.as_str().to_string())
        .collect();

    // Then filter concrete attributes by those abstract paths
    let attributes: Vec<AttributeResponse> = store
        .attributes
        .values()
        .filter(|a| {
            a.product_id == pid && tagged_abstract_paths.contains(a.abstract_path.as_str())
        })
        .map(|a| a.into())
        .collect();

    let total_count = attributes.len() as i32;

    Ok(Json(ListAttributesResponse {
        items: attributes,
        next_page_token: String::new(),
        total_count,
    }))
}

/// Create a new concrete attribute
async fn create_attribute(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
    Json(req): Json<CreateAttributeRequest>,
) -> ApiResult<Json<AttributeResponse>> {
    // Validate input
    req.validate_input()?;

    // Validate that both value and rule_id are not provided
    if req.value.is_some() && req.rule_id.is_some() {
        return Err(ApiError::BadRequest(
            "Cannot specify both value and rule_id".to_string()
        ));
    }

    let mut store = state.store.write().await;

    // Verify product exists
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    // Validate abstract attribute exists and belongs to this product
    let abstract_path = AbstractPath::new(&req.abstract_path);

    let abstract_attr = store.abstract_attributes.get(&req.abstract_path)
        .ok_or_else(|| ApiError::BadRequest(format!(
            "Abstract attribute '{}' not found",
            req.abstract_path
        )))?;

    // Validate abstract attribute belongs to this product
    if abstract_attr.product_id != pid {
        return Err(ApiError::BadRequest(format!(
            "Abstract attribute '{}' does not belong to product '{}'",
            req.abstract_path, product_id
        )));
    }

    // Validate component type matches abstract attribute
    if abstract_attr.component_type != req.component_type {
        return Err(ApiError::BadRequest(format!(
            "Component type '{}' does not match abstract attribute component type '{}'",
            req.component_type, abstract_attr.component_type
        )));
    }

    // Validate abstract path contains the correct attribute name
    // Abstract path format: {productId}:abstract-path:{componentType}[:{componentId}]:{attributeName}
    let path_str = abstract_path.as_str();
    if !path_str.ends_with(&format!(":{}", req.attribute_name)) {
        return Err(ApiError::BadRequest(format!(
            "Attribute name '{}' does not match abstract path",
            req.attribute_name
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

    // For FIXED_VALUE attributes, validate the value against datatype constraints
    if value_type == AttributeValueType::FixedValue {
        if let Some(ref val) = value {
            // Get the abstract attribute to access the datatype
            let abstract_attr = store
                .abstract_attributes
                .get(&req.abstract_path)
                .ok_or_else(|| ApiError::BadRequest(format!(
                    "Abstract attribute '{}' not found",
                    req.abstract_path
                )))?;

            // Get the datatype
            let datatype = store
                .datatypes
                .get(abstract_attr.datatype_id.as_str())
                .ok_or_else(|| ApiError::BadRequest(format!(
                    "Datatype '{}' not found",
                    abstract_attr.datatype_id.as_str()
                )))?;

            // Validate the value against constraints
            validate_value_constraints(val, datatype)?;
        }
    }

    // Create the attribute based on value type
    let attr = match value_type {
        AttributeValueType::FixedValue => {
            if let Some(val) = value {
                Attribute::new_fixed_value(path.clone(), abstract_path, pid, val)
            } else {
                return Err(ApiError::BadRequest(
                    "FIXED_VALUE attribute requires a value".to_string()
                ));
            }
        }
        AttributeValueType::RuleDriven => {
            if let Some(rule_id_str) = &req.rule_id {
                let rule_id = RuleId::from_string(rule_id_str);
                // Verify the rule exists
                if !store.rules.contains_key(rule_id_str) {
                    return Err(ApiError::BadRequest(format!(
                        "Rule '{}' not found",
                        rule_id_str
                    )));
                }
                Attribute::new_rule_driven(path.clone(), abstract_path, pid, rule_id)
            } else {
                return Err(ApiError::BadRequest(
                    "RULE_DRIVEN attribute requires a rule_id".to_string()
                ));
            }
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
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> ApiResult<Json<AttributeResponse>> {
    // Validate path format
    validate_path(&path)?;

    let store = state.store.read().await;

    store
        .attributes
        .get(&path)
        .map(|a| Json(a.into()))
        .ok_or_else(|| ApiError::NotFound(format!("Attribute '{}' not found", path)))
}

/// Update a concrete attribute
async fn update_attribute(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(req): Json<UpdateAttributeRequest>,
) -> ApiResult<Json<AttributeResponse>> {
    // Validate path format
    validate_path(&path)?;

    let mut store = state.store.write().await;

    // Get attribute info for validation (need to do this before mutable borrow)
    let datatype_clone = {
        let attr = store
            .attributes
            .get(&path)
            .ok_or_else(|| ApiError::NotFound(format!("Attribute '{}' not found", path)))?;

        let abstract_key = attr.abstract_path.as_str().to_string();

        // Check if immutable
        if let Some(abstract_attr) = store.abstract_attributes.get(&abstract_key) {
            if abstract_attr.immutable {
                return Err(ApiError::PreconditionFailed(format!(
                    "Cannot update attribute '{}' - abstract attribute is immutable",
                    path
                )));
            }
        }

        // Get datatype for constraint validation
        let datatype_clone = store.abstract_attributes.get(&abstract_key)
            .and_then(|aa| store.datatypes.get(aa.datatype_id.as_str()))
            .cloned();

        let _ = abstract_key; // used above
        datatype_clone
    };

    // Validate new value against constraints if provided
    if let Some(value) = &req.value {
        let new_value = Value::from(value);

        if let Some(ref datatype) = datatype_clone {
            validate_value_constraints(&new_value, datatype)?;
        }
    }

    // Now do the mutable update
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
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // Validate path format
    validate_path(&path)?;

    let mut store = state.store.write().await;

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

/// Extract the path suffix (componentType/componentId/attributeName) from an abstract path
/// Abstract path format: {productId}:abstract-path:{componentType}[:{componentId}]:{attributeName}
fn extract_path_suffix(abstract_path: &str) -> String {
    // Split by ':' and extract the relevant parts
    let parts: Vec<&str> = abstract_path.split(':').collect();
    if parts.len() >= 4 {
        // Format: productId:abstract-path:componentType[:componentId]:attributeName
        // Skip productId and "abstract-path"
        let component_type = parts.get(2).unwrap_or(&"");
        if parts.len() == 4 {
            // No componentId: productId:abstract-path:componentType:attributeName
            let attr_name = parts.get(3).unwrap_or(&"");
            format!("{}/{}", component_type, attr_name)
        } else {
            // With componentId: productId:abstract-path:componentType:componentId:attributeName
            let component_id = parts.get(3).unwrap_or(&"");
            let attr_name = parts.get(4).unwrap_or(&"");
            format!("{}/{}/{}", component_type, component_id, attr_name)
        }
    } else {
        abstract_path.to_string()
    }
}

/// Check if a rule path matches an abstract path suffix
/// Rule paths can be: componentType/componentId/attributeName or componentType.componentId.attributeName
fn paths_match(rule_path: &str, abstract_suffix: &str) -> bool {
    // Normalize both paths to use '/' as separator
    let normalized_rule = rule_path.replace('.', "/");
    let normalized_suffix = abstract_suffix.replace('.', "/");
    normalized_rule == normalized_suffix
}

/// Validate that a value satisfies the datatype's constraints
fn validate_value_constraints(value: &Value, datatype: &DataType) -> ApiResult<()> {
    if let Some(constraints) = &datatype.constraints {
        match datatype.primitive_type {
            // Numeric constraint validation (Int, Float, Decimal)
            PrimitiveType::Int | PrimitiveType::Float | PrimitiveType::Decimal => {
                if let Some(val) = value.as_float() {
                    if let Some(min) = constraints.min {
                        if val < min {
                            return Err(ApiError::bad_request(format!(
                                "Value {} is below minimum constraint of {}",
                                val, min
                            )));
                        }
                    }
                    if let Some(max) = constraints.max {
                        if val > max {
                            return Err(ApiError::bad_request(format!(
                                "Value {} exceeds maximum constraint of {}",
                                val, max
                            )));
                        }
                    }
                }
            }
            // String constraint validation
            PrimitiveType::String => {
                if let Some(s) = value.as_str() {
                    let len = s.len();
                    if let Some(min_len) = constraints.min_length {
                        if len < min_len {
                            return Err(ApiError::bad_request(format!(
                                "String length {} is below minimum constraint of {}",
                                len, min_len
                            )));
                        }
                    }
                    if let Some(max_len) = constraints.max_length {
                        if len > max_len {
                            return Err(ApiError::bad_request(format!(
                                "String length {} exceeds maximum constraint of {}",
                                len, max_len
                            )));
                        }
                    }
                    if let Some(pattern) = &constraints.pattern {
                        // Limit pattern length to prevent resource exhaustion
                        if pattern.len() > MAX_PATTERN_LENGTH {
                            return Err(ApiError::bad_request(format!(
                                "Regex pattern exceeds maximum length of {} characters",
                                MAX_PATTERN_LENGTH
                            )));
                        }

                        // Use RegexBuilder with size limit for additional protection
                        // The regex crate is ReDoS-resistant by design, but we add
                        // a compiled size limit as defense in depth
                        match regex::RegexBuilder::new(pattern)
                            .size_limit(MAX_REGEX_SIZE)
                            .build()
                        {
                            Ok(regex) => {
                                if !regex.is_match(s) {
                                    return Err(ApiError::bad_request(format!(
                                        "Value '{}' does not match pattern constraint '{}'",
                                        s, pattern
                                    )));
                                }
                            }
                            Err(e) => {
                                return Err(ApiError::bad_request(format!(
                                    "Invalid regex pattern '{}': {}",
                                    pattern, e
                                )));
                            }
                        }
                    }
                }
            }
            _ => {} // Other types don't have constraints in current implementation
        }
    }
    Ok(())
}
