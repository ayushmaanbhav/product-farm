//! gRPC service implementations for Product-FARM
//!
//! Implements all service traits generated from product_farm.proto.
//!
//! # Module Structure
//!
//! - [`helpers`]: Shared helper functions for gRPC services
//! - [`evaluation`]: ProductFarmGrpcService for rule evaluation
//! - [`products`]: ProductGrpcService for product CRUD and lifecycle
//! - [`abstract_attributes`]: AbstractAttributeGrpcService for abstract attribute management
//! - [`attributes`]: AttributeGrpcService for concrete attribute management
//! - [`rules`]: RuleGrpcService for rule management
//! - [`datatypes`]: DatatypeGrpcService for datatype management
//! - [`functionalities`]: ProductFunctionalityGrpcService for functionality management
//! - [`templates`]: ProductTemplateGrpcService for template enumeration management

pub mod helpers;

mod abstract_attributes;
mod attributes;
mod datatypes;
mod evaluation;
mod functionalities;
mod products;
mod rules;
mod templates;

// Include the generated protobuf code
pub mod proto {
    tonic::include_proto!("product_farm");
}

// Re-export all services
pub use abstract_attributes::AbstractAttributeGrpcService;
pub use attributes::AttributeGrpcService;
pub use datatypes::DatatypeGrpcService;
pub use evaluation::ProductFarmGrpcService;
pub use functionalities::ProductFunctionalityGrpcService;
pub use products::ProductGrpcService;
pub use rules::RuleGrpcService;
pub use templates::ProductTemplateGrpcService;

use crate::store::SharedStore;

/// All gRPC services bundled together
pub struct AllServices {
    pub product_farm: ProductFarmGrpcService,
    pub product: ProductGrpcService,
    pub abstract_attribute: AbstractAttributeGrpcService,
    pub attribute: AttributeGrpcService,
    pub rule: RuleGrpcService,
    pub datatype: DatatypeGrpcService,
    pub functionality: ProductFunctionalityGrpcService,
    pub template: ProductTemplateGrpcService,
}

/// Create all gRPC services with a shared store
pub fn create_all_services(store: SharedStore) -> AllServices {
    AllServices {
        product_farm: ProductFarmGrpcService::new(store.clone()),
        product: ProductGrpcService::new(store.clone()),
        abstract_attribute: AbstractAttributeGrpcService::new(store.clone()),
        attribute: AttributeGrpcService::new(store.clone()),
        rule: RuleGrpcService::new(store.clone()),
        datatype: DatatypeGrpcService::new(store.clone()),
        functionality: ProductFunctionalityGrpcService::new(store.clone()),
        template: ProductTemplateGrpcService::new(store),
    }
}
