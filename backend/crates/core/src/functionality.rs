//! Product Functionality - groupings of related attributes within a product
//!
//! ProductFunctionality represents logical groupings of attributes (like "cover", "premium", etc.)
//! with their required attributes and approval status.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{validation, AbstractPath, CoreError, CoreResult, FunctionalityId, ProductId};

/// Product Functionality status lifecycle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum ProductFunctionalityStatus {
    /// Functionality is being configured
    #[default]
    Draft,
    /// Functionality is awaiting approval
    PendingApproval,
    /// Functionality is active and can be used
    Active,
}

impl ProductFunctionalityStatus {
    /// Check if transition to new status is valid
    pub fn can_transition_to(&self, new_status: &ProductFunctionalityStatus) -> bool {
        matches!(
            (self, new_status),
            (ProductFunctionalityStatus::Draft, ProductFunctionalityStatus::PendingApproval)
                | (ProductFunctionalityStatus::PendingApproval, ProductFunctionalityStatus::Active)
                | (ProductFunctionalityStatus::PendingApproval, ProductFunctionalityStatus::Draft)
        )
    }

    /// Transition to new status
    pub fn transition(&mut self, new_status: ProductFunctionalityStatus) -> CoreResult<()> {
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


/// A required attribute within a functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalityRequiredAttribute {
    /// Functionality this belongs to
    pub functionality_id: FunctionalityId,
    /// Abstract path to the required attribute
    pub abstract_path: AbstractPath,
    /// Description of why this attribute is required
    pub description: String,
    /// Order index for this required attribute
    pub order: i32,
}

impl FunctionalityRequiredAttribute {
    pub fn new(
        functionality_id: impl Into<FunctionalityId>,
        abstract_path: impl Into<AbstractPath>,
        description: impl Into<String>,
        order: i32,
    ) -> Self {
        Self {
            functionality_id: functionality_id.into(),
            abstract_path: abstract_path.into(),
            description: description.into(),
            order,
        }
    }
}

/// A product functionality definition
///
/// Functionalities group related attributes together (e.g., "premium", "cover", "benefit")
/// and define which attributes are required for that functionality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductFunctionality {
    /// Unique functionality identifier
    pub id: FunctionalityId,
    /// Name of the functionality (e.g., "premium", "cover")
    pub name: String,
    /// Product this functionality belongs to
    pub product_id: ProductId,
    /// Whether this functionality is immutable (cannot be modified after creation)
    pub immutable: bool,
    /// Description of the functionality
    pub description: String,
    /// Required attributes for this functionality (ordered)
    pub required_attributes: Vec<FunctionalityRequiredAttribute>,
    /// Current status in lifecycle
    pub status: ProductFunctionalityStatus,
    /// When the functionality was created
    pub created_at: DateTime<Utc>,
    /// When the functionality was last updated
    pub updated_at: DateTime<Utc>,
}

impl ProductFunctionality {
    /// Create a new functionality
    pub fn new(
        id: impl Into<FunctionalityId>,
        name: impl Into<String>,
        product_id: impl Into<ProductId>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            product_id: product_id.into(),
            immutable: false,
            description: description.into(),
            required_attributes: Vec::new(),
            status: ProductFunctionalityStatus::Draft,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set immutability flag
    pub fn with_immutable(mut self, immutable: bool) -> Self {
        self.immutable = immutable;
        self
    }

    /// Add required attributes
    pub fn with_required_attributes(
        mut self,
        attrs: impl IntoIterator<Item = (impl Into<AbstractPath>, impl Into<String>)>,
    ) -> Self {
        let functionality_id = self.id.clone();
        self.required_attributes = attrs
            .into_iter()
            .enumerate()
            .map(|(i, (path, desc))| {
                FunctionalityRequiredAttribute::new(
                    functionality_id.clone(),
                    path,
                    desc,
                    i as i32,
                )
            })
            .collect();
        self
    }

    /// Add a single required attribute
    pub fn add_required_attribute(
        &mut self,
        abstract_path: impl Into<AbstractPath>,
        description: impl Into<String>,
    ) {
        let order = self.required_attributes.len() as i32;
        self.required_attributes.push(FunctionalityRequiredAttribute::new(
            self.id.clone(),
            abstract_path,
            description,
            order,
        ));
    }

    /// Validate the functionality
    pub fn validate(&self) -> CoreResult<()> {
        // Validate functionality name
        if !validation::is_valid_functionality_name(&self.name) {
            return Err(CoreError::ValidationFailed {
                field: "name".to_string(),
                message: format!(
                    "Functionality name '{}' does not match required pattern",
                    self.name
                ),
            });
        }

        // Validate description
        if !validation::is_valid_description(&self.description) {
            return Err(CoreError::ValidationFailed {
                field: "description".to_string(),
                message: "Description does not match required pattern".to_string(),
            });
        }

        // Validate each required attribute's abstract path
        for attr in &self.required_attributes {
            if !attr.abstract_path.is_valid() {
                return Err(CoreError::ValidationFailed {
                    field: "required_attributes".to_string(),
                    message: format!(
                        "Abstract path '{}' is not valid",
                        attr.abstract_path.as_str()
                    ),
                });
            }
        }

        Ok(())
    }

    /// Submit for approval
    pub fn submit(&mut self) -> CoreResult<()> {
        self.status.transition(ProductFunctionalityStatus::PendingApproval)
    }

    /// Approve functionality
    pub fn approve(&mut self) -> CoreResult<()> {
        self.status.transition(ProductFunctionalityStatus::Active)
    }

    /// Reject and return to draft
    pub fn reject(&mut self) -> CoreResult<()> {
        self.status.transition(ProductFunctionalityStatus::Draft)
    }

    /// Check if functionality is editable
    pub fn is_editable(&self) -> bool {
        !self.immutable && matches!(self.status, ProductFunctionalityStatus::Draft)
    }

    /// Check if functionality is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, ProductFunctionalityStatus::Active)
    }

    /// Check if modifications should be blocked
    pub fn check_modifiable(&self) -> CoreResult<()> {
        if self.immutable {
            return Err(CoreError::Immutable {
                entity_type: "ProductFunctionality".to_string(),
                id: self.id.as_str().to_string(),
            });
        }
        Ok(())
    }

    /// Check if an attribute is required by this functionality
    pub fn requires(&self, path: &AbstractPath) -> bool {
        self.required_attributes
            .iter()
            .any(|ra| &ra.abstract_path == path)
    }

    /// Get missing required attributes given a set of defined attributes
    pub fn missing_attributes(&self, defined: &[AbstractPath]) -> Vec<&AbstractPath> {
        self.required_attributes
            .iter()
            .filter(|ra| !defined.iter().any(|d| d == &ra.abstract_path))
            .map(|ra| &ra.abstract_path)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functionality_creation() {
        let func = ProductFunctionality::new(
            "premium-calc",
            "premium",
            "testProduct",
            "Premium calculation functionality",
        )
        .with_immutable(false)
        .with_required_attributes([
            ("testProduct:abstract-path:cover:base-rate", "Base rate for premium"),
            ("testProduct:abstract-path:customer:age", "Customer age for loading"),
        ]);

        assert_eq!(func.name, "premium");
        assert_eq!(func.required_attributes.len(), 2);
        assert_eq!(func.required_attributes[0].order, 0);
        assert_eq!(func.required_attributes[1].order, 1);
        assert_eq!(func.status, ProductFunctionalityStatus::Draft);
    }

    #[test]
    fn test_functionality_lifecycle() {
        let mut func = ProductFunctionality::new(
            "coverage-def",
            "coverage",
            "testProduct",
            "Coverage definition",
        );

        assert!(func.is_editable());

        // Submit for approval
        func.submit().unwrap();
        assert_eq!(func.status, ProductFunctionalityStatus::PendingApproval);
        assert!(!func.is_editable());

        // Approve
        func.approve().unwrap();
        assert_eq!(func.status, ProductFunctionalityStatus::Active);
        assert!(func.is_active());
    }

    #[test]
    fn test_functionality_immutability() {
        let func = ProductFunctionality::new(
            "immutable-func",
            "core",
            "testProduct",
            "Core immutable functionality",
        )
        .with_immutable(true);

        assert!(!func.is_editable());
        assert!(func.check_modifiable().is_err());
    }

    #[test]
    fn test_invalid_transition() {
        let mut func = ProductFunctionality::new(
            "test-func",
            "test",
            "testProduct",
            "Test functionality",
        );

        // Cannot directly activate from draft
        let result = func.status.transition(ProductFunctionalityStatus::Active);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_attributes() {
        let func = ProductFunctionality::new(
            "premium-calc",
            "premium",
            "testProduct",
            "Premium calculation",
        )
        .with_required_attributes([
            ("testProduct:abstract-path:cover:base-rate", "Base rate"),
            ("testProduct:abstract-path:cover:premium-amount", "Premium amount"),
            ("testProduct:abstract-path:customer:age", "Customer age"),
        ]);

        let defined = vec![
            AbstractPath::new("testProduct:abstract-path:cover:base-rate"),
            AbstractPath::new("testProduct:abstract-path:customer:age"),
        ];

        let missing = func.missing_attributes(&defined);
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].as_str(), "testProduct:abstract-path:cover:premium-amount");
    }
}
