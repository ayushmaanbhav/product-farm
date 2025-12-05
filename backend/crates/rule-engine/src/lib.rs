//! Rule Engine with DAG-based execution
//!
//! This crate provides:
//! - DAG construction from rule dependencies
//! - Topological sorting for execution order
//! - Parallel execution where possible
//! - Context management for rule evaluation

pub mod dag;
pub mod executor;
pub mod context;
pub mod error;

pub use dag::*;
pub use executor::*;
pub use context::*;
pub use error::*;
