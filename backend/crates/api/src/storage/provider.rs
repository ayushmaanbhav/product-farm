//! Storage provider - unified interface for all repositories

use std::sync::Arc;

use product_farm_persistence::{
    memory::{
        InMemoryAttributeRepository, InMemoryCompiledRuleRepository, InMemoryDataTypeRepository,
        InMemoryEnumerationRepository, InMemoryFunctionalityRepository, InMemoryGraphQueries,
        InMemoryProductRepository, InMemoryRuleRepository,
    },
    AttributeRepository, CompiledRuleRepository, DataTypeRepository, EnumerationRepository,
    FunctionalityRepository, GraphQueries, ProductRepository, RuleRepository,
};

use super::config::{StorageBackend, StorageConfig};

/// Storage provider - provides access to all repositories
///
/// This is the main entry point for storage operations.
/// It abstracts the underlying storage backend (memory, dgraph, hybrid).
#[derive(Clone)]
pub struct StorageProvider {
    /// Product repository
    pub products: Arc<dyn ProductRepository>,
    /// Attribute repository
    pub attributes: Arc<dyn AttributeRepository>,
    /// Rule repository
    pub rules: Arc<dyn RuleRepository>,
    /// Compiled rule repository
    pub compiled_rules: Arc<dyn CompiledRuleRepository>,
    /// DataType repository
    pub datatypes: Arc<dyn DataTypeRepository>,
    /// Functionality repository
    pub functionalities: Arc<dyn FunctionalityRepository>,
    /// Enumeration repository
    pub enumerations: Arc<dyn EnumerationRepository>,
    /// Graph queries for impact analysis
    pub graph: Arc<dyn GraphQueries>,
}

impl std::fmt::Debug for StorageProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageProvider")
            .field("backend", &"<repositories>")
            .finish()
    }
}

impl StorageProvider {
    /// Create a new storage provider with the given configuration
    pub fn new(config: &StorageConfig) -> Result<Self, StorageProviderError> {
        match &config.backend {
            StorageBackend::Memory => Ok(Self::memory()),
            StorageBackend::DGraph { endpoint } => {
                // DGraph implementation requires the dgraph feature
                #[cfg(feature = "dgraph")]
                {
                    Self::dgraph(endpoint)
                }
                #[cfg(not(feature = "dgraph"))]
                {
                    let _ = endpoint; // Suppress unused warning
                    Err(StorageProviderError::FeatureNotEnabled("dgraph".to_string()))
                }
            }
            StorageBackend::Hybrid {
                endpoint,
                cache_size,
            } => {
                // Hybrid implementation requires the dgraph feature
                #[cfg(feature = "dgraph")]
                {
                    Self::hybrid(endpoint, *cache_size)
                }
                #[cfg(not(feature = "dgraph"))]
                {
                    let _ = (endpoint, cache_size); // Suppress unused warning
                    Err(StorageProviderError::FeatureNotEnabled("dgraph".to_string()))
                }
            }
        }
    }

    /// Create an in-memory storage provider
    pub fn memory() -> Self {
        Self {
            products: Arc::new(InMemoryProductRepository::new()),
            attributes: Arc::new(InMemoryAttributeRepository::new()),
            rules: Arc::new(InMemoryRuleRepository::new()),
            compiled_rules: Arc::new(InMemoryCompiledRuleRepository::new()),
            datatypes: Arc::new(InMemoryDataTypeRepository::new()),
            functionalities: Arc::new(InMemoryFunctionalityRepository::new()),
            enumerations: Arc::new(InMemoryEnumerationRepository::new()),
            graph: Arc::new(InMemoryGraphQueries::new()),
        }
    }

    /// Create a DGraph-backed storage provider
    #[cfg(feature = "dgraph")]
    pub fn dgraph(endpoint: &str) -> Result<Self, StorageProviderError> {
        use product_farm_persistence::{DgraphConfig, DgraphRepositories};

        let config = DgraphConfig {
            endpoint: endpoint.to_string(),
        };
        let repos = DgraphRepositories::new(config)
            .map_err(|e| StorageProviderError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            products: Arc::new(repos.products),
            attributes: Arc::new(repos.attributes),
            rules: Arc::new(repos.rules),
            compiled_rules: Arc::new(repos.compiled_rules),
            // For now, use in-memory for types not yet in DGraph
            datatypes: Arc::new(InMemoryDataTypeRepository::new()),
            functionalities: Arc::new(InMemoryFunctionalityRepository::new()),
            enumerations: Arc::new(InMemoryEnumerationRepository::new()),
            // DGraph has native graph queries - use the graph component from DgraphRepositories
            graph: Arc::new(repos.graph),
        })
    }

    /// Create a hybrid storage provider (DGraph + caching)
    #[cfg(feature = "dgraph")]
    pub fn hybrid(endpoint: &str, cache_size: usize) -> Result<Self, StorageProviderError> {
        use product_farm_persistence::{
            CacheConfig, CachedAttributeRepository, CachedCompiledRuleRepository,
            CachedProductRepository, CachedRuleRepository, DgraphConfig, DgraphRepositories,
        };

        let config = DgraphConfig {
            endpoint: endpoint.to_string(),
        };
        let repos = DgraphRepositories::new(config)
            .map_err(|e| StorageProviderError::ConnectionFailed(e.to_string()))?;

        // Create cache config based on provided size
        let cache_config = CacheConfig::with_size(cache_size);

        // Wrap DGraph repositories with caching
        Ok(Self {
            products: Arc::new(CachedProductRepository::new(
                repos.products,
                cache_config.product_cache_size,
            )),
            attributes: Arc::new(CachedAttributeRepository::new(
                repos.attributes,
                cache_config.attribute_cache_size,
            )),
            rules: Arc::new(CachedRuleRepository::new(
                repos.rules,
                cache_config.rule_cache_size,
            )),
            compiled_rules: Arc::new(CachedCompiledRuleRepository::new(
                repos.compiled_rules,
                cache_config.compiled_rule_cache_size,
            )),
            // For now, use in-memory for types not yet in DGraph
            datatypes: Arc::new(InMemoryDataTypeRepository::new()),
            functionalities: Arc::new(InMemoryFunctionalityRepository::new()),
            enumerations: Arc::new(InMemoryEnumerationRepository::new()),
            // DGraph has native graph queries
            graph: Arc::new(repos.graph),
        })
    }
}

/// Errors that can occur when creating a storage provider
#[derive(Debug, thiserror::Error)]
pub enum StorageProviderError {
    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_provider() {
        let provider = StorageProvider::memory();
        // Just verify it creates successfully
        assert!(std::mem::size_of_val(&provider) > 0);
    }

    #[tokio::test]
    async fn test_memory_provider_operations() {
        use product_farm_core::{DataType, PrimitiveType, DataTypeId};

        let provider = StorageProvider::memory();

        // Test DataType repository
        let dt = DataType::new(DataTypeId::new("test-type"), PrimitiveType::String);
        provider.datatypes.save(&dt).await.unwrap();

        let fetched = provider.datatypes.get(&DataTypeId::new("test-type")).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id.as_str(), "test-type");
    }
}
