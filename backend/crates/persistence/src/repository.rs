//! Repository traits and implementations for persistence operations

use async_trait::async_trait;
use product_farm_core::{
    AbstractAttribute, AbstractPath, Product, ProductId, Rule, RuleId,
};
use product_farm_json_logic::PersistedRule;
use crate::error::PersistenceResult;

/// Repository for Product entities
#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn get(&self, id: &ProductId) -> PersistenceResult<Option<Product>>;
    async fn save(&self, product: &Product) -> PersistenceResult<()>;
    async fn delete(&self, id: &ProductId) -> PersistenceResult<()>;
    async fn list(&self, limit: usize, offset: usize) -> PersistenceResult<Vec<Product>>;
    async fn count(&self) -> PersistenceResult<usize>;
}

/// Repository for AbstractAttribute entities (attribute templates)
#[async_trait]
pub trait AttributeRepository: Send + Sync {
    async fn get(&self, path: &AbstractPath) -> PersistenceResult<Option<AbstractAttribute>>;
    async fn save(&self, attribute: &AbstractAttribute) -> PersistenceResult<()>;
    async fn delete(&self, path: &AbstractPath) -> PersistenceResult<()>;
    async fn find_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<AbstractAttribute>>;
    async fn find_by_tag(&self, product_id: &ProductId, tag: &str) -> PersistenceResult<Vec<AbstractAttribute>>;
}

/// Repository for Rule entities
#[async_trait]
pub trait RuleRepository: Send + Sync {
    async fn get(&self, id: &RuleId) -> PersistenceResult<Option<Rule>>;
    async fn save(&self, rule: &Rule) -> PersistenceResult<()>;
    async fn delete(&self, id: &RuleId) -> PersistenceResult<()>;
    async fn find_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<Rule>>;
    async fn find_enabled_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<Rule>>;
}

/// Repository for compiled rules with bytecode persistence
///
/// Stores pre-compiled rules (with AST and optional bytecode) for fast loading.
/// This enables warm starts where frequently-used rules don't need recompilation.
#[async_trait]
pub trait CompiledRuleRepository: Send + Sync {
    /// Get a compiled rule by its ID
    async fn get(&self, rule_id: &str) -> PersistenceResult<Option<PersistedRule>>;

    /// Save a compiled rule (with AST and optional bytecode)
    async fn save(&self, rule_id: &str, rule: &PersistedRule) -> PersistenceResult<()>;

    /// Delete a compiled rule
    async fn delete(&self, rule_id: &str) -> PersistenceResult<()>;

    /// List all compiled rule IDs
    async fn list_ids(&self) -> PersistenceResult<Vec<String>>;

    /// Check if a compiled rule exists
    async fn exists(&self, rule_id: &str) -> PersistenceResult<bool> {
        Ok(self.get(rule_id).await?.is_some())
    }
}

/// In-memory repositories for testing and development
pub mod memory {
    use super::*;
    use crate::error::PersistenceError;
    use std::collections::HashMap;
    use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard, PoisonError};

    /// Helper trait to convert poisoned lock errors to PersistenceError
    trait LockResultExt<'a, T> {
        fn map_lock_err(self) -> PersistenceResult<T>;
    }

    impl<'a, T> LockResultExt<'a, RwLockReadGuard<'a, T>> for Result<RwLockReadGuard<'a, T>, PoisonError<RwLockReadGuard<'a, T>>> {
        fn map_lock_err(self) -> PersistenceResult<RwLockReadGuard<'a, T>> {
            self.map_err(|_| PersistenceError::LockPoisoned)
        }
    }

    impl<'a, T> LockResultExt<'a, RwLockWriteGuard<'a, T>> for Result<RwLockWriteGuard<'a, T>, PoisonError<RwLockWriteGuard<'a, T>>> {
        fn map_lock_err(self) -> PersistenceResult<RwLockWriteGuard<'a, T>> {
            self.map_err(|_| PersistenceError::LockPoisoned)
        }
    }

    /// In-memory Product repository
    #[derive(Debug)]
    pub struct InMemoryProductRepository {
        products: RwLock<HashMap<ProductId, Product>>,
    }

    impl InMemoryProductRepository {
        pub fn new() -> Self {
            Self {
                products: RwLock::new(HashMap::new()),
            }
        }
    }

    impl Default for InMemoryProductRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl ProductRepository for InMemoryProductRepository {
        async fn get(&self, id: &ProductId) -> PersistenceResult<Option<Product>> {
            Ok(self.products.read().map_lock_err()?.get(id).cloned())
        }

        async fn save(&self, product: &Product) -> PersistenceResult<()> {
            self.products.write().map_lock_err()?.insert(product.id.clone(), product.clone());
            Ok(())
        }

        async fn delete(&self, id: &ProductId) -> PersistenceResult<()> {
            self.products.write().map_lock_err()?.remove(id);
            Ok(())
        }

        async fn list(&self, limit: usize, offset: usize) -> PersistenceResult<Vec<Product>> {
            let products = self.products.read().map_lock_err()?;
            let mut list: Vec<_> = products.values().cloned().collect();
            // Sort by id for consistent ordering
            list.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(list.into_iter().skip(offset).take(limit).collect())
        }

        async fn count(&self) -> PersistenceResult<usize> {
            Ok(self.products.read().map_lock_err()?.len())
        }
    }

    /// In-memory Attribute repository (for AbstractAttribute templates)
    #[derive(Debug)]
    pub struct InMemoryAttributeRepository {
        attributes: RwLock<HashMap<AbstractPath, AbstractAttribute>>,
        by_product: RwLock<HashMap<ProductId, Vec<AbstractPath>>>,
    }

    impl InMemoryAttributeRepository {
        pub fn new() -> Self {
            Self {
                attributes: RwLock::new(HashMap::new()),
                by_product: RwLock::new(HashMap::new()),
            }
        }
    }

    impl Default for InMemoryAttributeRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl AttributeRepository for InMemoryAttributeRepository {
        async fn get(&self, path: &AbstractPath) -> PersistenceResult<Option<AbstractAttribute>> {
            Ok(self.attributes.read().map_lock_err()?.get(path).cloned())
        }

        async fn save(&self, attribute: &AbstractAttribute) -> PersistenceResult<()> {
            let path = attribute.abstract_path.clone();
            let product_id = attribute.product_id.clone();

            self.attributes.write().map_lock_err()?.insert(path.clone(), attribute.clone());

            let mut by_product = self.by_product.write().map_lock_err()?;
            let paths = by_product.entry(product_id).or_default();
            if !paths.contains(&path) {
                paths.push(path);
            }

            Ok(())
        }

        async fn delete(&self, path: &AbstractPath) -> PersistenceResult<()> {
            if let Some(attr) = self.attributes.write().map_lock_err()?.remove(path) {
                let mut by_product = self.by_product.write().map_lock_err()?;
                if let Some(paths) = by_product.get_mut(&attr.product_id) {
                    paths.retain(|p| p != path);
                }
            }
            Ok(())
        }

        async fn find_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<AbstractAttribute>> {
            let by_product = self.by_product.read().map_lock_err()?;
            let attributes = self.attributes.read().map_lock_err()?;

            let paths = by_product.get(product_id).cloned().unwrap_or_default();
            Ok(paths.iter().filter_map(|p| attributes.get(p).cloned()).collect())
        }

        async fn find_by_tag(&self, product_id: &ProductId, tag: &str) -> PersistenceResult<Vec<AbstractAttribute>> {
            let attrs = self.find_by_product(product_id).await?;
            Ok(attrs.into_iter().filter(|a| a.has_tag(tag)).collect())
        }
    }

    /// In-memory Rule repository
    #[derive(Debug)]
    pub struct InMemoryRuleRepository {
        rules: RwLock<HashMap<RuleId, Rule>>,
        by_product: RwLock<HashMap<ProductId, Vec<RuleId>>>,
    }

    impl InMemoryRuleRepository {
        pub fn new() -> Self {
            Self {
                rules: RwLock::new(HashMap::new()),
                by_product: RwLock::new(HashMap::new()),
            }
        }
    }

    impl Default for InMemoryRuleRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl RuleRepository for InMemoryRuleRepository {
        async fn get(&self, id: &RuleId) -> PersistenceResult<Option<Rule>> {
            Ok(self.rules.read().map_lock_err()?.get(id).cloned())
        }

        async fn save(&self, rule: &Rule) -> PersistenceResult<()> {
            let id = rule.id.clone();
            let product_id = rule.product_id.clone();

            self.rules.write().map_lock_err()?.insert(id.clone(), rule.clone());

            let mut by_product = self.by_product.write().map_lock_err()?;
            let ids = by_product.entry(product_id).or_default();
            if !ids.contains(&id) {
                ids.push(id);
            }

            Ok(())
        }

        async fn delete(&self, id: &RuleId) -> PersistenceResult<()> {
            if let Some(rule) = self.rules.write().map_lock_err()?.remove(id) {
                let mut by_product = self.by_product.write().map_lock_err()?;
                if let Some(ids) = by_product.get_mut(&rule.product_id) {
                    ids.retain(|i| i != id);
                }
            }
            Ok(())
        }

        async fn find_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<Rule>> {
            let by_product = self.by_product.read().map_lock_err()?;
            let rules = self.rules.read().map_lock_err()?;

            let ids = by_product.get(product_id).cloned().unwrap_or_default();
            let mut result: Vec<Rule> = ids.iter()
                .filter_map(|id| rules.get(id).cloned())
                .collect();

            // Sort by order_index
            result.sort_by_key(|r| r.order_index);
            Ok(result)
        }

        async fn find_enabled_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<Rule>> {
            let all = self.find_by_product(product_id).await?;
            Ok(all.into_iter().filter(|r| r.enabled).collect())
        }
    }

    /// Combined repository with all entity types
    #[derive(Debug)]
    pub struct InMemoryRepositories {
        pub products: InMemoryProductRepository,
        pub attributes: InMemoryAttributeRepository,
        pub rules: InMemoryRuleRepository,
    }

    impl InMemoryRepositories {
        pub fn new() -> Self {
            Self {
                products: InMemoryProductRepository::new(),
                attributes: InMemoryAttributeRepository::new(),
                rules: InMemoryRuleRepository::new(),
            }
        }
    }

    impl Default for InMemoryRepositories {
        fn default() -> Self {
            Self::new()
        }
    }

    /// In-memory compiled rule repository
    ///
    /// Stores compiled rules with their bytecode for fast loading.
    #[derive(Debug)]
    pub struct InMemoryCompiledRuleRepository {
        rules: RwLock<HashMap<String, PersistedRule>>,
    }

    impl InMemoryCompiledRuleRepository {
        pub fn new() -> Self {
            Self {
                rules: RwLock::new(HashMap::new()),
            }
        }

        /// Get all compiled rules (for debugging/testing)
        pub fn get_all(&self) -> PersistenceResult<Vec<(String, PersistedRule)>> {
            Ok(self.rules.read().map_lock_err()?
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect())
        }

        /// Clear all compiled rules
        pub fn clear(&self) -> PersistenceResult<()> {
            self.rules.write().map_lock_err()?.clear();
            Ok(())
        }
    }

    impl Default for InMemoryCompiledRuleRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl CompiledRuleRepository for InMemoryCompiledRuleRepository {
        async fn get(&self, rule_id: &str) -> PersistenceResult<Option<PersistedRule>> {
            Ok(self.rules.read().map_lock_err()?.get(rule_id).cloned())
        }

        async fn save(&self, rule_id: &str, rule: &PersistedRule) -> PersistenceResult<()> {
            self.rules.write().map_lock_err()?.insert(rule_id.to_string(), rule.clone());
            Ok(())
        }

        async fn delete(&self, rule_id: &str) -> PersistenceResult<()> {
            self.rules.write().map_lock_err()?.remove(rule_id);
            Ok(())
        }

        async fn list_ids(&self) -> PersistenceResult<Vec<String>> {
            Ok(self.rules.read().map_lock_err()?.keys().cloned().collect())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use chrono::Utc;
        use serde_json::json;

        #[tokio::test]
        async fn test_product_repository() {
            let repo = InMemoryProductRepository::new();

            let product = Product::new("test-product", "Test Product", "strategy", Utc::now());

            // Save
            repo.save(&product).await.unwrap();

            // Get
            let fetched = repo.get(&product.id).await.unwrap();
            assert!(fetched.is_some());
            assert_eq!(fetched.unwrap().id.as_str(), "test-product");

            // List
            let products = repo.list(10, 0).await.unwrap();
            assert_eq!(products.len(), 1);

            // Count
            assert_eq!(repo.count().await.unwrap(), 1);

            // Delete
            repo.delete(&product.id).await.unwrap();
            assert!(repo.get(&product.id).await.unwrap().is_none());
        }

        #[tokio::test]
        async fn test_attribute_repository() {
            use product_farm_core::{AttributeDisplayName, DisplayNameFormat};

            let repo = InMemoryAttributeRepository::new();

            let attr = AbstractAttribute::new(
                "MARKET_DATA.current_price",
                "product-1",
                "MARKET_DATA",
                "price",
            )
            .with_display_name(AttributeDisplayName::for_abstract(
                "product-1",
                "MARKET_DATA.current_price",
                "Current Price",
                DisplayNameFormat::Human,
                0,
            ))
            .with_description("The current market price");

            // Save
            repo.save(&attr).await.unwrap();

            // Get
            let fetched = repo.get(&attr.abstract_path).await.unwrap();
            assert!(fetched.is_some());

            // Find by product
            let attrs = repo.find_by_product(&ProductId::new("product-1")).await.unwrap();
            assert_eq!(attrs.len(), 1);

            // Delete
            repo.delete(&attr.abstract_path).await.unwrap();
            assert!(repo.get(&attr.abstract_path).await.unwrap().is_none());
        }

        #[tokio::test]
        async fn test_attribute_repository_find_by_tag() {
            let repo = InMemoryAttributeRepository::new();

            // Create attributes with different tags
            let attr1 = AbstractAttribute::new(
                "MARKET_DATA.price",
                "product-1",
                "MARKET_DATA",
                "decimal",
            )
            .with_tag_name("pricing", 0)
            .with_tag_name("market", 1);

            let attr2 = AbstractAttribute::new(
                "MARKET_DATA.volume",
                "product-1",
                "MARKET_DATA",
                "integer",
            )
            .with_tag_name("market", 0);

            let attr3 = AbstractAttribute::new(
                "TRADE.quantity",
                "product-1",
                "TRADE",
                "integer",
            )
            .with_tag_name("trading", 0);

            // Save all
            repo.save(&attr1).await.unwrap();
            repo.save(&attr2).await.unwrap();
            repo.save(&attr3).await.unwrap();

            // Find by tag "market" - should find attr1 and attr2
            let market_attrs = repo.find_by_tag(&ProductId::new("product-1"), "market").await.unwrap();
            assert_eq!(market_attrs.len(), 2);

            // Find by tag "pricing" - should find only attr1
            let pricing_attrs = repo.find_by_tag(&ProductId::new("product-1"), "pricing").await.unwrap();
            assert_eq!(pricing_attrs.len(), 1);
            assert_eq!(pricing_attrs[0].abstract_path.as_str(), "MARKET_DATA.price");

            // Find by tag "trading" - should find only attr3
            let trading_attrs = repo.find_by_tag(&ProductId::new("product-1"), "trading").await.unwrap();
            assert_eq!(trading_attrs.len(), 1);
            assert_eq!(trading_attrs[0].abstract_path.as_str(), "TRADE.quantity");

            // Find by non-existent tag - should return empty
            let empty = repo.find_by_tag(&ProductId::new("product-1"), "nonexistent").await.unwrap();
            assert!(empty.is_empty());
        }

        #[tokio::test]
        async fn test_rule_repository() {
            let repo = InMemoryRuleRepository::new();

            let rule1 = Rule::from_json_logic("product-1", "calc", json!({"*": [{"var": "x"}, 2]}))
                .with_inputs(["x"])
                .with_outputs(["doubled"])
                .with_order(0);

            let rule2 = Rule::from_json_logic("product-1", "calc", json!({"+": [{"var": "doubled"}, 10]}))
                .with_inputs(["doubled"])
                .with_outputs(["result"])
                .with_order(1);

            let rule3 = Rule::from_json_logic("product-1", "calc", json!({"var": "x"}))
                .with_inputs(["x"])
                .with_outputs(["copy"])
                .with_order(2)
                .disabled();

            repo.save(&rule1).await.unwrap();
            repo.save(&rule2).await.unwrap();
            repo.save(&rule3).await.unwrap();

            // Find by product
            let rules = repo.find_by_product(&ProductId::new("product-1")).await.unwrap();
            assert_eq!(rules.len(), 3);
            // Should be sorted by order_index
            assert_eq!(rules[0].order_index, 0);
            assert_eq!(rules[1].order_index, 1);
            assert_eq!(rules[2].order_index, 2);

            // Find enabled only
            let enabled = repo.find_enabled_by_product(&ProductId::new("product-1")).await.unwrap();
            assert_eq!(enabled.len(), 2);
        }

        #[tokio::test]
        async fn test_compiled_rule_repository() {
            use product_farm_json_logic::{parse, CompiledRule, CompilationTier};

            let repo = InMemoryCompiledRuleRepository::new();

            // Create and compile a rule
            let expr = parse(&json!({"*": [{"var": "x"}, 2]})).unwrap();
            let rule = CompiledRule::new(expr);

            // Evaluate to build up count
            for i in 0..5 {
                rule.evaluate(&json!({"x": i})).unwrap();
            }

            // Persist
            let persisted = rule.to_persisted();
            repo.save("rule-1", &persisted).await.unwrap();

            // Get
            let fetched = repo.get("rule-1").await.unwrap();
            assert!(fetched.is_some());
            let fetched = fetched.unwrap();
            assert_eq!(fetched.tier, CompilationTier::Ast);
            assert_eq!(fetched.eval_count, 5);

            // List
            let ids = repo.list_ids().await.unwrap();
            assert_eq!(ids.len(), 1);
            assert!(ids.contains(&"rule-1".to_string()));

            // Exists
            assert!(repo.exists("rule-1").await.unwrap());
            assert!(!repo.exists("nonexistent").await.unwrap());

            // Delete
            repo.delete("rule-1").await.unwrap();
            assert!(repo.get("rule-1").await.unwrap().is_none());
        }

        #[tokio::test]
        async fn test_compiled_rule_repository_with_bytecode() {
            use product_farm_json_logic::{parse, CompiledRule, CompilationTier};

            let repo = InMemoryCompiledRuleRepository::new();

            // Create and promote to bytecode
            let expr = parse(&json!({
                "if": [
                    {">": [{"var": "score"}, 90]}, "A",
                    {">": [{"var": "score"}, 80]}, "B",
                    "C"
                ]
            })).unwrap();
            let mut rule = CompiledRule::new(expr);
            rule.promote_to_bytecode().unwrap();

            // Persist with bytecode
            let persisted = rule.to_persisted();
            assert!(persisted.bytecode.is_some());
            repo.save("grading-rule", &persisted).await.unwrap();

            // Restore and verify bytecode is preserved
            let fetched = repo.get("grading-rule").await.unwrap().unwrap();
            assert_eq!(fetched.tier, CompilationTier::Bytecode);
            assert!(fetched.bytecode.is_some());

            // Restore to CompiledRule and verify it works
            let restored = CompiledRule::from_persisted(fetched);
            assert_eq!(restored.tier(), CompilationTier::Bytecode);
            let result = restored.evaluate(&json!({"score": 85})).unwrap();
            assert_eq!(result, product_farm_core::Value::String("B".into()));
        }
    }
}
