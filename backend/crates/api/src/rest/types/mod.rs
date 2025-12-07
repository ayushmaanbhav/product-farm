//! REST API types module
//!
//! Provides request/response types and validation for the REST API.
//!
//! # Module Structure
//!
//! - [`validation`]: Input validation trait and helpers
//! - [`requests`]: All request types with validation implementations
//! - [`responses`]: All response types

pub mod requests;
pub mod responses;
pub mod validation;

// Re-export everything for backward compatibility
pub use requests::*;
pub use responses::*;
pub use validation::*;
