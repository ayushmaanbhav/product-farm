//! Migration tools for data export/import between storage backends
//!
//! This module provides utilities for:
//! - Exporting data from any repository to JSON files
//! - Importing data from JSON files to any repository
//! - Bulk operations for migrating between storage backends

use std::path::Path;

use product_farm_core::{AbstractAttribute, Product, Rule};
use product_farm_json_logic::PersistedRule;
use serde::{Deserialize, Serialize};

use crate::{
    AttributeRepository, CompiledRuleRepository, PersistenceError, PersistenceResult,
    ProductRepository, RuleRepository,
};

/// Export format containing all data from a storage backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    /// Version of the export format
    pub version: String,
    /// Exported products
    pub products: Vec<Product>,
    /// Exported attributes
    pub attributes: Vec<AbstractAttribute>,
    /// Exported rules
    pub rules: Vec<Rule>,
    /// Exported compiled rules (rule_id -> persisted rule)
    pub compiled_rules: Vec<(String, PersistedRule)>,
}

impl ExportData {
    /// Create a new empty export data container
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            products: Vec::new(),
            attributes: Vec::new(),
            rules: Vec::new(),
            compiled_rules: Vec::new(),
        }
    }
}

impl Default for ExportData {
    fn default() -> Self {
        Self::new()
    }
}

/// Export data from repositories to JSON file
pub async fn export_to_json<P, A, R, C>(
    products_repo: &P,
    attributes_repo: &A,
    rules_repo: &R,
    compiled_rules_repo: &C,
    path: impl AsRef<Path>,
) -> PersistenceResult<ExportStats>
where
    P: ProductRepository,
    A: AttributeRepository,
    R: RuleRepository,
    C: CompiledRuleRepository,
{
    let mut export = ExportData::new();
    let mut stats = ExportStats::default();

    // Export products
    let mut offset = 0;
    const PAGE_SIZE: usize = 100;
    loop {
        let products = products_repo.list(PAGE_SIZE, offset).await?;
        if products.is_empty() {
            break;
        }
        stats.products += products.len();
        export.products.extend(products);
        offset += PAGE_SIZE;
    }

    // Export attributes for each product
    for product in &export.products {
        let attrs = attributes_repo.find_by_product(&product.id).await?;
        stats.attributes += attrs.len();
        export.attributes.extend(attrs);
    }

    // Export rules for each product
    for product in &export.products {
        let rules = rules_repo.find_by_product(&product.id).await?;
        stats.rules += rules.len();
        export.rules.extend(rules);
    }

    // Export compiled rules
    let rule_ids = compiled_rules_repo.list_ids().await?;
    for rule_id in rule_ids {
        if let Some(compiled) = compiled_rules_repo.get(&rule_id).await? {
            stats.compiled_rules += 1;
            export.compiled_rules.push((rule_id, compiled));
        }
    }

    // Write to file
    let json = serde_json::to_string_pretty(&export)
        .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;

    std::fs::write(&path, json)
        .map_err(|e| PersistenceError::IoError(e.to_string()))?;

    stats.file_path = path.as_ref().to_string_lossy().to_string();

    Ok(stats)
}

/// Import data from JSON file to repositories
pub async fn import_from_json<P, A, R, C>(
    products_repo: &P,
    attributes_repo: &A,
    rules_repo: &R,
    compiled_rules_repo: &C,
    path: impl AsRef<Path>,
) -> PersistenceResult<ImportStats>
where
    P: ProductRepository,
    A: AttributeRepository,
    R: RuleRepository,
    C: CompiledRuleRepository,
{
    let mut stats = ImportStats::default();

    // Read file
    let json = std::fs::read_to_string(&path)
        .map_err(|e| PersistenceError::IoError(e.to_string()))?;

    let export: ExportData = serde_json::from_str(&json)
        .map_err(|e| PersistenceError::SerializationError(e.to_string()))?;

    stats.file_path = path.as_ref().to_string_lossy().to_string();

    // Import products
    for product in export.products {
        if let Err(e) = products_repo.save(&product).await {
            stats.errors.push(format!("Product {}: {}", product.id.as_str(), e));
        } else {
            stats.products += 1;
        }
    }

    // Import attributes
    for attr in export.attributes {
        if let Err(e) = attributes_repo.save(&attr).await {
            stats.errors.push(format!("Attribute {}: {}", attr.abstract_path.as_str(), e));
        } else {
            stats.attributes += 1;
        }
    }

    // Import rules
    for rule in export.rules {
        if let Err(e) = rules_repo.save(&rule).await {
            stats.errors.push(format!("Rule {}: {}", rule.id, e));
        } else {
            stats.rules += 1;
        }
    }

    // Import compiled rules
    for (rule_id, compiled) in export.compiled_rules {
        if let Err(e) = compiled_rules_repo.save(&rule_id, &compiled).await {
            stats.errors.push(format!("CompiledRule {}: {}", rule_id, e));
        } else {
            stats.compiled_rules += 1;
        }
    }

    Ok(stats)
}

/// Migrate data from one repository set to another
pub async fn migrate<P1, A1, R1, C1, P2, A2, R2, C2>(
    source_products: &P1,
    source_attributes: &A1,
    source_rules: &R1,
    source_compiled: &C1,
    dest_products: &P2,
    dest_attributes: &A2,
    dest_rules: &R2,
    dest_compiled: &C2,
) -> PersistenceResult<MigrationStats>
where
    P1: ProductRepository,
    A1: AttributeRepository,
    R1: RuleRepository,
    C1: CompiledRuleRepository,
    P2: ProductRepository,
    A2: AttributeRepository,
    R2: RuleRepository,
    C2: CompiledRuleRepository,
{
    let mut stats = MigrationStats::default();

    // Migrate products
    let mut offset = 0;
    const PAGE_SIZE: usize = 100;
    loop {
        let products = source_products.list(PAGE_SIZE, offset).await?;
        if products.is_empty() {
            break;
        }
        for product in products {
            // Migrate attributes for this product
            let attrs = source_attributes.find_by_product(&product.id).await?;
            for attr in attrs {
                if let Err(e) = dest_attributes.save(&attr).await {
                    stats.errors.push(format!("Attribute {}: {}", attr.abstract_path.as_str(), e));
                } else {
                    stats.attributes += 1;
                }
            }

            // Migrate rules for this product
            let rules = source_rules.find_by_product(&product.id).await?;
            for rule in rules {
                if let Err(e) = dest_rules.save(&rule).await {
                    stats.errors.push(format!("Rule {}: {}", rule.id, e));
                } else {
                    stats.rules += 1;
                }
            }

            // Migrate the product itself
            if let Err(e) = dest_products.save(&product).await {
                stats.errors.push(format!("Product {}: {}", product.id.as_str(), e));
            } else {
                stats.products += 1;
            }
        }
        offset += PAGE_SIZE;
    }

    // Migrate compiled rules
    let rule_ids = source_compiled.list_ids().await?;
    for rule_id in rule_ids {
        if let Some(compiled) = source_compiled.get(&rule_id).await? {
            if let Err(e) = dest_compiled.save(&rule_id, &compiled).await {
                stats.errors.push(format!("CompiledRule {}: {}", rule_id, e));
            } else {
                stats.compiled_rules += 1;
            }
        }
    }

    Ok(stats)
}

/// Statistics from an export operation
#[derive(Debug, Clone, Default)]
pub struct ExportStats {
    /// Path to the exported file
    pub file_path: String,
    /// Number of products exported
    pub products: usize,
    /// Number of attributes exported
    pub attributes: usize,
    /// Number of rules exported
    pub rules: usize,
    /// Number of compiled rules exported
    pub compiled_rules: usize,
}

impl ExportStats {
    /// Total number of entities exported
    pub fn total(&self) -> usize {
        self.products + self.attributes + self.rules + self.compiled_rules
    }
}

/// Statistics from an import operation
#[derive(Debug, Clone, Default)]
pub struct ImportStats {
    /// Path to the imported file
    pub file_path: String,
    /// Number of products imported
    pub products: usize,
    /// Number of attributes imported
    pub attributes: usize,
    /// Number of rules imported
    pub rules: usize,
    /// Number of compiled rules imported
    pub compiled_rules: usize,
    /// Errors encountered during import
    pub errors: Vec<String>,
}

impl ImportStats {
    /// Total number of entities imported successfully
    pub fn total(&self) -> usize {
        self.products + self.attributes + self.rules + self.compiled_rules
    }

    /// Whether the import completed without errors
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Statistics from a migration operation
#[derive(Debug, Clone, Default)]
pub struct MigrationStats {
    /// Number of products migrated
    pub products: usize,
    /// Number of attributes migrated
    pub attributes: usize,
    /// Number of rules migrated
    pub rules: usize,
    /// Number of compiled rules migrated
    pub compiled_rules: usize,
    /// Errors encountered during migration
    pub errors: Vec<String>,
}

impl MigrationStats {
    /// Total number of entities migrated successfully
    pub fn total(&self) -> usize {
        self.products + self.attributes + self.rules + self.compiled_rules
    }

    /// Whether the migration completed without errors
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{
        InMemoryAttributeRepository, InMemoryCompiledRuleRepository, InMemoryProductRepository,
        InMemoryRuleRepository,
    };
    use chrono::Utc;
    use product_farm_core::{ProductId, TemplateType};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_export_import_roundtrip() {
        let dir = tempdir().unwrap();
        let export_path = dir.path().join("export.json");

        // Create source repositories with some data
        let products = InMemoryProductRepository::new();
        let attributes = InMemoryAttributeRepository::new();
        let rules = InMemoryRuleRepository::new();
        let compiled = InMemoryCompiledRuleRepository::new();

        // Add a product
        let product = Product::new(
            ProductId::new("test-product"),
            "Test Product".to_string(),
            TemplateType::new("loan"),
            Utc::now(),
        );
        products.save(&product).await.unwrap();

        // Export
        let export_stats = export_to_json(&products, &attributes, &rules, &compiled, &export_path)
            .await
            .unwrap();
        assert_eq!(export_stats.products, 1);
        assert_eq!(export_stats.attributes, 0);
        assert_eq!(export_stats.rules, 0);

        // Create destination repositories
        let dest_products = InMemoryProductRepository::new();
        let dest_attributes = InMemoryAttributeRepository::new();
        let dest_rules = InMemoryRuleRepository::new();
        let dest_compiled = InMemoryCompiledRuleRepository::new();

        // Import
        let import_stats = import_from_json(
            &dest_products,
            &dest_attributes,
            &dest_rules,
            &dest_compiled,
            &export_path,
        )
        .await
        .unwrap();

        assert_eq!(import_stats.products, 1);
        assert!(import_stats.is_success());

        // Verify data
        let imported = dest_products.get(&ProductId::new("test-product")).await.unwrap();
        assert!(imported.is_some());
        assert_eq!(imported.unwrap().name, "Test Product");
    }

    #[tokio::test]
    async fn test_migration_between_repos() {
        // Create source repositories with some data
        let src_products = InMemoryProductRepository::new();
        let src_attributes = InMemoryAttributeRepository::new();
        let src_rules = InMemoryRuleRepository::new();
        let src_compiled = InMemoryCompiledRuleRepository::new();

        // Add products
        for i in 0..3 {
            let product = Product::new(
                ProductId::new(&format!("product-{}", i)),
                format!("Product {}", i),
                TemplateType::new("loan"),
                Utc::now(),
            );
            src_products.save(&product).await.unwrap();
        }

        // Create destination repositories
        let dest_products = InMemoryProductRepository::new();
        let dest_attributes = InMemoryAttributeRepository::new();
        let dest_rules = InMemoryRuleRepository::new();
        let dest_compiled = InMemoryCompiledRuleRepository::new();

        // Migrate
        let stats = migrate(
            &src_products,
            &src_attributes,
            &src_rules,
            &src_compiled,
            &dest_products,
            &dest_attributes,
            &dest_rules,
            &dest_compiled,
        )
        .await
        .unwrap();

        assert_eq!(stats.products, 3);
        assert!(stats.is_success());

        // Verify data
        let count = dest_products.count().await.unwrap();
        assert_eq!(count, 3);
    }
}
