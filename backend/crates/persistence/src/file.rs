//! File-based persistence for Product-FARM
//!
//! Provides JSON file storage for products, attributes, and rules.
//! Useful for development, testing, and simple deployments.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use product_farm_core::{AbstractAttribute, AbstractPath, Product, ProductId, Rule, RuleId};
use product_farm_json_logic::PersistedRule;
use crate::error::{PersistenceError, PersistenceResult};
use crate::repository::{AttributeRepository, CompiledRuleRepository, ProductRepository, RuleRepository};

/// File-based storage configuration
#[derive(Debug, Clone)]
pub struct FileStorageConfig {
    /// Base directory for storage
    pub base_dir: PathBuf,
    /// Whether to pretty-print JSON files
    pub pretty_print: bool,
}

impl FileStorageConfig {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            pretty_print: true,
        }
    }

    pub fn with_pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }
}

impl Default for FileStorageConfig {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("./data"),
            pretty_print: true,
        }
    }
}

/// File-based Product repository
pub struct FileProductRepository {
    config: FileStorageConfig,
}

impl FileProductRepository {
    pub async fn new(config: FileStorageConfig) -> PersistenceResult<Self> {
        let products_dir = config.base_dir.join("products");
        fs::create_dir_all(&products_dir).await
            .map_err(|e| PersistenceError::ConnectionError(format!("Failed to create directory: {}", e)))?;
        Ok(Self { config })
    }

    fn product_path(&self, id: &ProductId) -> PathBuf {
        self.config.base_dir.join("products").join(format!("{}.json", id.as_str()))
    }

    async fn read_product(&self, path: &Path) -> PersistenceResult<Product> {
        let mut file = fs::File::open(path).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to open file: {}", e)))?;

        let mut content = String::new();
        file.read_to_string(&mut content).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to read file: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| PersistenceError::SerializationError(format!("Failed to parse JSON: {}", e)))
    }
}

#[async_trait]
impl ProductRepository for FileProductRepository {
    async fn get(&self, id: &ProductId) -> PersistenceResult<Option<Product>> {
        let path = self.product_path(id);
        if !path.exists() {
            return Ok(None);
        }
        self.read_product(&path).await.map(Some)
    }

    async fn save(&self, product: &Product) -> PersistenceResult<()> {
        let path = self.product_path(&product.id);

        let content = if self.config.pretty_print {
            serde_json::to_string_pretty(product)
        } else {
            serde_json::to_string(product)
        }.map_err(|e| PersistenceError::SerializationError(format!("Failed to serialize: {}", e)))?;

        let mut file = fs::File::create(&path).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to create file: {}", e)))?;

        file.write_all(content.as_bytes()).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: &ProductId) -> PersistenceResult<()> {
        let path = self.product_path(id);
        if path.exists() {
            fs::remove_file(&path).await
                .map_err(|e| PersistenceError::QueryError(format!("Failed to delete file: {}", e)))?;
        }
        Ok(())
    }

    async fn list(&self, limit: usize, offset: usize) -> PersistenceResult<Vec<Product>> {
        let products_dir = self.config.base_dir.join("products");
        let mut entries = fs::read_dir(&products_dir).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to read directory: {}", e)))?;

        let mut products = Vec::new();
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to read entry: {}", e)))? {

            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(product) = self.read_product(&path).await {
                    products.push(product);
                }
            }
        }

        // Sort by id for consistent ordering
        products.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));

        Ok(products.into_iter().skip(offset).take(limit).collect())
    }

    async fn count(&self) -> PersistenceResult<usize> {
        let products_dir = self.config.base_dir.join("products");
        let mut entries = fs::read_dir(&products_dir).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to read directory: {}", e)))?;

        let mut count = 0;
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to read entry: {}", e)))? {
            if entry.path().extension().is_some_and(|ext| ext == "json") {
                count += 1;
            }
        }
        Ok(count)
    }
}

/// File-based Rule repository
pub struct FileRuleRepository {
    config: FileStorageConfig,
}

impl FileRuleRepository {
    pub async fn new(config: FileStorageConfig) -> PersistenceResult<Self> {
        let rules_dir = config.base_dir.join("rules");
        fs::create_dir_all(&rules_dir).await
            .map_err(|e| PersistenceError::ConnectionError(format!("Failed to create directory: {}", e)))?;
        Ok(Self { config })
    }

    fn rule_path(&self, id: &RuleId) -> PathBuf {
        self.config.base_dir.join("rules").join(format!("{}.json", id))
    }

    fn product_index_path(&self, product_id: &ProductId) -> PathBuf {
        self.config.base_dir.join("rules").join(format!("_index_{}.json", product_id.as_str()))
    }

    async fn read_rule(&self, path: &Path) -> PersistenceResult<Rule> {
        let mut file = fs::File::open(path).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to open file: {}", e)))?;

        let mut content = String::new();
        file.read_to_string(&mut content).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to read file: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| PersistenceError::SerializationError(format!("Failed to parse JSON: {}", e)))
    }

    async fn read_product_index(&self, product_id: &ProductId) -> PersistenceResult<Vec<String>> {
        let path = self.product_index_path(product_id);
        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut file = fs::File::open(&path).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to open index: {}", e)))?;

        let mut content = String::new();
        file.read_to_string(&mut content).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to read index: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| PersistenceError::SerializationError(format!("Failed to parse index: {}", e)))
    }

    async fn write_product_index(&self, product_id: &ProductId, rule_ids: &[String]) -> PersistenceResult<()> {
        let path = self.product_index_path(product_id);

        let content = serde_json::to_string_pretty(rule_ids)
            .map_err(|e| PersistenceError::SerializationError(format!("Failed to serialize index: {}", e)))?;

        let mut file = fs::File::create(&path).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to create index: {}", e)))?;

        file.write_all(content.as_bytes()).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to write index: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl RuleRepository for FileRuleRepository {
    async fn get(&self, id: &RuleId) -> PersistenceResult<Option<Rule>> {
        let path = self.rule_path(id);
        if !path.exists() {
            return Ok(None);
        }
        self.read_rule(&path).await.map(Some)
    }

    async fn save(&self, rule: &Rule) -> PersistenceResult<()> {
        let path = self.rule_path(&rule.id);

        let content = if self.config.pretty_print {
            serde_json::to_string_pretty(rule)
        } else {
            serde_json::to_string(rule)
        }.map_err(|e| PersistenceError::SerializationError(format!("Failed to serialize: {}", e)))?;

        let mut file = fs::File::create(&path).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to create file: {}", e)))?;

        file.write_all(content.as_bytes()).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to write file: {}", e)))?;

        // Update product index
        let mut index = self.read_product_index(&rule.product_id).await?;
        let rule_id_str = rule.id.to_string();
        if !index.contains(&rule_id_str) {
            index.push(rule_id_str);
            self.write_product_index(&rule.product_id, &index).await?;
        }

        Ok(())
    }

    async fn delete(&self, id: &RuleId) -> PersistenceResult<()> {
        // First get the rule to find its product_id
        if let Some(rule) = self.get(id).await? {
            let path = self.rule_path(id);
            if path.exists() {
                fs::remove_file(&path).await
                    .map_err(|e| PersistenceError::QueryError(format!("Failed to delete file: {}", e)))?;
            }

            // Update product index
            let mut index = self.read_product_index(&rule.product_id).await?;
            let rule_id_str = id.to_string();
            index.retain(|i| i != &rule_id_str);
            self.write_product_index(&rule.product_id, &index).await?;
        }
        Ok(())
    }

    async fn find_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<Rule>> {
        let index = self.read_product_index(product_id).await?;

        let mut rules = Vec::new();
        for rule_id_str in index {
            let rule_id = RuleId::from_string(&rule_id_str);
            if let Some(rule) = self.get(&rule_id).await? {
                rules.push(rule);
            }
        }

        // Sort by order_index
        rules.sort_by_key(|r| r.order_index);
        Ok(rules)
    }

    async fn find_enabled_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<Rule>> {
        let all = self.find_by_product(product_id).await?;
        Ok(all.into_iter().filter(|r| r.enabled).collect())
    }
}

/// File-based Attribute repository (for AbstractAttribute templates)
pub struct FileAttributeRepository {
    config: FileStorageConfig,
}

impl FileAttributeRepository {
    pub async fn new(config: FileStorageConfig) -> PersistenceResult<Self> {
        let attrs_dir = config.base_dir.join("attributes");
        fs::create_dir_all(&attrs_dir).await
            .map_err(|e| PersistenceError::ConnectionError(format!("Failed to create directory: {}", e)))?;
        Ok(Self { config })
    }

    fn attribute_path(&self, path: &AbstractPath) -> PathBuf {
        // Use URL-safe encoding for the path (replace dots with underscores for filename)
        let safe_name = path.as_str().replace('.', "_");
        self.config.base_dir.join("attributes").join(format!("{}.json", safe_name))
    }

    fn product_index_path(&self, product_id: &ProductId) -> PathBuf {
        self.config.base_dir.join("attributes").join(format!("_index_{}.json", product_id.as_str()))
    }

    async fn read_attribute(&self, path: &Path) -> PersistenceResult<AbstractAttribute> {
        let mut file = fs::File::open(path).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to open file: {}", e)))?;

        let mut content = String::new();
        file.read_to_string(&mut content).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to read file: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| PersistenceError::SerializationError(format!("Failed to parse JSON: {}", e)))
    }

    async fn read_product_index(&self, product_id: &ProductId) -> PersistenceResult<Vec<String>> {
        let path = self.product_index_path(product_id);
        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut file = fs::File::open(&path).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to open index: {}", e)))?;

        let mut content = String::new();
        file.read_to_string(&mut content).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to read index: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| PersistenceError::SerializationError(format!("Failed to parse index: {}", e)))
    }

    async fn write_product_index(&self, product_id: &ProductId, attr_paths: &[String]) -> PersistenceResult<()> {
        let path = self.product_index_path(product_id);

        let content = serde_json::to_string_pretty(attr_paths)
            .map_err(|e| PersistenceError::SerializationError(format!("Failed to serialize index: {}", e)))?;

        let mut file = fs::File::create(&path).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to create index: {}", e)))?;

        file.write_all(content.as_bytes()).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to write index: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl AttributeRepository for FileAttributeRepository {
    async fn get(&self, abstract_path: &AbstractPath) -> PersistenceResult<Option<AbstractAttribute>> {
        let path = self.attribute_path(abstract_path);
        if !path.exists() {
            return Ok(None);
        }
        self.read_attribute(&path).await.map(Some)
    }

    async fn save(&self, attribute: &AbstractAttribute) -> PersistenceResult<()> {
        let path = self.attribute_path(&attribute.abstract_path);

        let content = if self.config.pretty_print {
            serde_json::to_string_pretty(attribute)
        } else {
            serde_json::to_string(attribute)
        }.map_err(|e| PersistenceError::SerializationError(format!("Failed to serialize: {}", e)))?;

        let mut file = fs::File::create(&path).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to create file: {}", e)))?;

        file.write_all(content.as_bytes()).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to write file: {}", e)))?;

        // Update product index
        let mut index = self.read_product_index(&attribute.product_id).await?;
        let attr_path_str = attribute.abstract_path.as_str().to_string();
        if !index.contains(&attr_path_str) {
            index.push(attr_path_str);
            self.write_product_index(&attribute.product_id, &index).await?;
        }

        Ok(())
    }

    async fn delete(&self, abstract_path: &AbstractPath) -> PersistenceResult<()> {
        // First get the attribute to find its product_id
        if let Some(attr) = self.get(abstract_path).await? {
            let path = self.attribute_path(abstract_path);
            if path.exists() {
                fs::remove_file(&path).await
                    .map_err(|e| PersistenceError::QueryError(format!("Failed to delete file: {}", e)))?;
            }

            // Update product index
            let mut index = self.read_product_index(&attr.product_id).await?;
            let attr_path_str = abstract_path.as_str().to_string();
            index.retain(|i| i != &attr_path_str);
            self.write_product_index(&attr.product_id, &index).await?;
        }
        Ok(())
    }

    async fn find_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<AbstractAttribute>> {
        let index = self.read_product_index(product_id).await?;

        let mut attrs = Vec::new();
        for attr_path_str in index {
            let attr_path = AbstractPath::new(&attr_path_str);
            if let Some(attr) = self.get(&attr_path).await? {
                attrs.push(attr);
            }
        }

        Ok(attrs)
    }

    async fn find_by_tag(&self, product_id: &ProductId, tag: &str) -> PersistenceResult<Vec<AbstractAttribute>> {
        let attrs = self.find_by_product(product_id).await?;
        Ok(attrs.into_iter().filter(|a| a.has_tag(tag)).collect())
    }
}

/// File-based Compiled Rule repository (for bytecode persistence)
pub struct FileCompiledRuleRepository {
    config: FileStorageConfig,
}

impl FileCompiledRuleRepository {
    pub async fn new(config: FileStorageConfig) -> PersistenceResult<Self> {
        let compiled_dir = config.base_dir.join("compiled");
        fs::create_dir_all(&compiled_dir).await
            .map_err(|e| PersistenceError::ConnectionError(format!("Failed to create directory: {}", e)))?;
        Ok(Self { config })
    }

    fn compiled_rule_path(&self, rule_id: &str) -> PathBuf {
        // Sanitize rule_id for filename safety
        let safe_id = rule_id.replace(['/', '\\', ':'], "_");
        self.config.base_dir.join("compiled").join(format!("{}.json", safe_id))
    }
}

#[async_trait]
impl CompiledRuleRepository for FileCompiledRuleRepository {
    async fn get(&self, rule_id: &str) -> PersistenceResult<Option<PersistedRule>> {
        let path = self.compiled_rule_path(rule_id);
        if !path.exists() {
            return Ok(None);
        }

        let mut file = fs::File::open(&path).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to open file: {}", e)))?;

        let mut content = String::new();
        file.read_to_string(&mut content).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to read file: {}", e)))?;

        let rule: PersistedRule = serde_json::from_str(&content)
            .map_err(|e| PersistenceError::SerializationError(format!("Failed to parse JSON: {}", e)))?;

        Ok(Some(rule))
    }

    async fn save(&self, rule_id: &str, rule: &PersistedRule) -> PersistenceResult<()> {
        let path = self.compiled_rule_path(rule_id);

        let content = if self.config.pretty_print {
            serde_json::to_string_pretty(rule)
        } else {
            serde_json::to_string(rule)
        }.map_err(|e| PersistenceError::SerializationError(format!("Failed to serialize: {}", e)))?;

        let mut file = fs::File::create(&path).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to create file: {}", e)))?;

        file.write_all(content.as_bytes()).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, rule_id: &str) -> PersistenceResult<()> {
        let path = self.compiled_rule_path(rule_id);
        if path.exists() {
            fs::remove_file(&path).await
                .map_err(|e| PersistenceError::QueryError(format!("Failed to delete file: {}", e)))?;
        }
        Ok(())
    }

    async fn list_ids(&self) -> PersistenceResult<Vec<String>> {
        let compiled_dir = self.config.base_dir.join("compiled");
        let mut entries = fs::read_dir(&compiled_dir).await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to read directory: {}", e)))?;

        let mut ids = Vec::new();
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| PersistenceError::QueryError(format!("Failed to read entry: {}", e)))? {

            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Some(stem) = path.file_stem() {
                    if let Some(id) = stem.to_str() {
                        ids.push(id.to_string());
                    }
                }
            }
        }

        ids.sort();
        Ok(ids)
    }
}

/// Combined file-based repositories
pub struct FileRepositories {
    pub products: FileProductRepository,
    pub attributes: FileAttributeRepository,
    pub rules: FileRuleRepository,
    pub compiled_rules: FileCompiledRuleRepository,
}

impl FileRepositories {
    pub async fn new(config: FileStorageConfig) -> PersistenceResult<Self> {
        Ok(Self {
            products: FileProductRepository::new(config.clone()).await?,
            attributes: FileAttributeRepository::new(config.clone()).await?,
            rules: FileRuleRepository::new(config.clone()).await?,
            compiled_rules: FileCompiledRuleRepository::new(config).await?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_product_repository() {
        let temp_dir = TempDir::new().unwrap();
        let config = FileStorageConfig::new(temp_dir.path());
        let repo = FileProductRepository::new(config).await.unwrap();

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
    async fn test_file_rule_repository() {
        let temp_dir = TempDir::new().unwrap();
        let config = FileStorageConfig::new(temp_dir.path());
        let repo = FileRuleRepository::new(config).await.unwrap();

        let rule = Rule::from_json_logic("product-1", "calc", json!({"*": [{"var": "x"}, 2]}))
            .with_inputs(["x"])
            .with_outputs(["doubled"]);

        // Save
        repo.save(&rule).await.unwrap();

        // Get
        let fetched = repo.get(&rule.id).await.unwrap();
        assert!(fetched.is_some());

        // Find by product
        let rules = repo.find_by_product(&ProductId::new("product-1")).await.unwrap();
        assert_eq!(rules.len(), 1);

        // Delete
        repo.delete(&rule.id).await.unwrap();
        assert!(repo.get(&rule.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_file_attribute_repository() {
        use product_farm_core::{AttributeDisplayName, DisplayNameFormat};

        let temp_dir = TempDir::new().unwrap();
        let config = FileStorageConfig::new(temp_dir.path());
        let repo = FileAttributeRepository::new(config).await.unwrap();

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
    async fn test_file_compiled_rule_repository() {
        use product_farm_json_logic::{parse, CompiledRule, CompilationTier};

        let temp_dir = TempDir::new().unwrap();
        let config = FileStorageConfig::new(temp_dir.path());
        let repo = FileCompiledRuleRepository::new(config).await.unwrap();

        // Create and promote a rule to bytecode
        let expr = parse(&json!({
            "if": [
                {">": [{"var": "x"}, 10]}, "big",
                "small"
            ]
        })).unwrap();
        let mut rule = CompiledRule::new(expr);
        rule.promote_to_bytecode().unwrap();

        // Save compiled rule
        let persisted = rule.to_persisted();
        repo.save("test-rule", &persisted).await.unwrap();

        // Verify file exists
        let file_path = temp_dir.path().join("compiled").join("test-rule.json");
        assert!(file_path.exists());

        // Get compiled rule
        let fetched = repo.get("test-rule").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.tier, CompilationTier::Bytecode);
        assert!(fetched.bytecode.is_some());

        // Restore and verify it works
        let restored = CompiledRule::from_persisted(fetched);
        let result = restored.evaluate(&json!({"x": 15})).unwrap();
        assert_eq!(result, product_farm_core::Value::String("big".into()));

        // List IDs
        let ids = repo.list_ids().await.unwrap();
        assert_eq!(ids, vec!["test-rule"]);

        // Delete
        repo.delete("test-rule").await.unwrap();
        assert!(repo.get("test-rule").await.unwrap().is_none());
    }
}
