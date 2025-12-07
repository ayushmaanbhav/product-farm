//! Hybrid storage layer with LRU caching
//!
//! Provides write-through caching for repository operations.
//! Caches are bounded by size and entries are evicted using LRU policy.

use std::num::NonZeroUsize;
use std::sync::Arc;

use async_trait::async_trait;
use lru::LruCache;
use product_farm_core::{AbstractAttribute, AbstractPath, Product, ProductId, Rule, RuleId};
use product_farm_json_logic::PersistedRule;
use tokio::sync::RwLock;

use crate::{
    AttributeRepository, CompiledRuleRepository, PersistenceResult, ProductRepository,
    RuleRepository,
};

/// Configuration for the caching layer
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of products to cache
    pub product_cache_size: usize,
    /// Maximum number of attributes to cache
    pub attribute_cache_size: usize,
    /// Maximum number of rules to cache
    pub rule_cache_size: usize,
    /// Maximum number of compiled rules to cache
    pub compiled_rule_cache_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            product_cache_size: 100,
            attribute_cache_size: 10_000,
            rule_cache_size: 10_000,
            compiled_rule_cache_size: 10_000,
        }
    }
}

impl CacheConfig {
    /// Create a cache config with uniform size for all caches
    pub fn with_size(size: usize) -> Self {
        Self {
            product_cache_size: size.min(1000), // Products are larger, cap at 1000
            attribute_cache_size: size,
            rule_cache_size: size,
            compiled_rule_cache_size: size,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Current number of items in cache
    pub len: usize,
    /// Maximum capacity of cache
    pub cap: usize,
}

impl CacheStats {
    /// Returns the cache utilization as a percentage
    pub fn utilization(&self) -> f64 {
        if self.cap == 0 {
            0.0
        } else {
            (self.len as f64 / self.cap as f64) * 100.0
        }
    }
}

/// Cached product repository with write-through caching
pub struct CachedProductRepository<T: ProductRepository> {
    inner: T,
    cache: Arc<RwLock<LruCache<ProductId, Product>>>,
}

impl<T: ProductRepository> CachedProductRepository<T> {
    /// Create a new cached repository wrapping the given inner repository
    pub fn new(inner: T, cache_size: usize) -> Self {
        let size = NonZeroUsize::new(cache_size.max(1)).unwrap();
        Self {
            inner,
            cache: Arc::new(RwLock::new(LruCache::new(size))),
        }
    }

    /// Invalidate a specific entry in the cache
    pub async fn invalidate(&self, id: &ProductId) {
        self.cache.write().await.pop(id);
    }

    /// Clear the entire cache
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        CacheStats {
            len: cache.len(),
            cap: cache.cap().get(),
        }
    }
}

#[async_trait]
impl<T: ProductRepository + Send + Sync> ProductRepository for CachedProductRepository<T> {
    async fn get(&self, id: &ProductId) -> PersistenceResult<Option<Product>> {
        // Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(product) = cache.get(id) {
                return Ok(Some(product.clone()));
            }
        }

        // Cache miss - fetch from inner store
        let result = self.inner.get(id).await?;

        // Update cache on hit
        if let Some(ref product) = result {
            let mut cache = self.cache.write().await;
            cache.put(id.clone(), product.clone());
        }

        Ok(result)
    }

    async fn save(&self, product: &Product) -> PersistenceResult<()> {
        // Write-through: save to inner store first
        self.inner.save(product).await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.put(product.id.clone(), product.clone());

        Ok(())
    }

    async fn delete(&self, id: &ProductId) -> PersistenceResult<()> {
        // Delete from inner store first
        self.inner.delete(id).await?;

        // Remove from cache
        self.cache.write().await.pop(id);

        Ok(())
    }

    async fn list(&self, limit: usize, offset: usize) -> PersistenceResult<Vec<Product>> {
        // List operations always go to inner store (no caching for lists)
        self.inner.list(limit, offset).await
    }

    async fn count(&self) -> PersistenceResult<usize> {
        // Count goes to inner store
        self.inner.count().await
    }
}

/// Cached attribute repository with write-through caching
pub struct CachedAttributeRepository<T: AttributeRepository> {
    inner: T,
    cache: Arc<RwLock<LruCache<AbstractPath, AbstractAttribute>>>,
}

impl<T: AttributeRepository> CachedAttributeRepository<T> {
    /// Create a new cached repository wrapping the given inner repository
    pub fn new(inner: T, cache_size: usize) -> Self {
        let size = NonZeroUsize::new(cache_size.max(1)).unwrap();
        Self {
            inner,
            cache: Arc::new(RwLock::new(LruCache::new(size))),
        }
    }

    /// Invalidate a specific entry in the cache
    pub async fn invalidate(&self, path: &AbstractPath) {
        self.cache.write().await.pop(path);
    }

    /// Clear the entire cache
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        CacheStats {
            len: cache.len(),
            cap: cache.cap().get(),
        }
    }
}

#[async_trait]
impl<T: AttributeRepository + Send + Sync> AttributeRepository for CachedAttributeRepository<T> {
    async fn get(&self, path: &AbstractPath) -> PersistenceResult<Option<AbstractAttribute>> {
        // Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(attr) = cache.get(path) {
                return Ok(Some(attr.clone()));
            }
        }

        // Cache miss - fetch from inner store
        let result = self.inner.get(path).await?;

        // Update cache on hit
        if let Some(ref attr) = result {
            let mut cache = self.cache.write().await;
            cache.put(path.clone(), attr.clone());
        }

        Ok(result)
    }

    async fn save(&self, attr: &AbstractAttribute) -> PersistenceResult<()> {
        // Write-through: save to inner store first
        self.inner.save(attr).await?;

        // Update cache using the attribute's path
        let mut cache = self.cache.write().await;
        cache.put(attr.abstract_path.clone(), attr.clone());

        Ok(())
    }

    async fn delete(&self, path: &AbstractPath) -> PersistenceResult<()> {
        // Delete from inner store first
        self.inner.delete(path).await?;

        // Remove from cache
        self.cache.write().await.pop(path);

        Ok(())
    }

    async fn find_by_product(
        &self,
        product_id: &ProductId,
    ) -> PersistenceResult<Vec<AbstractAttribute>> {
        // List operations always go to inner store (no caching for lists)
        self.inner.find_by_product(product_id).await
    }

    async fn find_by_tag(
        &self,
        product_id: &ProductId,
        tag: &str,
    ) -> PersistenceResult<Vec<AbstractAttribute>> {
        // Tag queries go to inner store
        self.inner.find_by_tag(product_id, tag).await
    }
}

/// Cached rule repository with write-through caching
pub struct CachedRuleRepository<T: RuleRepository> {
    inner: T,
    cache: Arc<RwLock<LruCache<RuleId, Rule>>>,
}

impl<T: RuleRepository> CachedRuleRepository<T> {
    /// Create a new cached repository wrapping the given inner repository
    pub fn new(inner: T, cache_size: usize) -> Self {
        let size = NonZeroUsize::new(cache_size.max(1)).unwrap();
        Self {
            inner,
            cache: Arc::new(RwLock::new(LruCache::new(size))),
        }
    }

    /// Invalidate a specific entry in the cache
    pub async fn invalidate(&self, id: &RuleId) {
        self.cache.write().await.pop(id);
    }

    /// Clear the entire cache
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        CacheStats {
            len: cache.len(),
            cap: cache.cap().get(),
        }
    }
}

#[async_trait]
impl<T: RuleRepository + Send + Sync> RuleRepository for CachedRuleRepository<T> {
    async fn get(&self, id: &RuleId) -> PersistenceResult<Option<Rule>> {
        // Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(rule) = cache.get(id) {
                return Ok(Some(rule.clone()));
            }
        }

        // Cache miss - fetch from inner store
        let result = self.inner.get(id).await?;

        // Update cache on hit
        if let Some(ref rule) = result {
            let mut cache = self.cache.write().await;
            cache.put(id.clone(), rule.clone());
        }

        Ok(result)
    }

    async fn save(&self, rule: &Rule) -> PersistenceResult<()> {
        // Write-through: save to inner store first
        self.inner.save(rule).await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.put(rule.id.clone(), rule.clone());

        Ok(())
    }

    async fn delete(&self, id: &RuleId) -> PersistenceResult<()> {
        // Delete from inner store first
        self.inner.delete(id).await?;

        // Remove from cache
        self.cache.write().await.pop(id);

        Ok(())
    }

    async fn find_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<Rule>> {
        // List operations always go to inner store
        self.inner.find_by_product(product_id).await
    }

    async fn find_enabled_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<Rule>> {
        self.inner.find_enabled_by_product(product_id).await
    }
}

/// Cached compiled rule repository with write-through caching
pub struct CachedCompiledRuleRepository<T: CompiledRuleRepository> {
    inner: T,
    cache: Arc<RwLock<LruCache<String, PersistedRule>>>,
}

impl<T: CompiledRuleRepository> CachedCompiledRuleRepository<T> {
    /// Create a new cached repository wrapping the given inner repository
    pub fn new(inner: T, cache_size: usize) -> Self {
        let size = NonZeroUsize::new(cache_size.max(1)).unwrap();
        Self {
            inner,
            cache: Arc::new(RwLock::new(LruCache::new(size))),
        }
    }

    /// Invalidate a specific entry in the cache
    pub async fn invalidate(&self, id: &str) {
        self.cache.write().await.pop(id);
    }

    /// Clear the entire cache
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        CacheStats {
            len: cache.len(),
            cap: cache.cap().get(),
        }
    }
}

#[async_trait]
impl<T: CompiledRuleRepository + Send + Sync> CompiledRuleRepository
    for CachedCompiledRuleRepository<T>
{
    async fn get(&self, rule_id: &str) -> PersistenceResult<Option<PersistedRule>> {
        // Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(compiled) = cache.get(rule_id) {
                return Ok(Some(compiled.clone()));
            }
        }

        // Cache miss - fetch from inner store
        let result = self.inner.get(rule_id).await?;

        // Update cache on hit
        if let Some(ref compiled) = result {
            let mut cache = self.cache.write().await;
            cache.put(rule_id.to_string(), compiled.clone());
        }

        Ok(result)
    }

    async fn save(&self, rule_id: &str, rule: &PersistedRule) -> PersistenceResult<()> {
        // Write-through: save to inner store first
        self.inner.save(rule_id, rule).await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.put(rule_id.to_string(), rule.clone());

        Ok(())
    }

    async fn delete(&self, rule_id: &str) -> PersistenceResult<()> {
        // Delete from inner store first
        self.inner.delete(rule_id).await?;

        // Remove from cache
        self.cache.write().await.pop(rule_id);

        Ok(())
    }

    async fn list_ids(&self) -> PersistenceResult<Vec<String>> {
        // List operations always go to inner store
        self.inner.list_ids().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::InMemoryProductRepository;
    use chrono::Utc;
    use product_farm_core::TemplateType;

    fn make_product(id: &str, name: &str) -> Product {
        Product::new(
            ProductId::new(id),
            name.to_string(),
            TemplateType::new("loan"),
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn test_cached_product_repository() {
        let inner = InMemoryProductRepository::new();
        let cached = CachedProductRepository::new(inner, 10);

        // Create and save a product
        let product = make_product("test", "Test Product");
        cached.save(&product).await.unwrap();

        // First get should come from cache (was populated on save)
        let result = cached.get(&ProductId::new("test")).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Test Product");

        // Check cache stats
        let stats = cached.cache_stats().await;
        assert_eq!(stats.len, 1);
        assert_eq!(stats.cap, 10);
    }

    #[tokio::test]
    async fn test_cache_miss_populates_cache() {
        let inner = InMemoryProductRepository::new();

        // Save directly to inner store
        let product = make_product("test", "Test Product");
        inner.save(&product).await.unwrap();

        // Wrap in cache after saving
        let cached = CachedProductRepository::new(inner, 10);

        // Cache should be empty initially
        let stats = cached.cache_stats().await;
        assert_eq!(stats.len, 0);

        // Get should populate cache
        let result = cached.get(&ProductId::new("test")).await.unwrap();
        assert!(result.is_some());

        // Cache should now have one entry
        let stats = cached.cache_stats().await;
        assert_eq!(stats.len, 1);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let inner = InMemoryProductRepository::new();
        let cached = CachedProductRepository::new(inner, 10);

        // Save product
        let product = make_product("test", "Test Product");
        cached.save(&product).await.unwrap();

        // Verify in cache
        let stats = cached.cache_stats().await;
        assert_eq!(stats.len, 1);

        // Invalidate
        cached.invalidate(&ProductId::new("test")).await;

        // Cache should be empty
        let stats = cached.cache_stats().await;
        assert_eq!(stats.len, 0);

        // But data should still be in inner store
        let result = cached.get(&ProductId::new("test")).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_delete_removes_from_cache() {
        let inner = InMemoryProductRepository::new();
        let cached = CachedProductRepository::new(inner, 10);

        // Save product
        let product = make_product("test", "Test Product");
        cached.save(&product).await.unwrap();

        // Verify in cache
        let stats = cached.cache_stats().await;
        assert_eq!(stats.len, 1);

        // Delete
        cached.delete(&ProductId::new("test")).await.unwrap();

        // Cache should be empty
        let stats = cached.cache_stats().await;
        assert_eq!(stats.len, 0);

        // And data should be gone from inner store
        let result = cached.get(&ProductId::new("test")).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let inner = InMemoryProductRepository::new();
        let cached = CachedProductRepository::new(inner, 2); // Small cache

        // Save 3 products
        for i in 0..3 {
            let product = make_product(&format!("product-{}", i), &format!("Product {}", i));
            cached.save(&product).await.unwrap();
        }

        // Cache should only have 2 entries (LRU evicted the first)
        let stats = cached.cache_stats().await;
        assert_eq!(stats.len, 2);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let inner = InMemoryProductRepository::new();
        let cached = CachedProductRepository::new(inner, 10);

        // Save multiple products
        for i in 0..5 {
            let product = make_product(&format!("product-{}", i), &format!("Product {}", i));
            cached.save(&product).await.unwrap();
        }

        // Verify cache has entries
        let stats = cached.cache_stats().await;
        assert_eq!(stats.len, 5);

        // Clear cache
        cached.clear_cache().await;

        // Cache should be empty
        let stats = cached.cache_stats().await;
        assert_eq!(stats.len, 0);

        // Data should still be in inner store
        let result = cached.get(&ProductId::new("product-0")).await.unwrap();
        assert!(result.is_some());
    }
}
