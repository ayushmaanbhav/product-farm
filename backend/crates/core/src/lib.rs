//! Product-FARM Core Domain Types
//!
//! This crate defines the core domain types for the Product-FARM rule engine.
//! All types are generic and domain-agnostic - specific domains (insurance, trading, etc.)
//! are configured through DataTypes, Enums, Attributes, and Rules.
//!
//! ## Path Formats (Legacy Compatible)
//!
//! - **Concrete Path**: `{productId}:{componentType}:{componentId}:{attributeName}`
//!   - Example: `insuranceV1:cover:basic:premium`
//!
//! - **Abstract Path**: `{productId}:abstract-path:{componentType}[:{componentId}]:{attributeName}`
//!   - Example: `insuranceV1:abstract-path:cover:premium`
//!   - Example with component ID: `insuranceV1:abstract-path:cover:basic:premium`

pub mod types;
pub mod product;
pub mod attribute;
pub mod rule;
pub mod datatype;
pub mod functionality;
pub mod template;
pub mod error;
pub mod value;
pub mod builders;
pub mod validation;
pub mod clone;
pub mod evaluator;

pub use types::*;
pub use product::*;
pub use attribute::*;
pub use rule::*;
pub use datatype::*;
pub use functionality::*;
pub use template::*;
pub use error::*;
pub use value::*;
pub use builders::*;
pub use validation::*;
pub use clone::*;
pub use evaluator::*;
