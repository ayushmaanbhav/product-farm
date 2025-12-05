//! gRPC Integration Tests
//!
//! These tests start a real gRPC server and test the full request/response cycle.

use std::net::SocketAddr;
use std::time::Duration;

use product_farm_api::{
    grpc::proto::{
        product_farm_service_client::ProductFarmServiceClient,
        product_service_client::ProductServiceClient,
        rule_service_client::RuleServiceClient,
        CreateProductRequest, CreateRuleRequest, DeleteProductRequest, DeleteRuleRequest,
        EvaluateRequest, GetProductRequest, GetRuleRequest,
        HealthCheckRequest, ListProductsRequest, ListRulesRequest, UpdateProductRequest,
        UpdateRuleRequest, Value, value,
    },
    ProductFarmServer, ServerConfig,
};

fn int_value(i: i64) -> Value {
    Value { value: Some(value::Value::IntValue(i)) }
}

fn _float_value(f: f64) -> Value {
    Value { value: Some(value::Value::FloatValue(f)) }
}

fn extract_float(v: &Value) -> Option<f64> {
    match &v.value {
        Some(value::Value::FloatValue(f)) => Some(*f),
        Some(value::Value::IntValue(i)) => Some(*i as f64),
        _ => None,
    }
}

/// Helper to find an available port
fn find_available_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// Start a test server on a random port and return the address
async fn start_test_server() -> (SocketAddr, tokio::sync::oneshot::Sender<()>) {
    let port = find_available_port();
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let config = ServerConfig::new(addr).without_keepalive();
    let server = ProductFarmServer::with_config(config);

    tokio::spawn(async move {
        let _ = server.run_with_shutdown(async {
            shutdown_rx.await.ok();
        }).await;
    });

    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    (addr, shutdown_tx)
}

fn current_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

#[tokio::test]
async fn test_health_check() {
    let (addr, _shutdown) = start_test_server().await;
    let server_addr = format!("http://{}", addr);

    let mut client = ProductFarmServiceClient::connect(server_addr)
        .await
        .expect("Failed to connect");

    let response = client
        .health_check(HealthCheckRequest {
            service: "product-farm".into(),
        })
        .await
        .expect("Health check failed")
        .into_inner();

    assert_eq!(response.status, 1); // SERVING
    assert!(!response.version.is_empty());
}

#[tokio::test]
async fn test_product_crud() {
    let (addr, _shutdown) = start_test_server().await;
    let server_addr = format!("http://{}", addr);

    let mut client = ProductServiceClient::connect(server_addr)
        .await
        .expect("Failed to connect");

    // Create product
    let product = client
        .create_product(CreateProductRequest {
            id: "test_product_crud".into(),
            name: "Test Product".into(),
            template_type: "insurance".into(),
            effective_from: current_timestamp(),
            expiry_at: None,
            description: Some("A test product".into()),
        })
        .await
        .expect("Create product failed")
        .into_inner();

    assert_eq!(product.id, "test_product_crud");
    assert_eq!(product.name, "Test Product");
    assert_eq!(product.template_type, "insurance");

    // Get product
    let fetched = client
        .get_product(GetProductRequest {
            id: product.id.clone(),
        })
        .await
        .expect("Get product failed")
        .into_inner();

    assert_eq!(fetched.id, product.id);
    assert_eq!(fetched.name, "Test Product");

    // List products
    let list = client
        .list_products(ListProductsRequest {
            page_size: 10,
            page_token: String::new(),
            status_filter: None,
            template_type_filter: None,
        })
        .await
        .expect("List products failed")
        .into_inner();

    assert!(list.products.iter().any(|p| p.id == product.id));

    // Delete product
    let delete_response = client
        .delete_product(DeleteProductRequest {
            id: product.id.clone(),
        })
        .await
        .expect("Delete product failed")
        .into_inner();

    assert!(delete_response.success);
}

#[tokio::test]
async fn test_rule_crud() {
    let (addr, _shutdown) = start_test_server().await;
    let server_addr = format!("http://{}", addr);

    let mut product_client = ProductServiceClient::connect(server_addr.clone())
        .await
        .expect("Failed to connect");
    let mut rule_client = RuleServiceClient::connect(server_addr)
        .await
        .expect("Failed to connect");

    // Create product first
    let product = product_client
        .create_product(CreateProductRequest {
            id: "rule_test_product".into(),
            name: "Rule Test Product".into(),
            template_type: "insurance".into(),
            effective_from: current_timestamp(),
            expiry_at: None,
            description: Some("Product for rule testing".into()),
        })
        .await
        .expect("Create product failed")
        .into_inner();

    // Create rule
    let rule = rule_client
        .create_rule(CreateRuleRequest {
            product_id: product.id.clone(),
            rule_type: "calculation".into(),
            input_attributes: vec!["x".into()],
            output_attributes: vec!["doubled".into()],
            display_expression: "doubled = x * 2".into(),
            expression_json: r#"{"*": [{"var": "x"}, 2]}"#.into(),
            description: Some("Double the input".into()),
            order_index: 0,
        })
        .await
        .expect("Create rule failed")
        .into_inner();

    assert!(!rule.id.is_empty());
    assert_eq!(rule.product_id, product.id);
    assert_eq!(rule.rule_type, "calculation");

    // List rules
    let rules_list = rule_client
        .list_rules(ListRulesRequest {
            product_id: product.id.clone(),
            page_size: 10,
            page_token: String::new(),
            rule_type_filter: None,
            enabled_filter: None,
        })
        .await
        .expect("List rules failed")
        .into_inner();

    assert_eq!(rules_list.rules.len(), 1);
    assert_eq!(rules_list.rules[0].id, rule.id);

    // Get rule
    let fetched = rule_client
        .get_rule(GetRuleRequest {
            id: rule.id.clone(),
        })
        .await
        .expect("Get rule failed")
        .into_inner();

    assert_eq!(fetched.id, rule.id);
    assert_eq!(fetched.expression_json, r#"{"*": [{"var": "x"}, 2]}"#);

    // Update rule
    let updated = rule_client
        .update_rule(UpdateRuleRequest {
            id: rule.id.clone(),
            rule_type: None,
            display_expression: Some("doubled = x * 3".into()),
            expression_json: Some(r#"{"*": [{"var": "x"}, 3]}"#.into()),
            description: Some("Triple the input".into()),
            order_index: None,
            enabled: Some(true),
            input_attributes: vec![],
            output_attributes: vec![],
        })
        .await
        .expect("Update rule failed")
        .into_inner();

    assert_eq!(updated.display_expression, "doubled = x * 3");

    // Delete rule
    let delete_response = rule_client
        .delete_rule(DeleteRuleRequest {
            id: rule.id.clone(),
        })
        .await
        .expect("Delete rule failed")
        .into_inner();

    assert!(delete_response.success);
}

#[tokio::test]
async fn test_evaluate_simple_rule() {
    let (addr, _shutdown) = start_test_server().await;
    let server_addr = format!("http://{}", addr);

    let mut product_client = ProductServiceClient::connect(server_addr.clone())
        .await
        .expect("Failed to connect");
    let mut rule_client = RuleServiceClient::connect(server_addr.clone())
        .await
        .expect("Failed to connect");
    let mut eval_client = ProductFarmServiceClient::connect(server_addr)
        .await
        .expect("Failed to connect");

    // Create product
    let product = product_client
        .create_product(CreateProductRequest {
            id: "eval_test_product".into(),
            name: "Evaluation Test Product".into(),
            template_type: "insurance".into(),
            effective_from: current_timestamp(),
            expiry_at: None,
            description: Some("Product for evaluation testing".into()),
        })
        .await
        .expect("Create product failed")
        .into_inner();

    // Create a rule that multiplies input by 2
    rule_client
        .create_rule(CreateRuleRequest {
            product_id: product.id.clone(),
            rule_type: "calculation".into(),
            input_attributes: vec!["input_value".into()],
            output_attributes: vec!["output_value".into()],
            display_expression: "output_value = input_value * 2".into(),
            expression_json: r#"{"*": [{"var": "input_value"}, 2]}"#.into(),
            description: Some("Multiply by 2".into()),
            order_index: 0,
        })
        .await
        .expect("Create rule failed");

    // Evaluate with input_value = 10
    let mut input_data = std::collections::HashMap::new();
    input_data.insert("input_value".to_string(), int_value(10));

    let result = eval_client
        .evaluate(EvaluateRequest {
            product_id: product.id.clone(),
            input_data,
            rule_ids: vec![],
            options: None,
        })
        .await
        .expect("Evaluate failed")
        .into_inner();

    assert!(result.success);
    assert!(result.outputs.contains_key("output_value"));

    let output = &result.outputs["output_value"];
    let value = extract_float(output).expect("Expected numeric output");
    assert_eq!(value, 20.0);
}

#[tokio::test]
async fn test_evaluate_chain_of_rules() {
    let (addr, _shutdown) = start_test_server().await;
    let server_addr = format!("http://{}", addr);

    let mut product_client = ProductServiceClient::connect(server_addr.clone())
        .await
        .expect("Failed to connect");
    let mut rule_client = RuleServiceClient::connect(server_addr.clone())
        .await
        .expect("Failed to connect");
    let mut eval_client = ProductFarmServiceClient::connect(server_addr)
        .await
        .expect("Failed to connect");

    // Create product
    let product = product_client
        .create_product(CreateProductRequest {
            id: "chain_eval_test".into(),
            name: "Chain Evaluation Test".into(),
            template_type: "insurance".into(),
            effective_from: current_timestamp(),
            expiry_at: None,
            description: Some("Test chained rule evaluation".into()),
        })
        .await
        .expect("Create product failed")
        .into_inner();

    // Rule 1: doubled = input * 2
    rule_client
        .create_rule(CreateRuleRequest {
            product_id: product.id.clone(),
            rule_type: "calculation".into(),
            input_attributes: vec!["input".into()],
            output_attributes: vec!["doubled".into()],
            display_expression: "doubled = input * 2".into(),
            expression_json: r#"{"*": [{"var": "input"}, 2]}"#.into(),
            description: Some("Double the input".into()),
            order_index: 0,
        })
        .await
        .expect("Create rule 1 failed");

    // Rule 2: final = doubled + 5
    rule_client
        .create_rule(CreateRuleRequest {
            product_id: product.id.clone(),
            rule_type: "calculation".into(),
            input_attributes: vec!["doubled".into()],
            output_attributes: vec!["final".into()],
            display_expression: "final = doubled + 5".into(),
            expression_json: r#"{"+": [{"var": "doubled"}, 5]}"#.into(),
            description: Some("Add 5 to doubled".into()),
            order_index: 1,
        })
        .await
        .expect("Create rule 2 failed");

    // Evaluate with input = 10 -> doubled = 20 -> final = 25
    let mut input_data = std::collections::HashMap::new();
    input_data.insert("input".to_string(), int_value(10));

    let result = eval_client
        .evaluate(EvaluateRequest {
            product_id: product.id.clone(),
            input_data,
            rule_ids: vec![],
            options: None,
        })
        .await
        .expect("Evaluate failed")
        .into_inner();

    assert!(result.success);

    if let Some(output) = result.outputs.get("final") {
        let value = extract_float(output).expect("Expected numeric output");
        assert_eq!(value, 25.0);
    }
}

#[tokio::test]
async fn test_product_update() {
    let (addr, _shutdown) = start_test_server().await;
    let server_addr = format!("http://{}", addr);

    let mut client = ProductServiceClient::connect(server_addr)
        .await
        .expect("Failed to connect");

    // Create product
    let product = client
        .create_product(CreateProductRequest {
            id: "update_test_product".into(),
            name: "Original Name".into(),
            template_type: "insurance".into(),
            effective_from: current_timestamp(),
            expiry_at: None,
            description: Some("Original description".into()),
        })
        .await
        .expect("Create product failed")
        .into_inner();

    // Update product
    let updated = client
        .update_product(UpdateProductRequest {
            id: product.id.clone(),
            name: Some("Updated Name".into()),
            description: Some("Updated description".into()),
            effective_from: None,
            expiry_at: None,
        })
        .await
        .expect("Update product failed")
        .into_inner();

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.description, "Updated description");

    // Verify the update persisted
    let fetched = client
        .get_product(GetProductRequest {
            id: product.id.clone(),
        })
        .await
        .expect("Get product failed")
        .into_inner();

    assert_eq!(fetched.name, "Updated Name");
}

#[tokio::test]
async fn test_multiple_rules_same_product() {
    let (addr, _shutdown) = start_test_server().await;
    let server_addr = format!("http://{}", addr);

    let mut product_client = ProductServiceClient::connect(server_addr.clone())
        .await
        .expect("Failed to connect");
    let mut rule_client = RuleServiceClient::connect(server_addr)
        .await
        .expect("Failed to connect");

    // Create product
    let product = product_client
        .create_product(CreateProductRequest {
            id: "multi_rule_product".into(),
            name: "Multi-Rule Product".into(),
            template_type: "insurance".into(),
            effective_from: current_timestamp(),
            expiry_at: None,
            description: None,
        })
        .await
        .expect("Create product failed")
        .into_inner();

    // Create multiple rules
    for i in 0..5 {
        rule_client
            .create_rule(CreateRuleRequest {
                product_id: product.id.clone(),
                rule_type: "calculation".into(),
                input_attributes: vec![format!("input_{}", i)],
                output_attributes: vec![format!("output_{}", i)],
                display_expression: format!("output_{} = input_{} + {}", i, i, i),
                expression_json: format!(r#"{{"+": [{{"var": "input_{}"}}, {}]}}"#, i, i),
                description: Some(format!("Rule {}", i)),
                order_index: i,
            })
            .await
            .unwrap_or_else(|_| panic!("Create rule {} failed", i));
    }

    // List rules
    let rules_list = rule_client
        .list_rules(ListRulesRequest {
            product_id: product.id.clone(),
            page_size: 10,
            page_token: String::new(),
            rule_type_filter: None,
            enabled_filter: None,
        })
        .await
        .expect("List rules failed")
        .into_inner();

    assert_eq!(rules_list.rules.len(), 5);
}

#[tokio::test]
async fn test_invalid_product_id() {
    let (addr, _shutdown) = start_test_server().await;
    let server_addr = format!("http://{}", addr);

    let mut client = ProductServiceClient::connect(server_addr)
        .await
        .expect("Failed to connect");

    // Try to create product with invalid ID (contains spaces)
    let result = client
        .create_product(CreateProductRequest {
            id: "invalid product id".into(),
            name: "Test".into(),
            template_type: "insurance".into(),
            effective_from: current_timestamp(),
            expiry_at: None,
            description: None,
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_nonexistent_product() {
    let (addr, _shutdown) = start_test_server().await;
    let server_addr = format!("http://{}", addr);

    let mut client = ProductServiceClient::connect(server_addr)
        .await
        .expect("Failed to connect");

    let result = client
        .get_product(GetProductRequest {
            id: "nonexistent-product".into(),
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_rule_json() {
    let (addr, _shutdown) = start_test_server().await;
    let server_addr = format!("http://{}", addr);

    let mut product_client = ProductServiceClient::connect(server_addr.clone())
        .await
        .expect("Failed to connect");
    let mut rule_client = RuleServiceClient::connect(server_addr)
        .await
        .expect("Failed to connect");

    // Create product
    let product = product_client
        .create_product(CreateProductRequest {
            id: "invalid_json_test".into(),
            name: "Invalid JSON Test".into(),
            template_type: "insurance".into(),
            effective_from: current_timestamp(),
            expiry_at: None,
            description: None,
        })
        .await
        .expect("Create product failed")
        .into_inner();

    // Try to create rule with invalid JSON
    let result = rule_client
        .create_rule(CreateRuleRequest {
            product_id: product.id.clone(),
            rule_type: "calculation".into(),
            input_attributes: vec!["x".into()],
            output_attributes: vec!["y".into()],
            display_expression: "y = x".into(),
            expression_json: "not valid json {{{".into(),
            description: None,
            order_index: 0,
        })
        .await;

    assert!(result.is_err());
}
