//! Repository traits and implementations for persistence operations

use async_trait::async_trait;
use product_farm_core::{
    AbstractAttribute, AbstractPath, DataType, DataTypeId, FunctionalityId,
    Product, ProductFunctionality, ProductId, ProductTemplateEnumeration, Rule, RuleId,
    TemplateEnumerationId, TemplateType,
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

/// Repository for DataType entities
#[async_trait]
pub trait DataTypeRepository: Send + Sync {
    /// Get a data type by its ID
    async fn get(&self, id: &DataTypeId) -> PersistenceResult<Option<DataType>>;

    /// Save a data type
    async fn save(&self, datatype: &DataType) -> PersistenceResult<()>;

    /// Delete a data type
    async fn delete(&self, id: &DataTypeId) -> PersistenceResult<()>;

    /// List all data types with pagination
    async fn list(&self, limit: usize, offset: usize) -> PersistenceResult<Vec<DataType>>;

    /// Count total data types
    async fn count(&self) -> PersistenceResult<usize>;
}

/// Repository for ProductFunctionality entities
#[async_trait]
pub trait FunctionalityRepository: Send + Sync {
    /// Get a functionality by its ID
    async fn get(&self, id: &FunctionalityId) -> PersistenceResult<Option<ProductFunctionality>>;

    /// Get a functionality by product and name
    async fn get_by_name(&self, product_id: &ProductId, name: &str) -> PersistenceResult<Option<ProductFunctionality>>;

    /// Save a functionality
    async fn save(&self, functionality: &ProductFunctionality) -> PersistenceResult<()>;

    /// Delete a functionality
    async fn delete(&self, id: &FunctionalityId) -> PersistenceResult<()>;

    /// Find all functionalities for a product
    async fn find_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<ProductFunctionality>>;

    /// Count functionalities for a product
    async fn count_by_product(&self, product_id: &ProductId) -> PersistenceResult<usize>;
}

/// Repository for ProductTemplateEnumeration entities
#[async_trait]
pub trait EnumerationRepository: Send + Sync {
    /// Get an enumeration by its ID
    async fn get(&self, id: &TemplateEnumerationId) -> PersistenceResult<Option<ProductTemplateEnumeration>>;

    /// Get an enumeration by template type and name
    async fn get_by_name(&self, template_type: &TemplateType, name: &str) -> PersistenceResult<Option<ProductTemplateEnumeration>>;

    /// Save an enumeration
    async fn save(&self, enumeration: &ProductTemplateEnumeration) -> PersistenceResult<()>;

    /// Delete an enumeration
    async fn delete(&self, id: &TemplateEnumerationId) -> PersistenceResult<()>;

    /// Find all enumerations for a template type
    async fn find_by_template(&self, template_type: &TemplateType) -> PersistenceResult<Vec<ProductTemplateEnumeration>>;

    /// List all enumerations with pagination
    async fn list(&self, limit: usize, offset: usize) -> PersistenceResult<Vec<ProductTemplateEnumeration>>;

    /// Count total enumerations
    async fn count(&self) -> PersistenceResult<usize>;
}

/// Result of impact analysis for a rule or attribute change
#[derive(Debug, Clone, Default)]
pub struct ImpactAnalysisResult {
    /// ID of the source (rule or attribute path)
    pub source_id: String,
    /// Directly affected attributes (distance=1)
    pub direct_attributes: Vec<String>,
    /// All affected attributes including transitive
    pub all_affected_attributes: Vec<String>,
    /// Directly affected rules
    pub direct_rules: Vec<RuleId>,
    /// All affected rules including transitive
    pub all_affected_rules: Vec<RuleId>,
    /// Maximum depth traversed
    pub max_depth: usize,
}

/// Graph traversal queries for dependency and impact analysis
///
/// These operations leverage graph structure for efficient traversal.
/// With DGraph, these are native graph queries (O(d) where d=depth).
/// With in-memory storage, they use BFS/DFS (O(n) worst case).
#[async_trait]
pub trait GraphQueries: Send + Sync {
    /// Find all rules that directly depend on a given attribute
    async fn find_rules_depending_on(&self, attribute_path: &str) -> PersistenceResult<Vec<RuleId>>;

    /// Find all attributes directly computed by a rule
    async fn find_computed_attributes(&self, rule_id: &RuleId) -> PersistenceResult<Vec<String>>;

    /// Find transitive upstream dependencies of a rule
    /// Returns all rules that must execute before this rule
    async fn find_upstream_dependencies(&self, rule_id: &RuleId, max_depth: usize) -> PersistenceResult<Vec<RuleId>>;

    /// Find transitive downstream impact of changing a rule
    /// Returns all rules and attributes affected by this change
    async fn find_downstream_impact(&self, rule_id: &RuleId, max_depth: usize) -> PersistenceResult<ImpactAnalysisResult>;

    /// Analyze the impact of changing an attribute
    /// Returns all rules and other attributes affected by this change
    async fn analyze_attribute_impact(&self, attribute_path: &str, max_depth: usize) -> PersistenceResult<ImpactAnalysisResult>;

    /// Get the execution order for rules in a product (topological sort)
    async fn get_execution_order(&self, product_id: &ProductId) -> PersistenceResult<Vec<Vec<RuleId>>>;
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

    /// In-memory DataType repository
    #[derive(Debug)]
    pub struct InMemoryDataTypeRepository {
        datatypes: RwLock<HashMap<DataTypeId, DataType>>,
    }

    impl InMemoryDataTypeRepository {
        pub fn new() -> Self {
            Self {
                datatypes: RwLock::new(HashMap::new()),
            }
        }
    }

    impl Default for InMemoryDataTypeRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl DataTypeRepository for InMemoryDataTypeRepository {
        async fn get(&self, id: &DataTypeId) -> PersistenceResult<Option<DataType>> {
            Ok(self.datatypes.read().map_lock_err()?.get(id).cloned())
        }

        async fn save(&self, datatype: &DataType) -> PersistenceResult<()> {
            self.datatypes.write().map_lock_err()?.insert(datatype.id.clone(), datatype.clone());
            Ok(())
        }

        async fn delete(&self, id: &DataTypeId) -> PersistenceResult<()> {
            self.datatypes.write().map_lock_err()?.remove(id);
            Ok(())
        }

        async fn list(&self, limit: usize, offset: usize) -> PersistenceResult<Vec<DataType>> {
            let datatypes = self.datatypes.read().map_lock_err()?;
            let mut list: Vec<_> = datatypes.values().cloned().collect();
            list.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(list.into_iter().skip(offset).take(limit).collect())
        }

        async fn count(&self) -> PersistenceResult<usize> {
            Ok(self.datatypes.read().map_lock_err()?.len())
        }
    }

    /// In-memory Functionality repository
    #[derive(Debug)]
    pub struct InMemoryFunctionalityRepository {
        functionalities: RwLock<HashMap<FunctionalityId, ProductFunctionality>>,
        by_product: RwLock<HashMap<ProductId, Vec<FunctionalityId>>>,
    }

    impl InMemoryFunctionalityRepository {
        pub fn new() -> Self {
            Self {
                functionalities: RwLock::new(HashMap::new()),
                by_product: RwLock::new(HashMap::new()),
            }
        }
    }

    impl Default for InMemoryFunctionalityRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl FunctionalityRepository for InMemoryFunctionalityRepository {
        async fn get(&self, id: &FunctionalityId) -> PersistenceResult<Option<ProductFunctionality>> {
            Ok(self.functionalities.read().map_lock_err()?.get(id).cloned())
        }

        async fn get_by_name(&self, product_id: &ProductId, name: &str) -> PersistenceResult<Option<ProductFunctionality>> {
            let functionalities = self.functionalities.read().map_lock_err()?;
            let by_product = self.by_product.read().map_lock_err()?;

            if let Some(ids) = by_product.get(product_id) {
                for id in ids {
                    if let Some(func) = functionalities.get(id) {
                        if func.name == name {
                            return Ok(Some(func.clone()));
                        }
                    }
                }
            }
            Ok(None)
        }

        async fn save(&self, functionality: &ProductFunctionality) -> PersistenceResult<()> {
            let id = functionality.id.clone();
            let product_id = functionality.product_id.clone();

            self.functionalities.write().map_lock_err()?.insert(id.clone(), functionality.clone());

            let mut by_product = self.by_product.write().map_lock_err()?;
            let ids = by_product.entry(product_id).or_default();
            if !ids.contains(&id) {
                ids.push(id);
            }

            Ok(())
        }

        async fn delete(&self, id: &FunctionalityId) -> PersistenceResult<()> {
            if let Some(func) = self.functionalities.write().map_lock_err()?.remove(id) {
                let mut by_product = self.by_product.write().map_lock_err()?;
                if let Some(ids) = by_product.get_mut(&func.product_id) {
                    ids.retain(|i| i != id);
                }
            }
            Ok(())
        }

        async fn find_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<ProductFunctionality>> {
            let by_product = self.by_product.read().map_lock_err()?;
            let functionalities = self.functionalities.read().map_lock_err()?;

            let ids = by_product.get(product_id).cloned().unwrap_or_default();
            Ok(ids.iter().filter_map(|id| functionalities.get(id).cloned()).collect())
        }

        async fn count_by_product(&self, product_id: &ProductId) -> PersistenceResult<usize> {
            let by_product = self.by_product.read().map_lock_err()?;
            Ok(by_product.get(product_id).map(|v| v.len()).unwrap_or(0))
        }
    }

    /// In-memory Enumeration repository
    #[derive(Debug)]
    pub struct InMemoryEnumerationRepository {
        enumerations: RwLock<HashMap<TemplateEnumerationId, ProductTemplateEnumeration>>,
        by_template: RwLock<HashMap<TemplateType, Vec<TemplateEnumerationId>>>,
    }

    impl InMemoryEnumerationRepository {
        pub fn new() -> Self {
            Self {
                enumerations: RwLock::new(HashMap::new()),
                by_template: RwLock::new(HashMap::new()),
            }
        }
    }

    impl Default for InMemoryEnumerationRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl EnumerationRepository for InMemoryEnumerationRepository {
        async fn get(&self, id: &TemplateEnumerationId) -> PersistenceResult<Option<ProductTemplateEnumeration>> {
            Ok(self.enumerations.read().map_lock_err()?.get(id).cloned())
        }

        async fn get_by_name(&self, template_type: &TemplateType, name: &str) -> PersistenceResult<Option<ProductTemplateEnumeration>> {
            let enumerations = self.enumerations.read().map_lock_err()?;
            let by_template = self.by_template.read().map_lock_err()?;

            if let Some(ids) = by_template.get(template_type) {
                for id in ids {
                    if let Some(enum_def) = enumerations.get(id) {
                        if enum_def.name == name {
                            return Ok(Some(enum_def.clone()));
                        }
                    }
                }
            }
            Ok(None)
        }

        async fn save(&self, enumeration: &ProductTemplateEnumeration) -> PersistenceResult<()> {
            let id = enumeration.id.clone();
            let template_type = enumeration.template_type.clone();

            self.enumerations.write().map_lock_err()?.insert(id.clone(), enumeration.clone());

            let mut by_template = self.by_template.write().map_lock_err()?;
            let ids = by_template.entry(template_type).or_default();
            if !ids.contains(&id) {
                ids.push(id);
            }

            Ok(())
        }

        async fn delete(&self, id: &TemplateEnumerationId) -> PersistenceResult<()> {
            if let Some(enum_def) = self.enumerations.write().map_lock_err()?.remove(id) {
                let mut by_template = self.by_template.write().map_lock_err()?;
                if let Some(ids) = by_template.get_mut(&enum_def.template_type) {
                    ids.retain(|i| i != id);
                }
            }
            Ok(())
        }

        async fn find_by_template(&self, template_type: &TemplateType) -> PersistenceResult<Vec<ProductTemplateEnumeration>> {
            let by_template = self.by_template.read().map_lock_err()?;
            let enumerations = self.enumerations.read().map_lock_err()?;

            let ids = by_template.get(template_type).cloned().unwrap_or_default();
            Ok(ids.iter().filter_map(|id| enumerations.get(id).cloned()).collect())
        }

        async fn list(&self, limit: usize, offset: usize) -> PersistenceResult<Vec<ProductTemplateEnumeration>> {
            let enumerations = self.enumerations.read().map_lock_err()?;
            let mut list: Vec<_> = enumerations.values().cloned().collect();
            list.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
            Ok(list.into_iter().skip(offset).take(limit).collect())
        }

        async fn count(&self) -> PersistenceResult<usize> {
            Ok(self.enumerations.read().map_lock_err()?.len())
        }
    }

    /// In-memory graph queries implementation
    ///
    /// Uses BFS traversal over the rule dependency data.
    /// For production with large rule sets, use DGraph implementation instead.
    #[derive(Debug)]
    pub struct InMemoryGraphQueries {
        rules: RwLock<HashMap<RuleId, Rule>>,
    }

    impl InMemoryGraphQueries {
        pub fn new() -> Self {
            Self {
                rules: RwLock::new(HashMap::new()),
            }
        }

        /// Create with initial rules
        pub fn with_rules(rules: Vec<Rule>) -> Self {
            let map: HashMap<RuleId, Rule> = rules.into_iter().map(|r| (r.id.clone(), r)).collect();
            Self {
                rules: RwLock::new(map),
            }
        }

        /// Add or update a rule
        pub fn add_rule(&self, rule: Rule) -> PersistenceResult<()> {
            self.rules.write().map_lock_err()?.insert(rule.id.clone(), rule);
            Ok(())
        }

        /// Remove a rule
        pub fn remove_rule(&self, rule_id: &RuleId) -> PersistenceResult<()> {
            self.rules.write().map_lock_err()?.remove(rule_id);
            Ok(())
        }

        /// Sync rules from an external source
        pub fn sync_rules(&self, rules: Vec<Rule>) -> PersistenceResult<()> {
            let mut map = self.rules.write().map_lock_err()?;
            map.clear();
            for rule in rules {
                map.insert(rule.id.clone(), rule);
            }
            Ok(())
        }
    }

    impl Default for InMemoryGraphQueries {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl GraphQueries for InMemoryGraphQueries {
        async fn find_rules_depending_on(&self, attribute_path: &str) -> PersistenceResult<Vec<RuleId>> {
            let rules = self.rules.read().map_lock_err()?;
            let mut result = Vec::new();

            for rule in rules.values() {
                if rule.input_attributes.iter().any(|a| a.path.as_str() == attribute_path) {
                    result.push(rule.id.clone());
                }
            }

            Ok(result)
        }

        async fn find_computed_attributes(&self, rule_id: &RuleId) -> PersistenceResult<Vec<String>> {
            let rules = self.rules.read().map_lock_err()?;

            if let Some(rule) = rules.get(rule_id) {
                Ok(rule.output_attributes.iter().map(|a| a.path.as_str().to_string()).collect())
            } else {
                Ok(Vec::new())
            }
        }

        async fn find_upstream_dependencies(&self, rule_id: &RuleId, max_depth: usize) -> PersistenceResult<Vec<RuleId>> {
            let rules = self.rules.read().map_lock_err()?;

            // Build output -> rule map
            let mut output_to_rule: HashMap<String, RuleId> = HashMap::new();
            for rule in rules.values() {
                for output in &rule.output_attributes {
                    output_to_rule.insert(output.path.as_str().to_string(), rule.id.clone());
                }
            }

            let mut result = Vec::new();
            let mut visited: std::collections::HashSet<RuleId> = std::collections::HashSet::new();
            let mut queue: std::collections::VecDeque<(RuleId, usize)> = std::collections::VecDeque::new();

            if let Some(rule) = rules.get(rule_id) {
                for input in &rule.input_attributes {
                    if let Some(producing_rule_id) = output_to_rule.get(input.path.as_str()) {
                        if !visited.contains(producing_rule_id) {
                            visited.insert(producing_rule_id.clone());
                            queue.push_back((producing_rule_id.clone(), 1));
                        }
                    }
                }
            }

            while let Some((current_id, depth)) = queue.pop_front() {
                result.push(current_id.clone());

                if depth < max_depth {
                    if let Some(current_rule) = rules.get(&current_id) {
                        for input in &current_rule.input_attributes {
                            if let Some(producing_rule_id) = output_to_rule.get(input.path.as_str()) {
                                if !visited.contains(producing_rule_id) {
                                    visited.insert(producing_rule_id.clone());
                                    queue.push_back((producing_rule_id.clone(), depth + 1));
                                }
                            }
                        }
                    }
                }
            }

            Ok(result)
        }

        async fn find_downstream_impact(&self, rule_id: &RuleId, max_depth: usize) -> PersistenceResult<ImpactAnalysisResult> {
            let rules = self.rules.read().map_lock_err()?;

            // Build input -> rules map (rules that consume each attribute)
            let mut input_to_rules: HashMap<String, Vec<RuleId>> = HashMap::new();
            for rule in rules.values() {
                for input in &rule.input_attributes {
                    input_to_rules
                        .entry(input.path.as_str().to_string())
                        .or_default()
                        .push(rule.id.clone());
                }
            }

            let mut all_affected_attributes = Vec::new();
            let mut all_affected_rules = Vec::new();
            let mut direct_attributes = Vec::new();
            let mut direct_rules = Vec::new();
            let mut visited_attrs: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut visited_rules: std::collections::HashSet<RuleId> = std::collections::HashSet::new();

            // Start with the rule's outputs
            if let Some(rule) = rules.get(rule_id) {
                for output in &rule.output_attributes {
                    let path = output.path.as_str().to_string();
                    if !visited_attrs.contains(&path) {
                        visited_attrs.insert(path.clone());
                        direct_attributes.push(path.clone());
                        all_affected_attributes.push(path);
                    }
                }
            }

            // BFS through downstream dependencies
            let mut current_level_attrs: Vec<String> = direct_attributes.clone();
            let mut depth = 1;

            while !current_level_attrs.is_empty() && depth <= max_depth {
                let mut next_level_attrs = Vec::new();

                for attr_path in &current_level_attrs {
                    // Find rules that consume this attribute
                    if let Some(consuming_rules) = input_to_rules.get(attr_path) {
                        for consuming_rule_id in consuming_rules {
                            if consuming_rule_id != rule_id && !visited_rules.contains(consuming_rule_id) {
                                visited_rules.insert(consuming_rule_id.clone());

                                if depth == 1 {
                                    direct_rules.push(consuming_rule_id.clone());
                                }
                                all_affected_rules.push(consuming_rule_id.clone());

                                // Add this rule's outputs to next level
                                if let Some(consuming_rule) = rules.get(consuming_rule_id) {
                                    for output in &consuming_rule.output_attributes {
                                        let path = output.path.as_str().to_string();
                                        if !visited_attrs.contains(&path) {
                                            visited_attrs.insert(path.clone());
                                            all_affected_attributes.push(path.clone());
                                            next_level_attrs.push(path);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                current_level_attrs = next_level_attrs;
                depth += 1;
            }

            Ok(ImpactAnalysisResult {
                source_id: rule_id.to_string(),
                direct_attributes,
                all_affected_attributes,
                direct_rules,
                all_affected_rules,
                max_depth,
            })
        }

        async fn analyze_attribute_impact(&self, attribute_path: &str, max_depth: usize) -> PersistenceResult<ImpactAnalysisResult> {
            // Find the rule that produces this attribute
            let rules = self.rules.read().map_lock_err()?;

            // Find rules that consume this attribute (downstream)
            let mut direct_rules = Vec::new();
            for rule in rules.values() {
                if rule.input_attributes.iter().any(|a| a.path.as_str() == attribute_path) {
                    direct_rules.push(rule.id.clone());
                }
            }

            // Build result by analyzing downstream impact from consuming rules
            let mut all_affected_attributes = Vec::new();
            let mut all_affected_rules = direct_rules.clone();
            let mut visited_attrs: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut visited_rules: std::collections::HashSet<RuleId> = direct_rules.iter().cloned().collect();

            // Add outputs of direct consuming rules
            let mut direct_attributes = Vec::new();
            for rule_id in &direct_rules {
                if let Some(rule) = rules.get(rule_id) {
                    for output in &rule.output_attributes {
                        let path = output.path.as_str().to_string();
                        if !visited_attrs.contains(&path) {
                            visited_attrs.insert(path.clone());
                            direct_attributes.push(path.clone());
                            all_affected_attributes.push(path);
                        }
                    }
                }
            }

            // Continue BFS for transitive impact
            // Build input -> rules map
            let mut input_to_rules: HashMap<String, Vec<RuleId>> = HashMap::new();
            for rule in rules.values() {
                for input in &rule.input_attributes {
                    input_to_rules
                        .entry(input.path.as_str().to_string())
                        .or_default()
                        .push(rule.id.clone());
                }
            }

            let mut current_level_attrs: Vec<String> = direct_attributes.clone();
            let mut depth = 2; // Start at 2 since we already processed depth 1

            while !current_level_attrs.is_empty() && depth <= max_depth {
                let mut next_level_attrs = Vec::new();

                for attr_path in &current_level_attrs {
                    if let Some(consuming_rules) = input_to_rules.get(attr_path) {
                        for consuming_rule_id in consuming_rules {
                            if !visited_rules.contains(consuming_rule_id) {
                                visited_rules.insert(consuming_rule_id.clone());
                                all_affected_rules.push(consuming_rule_id.clone());

                                if let Some(consuming_rule) = rules.get(consuming_rule_id) {
                                    for output in &consuming_rule.output_attributes {
                                        let path = output.path.as_str().to_string();
                                        if !visited_attrs.contains(&path) {
                                            visited_attrs.insert(path.clone());
                                            all_affected_attributes.push(path.clone());
                                            next_level_attrs.push(path);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                current_level_attrs = next_level_attrs;
                depth += 1;
            }

            Ok(ImpactAnalysisResult {
                source_id: attribute_path.to_string(),
                direct_attributes,
                all_affected_attributes,
                direct_rules,
                all_affected_rules,
                max_depth,
            })
        }

        async fn get_execution_order(&self, product_id: &ProductId) -> PersistenceResult<Vec<Vec<RuleId>>> {
            let rules = self.rules.read().map_lock_err()?;

            // Filter rules for this product
            let product_rules: Vec<&Rule> = rules.values()
                .filter(|r| r.product_id == *product_id && r.enabled)
                .collect();

            if product_rules.is_empty() {
                return Ok(Vec::new());
            }

            // Build output -> rule map
            let mut output_to_rule: HashMap<String, RuleId> = HashMap::new();
            for rule in &product_rules {
                for output in &rule.output_attributes {
                    output_to_rule.insert(output.path.as_str().to_string(), rule.id.clone());
                }
            }

            // Calculate in-degree for each rule (number of dependencies)
            let mut in_degree: HashMap<RuleId, usize> = HashMap::new();
            let mut adj: HashMap<RuleId, Vec<RuleId>> = HashMap::new();

            for rule in &product_rules {
                in_degree.entry(rule.id.clone()).or_insert(0);

                for input in &rule.input_attributes {
                    if let Some(producing_rule_id) = output_to_rule.get(input.path.as_str()) {
                        if producing_rule_id != &rule.id {
                            adj.entry(producing_rule_id.clone()).or_default().push(rule.id.clone());
                            *in_degree.entry(rule.id.clone()).or_insert(0) += 1;
                        }
                    }
                }
            }

            // Kahn's algorithm for topological sort with levels
            let mut levels: Vec<Vec<RuleId>> = Vec::new();
            let mut queue: Vec<RuleId> = in_degree
                .iter()
                .filter(|(_, &deg)| deg == 0)
                .map(|(id, _)| id.clone())
                .collect();

            while !queue.is_empty() {
                levels.push(queue.clone());

                let mut next_queue = Vec::new();
                for rule_id in &queue {
                    if let Some(dependents) = adj.get(rule_id) {
                        for dependent in dependents {
                            if let Some(deg) = in_degree.get_mut(dependent) {
                                *deg -= 1;
                                if *deg == 0 {
                                    next_queue.push(dependent.clone());
                                }
                            }
                        }
                    }
                }
                queue = next_queue;
            }

            Ok(levels)
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
