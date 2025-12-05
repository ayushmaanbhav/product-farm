//! Product entity and lifecycle management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult, ProductId, validation};

/// Product status lifecycle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum ProductStatus {
    /// Product is being configured
    #[default]
    Draft,
    /// Product is awaiting approval
    PendingApproval,
    /// Product is active and can be used
    Active,
    /// Product has been discontinued
    Discontinued,
}

impl ProductStatus {
    /// Check if transition to new status is valid
    pub fn can_transition_to(&self, new_status: &ProductStatus) -> bool {
        matches!(
            (self, new_status),
            (ProductStatus::Draft, ProductStatus::PendingApproval)
                | (ProductStatus::PendingApproval, ProductStatus::Active)
                | (ProductStatus::PendingApproval, ProductStatus::Draft)
                | (ProductStatus::Active, ProductStatus::Discontinued)
        )
    }

    /// Transition to new status
    pub fn transition(&mut self, new_status: ProductStatus) -> CoreResult<()> {
        if self.can_transition_to(&new_status) {
            *self = new_status;
            Ok(())
        } else {
            Err(CoreError::InvalidStateTransition {
                from: format!("{:?}", self),
                to: format!("{:?}", new_status),
            })
        }
    }
}


/// Product template type (dynamically defined)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateType(pub String);

impl TemplateType {
    pub fn new(t: impl Into<String>) -> Self {
        Self(t.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for TemplateType {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for TemplateType {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A product definition - the root entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    /// Unique product identifier (must match PRODUCT_ID_REGEX)
    pub id: ProductId,
    /// Human-readable name (must match PRODUCT_NAME_REGEX)
    pub name: String,
    /// Current status in lifecycle
    pub status: ProductStatus,
    /// Template type (e.g., "insurance", "trading") - dynamically defined
    pub template_type: TemplateType,
    /// Optional parent product (if cloned)
    pub parent_product_id: Option<ProductId>,
    /// When this product becomes effective
    pub effective_from: DateTime<Utc>,
    /// When this product expires
    pub expiry_at: Option<DateTime<Utc>>,
    /// Human-readable description (must match DESCRIPTION_REGEX)
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Version for optimistic locking
    pub version: u64,
}

impl Product {
    /// Create a new product without validation.
    ///
    /// Note: Prefer `try_new()` for API boundaries to ensure valid data.
    pub fn new(
        id: impl Into<ProductId>,
        name: impl Into<String>,
        template_type: impl Into<TemplateType>,
        effective_from: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            status: ProductStatus::Draft,
            template_type: template_type.into(),
            parent_product_id: None,
            effective_from,
            expiry_at: None,
            description: None,
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    /// Create a new product with validation.
    ///
    /// Returns an error if the product ID or name is invalid.
    /// Use this at API boundaries to ensure data integrity.
    pub fn try_new(
        id: impl Into<ProductId>,
        name: impl Into<String>,
        template_type: impl Into<TemplateType>,
        effective_from: DateTime<Utc>,
    ) -> CoreResult<Self> {
        let product = Self::new(id, name, template_type, effective_from);
        product.validate()?;
        Ok(product)
    }

    /// Create a new product cloned from a parent
    pub fn clone_from(
        parent: &Product,
        new_id: impl Into<ProductId>,
        new_name: impl Into<String>,
        effective_from: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: new_id.into(),
            name: new_name.into(),
            status: ProductStatus::Draft,
            template_type: parent.template_type.clone(),
            parent_product_id: Some(parent.id.clone()),
            effective_from,
            expiry_at: None,
            description: parent.description.clone(),
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    /// Set the product name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the expiry date
    pub fn with_expiry(mut self, expiry: DateTime<Utc>) -> Self {
        self.expiry_at = Some(expiry);
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Validate the product fields against regex patterns
    pub fn validate(&self) -> CoreResult<()> {
        if !validation::is_valid_product_id(self.id.as_str()) {
            return Err(CoreError::ValidationFailed {
                field: "id".to_string(),
                message: format!(
                    "Product ID '{}' does not match required pattern",
                    self.id.as_str()
                ),
            });
        }

        if !validation::is_valid_product_name(&self.name) {
            return Err(CoreError::ValidationFailed {
                field: "name".to_string(),
                message: format!(
                    "Product name '{}' does not match required pattern",
                    self.name
                ),
            });
        }

        if let Some(desc) = &self.description {
            if !validation::is_valid_description(desc) {
                return Err(CoreError::ValidationFailed {
                    field: "description".to_string(),
                    message: "Description does not match required pattern".to_string(),
                });
            }
        }

        // Validate date constraints
        if let Some(expiry) = &self.expiry_at {
            if expiry <= &self.effective_from {
                return Err(CoreError::ValidationFailed {
                    field: "expiry_at".to_string(),
                    message: "Expiry date must be after effective date".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Submit product for approval
    pub fn submit(&mut self) -> CoreResult<()> {
        self.status.transition(ProductStatus::PendingApproval)?;
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    /// Approve product
    pub fn approve(&mut self) -> CoreResult<()> {
        self.status.transition(ProductStatus::Active)?;
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    /// Reject and return to draft
    pub fn reject(&mut self) -> CoreResult<()> {
        self.status.transition(ProductStatus::Draft)?;
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    /// Discontinue product
    pub fn discontinue(&mut self) -> CoreResult<()> {
        self.status.transition(ProductStatus::Discontinued)?;
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    /// Check if product is editable
    pub fn is_editable(&self) -> bool {
        matches!(self.status, ProductStatus::Draft)
    }

    /// Check if product is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, ProductStatus::Active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_lifecycle() {
        let mut product = Product::new("testProduct", "Test Product", "insurance", Utc::now());
        assert_eq!(product.status, ProductStatus::Draft);
        assert!(product.is_editable());

        // Submit for approval
        product.submit().unwrap();
        assert_eq!(product.status, ProductStatus::PendingApproval);
        assert!(!product.is_editable());

        // Approve
        product.approve().unwrap();
        assert_eq!(product.status, ProductStatus::Active);
        assert!(product.is_active());

        // Discontinue
        product.discontinue().unwrap();
        assert_eq!(product.status, ProductStatus::Discontinued);
    }

    #[test]
    fn test_invalid_transition() {
        let mut product = Product::new("tradingProduct", "Trading Product", "trading", Utc::now());

        // Cannot directly activate from draft
        let result = product.status.transition(ProductStatus::Active);
        assert!(result.is_err());
    }

    #[test]
    fn test_clone_product() {
        let parent = Product::new("parentProduct", "Parent Product", "insurance", Utc::now());
        let child = Product::clone_from(&parent, "childProduct", "Child Product", Utc::now());

        assert_eq!(child.parent_product_id, Some(parent.id.clone()));
        assert_eq!(child.template_type, parent.template_type);
        assert_eq!(child.status, ProductStatus::Draft);
        assert_eq!(child.name, "Child Product");
    }

    #[test]
    fn test_product_validation() {
        // Valid product
        let product = Product::new("validProduct", "Valid Name", "insurance", Utc::now());
        assert!(product.validate().is_ok());

        // Invalid product ID (starts with number)
        let product = Product::new("123invalid", "Valid Name", "insurance", Utc::now());
        assert!(product.validate().is_err());

        // Invalid product ID (contains hyphen - not allowed)
        let product = Product::new("invalid-id", "Valid Name", "insurance", Utc::now());
        assert!(product.validate().is_err());
    }

    #[test]
    fn test_date_validation() {
        let now = Utc::now();
        let past = now - chrono::Duration::days(1);

        // Invalid: expiry before effective
        let product = Product::new("validProduct", "Valid Name", "insurance", now)
            .with_expiry(past);
        assert!(product.validate().is_err());

        // Valid: expiry after effective
        let future = now + chrono::Duration::days(365);
        let product = Product::new("validProduct", "Valid Name", "insurance", now)
            .with_expiry(future);
        assert!(product.validate().is_ok());
    }
}
