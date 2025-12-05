//! Product REST API handlers

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{TimeZone, Utc};
use product_farm_core::{
    CloneProductRequest as CoreCloneRequest, CloneSelections, Product, ProductCloneService,
    ProductId, ProductStatus,
};
use serde::Deserialize;

use crate::store::SharedStore;

use super::error::{ApiError, ApiResult};
use super::types::*;

/// Create product routes
pub fn routes() -> Router<SharedStore> {
    Router::new()
        .route("/api/products", get(list_products).post(create_product))
        .route(
            "/api/products/:id",
            get(get_product).put(update_product).delete(delete_product),
        )
        .route("/api/products/:id/clone", post(clone_product))
        .route("/api/products/:id/submit", post(submit_product))
        .route("/api/products/:id/approve", post(approve_product))
        .route("/api/products/:id/reject", post(reject_product))
        .route("/api/products/:id/discontinue", post(discontinue_product))
        .route("/api/product-templates", get(list_templates))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProductsQuery {
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub page_token: String,
    #[serde(default)]
    pub status_filter: Option<String>,
    #[serde(default)]
    pub template_type_filter: Option<String>,
}

fn default_page_size() -> usize {
    20
}

/// List all products with pagination
pub async fn list_products(
    State(store): State<SharedStore>,
    Query(query): Query<ListProductsQuery>,
) -> ApiResult<Json<PaginatedResponse<ProductResponse>>> {
    let store = store.read().await;

    let offset: usize = if query.page_token.is_empty() {
        0
    } else {
        query
            .page_token
            .parse()
            .map_err(|_| ApiError::bad_request(format!("Invalid page_token: '{}'", query.page_token)))?
    };
    let page_size = query.page_size.min(100);

    // Filter products
    let mut products: Vec<&Product> = store.products.values().collect();

    // Apply status filter
    if let Some(status_str) = &query.status_filter {
        let status = match status_str.to_uppercase().as_str() {
            "DRAFT" => Some(ProductStatus::Draft),
            "PENDING_APPROVAL" => Some(ProductStatus::PendingApproval),
            "ACTIVE" => Some(ProductStatus::Active),
            "DISCONTINUED" => Some(ProductStatus::Discontinued),
            _ => None,
        };
        if let Some(s) = status {
            products.retain(|p| p.status == s);
        }
    }

    // Apply template type filter
    if let Some(template_type) = &query.template_type_filter {
        products.retain(|p| p.template_type.as_str() == template_type.as_str());
    }

    let total_count = products.len() as i32;

    // Sort by name for consistent ordering
    products.sort_by(|a, b| a.name.cmp(&b.name));

    // Paginate
    let paginated: Vec<ProductResponse> = products
        .into_iter()
        .skip(offset)
        .take(page_size)
        .map(|p| p.into())
        .collect();

    let next_token = if paginated.len() == page_size {
        (offset + page_size).to_string()
    } else {
        String::new()
    };

    Ok(Json(PaginatedResponse {
        items: paginated,
        next_page_token: next_token,
        total_count,
    }))
}

/// Get a single product by ID
pub async fn get_product(
    State(store): State<SharedStore>,
    Path(id): Path<String>,
) -> ApiResult<Json<ProductResponse>> {
    let store = store.read().await;

    store
        .products
        .get(&id)
        .map(|p| Json(p.into()))
        .ok_or_else(|| ApiError::NotFound(format!("Product '{}' not found", id)))
}

/// Create a new product
pub async fn create_product(
    State(store): State<SharedStore>,
    Json(req): Json<CreateProductRequest>,
) -> ApiResult<Json<ProductResponse>> {
    // Validate input lengths
    req.validate_input()?;

    let mut store = store.write().await;

    // Check if product already exists
    if store.products.contains_key(&req.id) {
        return Err(ApiError::Conflict(format!(
            "Product '{}' already exists",
            req.id
        )));
    }

    // Validate product ID
    if !product_farm_core::validation::is_valid_product_id(&req.id) {
        return Err(ApiError::BadRequest(format!(
            "Invalid product ID '{}'. Must be alphanumeric with hyphens.",
            req.id
        )));
    }

    let effective_from = Utc
        .timestamp_opt(req.effective_from, 0)
        .single()
        .ok_or_else(|| ApiError::BadRequest("Invalid effective_from timestamp".to_string()))?;

    let product_id = req.id.clone();
    let mut product = Product::new(req.id, req.name, req.template_type, effective_from);

    if let Some(desc) = req.description {
        product = product.with_description(desc);
    }

    if let Some(expiry) = req.expiry_at {
        if let Some(dt) = Utc.timestamp_opt(expiry, 0).single() {
            product = product.with_expiry(dt);
        }
    }

    let response: ProductResponse = (&product).into();
    store.products.insert(product_id, product);

    Ok(Json(response))
}

/// Update an existing product
pub async fn update_product(
    State(store): State<SharedStore>,
    Path(id): Path<String>,
    Json(req): Json<UpdateProductRequest>,
) -> ApiResult<Json<ProductResponse>> {
    // Validate input lengths
    req.validate_input()?;

    let mut store = store.write().await;

    let product = store
        .products
        .get_mut(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Product '{}' not found", id)))?;

    // Only allow updates to DRAFT products
    if product.status != ProductStatus::Draft {
        return Err(ApiError::PreconditionFailed(format!(
            "Cannot update product '{}' with status {:?}. Only DRAFT products can be updated.",
            id, product.status
        )));
    }

    if let Some(name) = req.name {
        product.name = name;
    }

    if let Some(desc) = req.description {
        product.description = Some(desc);
    }

    if let Some(effective_from) = req.effective_from {
        if let Some(dt) = Utc.timestamp_opt(effective_from, 0).single() {
            product.effective_from = dt;
        }
    }

    if let Some(expiry_at) = req.expiry_at {
        product.expiry_at = Utc.timestamp_opt(expiry_at, 0).single();
    }

    product.updated_at = Utc::now();
    product.version += 1;

    Ok(Json((&*product).into()))
}

/// Delete a product
pub async fn delete_product(
    State(store): State<SharedStore>,
    Path(id): Path<String>,
) -> ApiResult<Json<DeleteResponse>> {
    let mut store = store.write().await;

    // Check if product exists
    let product = store
        .products
        .get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Product '{}' not found", id)))?;

    // Only allow deletion of DRAFT products
    if product.status != ProductStatus::Draft {
        return Err(ApiError::PreconditionFailed(format!(
            "Cannot delete product '{}' with status {:?}. Only DRAFT products can be deleted.",
            id, product.status
        )));
    }

    // Remove product and all related entities
    store.products.remove(&id);
    store.abstract_attrs_by_product.remove(&id);
    store.attrs_by_product.remove(&id);
    store.rules_by_product.remove(&id);
    store.funcs_by_product.remove(&id);

    // Remove the actual entities
    store.abstract_attributes.retain(|_, a| a.product_id.as_str() != id);
    store.attributes.retain(|_, a| a.product_id.as_str() != id);
    store.rules.retain(|_, r| r.product_id.as_str() != id);
    store.functionalities.retain(|k, _| !k.starts_with(&format!("{}:", id)));

    Ok(Json(DeleteResponse {
        success: true,
        message: Some(format!("Product '{}' deleted", id)),
    }))
}

/// Clone a product
pub async fn clone_product(
    State(store): State<SharedStore>,
    Path(source_id): Path<String>,
    Json(req): Json<CloneProductRequest>,
) -> ApiResult<Json<CloneProductResponse>> {
    // Validate input lengths
    req.validate_input()?;

    let mut store = store.write().await;

    // Get source product
    let source_product = store
        .products
        .get(&source_id)
        .ok_or_else(|| ApiError::NotFound(format!("Source product '{}' not found", source_id)))?
        .clone();

    // Check new ID doesn't exist
    if store.products.contains_key(&req.new_product_id) {
        return Err(ApiError::Conflict(format!(
            "Product '{}' already exists",
            req.new_product_id
        )));
    }

    // Validate new product ID
    if !product_farm_core::validation::is_valid_product_id(&req.new_product_id) {
        return Err(ApiError::BadRequest(format!(
            "Invalid product ID '{}'",
            req.new_product_id
        )));
    }

    // Gather source data
    let source_abstract_attrs: Vec<_> = store
        .get_abstract_attrs_for_product(&source_id)
        .into_iter()
        .cloned()
        .collect();
    let source_attrs: Vec<_> = store
        .get_attrs_for_product(&source_id)
        .into_iter()
        .cloned()
        .collect();
    let source_rules: Vec<_> = store
        .get_rules_for_product(&source_id)
        .into_iter()
        .cloned()
        .collect();
    let source_funcs: Vec<_> = store
        .get_funcs_for_product(&source_id)
        .into_iter()
        .cloned()
        .collect();

    let effective_from = Utc::now();

    // Build selections
    let selections = if req.selected_components.is_empty()
        && req.selected_datatypes.is_empty()
        && req.selected_enumerations.is_empty()
        && req.selected_functionalities.is_empty()
        && req.selected_abstract_attributes.is_empty()
    {
        None
    } else {
        Some(CloneSelections {
            components: req.selected_components,
            datatypes: req.selected_datatypes,
            enumerations: req.selected_enumerations,
            functionalities: req.selected_functionalities,
            abstract_attributes: req.selected_abstract_attributes,
        })
    };

    let clone_request = CoreCloneRequest {
        new_product_id: ProductId::new(req.new_product_id.clone()),
        new_name: req.new_product_name,
        new_description: req.new_product_description,
        effective_from,
        selections,
        clone_concrete_attributes: req.clone_concrete_attributes,
    };

    let result = ProductCloneService::clone_product(
        &source_product,
        &source_abstract_attrs,
        &source_attrs,
        &source_rules,
        &source_funcs,
        clone_request,
    )
    .map_err(|e| ApiError::Internal(format!("Clone failed: {}", e)))?;

    // Store cloned entities
    let abstract_attributes_cloned = result.abstract_attributes.len() as i32;
    let attributes_cloned = result.attributes.len() as i32;
    let rules_cloned = result.rules.len() as i32;
    let functionalities_cloned = result.functionalities.len() as i32;

    let new_product_id = result.product.id.as_str().to_string();

    // Store product
    let product_response: ProductResponse = (&result.product).into();
    store.products.insert(new_product_id.clone(), result.product);

    // Store abstract attributes
    let mut aa_paths = vec![];
    for attr in result.abstract_attributes {
        let path = attr.abstract_path.as_str().to_string();
        aa_paths.push(path.clone());
        store.abstract_attributes.insert(path, attr);
    }
    store.abstract_attrs_by_product.insert(new_product_id.clone(), aa_paths);

    // Store attributes
    let mut attr_paths = vec![];
    for attr in result.attributes {
        let path = attr.path.as_str().to_string();
        attr_paths.push(path.clone());
        store.attributes.insert(path, attr);
    }
    store.attrs_by_product.insert(new_product_id.clone(), attr_paths);

    // Store rules
    let mut rule_ids = vec![];
    for rule in result.rules {
        let id = rule.id.to_string();
        rule_ids.push(id.clone());
        store.rules.insert(id, rule);
    }
    store.rules_by_product.insert(new_product_id.clone(), rule_ids);

    // Store functionalities
    let mut func_keys = vec![];
    for func in result.functionalities {
        let key = crate::store::EntityStore::functionality_key(&new_product_id, &func.name);
        func_keys.push(key.clone());
        store.functionalities.insert(key, func);
    }
    store.funcs_by_product.insert(new_product_id, func_keys);

    Ok(Json(CloneProductResponse {
        product: product_response,
        abstract_attributes_cloned,
        attributes_cloned,
        rules_cloned,
        functionalities_cloned,
    }))
}

/// Submit product for approval
pub async fn submit_product(
    State(store): State<SharedStore>,
    Path(id): Path<String>,
) -> ApiResult<Json<ProductResponse>> {
    let mut store = store.write().await;

    let product = store
        .products
        .get_mut(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Product '{}' not found", id)))?;

    if product.status != ProductStatus::Draft {
        return Err(ApiError::PreconditionFailed(format!(
            "Cannot submit product '{}' with status {:?}. Only DRAFT products can be submitted.",
            id, product.status
        )));
    }

    product.status = ProductStatus::PendingApproval;
    product.updated_at = Utc::now();
    product.version += 1;

    Ok(Json((&*product).into()))
}

/// Approve a product
pub async fn approve_product(
    State(store): State<SharedStore>,
    Path(id): Path<String>,
    Json(_req): Json<ApprovalRequest>,
) -> ApiResult<Json<ProductResponse>> {
    let mut store = store.write().await;

    let product = store
        .products
        .get_mut(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Product '{}' not found", id)))?;

    if product.status != ProductStatus::PendingApproval {
        return Err(ApiError::PreconditionFailed(format!(
            "Cannot approve product '{}' with status {:?}. Only PENDING_APPROVAL products can be approved.",
            id, product.status
        )));
    }

    product.status = ProductStatus::Active;
    product.updated_at = Utc::now();
    product.version += 1;

    Ok(Json((&*product).into()))
}

/// Reject a product
pub async fn reject_product(
    State(store): State<SharedStore>,
    Path(id): Path<String>,
    Json(_req): Json<ApprovalRequest>,
) -> ApiResult<Json<ProductResponse>> {
    let mut store = store.write().await;

    let product = store
        .products
        .get_mut(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Product '{}' not found", id)))?;

    if product.status != ProductStatus::PendingApproval {
        return Err(ApiError::PreconditionFailed(format!(
            "Cannot reject product '{}' with status {:?}. Only PENDING_APPROVAL products can be rejected.",
            id, product.status
        )));
    }

    product.status = ProductStatus::Draft;
    product.updated_at = Utc::now();
    product.version += 1;

    Ok(Json((&*product).into()))
}

/// Discontinue a product
pub async fn discontinue_product(
    State(store): State<SharedStore>,
    Path(id): Path<String>,
) -> ApiResult<Json<ProductResponse>> {
    let mut store = store.write().await;

    let product = store
        .products
        .get_mut(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Product '{}' not found", id)))?;

    if product.status != ProductStatus::Active {
        return Err(ApiError::PreconditionFailed(format!(
            "Cannot discontinue product '{}' with status {:?}. Only ACTIVE products can be discontinued.",
            id, product.status
        )));
    }

    product.status = ProductStatus::Discontinued;
    product.updated_at = Utc::now();
    product.version += 1;

    Ok(Json((&*product).into()))
}

/// List product templates
pub async fn list_templates(
    State(_store): State<SharedStore>,
) -> ApiResult<Json<Vec<ProductTemplateResponse>>> {
    // Return hardcoded templates for now (matching frontend mock data)
    let templates = vec![
        ProductTemplateResponse {
            id: "insurance".to_string(),
            name: "Insurance Product".to_string(),
            template_type: "insurance".to_string(),
            description: "Template for insurance products with covers, premiums, and claims".to_string(),
            components: vec![
                TemplateComponentJson {
                    id: "cover".to_string(),
                    name: "Cover".to_string(),
                    description: "Insurance coverage details".to_string(),
                },
                TemplateComponentJson {
                    id: "premium".to_string(),
                    name: "Premium".to_string(),
                    description: "Premium calculation".to_string(),
                },
                TemplateComponentJson {
                    id: "discount".to_string(),
                    name: "Discount".to_string(),
                    description: "Discount rules".to_string(),
                },
                TemplateComponentJson {
                    id: "eligibility".to_string(),
                    name: "Eligibility".to_string(),
                    description: "Eligibility criteria".to_string(),
                },
                TemplateComponentJson {
                    id: "claim".to_string(),
                    name: "Claim".to_string(),
                    description: "Claim processing".to_string(),
                },
                TemplateComponentJson {
                    id: "underwriting".to_string(),
                    name: "Underwriting".to_string(),
                    description: "Underwriting rules".to_string(),
                },
            ],
        },
        ProductTemplateResponse {
            id: "loan".to_string(),
            name: "Loan Product".to_string(),
            template_type: "loan".to_string(),
            description: "Template for loan products with interest rates and repayment schedules".to_string(),
            components: vec![
                TemplateComponentJson {
                    id: "principal".to_string(),
                    name: "Principal".to_string(),
                    description: "Loan principal amount".to_string(),
                },
                TemplateComponentJson {
                    id: "interest".to_string(),
                    name: "Interest".to_string(),
                    description: "Interest calculation".to_string(),
                },
                TemplateComponentJson {
                    id: "repayment".to_string(),
                    name: "Repayment".to_string(),
                    description: "Repayment schedule".to_string(),
                },
                TemplateComponentJson {
                    id: "eligibility".to_string(),
                    name: "Eligibility".to_string(),
                    description: "Loan eligibility criteria".to_string(),
                },
            ],
        },
        ProductTemplateResponse {
            id: "trading".to_string(),
            name: "Trading Product".to_string(),
            template_type: "trading".to_string(),
            description: "Template for trading products with market data and signals".to_string(),
            components: vec![
                TemplateComponentJson {
                    id: "market".to_string(),
                    name: "Market".to_string(),
                    description: "Market data".to_string(),
                },
                TemplateComponentJson {
                    id: "signal".to_string(),
                    name: "Signal".to_string(),
                    description: "Trading signals".to_string(),
                },
                TemplateComponentJson {
                    id: "risk".to_string(),
                    name: "Risk".to_string(),
                    description: "Risk management".to_string(),
                },
            ],
        },
    ];

    Ok(Json(templates))
}
