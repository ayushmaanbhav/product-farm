//! Rule Engine with DAG-based execution
//!
//! This crate provides:
//! - DAG construction from rule dependencies
//! - Topological sorting for execution order
//! - Parallel execution where possible
//! - Context management for rule evaluation
//! - Pattern analysis for semantic grouping and insights

pub mod dag;
pub mod executor;
pub mod context;
pub mod error;
pub mod pattern_analyzer;

pub use dag::*;
pub use executor::*;
pub use context::*;
pub use error::*;
pub use pattern_analyzer::*;
