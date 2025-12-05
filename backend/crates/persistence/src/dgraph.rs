//! DGraph persistence layer for Product-FARM
//!
//! This module provides graph database storage using DGraph for:
//! - Products, Attributes, Rules as nodes
//! - Dependencies and computation relationships as edges
//! - Graph traversal queries for dependency analysis and impact analysis

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dgraph_tonic::{Client, Mutate, Mutation, Operation, Query};
use product_farm_core::{
    AbstractAttribute, AbstractPath, Product, ProductId, ProductStatus, Rule, RuleId,
    TemplateType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument};

use crate::error::{PersistenceError, PersistenceResult};
use crate::{AttributeRepository, CompiledRuleRepository, ProductRepository, RuleRepository};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use product_farm_json_logic::{CompilationTier, CompiledBytecode, Expr, PersistedRule};

/// DGraph client configuration
#[derive(Debug, Clone)]
pub struct DgraphConfig {
    /// DGraph Alpha gRPC endpoint (default: localhost:9080)
    pub endpoint: String,
}

impl Default for DgraphConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:9080".to_string(),
        }
    }
}

/// DGraph client wrapper
pub struct DgraphClient {
    client: Client,
    config: DgraphConfig,
}

impl std::fmt::Debug for DgraphClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DgraphClient")
            .field("config", &self.config)
            .finish()
    }
}

impl DgraphClient {
    /// Create a new DGraph client
    pub fn new(config: DgraphConfig) -> PersistenceResult<Self> {
        let client = Client::new(&config.endpoint)
            .map_err(|e| PersistenceError::ConnectionError(e.to_string()))?;

        info!(endpoint = %config.endpoint, "Connected to DGraph");

        Ok(Self { client, config })
    }

    /// Create with default configuration
    pub fn default_client() -> PersistenceResult<Self> {
        Self::new(DgraphConfig::default())
    }

    /// Apply the schema to DGraph
    #[instrument(skip(self, schema))]
    pub async fn apply_schema(&self, schema: &str) -> PersistenceResult<()> {
        let op = Operation {
            schema: schema.to_string(),
            ..Default::default()
        };

        self.client
            .alter(op)
            .await
            .map_err(|e| PersistenceError::SchemaError(e.to_string()))?;

        info!("Schema applied successfully");
        Ok(())
    }

    /// Execute a raw DQL query
    pub async fn query<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        vars: HashMap<String, String>,
    ) -> PersistenceResult<T> {
        let mut txn = self.client.new_read_only_txn();
        let response = txn
            .query_with_vars(query, vars)
            .await
            .map_err(|e| PersistenceError::QueryError(e.to_string()))?;

        serde_json::from_slice(&response.json)
            .map_err(|e| PersistenceError::DeserializationError(e.to_string()))
    }

    /// Execute a query without variables
    pub async fn query_simple<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
    ) -> PersistenceResult<T> {
        let mut txn = self.client.new_read_only_txn();
        let response = txn
            .query(query)
            .await
            .map_err(|e| PersistenceError::QueryError(e.to_string()))?;

        serde_json::from_slice(&response.json)
            .map_err(|e| PersistenceError::DeserializationError(e.to_string()))
    }

    /// Execute a mutation with NQuads format
    ///
    /// NQuads are more reliable than JSON mutations via gRPC in dgraph-tonic.
    /// Returns a map of blank node labels to UIDs.
    pub async fn mutate_nquads(&self, nquads: &str) -> PersistenceResult<HashMap<String, String>> {
        let mut txn = self.client.new_mutated_txn();

        let mut mu = Mutation::new();
        mu.set_set_nquads(nquads);

        let response = txn
            .mutate(mu)
            .await
            .map_err(|e| PersistenceError::MutationError(e.to_string()))?;

        txn.commit()
            .await
            .map_err(|e| PersistenceError::MutationError(e.to_string()))?;

        Ok(response.uids)
    }

    /// Execute an upsert mutation (query + conditional mutation)
    ///
    /// This is useful for updating existing nodes or inserting new ones.
    /// The query finds existing nodes, and the nquads reference them via uid(varname).
    pub async fn upsert(&self, query: &str, nquads: &str) -> PersistenceResult<HashMap<String, String>> {
        let mut txn = self.client.new_mutated_txn();

        let mut mu = Mutation::new();
        mu.set_set_nquads(nquads);

        let response = txn
            .upsert(query, mu)
            .await
            .map_err(|e| PersistenceError::MutationError(e.to_string()))?;

        txn.commit()
            .await
            .map_err(|e| PersistenceError::MutationError(e.to_string()))?;

        Ok(response.uids)
    }

    /// Delete nodes by UID
    pub async fn delete_node(&self, uid: &str) -> PersistenceResult<()> {
        let mut txn = self.client.new_mutated_txn();

        let mut mu = Mutation::new();
        let del_nquads = format!("<{}> * * .", uid);
        mu.set_delete_nquads(del_nquads);

        txn.mutate(mu)
            .await
            .map_err(|e| PersistenceError::MutationError(e.to_string()))?;

        txn.commit()
            .await
            .map_err(|e| PersistenceError::MutationError(e.to_string()))?;

        Ok(())
    }
}

// ============================================================================
// DGraph Data Transfer Objects (DTOs)
// ============================================================================

/// Product DTO for DGraph serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    #[serde(rename = "dgraph.type", default)]
    pub dgraph_type: Vec<String>,
    #[serde(rename = "Product.id")]
    pub id: String,
    #[serde(rename = "Product.status")]
    pub status: String,
    #[serde(rename = "Product.template_type")]
    pub template_type: String,
    #[serde(rename = "Product.effective_from")]
    pub effective_from: String,
    #[serde(rename = "Product.expiry_at", skip_serializing_if = "Option::is_none")]
    pub expiry_at: Option<String>,
    #[serde(rename = "Product.description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "Product.created_at")]
    pub created_at: String,
    #[serde(rename = "Product.updated_at")]
    pub updated_at: String,
    #[serde(rename = "Product.version")]
    pub version: i64,
    #[serde(rename = "Product.parent_product", skip_serializing_if = "Option::is_none")]
    pub parent_product_uid: Option<UidRef>,
}

/// Rule DTO for DGraph serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    #[serde(rename = "dgraph.type", default)]
    pub dgraph_type: Vec<String>,
    #[serde(rename = "Rule.id")]
    pub id: String,
    #[serde(rename = "Rule.rule_type")]
    pub rule_type: String,
    #[serde(rename = "Rule.display_expression", skip_serializing_if = "Option::is_none")]
    pub display_expression: Option<String>,
    #[serde(rename = "Rule.expression")]
    pub expression: String,
    #[serde(rename = "Rule.description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "Rule.enabled")]
    pub enabled: bool,
    #[serde(rename = "Rule.order_index")]
    pub order_index: i32,
    #[serde(rename = "Rule.bytecode", skip_serializing_if = "Option::is_none")]
    pub bytecode: Option<String>,
    #[serde(rename = "Rule.compilation_tier", skip_serializing_if = "Option::is_none")]
    pub compilation_tier: Option<String>,
    #[serde(rename = "Rule.eval_count", skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<i64>,
    #[serde(rename = "Rule.product", skip_serializing_if = "Option::is_none")]
    pub product_uid: Option<UidRef>,
    #[serde(rename = "Rule.depends_on", skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<UidRef>>,
    #[serde(rename = "Rule.computes", skip_serializing_if = "Option::is_none")]
    pub computes: Option<Vec<UidRef>>,
}

/// AbstractAttribute DTO for DGraph serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractAttributeDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    #[serde(rename = "dgraph.type", default)]
    pub dgraph_type: Vec<String>,
    #[serde(rename = "AbstractAttribute.abstract_path")]
    pub abstract_path: String,
    #[serde(rename = "AbstractAttribute.component_type")]
    pub component_type: String,
    #[serde(rename = "AbstractAttribute.component_id", skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,
    #[serde(rename = "AbstractAttribute.datatype_id")]
    pub datatype_id: String,
    #[serde(rename = "AbstractAttribute.enum_name", skip_serializing_if = "Option::is_none")]
    pub enum_name: Option<String>,
    #[serde(rename = "AbstractAttribute.tags", skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(rename = "AbstractAttribute.display_names", skip_serializing_if = "Option::is_none")]
    pub display_names: Option<Vec<String>>,
    #[serde(rename = "AbstractAttribute.constraint_expression", skip_serializing_if = "Option::is_none")]
    pub constraint_expression: Option<String>,
    #[serde(rename = "AbstractAttribute.immutable")]
    pub immutable: bool,
    #[serde(rename = "AbstractAttribute.description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "AbstractAttribute.product", skip_serializing_if = "Option::is_none")]
    pub product_uid: Option<UidRef>,
}

/// UID reference for edges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UidRef {
    pub uid: String,
}

impl UidRef {
    pub fn new(uid: &str) -> Self {
        Self {
            uid: uid.to_string(),
        }
    }
}

/// Product query result
#[derive(Debug, Deserialize)]
pub struct ProductQueryResult {
    #[serde(default)]
    pub products: Vec<ProductDto>,
}

/// Rule query result
#[derive(Debug, Deserialize)]
pub struct RuleQueryResult {
    #[serde(default)]
    pub rules: Vec<RuleDto>,
}

/// Attribute query result
#[derive(Debug, Deserialize)]
pub struct AttributeQueryResult {
    #[serde(default)]
    pub attributes: Vec<AbstractAttributeDto>,
}

// ============================================================================
// UID-only query results (for find/check existence queries)
// ============================================================================

/// Simple UID holder for queries that only need UID
#[derive(Debug, Deserialize)]
pub struct UidOnly {
    pub uid: Option<String>,
}

/// Product UID query result
#[derive(Debug, Deserialize)]
pub struct ProductUidResult {
    #[serde(default)]
    pub products: Vec<UidOnly>,
}

/// Rule UID query result
#[derive(Debug, Deserialize)]
pub struct RuleUidResult {
    #[serde(default)]
    pub rules: Vec<UidOnly>,
}

/// Attribute UID query result
#[derive(Debug, Deserialize)]
pub struct AttributeUidResult {
    #[serde(default)]
    pub attributes: Vec<UidOnly>,
}

/// CompiledRule UID query result
#[derive(Debug, Deserialize)]
pub struct CompiledRuleUidResult {
    #[serde(default)]
    pub compiled_rules: Vec<UidOnly>,
}

// ============================================================================
// NQuads Helpers
// ============================================================================

/// Escape a string for use in NQuads format
fn escape_nquad_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Generate NQuads for a Product
fn product_to_nquads(product: &Product, uid: Option<&str>) -> String {
    let subject = uid.map(|u| format!("<{}>", u)).unwrap_or_else(|| "_:product".to_string());

    let status = match product.status {
        ProductStatus::Draft => "DRAFT",
        ProductStatus::PendingApproval => "PENDING_APPROVAL",
        ProductStatus::Active => "ACTIVE",
        ProductStatus::Discontinued => "DISCONTINUED",
    };

    let mut nquads = vec![
        format!("{} <dgraph.type> \"Product\" .", subject),
        format!("{} <Product.id> \"{}\" .", subject, escape_nquad_string(product.id.as_str())),
        format!("{} <Product.status> \"{}\" .", subject, status),
        format!("{} <Product.template_type> \"{}\" .", subject, escape_nquad_string(product.template_type.as_str())),
        format!("{} <Product.effective_from> \"{}\"^^<xs:dateTime> .", subject, product.effective_from.to_rfc3339()),
        format!("{} <Product.created_at> \"{}\"^^<xs:dateTime> .", subject, product.created_at.to_rfc3339()),
        format!("{} <Product.updated_at> \"{}\"^^<xs:dateTime> .", subject, product.updated_at.to_rfc3339()),
        format!("{} <Product.version> \"{}\"^^<xs:int> .", subject, product.version),
    ];

    if let Some(expiry) = product.expiry_at {
        nquads.push(format!("{} <Product.expiry_at> \"{}\"^^<xs:dateTime> .", subject, expiry.to_rfc3339()));
    }

    if let Some(ref desc) = product.description {
        nquads.push(format!("{} <Product.description> \"{}\" .", subject, escape_nquad_string(desc)));
    }

    nquads.join("\n")
}

/// Generate NQuads for a Rule
fn rule_to_nquads(rule: &Rule, uid: Option<&str>, product_uid: &str) -> String {
    let subject = uid.map(|u| format!("<{}>", u)).unwrap_or_else(|| "_:rule".to_string());

    let mut nquads = vec![
        format!("{} <dgraph.type> \"Rule\" .", subject),
        format!("{} <Rule.id> \"{}\" .", subject, escape_nquad_string(&rule.id.to_string())),
        format!("{} <Rule.rule_type> \"{}\" .", subject, escape_nquad_string(&rule.rule_type)),
        format!("{} <Rule.expression> \"{}\" .", subject, escape_nquad_string(&serde_json::to_string(&rule.expression).unwrap_or_default())),
        format!("{} <Rule.enabled> \"{}\"^^<xs:boolean> .", subject, rule.enabled),
        format!("{} <Rule.order_index> \"{}\"^^<xs:int> .", subject, rule.order_index),
        format!("{} <Rule.product> <{}> .", subject, product_uid),
    ];

    if !rule.display_expression.is_empty() {
        nquads.push(format!("{} <Rule.display_expression> \"{}\" .", subject, escape_nquad_string(&rule.display_expression)));
    }

    if let Some(ref desc) = rule.description {
        nquads.push(format!("{} <Rule.description> \"{}\" .", subject, escape_nquad_string(desc)));
    }

    nquads.join("\n")
}

/// Generate NQuads for an AbstractAttribute
fn attribute_to_nquads(attr: &AbstractAttribute, uid: Option<&str>, product_uid: &str) -> String {
    let subject = uid.map(|u| format!("<{}>", u)).unwrap_or_else(|| "_:attr".to_string());

    let mut nquads = vec![
        format!("{} <dgraph.type> \"AbstractAttribute\" .", subject),
        format!("{} <AbstractAttribute.abstract_path> \"{}\" .", subject, escape_nquad_string(attr.abstract_path.as_str())),
        format!("{} <AbstractAttribute.component_type> \"{}\" .", subject, escape_nquad_string(&attr.component_type)),
        format!("{} <AbstractAttribute.datatype_id> \"{}\" .", subject, escape_nquad_string(attr.datatype_id.as_str())),
        format!("{} <AbstractAttribute.immutable> \"{}\"^^<xs:boolean> .", subject, attr.immutable),
        format!("{} <AbstractAttribute.product> <{}> .", subject, product_uid),
    ];

    if let Some(ref comp_id) = attr.component_id {
        nquads.push(format!("{} <AbstractAttribute.component_id> \"{}\" .", subject, escape_nquad_string(comp_id)));
    }

    if let Some(ref enum_name) = attr.enum_name {
        nquads.push(format!("{} <AbstractAttribute.enum_name> \"{}\" .", subject, escape_nquad_string(enum_name)));
    }

    if let Some(ref desc) = attr.description {
        nquads.push(format!("{} <AbstractAttribute.description> \"{}\" .", subject, escape_nquad_string(desc)));
    }

    for tag in &attr.tags {
        nquads.push(format!("{} <AbstractAttribute.tags> \"{}\" .", subject, escape_nquad_string(tag.as_str())));
    }

    for name in &attr.display_names {
        nquads.push(format!("{} <AbstractAttribute.display_names> \"{}\" .", subject, escape_nquad_string(name.as_str())));
    }

    nquads.join("\n")
}

/// Generate NQuads for a CompiledRule (PersistedRule)
fn compiled_rule_to_nquads(rule_id: &str, rule: &PersistedRule, uid: Option<&str>) -> String {
    let subject = uid.map(|u| format!("<{}>", u)).unwrap_or_else(|| "_:compiled".to_string());

    let tier = match rule.tier {
        CompilationTier::Ast => "AST",
        CompilationTier::Bytecode => "BYTECODE",
        CompilationTier::Jit => "JIT",
    };

    let ast_json = serde_json::to_string(&rule.ast).unwrap_or_default();

    let mut nquads = vec![
        format!("{} <dgraph.type> \"CompiledRule\" .", subject),
        format!("{} <CompiledRule.rule_id> \"{}\" .", subject, escape_nquad_string(rule_id)),
        format!("{} <CompiledRule.tier> \"{}\" .", subject, tier),
        format!("{} <CompiledRule.eval_count> \"{}\"^^<xs:int> .", subject, rule.eval_count),
        format!("{} <CompiledRule.ast_json> \"{}\" .", subject, escape_nquad_string(&ast_json)),
    ];

    if let Some(ref bc) = rule.bytecode {
        let json = serde_json::to_string(bc).unwrap_or_default();
        let encoded = BASE64.encode(json.as_bytes());
        nquads.push(format!("{} <CompiledRule.bytecode> \"{}\" .", subject, encoded));
    }

    nquads.join("\n")
}

// ============================================================================
// Conversions
// ============================================================================

impl From<&Product> for ProductDto {
    fn from(p: &Product) -> Self {
        let status = match p.status {
            ProductStatus::Draft => "DRAFT",
            ProductStatus::PendingApproval => "PENDING_APPROVAL",
            ProductStatus::Active => "ACTIVE",
            ProductStatus::Discontinued => "DISCONTINUED",
        };

        Self {
            uid: None,
            dgraph_type: vec!["Product".to_string()],
            id: p.id.as_str().to_string(),
            status: status.to_string(),
            template_type: p.template_type.as_str().to_string(),
            effective_from: p.effective_from.to_rfc3339(),
            expiry_at: p.expiry_at.map(|d| d.to_rfc3339()),
            description: p.description.clone(),
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
            version: p.version as i64,
            parent_product_uid: None,
        }
    }
}

impl TryFrom<ProductDto> for Product {
    type Error = PersistenceError;

    fn try_from(dto: ProductDto) -> Result<Self, Self::Error> {
        let status = match dto.status.as_str() {
            "DRAFT" => ProductStatus::Draft,
            "PENDING_APPROVAL" => ProductStatus::PendingApproval,
            "ACTIVE" => ProductStatus::Active,
            "DISCONTINUED" => ProductStatus::Discontinued,
            s => {
                return Err(PersistenceError::DeserializationError(format!(
                    "Unknown status: {}",
                    s
                )))
            }
        };

        let effective_from = DateTime::parse_from_rfc3339(&dto.effective_from)
            .map_err(|e| PersistenceError::DeserializationError(e.to_string()))?
            .with_timezone(&Utc);

        let expiry_at = dto
            .expiry_at
            .map(|s| DateTime::parse_from_rfc3339(&s))
            .transpose()
            .map_err(|e| PersistenceError::DeserializationError(e.to_string()))?
            .map(|d| d.with_timezone(&Utc));

        let created_at = DateTime::parse_from_rfc3339(&dto.created_at)
            .map_err(|e| PersistenceError::DeserializationError(e.to_string()))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&dto.updated_at)
            .map_err(|e| PersistenceError::DeserializationError(e.to_string()))?
            .with_timezone(&Utc);

        Ok(Product {
            id: ProductId::new(dto.id),
            status,
            template_type: TemplateType::new(dto.template_type),
            parent_product_id: None,
            effective_from,
            expiry_at,
            description: dto.description,
            created_at,
            updated_at,
            version: dto.version as u64,
        })
    }
}

impl From<&Rule> for RuleDto {
    fn from(r: &Rule) -> Self {
        Self {
            uid: None,
            dgraph_type: vec!["Rule".to_string()],
            id: r.id.to_string(),
            rule_type: r.rule_type.clone(),
            display_expression: if r.display_expression.is_empty() {
                None
            } else {
                Some(r.display_expression.clone())
            },
            expression: serde_json::to_string(&r.expression).unwrap_or_default(),
            description: r.description.clone(),
            enabled: r.enabled,
            order_index: r.order_index,
            bytecode: None,
            compilation_tier: None,
            eval_count: None,
            product_uid: Some(UidRef::new("_:product")),
            depends_on: None,
            computes: None,
        }
    }
}

// ============================================================================
// DGraph Product Repository
// ============================================================================

/// DGraph-backed Product Repository
pub struct DgraphProductRepository {
    client: Arc<DgraphClient>,
}

impl DgraphProductRepository {
    pub fn new(client: Arc<DgraphClient>) -> Self {
        Self { client }
    }
}

impl std::fmt::Debug for DgraphProductRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DgraphProductRepository").finish()
    }
}

#[async_trait]
impl ProductRepository for DgraphProductRepository {
    #[instrument(skip(self))]
    async fn get(&self, id: &ProductId) -> PersistenceResult<Option<Product>> {
        let query = r#"
            query GetProduct($id: string) {
                products(func: eq(Product.id, $id)) {
                    uid
                    dgraph.type
                    Product.id
                    Product.status
                    Product.template_type
                    Product.effective_from
                    Product.expiry_at
                    Product.description
                    Product.created_at
                    Product.updated_at
                    Product.version
                    Product.parent_product {
                        Product.id
                    }
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$id".to_string(), id.as_str().to_string());

        let result: ProductQueryResult = self.client.query(query, vars).await?;

        if let Some(dto) = result.products.into_iter().next() {
            let product = Product::try_from(dto)?;
            Ok(Some(product))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self, product))]
    async fn save(&self, product: &Product) -> PersistenceResult<()> {
        // First, check if product exists and get its UID
        let find_query = r#"
            query FindProduct($id: string) {
                products(func: eq(Product.id, $id)) {
                    uid
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$id".to_string(), product.id.as_str().to_string());

        let result: ProductUidResult = self.client.query(find_query, vars).await?;
        let existing_uid = result.products.into_iter().next().and_then(|p| p.uid);

        // Use NQuads for mutation (JSON mutations have compatibility issues with dgraph-tonic)
        let nquads = product_to_nquads(product, existing_uid.as_deref());
        self.client.mutate_nquads(&nquads).await?;
        debug!(product_id = %product.id.as_str(), "Product saved");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &ProductId) -> PersistenceResult<()> {
        let find_query = r#"
            query FindProduct($id: string) {
                products(func: eq(Product.id, $id)) {
                    uid
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$id".to_string(), id.as_str().to_string());

        let result: ProductUidResult = self.client.query(find_query, vars).await?;

        if let Some(uid_only) = result.products.into_iter().next() {
            if let Some(uid) = uid_only.uid {
                self.client.delete_node(&uid).await?;
                debug!(product_id = %id.as_str(), "Product deleted");
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn list(&self, limit: usize, offset: usize) -> PersistenceResult<Vec<Product>> {
        let query = format!(
            r#"{{
                products(func: type(Product), first: {}, offset: {}, orderasc: Product.id) {{
                    uid
                    dgraph.type
                    Product.id
                    Product.status
                    Product.template_type
                    Product.effective_from
                    Product.expiry_at
                    Product.description
                    Product.created_at
                    Product.updated_at
                    Product.version
                }}
            }}"#,
            limit, offset
        );

        let result: ProductQueryResult = self.client.query_simple(&query).await?;

        result
            .products
            .into_iter()
            .map(Product::try_from)
            .collect()
    }

    #[instrument(skip(self))]
    async fn count(&self) -> PersistenceResult<usize> {
        let query = r#"{
            products(func: type(Product)) {
                count(uid)
            }
        }"#;

        #[derive(Deserialize)]
        struct CountResult {
            #[serde(default)]
            products: Vec<CountValue>,
        }
        #[derive(Deserialize)]
        struct CountValue {
            count: usize,
        }

        let result: CountResult = self.client.query_simple(query).await?;
        Ok(result.products.first().map(|c| c.count).unwrap_or(0))
    }
}

// ============================================================================
// DGraph Rule Repository
// ============================================================================

/// DGraph-backed Rule Repository
pub struct DgraphRuleRepository {
    client: Arc<DgraphClient>,
}

impl DgraphRuleRepository {
    pub fn new(client: Arc<DgraphClient>) -> Self {
        Self { client }
    }
}

impl std::fmt::Debug for DgraphRuleRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DgraphRuleRepository").finish()
    }
}

#[async_trait]
impl RuleRepository for DgraphRuleRepository {
    #[instrument(skip(self))]
    async fn get(&self, id: &RuleId) -> PersistenceResult<Option<Rule>> {
        let query = r#"
            query GetRule($id: string) {
                rules(func: eq(Rule.id, $id)) {
                    uid
                    dgraph.type
                    Rule.id
                    Rule.rule_type
                    Rule.display_expression
                    Rule.expression
                    Rule.description
                    Rule.enabled
                    Rule.order_index
                    Rule.product {
                        uid
                        Product.id
                    }
                    Rule.depends_on {
                        uid
                        AbstractAttribute.abstract_path
                    }
                    Rule.computes {
                        uid
                        AbstractAttribute.abstract_path
                    }
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$id".to_string(), id.to_string());

        let result: RuleQueryResult = self.client.query(query, vars).await?;

        if let Some(dto) = result.rules.into_iter().next() {
            let rule = rule_from_dto(dto)?;
            Ok(Some(rule))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self, rule))]
    async fn save(&self, rule: &Rule) -> PersistenceResult<()> {
        // First, find the product UID
        let find_product_query = r#"
            query FindProduct($id: string) {
                products(func: eq(Product.id, $id)) {
                    uid
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$id".to_string(), rule.product_id.as_str().to_string());

        let result: ProductUidResult = self.client.query(find_product_query, vars).await?;

        let product_uid = result
            .products
            .into_iter()
            .next()
            .and_then(|p| p.uid)
            .ok_or_else(|| {
                PersistenceError::NotFound(format!(
                    "Product not found: {}",
                    rule.product_id.as_str()
                ))
            })?;

        // Check if rule exists
        let find_rule_query = r#"
            query FindRule($id: string) {
                rules(func: eq(Rule.id, $id)) {
                    uid
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$id".to_string(), rule.id.to_string());

        let result: RuleUidResult = self.client.query(find_rule_query, vars).await?;
        let existing_uid = result.rules.into_iter().next().and_then(|r| r.uid);

        // Use NQuads for mutation (JSON mutations have compatibility issues with dgraph-tonic)
        let nquads = rule_to_nquads(rule, existing_uid.as_deref(), &product_uid);
        self.client.mutate_nquads(&nquads).await?;
        debug!(rule_id = %rule.id.to_string(), "Rule saved");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &RuleId) -> PersistenceResult<()> {
        let find_query = r#"
            query FindRule($id: string) {
                rules(func: eq(Rule.id, $id)) {
                    uid
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$id".to_string(), id.to_string());

        let result: RuleUidResult = self.client.query(find_query, vars).await?;

        if let Some(uid_only) = result.rules.into_iter().next() {
            if let Some(uid) = uid_only.uid {
                self.client.delete_node(&uid).await?;
                debug!(rule_id = %id.to_string(), "Rule deleted");
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn find_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<Rule>> {
        let query = r#"
            query FindRulesByProduct($product_id: string) {
                var(func: eq(Product.id, $product_id)) {
                    rules as ~Rule.product
                }
                rules(func: uid(rules), orderasc: Rule.order_index) {
                    uid
                    dgraph.type
                    Rule.id
                    Rule.rule_type
                    Rule.display_expression
                    Rule.expression
                    Rule.description
                    Rule.enabled
                    Rule.order_index
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$product_id".to_string(), product_id.as_str().to_string());

        let result: RuleQueryResult = self.client.query(query, vars).await?;

        result
            .rules
            .into_iter()
            .map(rule_from_dto)
            .collect()
    }

    #[instrument(skip(self))]
    async fn find_enabled_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<Rule>> {
        let all = self.find_by_product(product_id).await?;
        Ok(all.into_iter().filter(|r| r.enabled).collect())
    }
}

/// Convert RuleDto to Rule
fn rule_from_dto(dto: RuleDto) -> PersistenceResult<Rule> {
    let expression: serde_json::Value = serde_json::from_str(&dto.expression)
        .map_err(|e| PersistenceError::DeserializationError(e.to_string()))?;

    let product_id = ProductId::new("unknown");

    let mut rule = Rule::from_json_logic(product_id, dto.rule_type, expression)
        .with_id(RuleId::from_string(&dto.id))
        .with_order(dto.order_index);

    if let Some(desc) = dto.description {
        rule = rule.with_description(desc);
    }
    if let Some(display) = dto.display_expression {
        rule = rule.with_display(display);
    }
    if !dto.enabled {
        rule = rule.disabled();
    }

    Ok(rule)
}

// ============================================================================
// DGraph Attribute Repository
// ============================================================================

/// DGraph-backed Attribute Repository
pub struct DgraphAttributeRepository {
    client: Arc<DgraphClient>,
}

impl DgraphAttributeRepository {
    pub fn new(client: Arc<DgraphClient>) -> Self {
        Self { client }
    }
}

impl std::fmt::Debug for DgraphAttributeRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DgraphAttributeRepository").finish()
    }
}

#[async_trait]
impl AttributeRepository for DgraphAttributeRepository {
    #[instrument(skip(self))]
    async fn get(&self, path: &AbstractPath) -> PersistenceResult<Option<AbstractAttribute>> {
        let query = r#"
            query GetAttribute($path: string) {
                attributes(func: eq(AbstractAttribute.abstract_path, $path)) {
                    uid
                    dgraph.type
                    AbstractAttribute.abstract_path
                    AbstractAttribute.component_type
                    AbstractAttribute.component_id
                    AbstractAttribute.datatype_id
                    AbstractAttribute.enum_name
                    AbstractAttribute.tags
                    AbstractAttribute.display_names
                    AbstractAttribute.constraint_expression
                    AbstractAttribute.immutable
                    AbstractAttribute.description
                    AbstractAttribute.product {
                        uid
                        Product.id
                    }
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$path".to_string(), path.as_str().to_string());

        let result: AttributeQueryResult = self.client.query(query, vars).await?;

        if let Some(dto) = result.attributes.into_iter().next() {
            let attr = abstract_attribute_from_dto(dto)?;
            Ok(Some(attr))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self, attribute))]
    async fn save(&self, attribute: &AbstractAttribute) -> PersistenceResult<()> {
        // Find the product UID
        let find_product_query = r#"
            query FindProduct($id: string) {
                products(func: eq(Product.id, $id)) {
                    uid
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$id".to_string(), attribute.product_id.as_str().to_string());

        let result: ProductUidResult = self.client.query(find_product_query, vars).await?;

        let product_uid = result
            .products
            .into_iter()
            .next()
            .and_then(|p| p.uid)
            .ok_or_else(|| {
                PersistenceError::NotFound(format!(
                    "Product not found: {}",
                    attribute.product_id.as_str()
                ))
            })?;

        // Check if attribute exists
        let find_attr_query = r#"
            query FindAttribute($path: string) {
                attributes(func: eq(AbstractAttribute.abstract_path, $path)) {
                    uid
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$path".to_string(), attribute.abstract_path.as_str().to_string());

        let result: AttributeUidResult = self.client.query(find_attr_query, vars).await?;
        let existing_uid = result.attributes.into_iter().next().and_then(|a| a.uid);

        // Use NQuads for mutation (JSON mutations have compatibility issues with dgraph-tonic)
        let nquads = attribute_to_nquads(attribute, existing_uid.as_deref(), &product_uid);
        self.client.mutate_nquads(&nquads).await?;
        debug!(path = %attribute.abstract_path.as_str(), "Attribute saved");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, path: &AbstractPath) -> PersistenceResult<()> {
        let find_query = r#"
            query FindAttribute($path: string) {
                attributes(func: eq(AbstractAttribute.abstract_path, $path)) {
                    uid
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$path".to_string(), path.as_str().to_string());

        let result: AttributeUidResult = self.client.query(find_query, vars).await?;

        if let Some(entry) = result.attributes.into_iter().next() {
            if let Some(uid) = entry.uid {
                self.client.delete_node(&uid).await?;
                debug!(path = %path.as_str(), "Attribute deleted");
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn find_by_product(&self, product_id: &ProductId) -> PersistenceResult<Vec<AbstractAttribute>> {
        let query = r#"
            query FindAttributesByProduct($product_id: string) {
                var(func: eq(Product.id, $product_id)) {
                    attrs as ~AbstractAttribute.product
                }
                attributes(func: uid(attrs)) {
                    uid
                    dgraph.type
                    AbstractAttribute.abstract_path
                    AbstractAttribute.component_type
                    AbstractAttribute.component_id
                    AbstractAttribute.datatype_id
                    AbstractAttribute.enum_name
                    AbstractAttribute.tags
                    AbstractAttribute.display_names
                    AbstractAttribute.immutable
                    AbstractAttribute.description
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$product_id".to_string(), product_id.as_str().to_string());

        let result: AttributeQueryResult = self.client.query(query, vars).await?;

        result
            .attributes
            .into_iter()
            .map(abstract_attribute_from_dto)
            .collect()
    }

    #[instrument(skip(self))]
    async fn find_by_tag(&self, product_id: &ProductId, tag: &str) -> PersistenceResult<Vec<AbstractAttribute>> {
        let query = r#"
            query FindAttributesByTag($product_id: string, $tag: string) {
                var(func: eq(Product.id, $product_id)) {
                    attrs as ~AbstractAttribute.product @filter(
                        anyofterms(AbstractAttribute.tags, $tag)
                    )
                }
                attributes(func: uid(attrs)) {
                    uid
                    dgraph.type
                    AbstractAttribute.abstract_path
                    AbstractAttribute.component_type
                    AbstractAttribute.component_id
                    AbstractAttribute.datatype_id
                    AbstractAttribute.enum_name
                    AbstractAttribute.tags
                    AbstractAttribute.display_names
                    AbstractAttribute.immutable
                    AbstractAttribute.description
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$product_id".to_string(), product_id.as_str().to_string());
        vars.insert("$tag".to_string(), tag.to_string());

        let result: AttributeQueryResult = self.client.query(query, vars).await?;

        result
            .attributes
            .into_iter()
            .map(abstract_attribute_from_dto)
            .collect()
    }
}

/// Convert AbstractAttributeDto to AbstractAttribute
fn abstract_attribute_from_dto(dto: AbstractAttributeDto) -> PersistenceResult<AbstractAttribute> {
    use product_farm_core::{DataTypeId, DisplayName, Tag};

    let product_id = ProductId::new("unknown");

    let mut attr = AbstractAttribute::new(
        dto.abstract_path,
        product_id,
        dto.component_type,
        DataTypeId::new(dto.datatype_id),
    );

    if let Some(comp_id) = dto.component_id {
        attr = attr.with_component_id(comp_id);
    }
    if let Some(enum_name) = dto.enum_name {
        attr = attr.with_enum(enum_name);
    }
    if let Some(tags) = dto.tags {
        for tag in tags {
            attr = attr.with_tag(Tag::new(tag));
        }
    }
    if let Some(display_names) = dto.display_names {
        for name in display_names {
            attr = attr.with_display_name(DisplayName::new(name));
        }
    }
    if let Some(desc) = dto.description {
        attr = attr.with_description(desc);
    }
    if dto.immutable {
        attr = attr.immutable();
    }

    Ok(attr)
}

// ============================================================================
// Graph Traversal Queries
// ============================================================================

/// Graph traversal operations specific to DGraph
pub struct DgraphGraphQueries {
    client: Arc<DgraphClient>,
}

impl DgraphGraphQueries {
    pub fn new(client: Arc<DgraphClient>) -> Self {
        Self { client }
    }

    /// Find all rules that depend on a given attribute (upstream dependencies)
    #[instrument(skip(self))]
    pub async fn find_rules_depending_on(
        &self,
        attribute_path: &str,
    ) -> PersistenceResult<Vec<RuleId>> {
        let query = r#"
            query FindDependentRules($path: string) {
                var(func: eq(AbstractAttribute.abstract_path, $path)) {
                    rules as ~Rule.depends_on
                }
                rules(func: uid(rules)) {
                    Rule.id
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$path".to_string(), attribute_path.to_string());

        #[derive(Deserialize)]
        struct Result {
            #[serde(default)]
            rules: Vec<RuleIdOnly>,
        }
        #[derive(Deserialize)]
        struct RuleIdOnly {
            #[serde(rename = "Rule.id")]
            id: String,
        }

        let result: Result = self.client.query(query, vars).await?;

        Ok(result
            .rules
            .into_iter()
            .map(|r| RuleId::from_string(&r.id))
            .collect())
    }

    /// Find all attributes computed by a given rule (downstream impact)
    #[instrument(skip(self))]
    pub async fn find_computed_attributes(&self, rule_id: &str) -> PersistenceResult<Vec<String>> {
        let query = r#"
            query FindComputedAttributes($rule_id: string) {
                rules(func: eq(Rule.id, $rule_id)) {
                    Rule.computes {
                        AbstractAttribute.abstract_path
                    }
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$rule_id".to_string(), rule_id.to_string());

        #[derive(Deserialize)]
        struct Result {
            #[serde(default)]
            rules: Vec<RuleWithComputes>,
        }
        #[derive(Deserialize)]
        struct RuleWithComputes {
            #[serde(rename = "Rule.computes", default)]
            computes: Vec<AttrPathOnly>,
        }
        #[derive(Deserialize)]
        struct AttrPathOnly {
            #[serde(rename = "AbstractAttribute.abstract_path")]
            path: String,
        }

        let result: Result = self.client.query(query, vars).await?;

        Ok(result
            .rules
            .into_iter()
            .flat_map(|r| r.computes)
            .map(|a| a.path)
            .collect())
    }

    /// Find transitive dependencies of a rule (all upstream rules, recursively)
    #[instrument(skip(self))]
    pub async fn find_transitive_dependencies(
        &self,
        rule_id: &str,
        max_depth: usize,
    ) -> PersistenceResult<Vec<RuleId>> {
        let query = format!(
            r#"{{
                var(func: eq(Rule.id, "{}")) {{
                    upstream as Rule.depends_on @recurse(depth: {}) {{
                        ~Rule.computes
                    }}
                }}
                rules(func: uid(upstream)) {{
                    Rule.id
                }}
            }}"#,
            rule_id, max_depth
        );

        #[derive(Deserialize)]
        struct Result {
            #[serde(default)]
            rules: Vec<RuleIdOnly>,
        }
        #[derive(Deserialize)]
        struct RuleIdOnly {
            #[serde(rename = "Rule.id")]
            id: String,
        }

        let result: Result = self.client.query_simple(&query).await?;

        Ok(result
            .rules
            .into_iter()
            .map(|r| RuleId::from_string(&r.id))
            .collect())
    }

    /// Find impact of changing a rule (all downstream rules and attributes affected)
    #[instrument(skip(self))]
    pub async fn find_impact_of_rule_change(
        &self,
        rule_id: &str,
        max_depth: usize,
    ) -> PersistenceResult<ImpactAnalysis> {
        let query = format!(
            r#"{{
                var(func: eq(Rule.id, "{}")) {{
                    downstream_attrs as Rule.computes @recurse(depth: {}) {{
                        ~Rule.depends_on {{
                            Rule.computes
                        }}
                    }}
                    downstream_rules as Rule.computes @recurse(depth: {}) {{
                        ~Rule.depends_on
                    }}
                }}
                affected_attributes(func: uid(downstream_attrs)) {{
                    AbstractAttribute.abstract_path
                }}
                affected_rules(func: uid(downstream_rules)) {{
                    Rule.id
                }}
            }}"#,
            rule_id, max_depth, max_depth
        );

        #[derive(Deserialize)]
        struct Result {
            #[serde(default)]
            affected_attributes: Vec<AttrPathOnly>,
            #[serde(default)]
            affected_rules: Vec<RuleIdOnly>,
        }
        #[derive(Deserialize)]
        struct AttrPathOnly {
            #[serde(rename = "AbstractAttribute.abstract_path")]
            path: String,
        }
        #[derive(Deserialize)]
        struct RuleIdOnly {
            #[serde(rename = "Rule.id")]
            id: String,
        }

        let result: Result = self.client.query_simple(&query).await?;

        Ok(ImpactAnalysis {
            source_rule_id: rule_id.to_string(),
            affected_attributes: result
                .affected_attributes
                .into_iter()
                .map(|a| a.path)
                .collect(),
            affected_rules: result
                .affected_rules
                .into_iter()
                .map(|r| RuleId::from_string(&r.id))
                .collect(),
        })
    }
}

/// Result of impact analysis
#[derive(Debug, Clone)]
pub struct ImpactAnalysis {
    pub source_rule_id: String,
    pub affected_attributes: Vec<String>,
    pub affected_rules: Vec<RuleId>,
}

// ============================================================================
// DGraph Compiled Rule Repository (Bytecode Persistence)
// ============================================================================

/// DTO for compiled rule storage in DGraph
///
/// Stores compiled rules with their AST and optional bytecode.
/// The AST is stored as JSON string, bytecode as base64-encoded JSON of CompiledBytecode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledRuleDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    #[serde(rename = "dgraph.type", default)]
    pub dgraph_type: Vec<String>,
    #[serde(rename = "CompiledRule.rule_id")]
    pub rule_id: String,
    #[serde(rename = "CompiledRule.tier")]
    pub tier: String,
    #[serde(rename = "CompiledRule.eval_count")]
    pub eval_count: u64,
    #[serde(rename = "CompiledRule.ast_json")]
    pub ast_json: String,
    /// Base64-encoded JSON of CompiledBytecode struct (includes bytecode, constants, variable_map, variable_names)
    #[serde(rename = "CompiledRule.bytecode", skip_serializing_if = "Option::is_none")]
    pub bytecode: Option<String>,
}

/// Query result for compiled rules
#[derive(Debug, Deserialize)]
pub struct CompiledRuleQueryResult {
    #[serde(default)]
    pub compiled_rules: Vec<CompiledRuleDto>,
}

impl From<&PersistedRule> for CompiledRuleDto {
    fn from(rule: &PersistedRule) -> Self {
        let tier = match rule.tier {
            CompilationTier::Ast => "AST",
            CompilationTier::Bytecode => "BYTECODE",
            CompilationTier::Jit => "JIT",
        };

        // Serialize the AST (Expr) to JSON
        let ast_json = serde_json::to_string(&rule.ast).unwrap_or_default();

        // Serialize the entire CompiledBytecode struct to JSON, then base64 encode
        let bytecode = rule.bytecode.as_ref().map(|bc| {
            let json = serde_json::to_string(bc).unwrap_or_default();
            BASE64.encode(json.as_bytes())
        });

        Self {
            uid: None,
            dgraph_type: vec!["CompiledRule".to_string()],
            rule_id: String::new(), // Set separately when saving
            tier: tier.to_string(),
            eval_count: rule.eval_count,
            ast_json,
            bytecode,
        }
    }
}

impl TryFrom<CompiledRuleDto> for PersistedRule {
    type Error = PersistenceError;

    fn try_from(dto: CompiledRuleDto) -> Result<Self, Self::Error> {
        let tier = match dto.tier.as_str() {
            "AST" => CompilationTier::Ast,
            "BYTECODE" => CompilationTier::Bytecode,
            "JIT" => CompilationTier::Jit,
            t => {
                return Err(PersistenceError::DeserializationError(format!(
                    "Unknown compilation tier: {}",
                    t
                )))
            }
        };

        // Deserialize the AST JSON back to Expr
        let ast: Expr = serde_json::from_str(&dto.ast_json)
            .map_err(|e| PersistenceError::DeserializationError(format!("Failed to deserialize AST: {}", e)))?;

        // Decode base64 and deserialize the CompiledBytecode struct
        let bytecode: Option<CompiledBytecode> = dto.bytecode
            .map(|b| {
                let json_bytes = BASE64.decode(&b)
                    .map_err(|e| PersistenceError::DeserializationError(format!("Failed to decode bytecode base64: {}", e)))?;
                serde_json::from_slice(&json_bytes)
                    .map_err(|e| PersistenceError::DeserializationError(format!("Failed to deserialize bytecode: {}", e)))
            })
            .transpose()?;

        Ok(PersistedRule {
            ast,
            bytecode,
            tier,
            eval_count: dto.eval_count,
        })
    }
}

/// DGraph-backed Compiled Rule Repository for bytecode persistence
pub struct DgraphCompiledRuleRepository {
    client: Arc<DgraphClient>,
}

impl DgraphCompiledRuleRepository {
    pub fn new(client: Arc<DgraphClient>) -> Self {
        Self { client }
    }
}

impl std::fmt::Debug for DgraphCompiledRuleRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DgraphCompiledRuleRepository").finish()
    }
}

#[async_trait]
impl CompiledRuleRepository for DgraphCompiledRuleRepository {
    #[instrument(skip(self))]
    async fn get(&self, rule_id: &str) -> PersistenceResult<Option<PersistedRule>> {
        let query = r#"
            query GetCompiledRule($rule_id: string) {
                compiled_rules(func: eq(CompiledRule.rule_id, $rule_id)) {
                    uid
                    dgraph.type
                    CompiledRule.rule_id
                    CompiledRule.tier
                    CompiledRule.eval_count
                    CompiledRule.ast_json
                    CompiledRule.bytecode
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$rule_id".to_string(), rule_id.to_string());

        let result: CompiledRuleQueryResult = self.client.query(query, vars).await?;

        if let Some(dto) = result.compiled_rules.into_iter().next() {
            let rule = PersistedRule::try_from(dto)?;
            Ok(Some(rule))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self, rule))]
    async fn save(&self, rule_id: &str, rule: &PersistedRule) -> PersistenceResult<()> {
        // Check if compiled rule exists
        let find_query = r#"
            query FindCompiledRule($rule_id: string) {
                compiled_rules(func: eq(CompiledRule.rule_id, $rule_id)) {
                    uid
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$rule_id".to_string(), rule_id.to_string());

        let result: CompiledRuleUidResult = self.client.query(find_query, vars).await?;
        let existing_uid = result.compiled_rules.into_iter().next().and_then(|r| r.uid);

        // Use NQuads for mutation (JSON mutations have compatibility issues with dgraph-tonic)
        let nquads = compiled_rule_to_nquads(rule_id, rule, existing_uid.as_deref());
        self.client.mutate_nquads(&nquads).await?;
        debug!(rule_id = %rule_id, tier = ?rule.tier, "Compiled rule saved");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, rule_id: &str) -> PersistenceResult<()> {
        let find_query = r#"
            query FindCompiledRule($rule_id: string) {
                compiled_rules(func: eq(CompiledRule.rule_id, $rule_id)) {
                    uid
                }
            }
        "#;

        let mut vars = HashMap::new();
        vars.insert("$rule_id".to_string(), rule_id.to_string());

        let result: CompiledRuleUidResult = self.client.query(find_query, vars).await?;

        if let Some(entry) = result.compiled_rules.into_iter().next() {
            if let Some(uid) = entry.uid {
                self.client.delete_node(&uid).await?;
                debug!(rule_id = %rule_id, "Compiled rule deleted");
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn list_ids(&self) -> PersistenceResult<Vec<String>> {
        let query = r#"{
            compiled_rules(func: type(CompiledRule)) {
                CompiledRule.rule_id
            }
        }"#;

        #[derive(Deserialize)]
        struct ListResult {
            #[serde(default)]
            compiled_rules: Vec<RuleIdOnly>,
        }
        #[derive(Deserialize)]
        struct RuleIdOnly {
            #[serde(rename = "CompiledRule.rule_id")]
            rule_id: String,
        }

        let result: ListResult = self.client.query_simple(query).await?;
        Ok(result.compiled_rules.into_iter().map(|r| r.rule_id).collect())
    }
}

// ============================================================================
// Combined DGraph Repositories
// ============================================================================

/// All DGraph repositories in one struct
pub struct DgraphRepositories {
    client: Arc<DgraphClient>,
    pub products: DgraphProductRepository,
    pub attributes: DgraphAttributeRepository,
    pub rules: DgraphRuleRepository,
    pub compiled_rules: DgraphCompiledRuleRepository,
    pub graph: DgraphGraphQueries,
}

impl DgraphRepositories {
    pub fn new(config: DgraphConfig) -> PersistenceResult<Self> {
        let client = Arc::new(DgraphClient::new(config)?);

        Ok(Self {
            products: DgraphProductRepository::new(client.clone()),
            attributes: DgraphAttributeRepository::new(client.clone()),
            rules: DgraphRuleRepository::new(client.clone()),
            compiled_rules: DgraphCompiledRuleRepository::new(client.clone()),
            graph: DgraphGraphQueries::new(client.clone()),
            client,
        })
    }

    /// Apply the schema to DGraph
    pub async fn apply_schema(&self) -> PersistenceResult<()> {
        let schema = include_str!("../schema/dgraph.dql");
        self.client.apply_schema(schema).await
    }

    /// Get the underlying client
    pub fn client(&self) -> &DgraphClient {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_dto_conversion() {
        use chrono::Utc;

        let product = Product::new("test-product", "TRADING", Utc::now());
        let dto = ProductDto::from(&product);

        assert_eq!(dto.id, "test-product");
        assert_eq!(dto.status, "DRAFT");
        assert_eq!(dto.template_type, "TRADING");
        assert_eq!(dto.dgraph_type, vec!["Product"]);
    }

    #[test]
    fn test_rule_dto_conversion() {
        use serde_json::json;

        let rule = Rule::from_json_logic("test-product", "calc", json!({"*": [{"var": "x"}, 2]}))
            .with_inputs(["x"])
            .with_outputs(["doubled"]);

        let dto = RuleDto::from(&rule);

        assert_eq!(dto.rule_type, "calc");
        assert!(dto.enabled);
        assert_eq!(dto.dgraph_type, vec!["Rule"]);
    }

    #[test]
    fn test_compiled_rule_dto_conversion() {
        use product_farm_core::Value;
        use product_farm_json_logic::{Expr, VarExpr};
        use std::collections::HashMap;

        // Test AST tier (no bytecode)
        // Create: {"*": [{"var": "x"}, 2]}
        let ast_expr = Expr::Mul(vec![
            Expr::Var(VarExpr {
                path: "x".to_string(),
                default: None,
            }),
            Expr::Literal(Value::Float(2.0)),
        ]);

        let persisted_ast = PersistedRule {
            ast: ast_expr.clone(),
            bytecode: None,
            tier: CompilationTier::Ast,
            eval_count: 100,
        };

        let dto = CompiledRuleDto::from(&persisted_ast);
        assert_eq!(dto.tier, "AST");
        assert_eq!(dto.eval_count, 100);
        assert!(dto.bytecode.is_none());
        assert_eq!(dto.dgraph_type, vec!["CompiledRule"]);

        // Convert back
        let restored = PersistedRule::try_from(dto).unwrap();
        assert_eq!(restored.tier, CompilationTier::Ast);
        assert_eq!(restored.eval_count, 100);
        assert!(restored.bytecode.is_none());

        // Test Bytecode tier with CompiledBytecode struct
        let compiled_bytecode = CompiledBytecode {
            bytecode: vec![1, 2, 3, 4, 5],
            constants: vec![
                Value::Int(42),
                Value::String("hello".into()),
            ],
            variable_map: {
                let mut map = HashMap::new();
                map.insert("x".to_string(), 0);
                map
            },
            variable_names: vec!["x".to_string()],
        };

        let persisted_bytecode = PersistedRule {
            ast: ast_expr,
            bytecode: Some(compiled_bytecode.clone()),
            tier: CompilationTier::Bytecode,
            eval_count: 5000,
        };

        let dto2 = CompiledRuleDto::from(&persisted_bytecode);
        assert_eq!(dto2.tier, "BYTECODE");
        assert_eq!(dto2.eval_count, 5000);
        assert!(dto2.bytecode.is_some());

        // Convert back
        let restored2 = PersistedRule::try_from(dto2).unwrap();
        assert_eq!(restored2.tier, CompilationTier::Bytecode);
        assert_eq!(restored2.eval_count, 5000);

        // Verify bytecode was round-tripped correctly
        let restored_bc = restored2.bytecode.unwrap();
        assert_eq!(restored_bc.bytecode, vec![1, 2, 3, 4, 5]);
        assert_eq!(restored_bc.constants.len(), 2);
        assert_eq!(restored_bc.variable_names, vec!["x"]);
    }
}
