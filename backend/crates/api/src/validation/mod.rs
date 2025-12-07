//! Rule validation service
//!
//! Provides comprehensive validation for rules before execution:
//! - JSON Logic expression syntax validation
//! - Dependency cycle detection
//! - Input/output attribute validation
//! - Missing dependency detection
//!
//! # Module Structure
//!
//! - [`errors`]: Validation result, error, and warning types
//! - [`rules`]: Rule validation logic and utilities

mod errors;
mod rules;

// Re-export all types
pub use errors::*;
pub use rules::*;
