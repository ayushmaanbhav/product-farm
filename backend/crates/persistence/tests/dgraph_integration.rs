//! Integration tests for DGraph repositories
//!
//! These tests require a running DGraph instance with gRPC on localhost:9080.
//!
//! **Note**: The dgraph-tonic library (v0.11) supports up to DGraph v21.03.x.
//! If running DGraph v25+, the gRPC API may not be compatible.
//! HTTP mutations via curl work fine - only gRPC client has compatibility issues.
//!
//! To run with a compatible DGraph version (e.g., v21.03):
//!   docker run -p 8080:8080 -p 9080:9080 dgraph/standalone:v21.03.0
//!
//! Run with: cargo test -p product-farm-persistence --features dgraph --test dgraph_integration -- --ignored

#![cfg(feature = "dgraph")]

use chrono::Utc;
use product_farm_core::{AbstractAttribute, DataTypeId, Product, ProductId, Rule, Value};
use product_farm_json_logic::{CompilationTier, CompiledBytecode, Expr, PersistedRule, VarExpr};
use product_farm_persistence::dgraph::{DgraphConfig, DgraphRepositories};
use product_farm_persistence::{
    AttributeRepository, CompiledRuleRepository, ProductRepository, RuleRepository,
};
use serde_json::json;
use std::collections::HashMap;

fn create_test_config() -> DgraphConfig {
    // DGraph gRPC endpoint
    DgraphConfig {
        endpoint: "http://127.0.0.1:9080".to_string(),
    }
}

#[tokio::test]
async fn test_product_crud() {
    let repos = DgraphRepositories::new(create_test_config()).expect("Failed to connect to DGraph");

    // Create a unique product ID for this test
    let product_id = format!("test-product-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
    let product = Product::new(product_id.as_str(), "TRADING", Utc::now());

    // Save product
    repos
        .products
        .save(&product)
        .await
        .expect("Failed to save product");

    // Get product
    let retrieved = repos
        .products
        .get(&ProductId::new(&product_id))
        .await
        .expect("Failed to get product");

    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id.as_str(), product_id);
    assert_eq!(retrieved.template_type.as_str(), "TRADING");

    // Update product (change description)
    let mut updated_product = product.clone();
    updated_product.description = Some("Updated description".to_string());
    repos
        .products
        .save(&updated_product)
        .await
        .expect("Failed to update product");

    // Verify update
    let retrieved = repos
        .products
        .get(&ProductId::new(&product_id))
        .await
        .expect("Failed to get updated product")
        .unwrap();
    assert_eq!(retrieved.description, Some("Updated description".to_string()));

    // Delete product
    repos
        .products
        .delete(&ProductId::new(&product_id))
        .await
        .expect("Failed to delete product");

    // Verify deletion
    let retrieved = repos
        .products
        .get(&ProductId::new(&product_id))
        .await
        .expect("Failed to check deleted product");
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_rule_crud() {
    let repos = DgraphRepositories::new(create_test_config()).expect("Failed to connect to DGraph");

    // First create a product (rules need a product)
    let product_id = format!("test-product-rule-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
    let product = Product::new(product_id.as_str(), "INSURANCE", Utc::now());
    repos.products.save(&product).await.expect("Failed to save product");

    // Create a rule
    let rule = Rule::from_json_logic(product_id.as_str(), "calc", json!({"*": [{"var": "base"}, 1.5]}))
        .with_inputs(["base"])
        .with_outputs(["premium"])
        .with_description("Calculate premium");

    let rule_id = rule.id.clone();

    // Save rule
    repos.rules.save(&rule).await.expect("Failed to save rule");

    // Get rule
    let retrieved = repos
        .rules
        .get(&rule_id)
        .await
        .expect("Failed to get rule");

    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.rule_type, "calc");
    assert!(retrieved.enabled);

    // Find rules by product
    let product_rules = repos
        .rules
        .find_by_product(&ProductId::new(&product_id))
        .await
        .expect("Failed to find rules by product");

    assert!(!product_rules.is_empty());

    // Delete rule
    repos.rules.delete(&rule_id).await.expect("Failed to delete rule");

    // Verify deletion
    let retrieved = repos.rules.get(&rule_id).await.expect("Failed to check deleted rule");
    assert!(retrieved.is_none());

    // Cleanup: delete product
    repos.products.delete(&ProductId::new(&product_id)).await.ok();
}

#[tokio::test]
async fn test_attribute_crud() {
    let repos = DgraphRepositories::new(create_test_config()).expect("Failed to connect to DGraph");

    // First create a product
    let product_id = format!("test-product-attr-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
    let product = Product::new(product_id.as_str(), "TRADING", Utc::now());
    repos.products.save(&product).await.expect("Failed to save product");

    // Create an attribute
    let attr_path = format!("$.test.attr.{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
    let attr = AbstractAttribute::new(
        attr_path.as_str(),
        product_id.as_str(),
        "INPUT",
        DataTypeId::new("DECIMAL"),
    )
    .with_description("Test attribute");

    // Save attribute
    repos.attributes.save(&attr).await.expect("Failed to save attribute");

    // Get attribute
    let retrieved = repos
        .attributes
        .get(&attr.abstract_path)
        .await
        .expect("Failed to get attribute");

    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.abstract_path.as_str(), attr_path);
    assert_eq!(retrieved.component_type, "INPUT");

    // Find attributes by product
    let product_attrs = repos
        .attributes
        .find_by_product(&ProductId::new(&product_id))
        .await
        .expect("Failed to find attributes by product");

    assert!(!product_attrs.is_empty());

    // Delete attribute
    repos
        .attributes
        .delete(&attr.abstract_path)
        .await
        .expect("Failed to delete attribute");

    // Verify deletion
    let retrieved = repos
        .attributes
        .get(&attr.abstract_path)
        .await
        .expect("Failed to check deleted attribute");
    assert!(retrieved.is_none());

    // Cleanup: delete product
    repos.products.delete(&ProductId::new(&product_id)).await.ok();
}

#[tokio::test]
async fn test_compiled_rule_crud() {
    let repos = DgraphRepositories::new(create_test_config()).expect("Failed to connect to DGraph");

    let rule_id = format!("test-compiled-rule-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));

    // Create an AST-tier persisted rule
    let ast_expr = Expr::Mul(vec![
        Expr::Var(VarExpr {
            path: "x".to_string(),
            default: None,
        }),
        Expr::Literal(Value::Float(2.0)),
    ]);

    let persisted_rule = PersistedRule {
        ast: ast_expr.clone(),
        bytecode: None,
        tier: CompilationTier::Ast,
        eval_count: 0,
    };

    // Save compiled rule
    repos
        .compiled_rules
        .save(&rule_id, &persisted_rule)
        .await
        .expect("Failed to save compiled rule");

    // Get compiled rule
    let retrieved = repos
        .compiled_rules
        .get(&rule_id)
        .await
        .expect("Failed to get compiled rule");

    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.tier, CompilationTier::Ast);
    assert_eq!(retrieved.eval_count, 0);
    assert!(retrieved.bytecode.is_none());

    // Update to bytecode tier
    let compiled_bytecode = CompiledBytecode {
        bytecode: vec![0x01, 0x02, 0x03, 0x04],
        constants: vec![Value::Float(2.0)],
        variable_map: {
            let mut map = HashMap::new();
            map.insert("x".to_string(), 0);
            map
        },
        variable_names: vec!["x".to_string()],
    };

    let promoted_rule = PersistedRule {
        ast: ast_expr,
        bytecode: Some(compiled_bytecode),
        tier: CompilationTier::Bytecode,
        eval_count: 1000,
    };

    repos
        .compiled_rules
        .save(&rule_id, &promoted_rule)
        .await
        .expect("Failed to update compiled rule");

    // Verify update
    let retrieved = repos
        .compiled_rules
        .get(&rule_id)
        .await
        .expect("Failed to get updated compiled rule")
        .unwrap();

    assert_eq!(retrieved.tier, CompilationTier::Bytecode);
    assert_eq!(retrieved.eval_count, 1000);
    assert!(retrieved.bytecode.is_some());

    let bc = retrieved.bytecode.unwrap();
    assert_eq!(bc.bytecode, vec![0x01, 0x02, 0x03, 0x04]);
    assert_eq!(bc.variable_names, vec!["x"]);

    // List rule IDs
    let ids = repos
        .compiled_rules
        .list_ids()
        .await
        .expect("Failed to list compiled rule IDs");

    assert!(ids.contains(&rule_id));

    // Delete compiled rule
    repos
        .compiled_rules
        .delete(&rule_id)
        .await
        .expect("Failed to delete compiled rule");

    // Verify deletion
    let retrieved = repos
        .compiled_rules
        .get(&rule_id)
        .await
        .expect("Failed to check deleted compiled rule");
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_graph_queries() {
    let repos = DgraphRepositories::new(create_test_config()).expect("Failed to connect to DGraph");

    // Create a product
    let product_id = format!("test-product-graph-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
    let product = Product::new(product_id.as_str(), "INSURANCE", Utc::now());
    repos.products.save(&product).await.expect("Failed to save product");

    // Create attributes
    let input_attr_path = format!("$.input.{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
    let output_attr_path = format!("$.output.{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));

    let input_attr = AbstractAttribute::new(
        input_attr_path.as_str(),
        product_id.as_str(),
        "INPUT",
        DataTypeId::new("DECIMAL"),
    );
    let output_attr = AbstractAttribute::new(
        output_attr_path.as_str(),
        product_id.as_str(),
        "COMPUTED",
        DataTypeId::new("DECIMAL"),
    );

    repos.attributes.save(&input_attr).await.expect("Failed to save input attr");
    repos.attributes.save(&output_attr).await.expect("Failed to save output attr");

    // Test finding rules (even if empty initially)
    let rules = repos
        .graph
        .find_rules_depending_on(&input_attr_path)
        .await
        .expect("Failed to query dependent rules");

    // This should return an empty list since we haven't created rules with dependencies
    assert!(rules.is_empty());

    // Cleanup
    repos.attributes.delete(&input_attr.abstract_path).await.ok();
    repos.attributes.delete(&output_attr.abstract_path).await.ok();
    repos.products.delete(&ProductId::new(&product_id)).await.ok();
}

#[tokio::test]
async fn test_product_list_and_count() {
    let repos = DgraphRepositories::new(create_test_config()).expect("Failed to connect to DGraph");

    // Create multiple products
    let base_id = format!("test-list-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
    let mut product_ids = Vec::new();

    for i in 0..3 {
        let product_id = format!("{}-{}", base_id, i);
        let product = Product::new(product_id.as_str(), "TRADING", Utc::now());
        repos.products.save(&product).await.expect("Failed to save product");
        product_ids.push(product_id);
    }

    // Count products (should be at least 3)
    let count = repos.products.count().await.expect("Failed to count products");
    assert!(count >= 3);

    // List products
    let products = repos
        .products
        .list(10, 0)
        .await
        .expect("Failed to list products");
    assert!(!products.is_empty());

    // Cleanup
    for id in product_ids {
        repos.products.delete(&ProductId::new(&id)).await.ok();
    }
}
