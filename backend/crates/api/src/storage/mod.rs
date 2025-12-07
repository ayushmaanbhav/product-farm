//! Storage abstraction layer for Product-FARM
//!
//! This module provides a unified interface for storage operations,
//! abstracting the underlying storage backend (in-memory, DGraph, hybrid).
//!
//! # Usage
//!
//! ```ignore
//! use product_farm_api::storage::{StorageConfig, StorageProvider};
//!
//! // Create from environment variables
//! let config = StorageConfig::from_env();
//! let provider = StorageProvider::new(&config)?;
//!
//! // Or create directly for testing
//! let provider = StorageProvider::memory();
//!
//! // Use repositories
//! let products = provider.products.list(10, 0).await?;
//! ```

mod config;
mod provider;

pub use config::{StorageBackend, StorageConfig};
pub use provider::{StorageProvider, StorageProviderError};
