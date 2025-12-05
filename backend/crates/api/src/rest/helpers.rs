//! Helper functions for REST API handlers
//!
//! Provides common utility functions to reduce code duplication.

use product_farm_core::Product;

use crate::store::EntityStore;

use super::error::{ApiError, ApiResult};

// =============================================================================
// ENTITY LOOKUP HELPERS
// =============================================================================

/// Get a product reference or return NotFound error.
///
/// # Example
/// ```ignore
/// let product = require_product(&store, "my-product")?;
/// ```
pub fn require_product<'a>(store: &'a EntityStore, product_id: &str) -> ApiResult<&'a Product> {
    store
        .products
        .get(product_id)
        .ok_or_else(|| ApiError::NotFound(format!("Product '{}' not found", product_id)))
}

/// Get a mutable product reference or return NotFound error.
pub fn require_product_mut<'a>(
    store: &'a mut EntityStore,
    product_id: &str,
) -> ApiResult<&'a mut Product> {
    store
        .products
        .get_mut(product_id)
        .ok_or_else(|| ApiError::NotFound(format!("Product '{}' not found", product_id)))
}

/// Check that a product exists, return error if not.
pub fn assert_product_exists(store: &EntityStore, product_id: &str) -> ApiResult<()> {
    if store.products.contains_key(product_id) {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )))
    }
}

// =============================================================================
// UNIQUENESS HELPERS
// =============================================================================

/// Ensure an entity does not already exist.
///
/// # Example
/// ```ignore
/// require_unique(!store.products.contains_key(&id), "Product", &id)?;
/// ```
pub fn require_unique(exists: bool, entity_type: &str, id: &str) -> ApiResult<()> {
    if exists {
        Err(ApiError::Conflict(format!(
            "{} '{}' already exists",
            entity_type, id
        )))
    } else {
        Ok(())
    }
}

/// Ensure a product ID is not already in use.
pub fn require_unique_product(store: &EntityStore, product_id: &str) -> ApiResult<()> {
    require_unique(store.products.contains_key(product_id), "Product", product_id)
}

// =============================================================================
// IMMUTABILITY HELPERS
// =============================================================================

/// Trait for entities that can be immutable
pub trait HasImmutable {
    fn is_immutable(&self) -> bool;
}

impl HasImmutable for Product {
    fn is_immutable(&self) -> bool {
        // Products are immutable if they're not in Draft status
        !matches!(self.status, product_farm_core::ProductStatus::Draft)
    }
}

impl HasImmutable for product_farm_core::AbstractAttribute {
    fn is_immutable(&self) -> bool {
        self.immutable
    }
}

impl HasImmutable for product_farm_core::ProductFunctionality {
    fn is_immutable(&self) -> bool {
        self.immutable
    }
}

/// Check that an entity is mutable, return error if immutable.
///
/// # Example
/// ```ignore
/// require_mutable(&product, "Product", &product_id)?;
/// ```
pub fn require_mutable<T: HasImmutable>(entity: &T, entity_type: &str, id: &str) -> ApiResult<()> {
    if entity.is_immutable() {
        Err(ApiError::PreconditionFailed(format!(
            "Cannot modify immutable {} '{}'",
            entity_type, id
        )))
    } else {
        Ok(())
    }
}

// =============================================================================
// KEY BUILDERS
// =============================================================================

/// Build a functionality key from product_id and name.
/// Format: "{product_id}:{name}"
pub fn functionality_key(product_id: &str, name: &str) -> String {
    format!("{}:{}", product_id, name)
}

/// Build an enumeration key from template_type and name.
/// Format: "{template_type}:{name}"
pub fn enumeration_key(template_type: &str, name: &str) -> String {
    format!("{}:{}", template_type, name)
}

// =============================================================================
// PAGINATION HELPERS
// =============================================================================

/// Paginate a vector based on page size and token.
///
/// Returns (items, next_page_token)
pub fn paginate<T>(items: Vec<T>, page_size: usize, page_token: Option<&str>) -> (Vec<T>, String) {
    let start = page_token
        .and_then(|t| t.parse::<usize>().ok())
        .unwrap_or(0);

    let end = (start + page_size).min(items.len());
    let page_items: Vec<T> = items.into_iter().skip(start).take(page_size).collect();

    let next_token = if end < start + page_size {
        String::new()
    } else {
        end.to_string()
    };

    (page_items, next_token)
}

// =============================================================================
// STRING HELPERS
// =============================================================================

/// Truncate a string to a maximum length.
pub fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len]
    }
}

/// Check if a string is within length limits.
pub fn validate_length(value: &str, field: &str, max_len: usize) -> ApiResult<()> {
    if value.len() > max_len {
        Err(ApiError::BadRequest(format!(
            "{} exceeds maximum length of {} characters",
            field, max_len
        )))
    } else {
        Ok(())
    }
}
