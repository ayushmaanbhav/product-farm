//! Persistence layer for Product-FARM
//!
//! Provides storage backends for:
//! - In-memory: Fast storage for testing and development
//! - File-based: JSON file storage for simple deployments
//! - DGraph: Graph-based storage for relationship traversal
//! - Hybrid: DGraph with LRU caching for production
//! - ScyllaDB: High-performance storage (future)
//!
//! Migration tools for data export/import between backends.

pub mod error;
pub mod file;
pub mod hybrid;
pub mod migration;
pub mod repository;

#[cfg(feature = "dgraph")]
pub mod dgraph;

pub use error::*;
pub use file::*;
pub use hybrid::*;
pub use migration::*;
pub use repository::*;

#[cfg(feature = "dgraph")]
pub use dgraph::*;
