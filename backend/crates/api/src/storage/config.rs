//! Storage configuration for selecting backend type

use std::env;

use crate::config::storage::{DEFAULT_CACHE_SIZE, DEFAULT_DGRAPH_ENDPOINT};

/// Storage backend configuration
#[derive(Debug, Clone)]
pub enum StorageBackend {
    /// In-memory storage (default, for development/testing)
    Memory,
    /// DGraph graph database
    DGraph {
        /// DGraph Alpha gRPC endpoint
        endpoint: String,
    },
    /// Hybrid: DGraph with in-memory caching
    Hybrid {
        /// DGraph Alpha gRPC endpoint
        endpoint: String,
        /// Maximum cache size (number of items)
        cache_size: usize,
    },
}

impl Default for StorageBackend {
    fn default() -> Self {
        Self::Memory
    }
}

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Backend type
    pub backend: StorageBackend,
}

impl StorageConfig {
    /// Create configuration from environment variables
    ///
    /// Environment variables:
    /// - `STORAGE_BACKEND`: "memory", "dgraph", or "hybrid" (default: memory)
    /// - `DGRAPH_ENDPOINT`: DGraph endpoint (default: http://localhost:9080)
    /// - `CACHE_SIZE`: Cache size for hybrid mode (default: 10000)
    pub fn from_env() -> Self {
        let backend_str = env::var("STORAGE_BACKEND").unwrap_or_else(|_| "memory".to_string());
        let endpoint = env::var("DGRAPH_ENDPOINT").unwrap_or_else(|_| DEFAULT_DGRAPH_ENDPOINT.to_string());
        let cache_size: usize = env::var("CACHE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_CACHE_SIZE);

        let backend = match backend_str.to_lowercase().as_str() {
            "dgraph" => StorageBackend::DGraph { endpoint },
            "hybrid" => StorageBackend::Hybrid { endpoint, cache_size },
            _ => StorageBackend::Memory,
        };

        Self { backend }
    }

    /// Create with memory backend
    pub fn memory() -> Self {
        Self {
            backend: StorageBackend::Memory,
        }
    }

    /// Create with DGraph backend
    pub fn dgraph(endpoint: impl Into<String>) -> Self {
        Self {
            backend: StorageBackend::DGraph {
                endpoint: endpoint.into(),
            },
        }
    }

    /// Create with Hybrid backend
    pub fn hybrid(endpoint: impl Into<String>, cache_size: usize) -> Self {
        Self {
            backend: StorageBackend::Hybrid {
                endpoint: endpoint.into(),
                cache_size,
            },
        }
    }

    /// Check if using in-memory storage
    pub fn is_memory(&self) -> bool {
        matches!(self.backend, StorageBackend::Memory)
    }

    /// Check if using DGraph
    pub fn is_dgraph(&self) -> bool {
        matches!(self.backend, StorageBackend::DGraph { .. })
    }

    /// Check if using hybrid mode
    pub fn is_hybrid(&self) -> bool {
        matches!(self.backend, StorageBackend::Hybrid { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = StorageConfig::default();
        assert!(config.is_memory());
    }

    #[test]
    fn test_dgraph_config() {
        let config = StorageConfig::dgraph("http://localhost:9080");
        assert!(config.is_dgraph());
    }

    #[test]
    fn test_hybrid_config() {
        let config = StorageConfig::hybrid("http://localhost:9080", 5000);
        assert!(config.is_hybrid());
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self::memory()
    }
}
