//! Type converters between proto and core types
//!
//! Provides bidirectional conversion between gRPC proto types
//! and internal core domain types.
//!
//! # Module Structure
//!
//! - [`values`]: Value type conversions
//! - [`products`]: Product and functionality conversions
//! - [`attributes`]: Abstract and concrete attribute conversions
//! - [`rules`]: Rule conversions
//! - [`datatypes`]: Datatype and template enumeration conversions

mod attributes;
mod datatypes;
mod products;
mod rules;
mod values;

// Re-export all converters for convenience
pub use attributes::*;
pub use datatypes::*;
pub use products::*;
pub use rules::*;
pub use values::*;
