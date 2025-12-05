//! Persistence layer for Product-FARM
//!
//! Provides storage backends for:
//! - In-memory: Fast storage for testing and development
//! - File-based: JSON file storage for simple deployments
//! - DGraph: Graph-based storage for relationship traversal
//! - ScyllaDB: High-performance storage (future)

pub mod error;
pub mod file;
pub mod repository;

#[cfg(feature = "dgraph")]
pub mod dgraph;

pub use error::*;
pub use file::*;
pub use repository::*;

#[cfg(feature = "dgraph")]
pub use dgraph::*;
