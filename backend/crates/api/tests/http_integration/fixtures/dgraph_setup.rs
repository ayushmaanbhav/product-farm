//! DGraph Test Context and Cleanup
//!
//! Provides DGraph connection management and test data cleanup.

use std::sync::atomic::{AtomicU64, Ordering};
use std::env;

use product_farm_core::{
    AbstractAttribute, AbstractPath, DataType, DataTypeId, PrimitiveType, Product, ProductId,
    ProductStatus, Rule, RuleId, TemplateType,
};
use product_farm_persistence::{
    AttributeRepository, CompiledRuleRepository, DgraphConfig, DgraphRepositories,
    ProductRepository, RuleRepository,
};
use chrono::Utc;

/// Global counter for unique test prefixes
static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// DGraph test context with repositories and cleanup capability
pub struct DgraphTestContext {
    /// Direct access to DGraph repositories
    pub repos: DgraphRepositories,
    /// Unique test prefix for isolation
    pub test_prefix: String,
    /// DGraph endpoint
    pub endpoint: String,
    /// Track created entity IDs for cleanup
    created_products: Vec<String>,
    created_rules: Vec<String>,
    created_attributes: Vec<String>,
}

impl DgraphTestContext {
    /// Create a new test context with DGraph connection
    pub async fn new() -> Result<Self, DgraphTestError> {
        let endpoint = Self::get_endpoint();
        let config = DgraphConfig { endpoint: endpoint.clone() };

        let repos = DgraphRepositories::new(config)
            .map_err(|e| DgraphTestError::ConnectionFailed(e.to_string()))?;

        // Generate unique test prefix
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let test_prefix = format!("test{}n{}", timestamp, counter);

        Ok(Self {
            repos,
            test_prefix,
            endpoint,
            created_products: Vec::new(),
            created_rules: Vec::new(),
            created_attributes: Vec::new(),
        })
    }

    /// Create a memory-only context (no DGraph connection, just unique ID generation)
    pub fn new_memory() -> Self {
        // Generate unique test prefix
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let test_prefix = format!("test{}n{}", timestamp, counter);

        // Create with dummy repos - they won't be used in memory-only mode
        let config = DgraphConfig { endpoint: "http://dummy:9080".to_string() };
        let repos = DgraphRepositories::new(config).expect("Dummy config should work");

        Self {
            repos,
            test_prefix,
            endpoint: "memory".to_string(),
            created_products: Vec::new(),
            created_rules: Vec::new(),
            created_attributes: Vec::new(),
        }
    }

    /// Get DGraph endpoint from environment or default
    pub fn get_endpoint() -> String {
        env::var("DGRAPH_TEST_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:9080".to_string())
    }

    /// Generate a unique ID with the test prefix
    pub fn unique_id(&self, base: &str) -> String {
        format!("{}_{}", self.test_prefix, base)
    }

    /// Generate a unique product ID (valid format)
    pub fn unique_product_id(&self, base: &str) -> String {
        // Product IDs must start with letter and contain alphanumeric with underscores
        // Hyphens are NOT allowed (despite what the error message says)
        format!("t{}_{}", self.test_prefix.replace('-', ""), base.replace('-', "_"))
    }

    // =========================================================================
    // Seeding Methods - Create test data directly in DGraph
    // =========================================================================

    /// Seed a product in DGraph
    pub async fn seed_product(&mut self, id: &str) -> Result<Product, DgraphTestError> {
        let product_id = self.unique_product_id(id);
        let product = Product::new(product_id.clone(), format!("Test {}", id), "insurance", Utc::now());

        self.repos.products.save(&product).await
            .map_err(|e| DgraphTestError::SeedError(format!("Failed to seed product: {}", e)))?;

        self.created_products.push(product_id);
        Ok(product)
    }

    /// Seed a product with specific template type
    pub async fn seed_product_with_template(
        &mut self,
        id: &str,
        template_type: &str,
    ) -> Result<Product, DgraphTestError> {
        let product_id = self.unique_product_id(id);
        let product = Product::new(product_id.clone(), format!("Test {}", id), template_type, Utc::now());

        self.repos.products.save(&product).await
            .map_err(|e| DgraphTestError::SeedError(format!("Failed to seed product: {}", e)))?;

        self.created_products.push(product_id);
        Ok(product)
    }

    /// Seed an abstract attribute in DGraph
    pub async fn seed_abstract_attribute(
        &mut self,
        product_id: &str,
        component_type: &str,
        attribute_name: &str,
    ) -> Result<AbstractAttribute, DgraphTestError> {
        let path = format!("{}:abstract-path:{}:{}", product_id, component_type, attribute_name);
        let attr = AbstractAttribute::new(
            path.clone(),
            product_id,
            component_type,
            DataTypeId::new("decimal"),
        );

        self.repos.attributes.save(&attr).await
            .map_err(|e| DgraphTestError::SeedError(format!("Failed to seed attribute: {}", e)))?;

        self.created_attributes.push(path);
        Ok(attr)
    }

    /// Seed a rule in DGraph
    pub async fn seed_rule(
        &mut self,
        product_id: &str,
        rule_type: &str,
        expression: serde_json::Value,
    ) -> Result<Rule, DgraphTestError> {
        let rule = Rule::from_json_logic(product_id, rule_type, expression)
            .with_inputs(["input"])
            .with_outputs(["output"]);

        let rule_id = rule.id.to_string();

        self.repos.rules.save(&rule).await
            .map_err(|e| DgraphTestError::SeedError(format!("Failed to seed rule: {}", e)))?;

        self.created_rules.push(rule_id);
        Ok(rule)
    }

    // =========================================================================
    // Verification Methods - Check DGraph state
    // =========================================================================

    /// Verify a product exists in DGraph
    pub async fn verify_product_exists(&self, id: &str) -> bool {
        self.repos.products.get(&ProductId::new(id)).await
            .map(|p| p.is_some())
            .unwrap_or(false)
    }

    /// Verify product status
    pub async fn verify_product_status(
        &self,
        id: &str,
        expected: ProductStatus,
    ) -> Result<bool, DgraphTestError> {
        let product = self.repos.products.get(&ProductId::new(id)).await
            .map_err(|e| DgraphTestError::QueryError(e.to_string()))?
            .ok_or_else(|| DgraphTestError::NotFound(format!("Product {} not found", id)))?;

        Ok(product.status == expected)
    }

    /// Get product version
    pub async fn get_product_version(&self, id: &str) -> Result<u64, DgraphTestError> {
        let product = self.repos.products.get(&ProductId::new(id)).await
            .map_err(|e| DgraphTestError::QueryError(e.to_string()))?
            .ok_or_else(|| DgraphTestError::NotFound(format!("Product {} not found", id)))?;

        Ok(product.version)
    }

    /// Get product from DGraph
    pub async fn get_product(&self, id: &str) -> Result<Option<Product>, DgraphTestError> {
        self.repos.products.get(&ProductId::new(id)).await
            .map_err(|e| DgraphTestError::QueryError(e.to_string()))
    }

    /// Get rule from DGraph
    pub async fn get_rule(&self, id: &str) -> Result<Option<Rule>, DgraphTestError> {
        self.repos.rules.get(&RuleId::from_string(id)).await
            .map_err(|e| DgraphTestError::QueryError(e.to_string()))
    }

    /// Get attribute from DGraph
    pub async fn get_attribute(&self, path: &str) -> Result<Option<AbstractAttribute>, DgraphTestError> {
        self.repos.attributes.get(&AbstractPath::new(path)).await
            .map_err(|e| DgraphTestError::QueryError(e.to_string()))
    }

    // =========================================================================
    // Cleanup
    // =========================================================================

    /// Clean up all test data created by this context
    pub async fn cleanup(&mut self) -> Result<(), DgraphTestError> {
        // Delete rules first (they reference products)
        for rule_id in self.created_rules.drain(..) {
            if let Err(e) = self.repos.rules.delete(&RuleId::from_string(&rule_id)).await {
                tracing::warn!("Failed to cleanup rule {}: {}", rule_id, e);
            }
        }

        // Delete attributes (they reference products)
        for attr_path in self.created_attributes.drain(..) {
            if let Err(e) = self.repos.attributes.delete(&AbstractPath::new(&attr_path)).await {
                tracing::warn!("Failed to cleanup attribute {}: {}", attr_path, e);
            }
        }

        // Delete products last
        for product_id in self.created_products.drain(..) {
            if let Err(e) = self.repos.products.delete(&ProductId::new(&product_id)).await {
                tracing::warn!("Failed to cleanup product {}: {}", product_id, e);
            }
        }

        Ok(())
    }

    /// Register a product ID for cleanup
    pub fn track_product(&mut self, id: String) {
        self.created_products.push(id);
    }

    /// Register a rule ID for cleanup
    pub fn track_rule(&mut self, id: String) {
        self.created_rules.push(id);
    }

    /// Register an attribute path for cleanup
    pub fn track_attribute(&mut self, path: String) {
        self.created_attributes.push(path);
    }
}

impl Drop for DgraphTestContext {
    fn drop(&mut self) {
        // Note: async cleanup in Drop is tricky
        // Tests should call cleanup() explicitly before dropping
        if !self.created_products.is_empty()
            || !self.created_rules.is_empty()
            || !self.created_attributes.is_empty()
        {
            tracing::warn!(
                "DgraphTestContext dropped with uncleaned data: {} products, {} rules, {} attributes",
                self.created_products.len(),
                self.created_rules.len(),
                self.created_attributes.len()
            );
        }
    }
}

/// Errors that can occur in test context operations
#[derive(Debug, thiserror::Error)]
pub enum DgraphTestError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Seed error: {0}")]
    SeedError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Cleanup error: {0}")]
    CleanupError(String),
}

/// Combined test context with both HTTP server and DGraph access
pub struct TestContext {
    /// HTTP test server
    pub server: super::TestServer,
    /// DGraph repositories
    pub dgraph: DgraphTestContext,
}

impl TestContext {
    /// Create a new test context with in-memory backend (fast for tests)
    pub async fn new() -> Result<Self, String> {
        let server = super::TestServer::new_memory().await
            .map_err(|e| format!("Failed to create test server: {}", e))?;

        // For in-memory tests, we don't need DGraph context
        // Use a dummy one that just generates unique IDs
        let dgraph = DgraphTestContext::new_memory();

        Ok(Self { server, dgraph })
    }

    /// Create a new test context with DGraph backend
    pub async fn new_with_dgraph() -> Result<Self, String> {
        let endpoint = DgraphTestContext::get_endpoint();

        let dgraph = DgraphTestContext::new().await
            .map_err(|e| format!("Failed to create DGraph context: {}", e))?;

        let server = super::TestServer::new_with_dgraph(&endpoint).await
            .map_err(|e| format!("Failed to create test server: {}", e))?;

        Ok(Self { server, dgraph })
    }

    /// Create a new test context with hybrid storage
    pub async fn new_hybrid(cache_size: usize) -> Result<Self, String> {
        let endpoint = DgraphTestContext::get_endpoint();

        let dgraph = DgraphTestContext::new().await
            .map_err(|e| format!("Failed to create DGraph context: {}", e))?;

        let server = super::TestServer::new_with_hybrid(&endpoint, cache_size).await
            .map_err(|e| format!("Failed to create test server: {}", e))?;

        Ok(Self { server, dgraph })
    }

    /// Generate a unique ID
    pub fn unique_id(&self, base: &str) -> String {
        self.dgraph.unique_id(base)
    }

    /// Generate a unique product ID
    pub fn unique_product_id(&self, base: &str) -> String {
        self.dgraph.unique_product_id(base)
    }

    /// Clean up and shutdown
    pub async fn cleanup(mut self) -> Result<(), String> {
        self.dgraph.cleanup().await
            .map_err(|e| format!("Cleanup failed: {}", e))?;
        self.server.shutdown().await;
        Ok(())
    }

    // =========================================================================
    // HTTP Delegate Methods - forward to server
    // =========================================================================

    /// GET request expecting JSON response
    pub async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        self.server.get(path).await.map_err(|e| e.to_string())
    }

    /// POST request expecting JSON response
    pub async fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<T, String> {
        self.server.post(path, body).await.map_err(|e| e.to_string())
    }

    /// POST request with empty body
    pub async fn post_empty<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, String> {
        self.server.post(path, &serde_json::json!({})).await.map_err(|e| e.to_string())
    }

    /// PUT request expecting JSON response
    pub async fn put<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<T, String> {
        self.server.put(path, body).await.map_err(|e| e.to_string())
    }

    /// DELETE request expecting JSON response
    pub async fn delete<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        self.server.delete(path).await.map_err(|e| e.to_string())
    }
}
