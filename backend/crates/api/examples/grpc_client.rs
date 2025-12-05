//! Example: gRPC Client Basics
//!
//! A simple example showing how to connect to the Product-FARM gRPC server
//! and perform basic operations.
//!
//! Run with:
//!   1. Start server: cargo run -p product-farm-api
//!   2. Run client: cargo run --example grpc_client
//!
//! Or set GRPC_SERVER environment variable for a different server address.

use std::collections::HashMap;

use product_farm_api::grpc::proto::{
    product_farm_service_client::ProductFarmServiceClient,
    product_service_client::ProductServiceClient,
    rule_service_client::RuleServiceClient,
    CreateProductRequest, CreateRuleRequest, DeleteProductRequest,
    EvaluateRequest, GetProductRequest, HealthCheckRequest, ListProductsRequest,
    GetExecutionPlanRequest, Value, value,
};

fn float_value(f: f64) -> Value {
    Value { value: Some(value::Value::FloatValue(f)) }
}

fn format_value(v: &Value) -> String {
    match &v.value {
        Some(value::Value::NullValue(_)) => "null".into(),
        Some(value::Value::BoolValue(b)) => b.to_string(),
        Some(value::Value::IntValue(i)) => i.to_string(),
        Some(value::Value::FloatValue(f)) => format!("{:.2}", f),
        Some(value::Value::StringValue(s)) => format!("\"{}\"", s),
        Some(value::Value::DecimalValue(d)) => d.clone(),
        Some(value::Value::ArrayValue(_)) => "[...]".into(),
        Some(value::Value::ObjectValue(_)) => "{...}".into(),
        None => "undefined".into(),
    }
}

fn current_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_addr = std::env::var("GRPC_SERVER")
        .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           Product-FARM gRPC Client Example                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // =========================================================================
    // Step 1: Health Check
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Step 1: Health Check                                           │");
    println!("└─────────────────────────────────────────────────────────────────┘");

    let mut client = ProductFarmServiceClient::connect(server_addr.clone()).await?;

    let health = client
        .health_check(HealthCheckRequest {
            service: "product-farm".into(),
        })
        .await?
        .into_inner();

    println!("Server Status: {:?}", health.status);
    println!("Server Version: {}", health.version);
    println!("Uptime: {}s", health.uptime_seconds);
    println!();

    // =========================================================================
    // Step 2: Create a Product
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Step 2: Create a Product                                       │");
    println!("└─────────────────────────────────────────────────────────────────┘");

    let mut product_client = ProductServiceClient::connect(server_addr.clone()).await?;

    let product = product_client
        .create_product(CreateProductRequest {
            id: "discount_calculator".into(),
            name: "Discount Calculator".into(),
            template_type: "pricing".into(),
            effective_from: current_timestamp(),
            expiry_at: None,
            description: Some("Calculate customer discounts based on order value".into()),
        })
        .await?
        .into_inner();

    println!("Created Product:");
    println!("  ID: {}", product.id);
    println!("  Name: {}", product.name);
    println!("  Type: {}", product.template_type);
    println!();

    // =========================================================================
    // Step 3: Create Rules
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Step 3: Create Rules                                           │");
    println!("└─────────────────────────────────────────────────────────────────┘");

    let mut rule_client = RuleServiceClient::connect(server_addr.clone()).await?;

    // Rule 1: Determine discount tier
    let rule1 = rule_client
        .create_rule(CreateRuleRequest {
            product_id: product.id.clone(),
            rule_type: "calculation".into(),
            input_attributes: vec!["order_value".into()],
            output_attributes: vec!["discount_rate".into()],
            display_expression: "20% if order >= 1000, 10% if >= 500, 5% if >= 100, else 0%".into(),
            expression_json: r#"{
                "if": [
                    {">=": [{"var": "order_value"}, 1000]}, 0.20,
                    {">=": [{"var": "order_value"}, 500]}, 0.10,
                    {">=": [{"var": "order_value"}, 100]}, 0.05,
                    0.0
                ]
            }"#.into(),
            description: Some("Calculate discount rate based on order value tiers".into()),
            order_index: 0,
        })
        .await?
        .into_inner();
    println!("Created Rule 1: {:?} ({})", rule1.description, rule1.id);

    // Rule 2: Calculate discount amount
    let rule2 = rule_client
        .create_rule(CreateRuleRequest {
            product_id: product.id.clone(),
            rule_type: "calculation".into(),
            input_attributes: vec!["order_value".into(), "discount_rate".into()],
            output_attributes: vec!["discount_amount".into()],
            display_expression: "discount = order_value * discount_rate".into(),
            expression_json: r#"{"*": [{"var": "order_value"}, {"var": "discount_rate"}]}"#.into(),
            description: Some("Calculate discount amount".into()),
            order_index: 1,
        })
        .await?
        .into_inner();
    println!("Created Rule 2: {:?} ({})", rule2.description, rule2.id);

    // Rule 3: Calculate final price
    let rule3 = rule_client
        .create_rule(CreateRuleRequest {
            product_id: product.id.clone(),
            rule_type: "calculation".into(),
            input_attributes: vec!["order_value".into(), "discount_amount".into()],
            output_attributes: vec!["final_price".into()],
            display_expression: "final_price = order_value - discount_amount".into(),
            expression_json: r#"{"-": [{"var": "order_value"}, {"var": "discount_amount"}]}"#.into(),
            description: Some("Calculate final price after discount".into()),
            order_index: 2,
        })
        .await?
        .into_inner();
    println!("Created Rule 3: {:?} ({})", rule3.description, rule3.id);
    println!();

    // =========================================================================
    // Step 4: Get Execution Plan
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Step 4: Get Execution Plan                                     │");
    println!("└─────────────────────────────────────────────────────────────────┘");

    let plan = client
        .get_execution_plan(GetExecutionPlanRequest {
            product_id: product.id.clone(),
            rule_ids: vec![],  // Empty = all rules
        })
        .await?
        .into_inner();

    println!("Execution Plan:");
    println!("  Levels: {}", plan.levels.len());
    for level in &plan.levels {
        println!("    Level {}: {} rules", level.level, level.rule_ids.len());
    }
    if !plan.dependencies.is_empty() {
        println!("  Dependencies:");
        for dep in &plan.dependencies {
            println!("    {} depends on: {:?}", dep.rule_id, dep.depends_on);
        }
    }
    if !plan.missing_inputs.is_empty() {
        println!("  Missing Inputs:");
        for missing in &plan.missing_inputs {
            println!("    Rule {}: {}", missing.rule_id, missing.input_path);
        }
    }
    println!();

    // =========================================================================
    // Step 5: Evaluate Rules
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Step 5: Evaluate Rules                                         │");
    println!("└─────────────────────────────────────────────────────────────────┘");

    let test_orders = vec![50.0, 150.0, 750.0, 1500.0];

    println!("┌────────────────┬───────────────┬─────────────────┬──────────────┐");
    println!("│ Order Value    │ Discount Rate │ Discount Amount │ Final Price  │");
    println!("├────────────────┼───────────────┼─────────────────┼──────────────┤");

    for order_value in test_orders {
        let response = client
            .evaluate(EvaluateRequest {
                product_id: product.id.clone(),
                input_data: HashMap::from([
                    ("order_value".into(), float_value(order_value)),
                ]),
                rule_ids: vec![],
                options: None,
            })
            .await?
            .into_inner();

        let discount_rate = response.outputs.get("discount_rate")
            .map(format_value)
            .unwrap_or_default();
        let discount_amount = response.outputs.get("discount_amount")
            .map(format_value)
            .unwrap_or_default();
        let final_price = response.outputs.get("final_price")
            .map(format_value)
            .unwrap_or_default();

        println!("│ ${:>12.2} │ {:>13} │ ${:>14} │ ${:>10} │",
            order_value, discount_rate, discount_amount, final_price);
    }
    println!("└────────────────┴───────────────┴─────────────────┴──────────────┘");
    println!();

    // =========================================================================
    // Step 6: List Products
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Step 6: List Products                                          │");
    println!("└─────────────────────────────────────────────────────────────────┘");

    let products = product_client
        .list_products(ListProductsRequest {
            page_size: 10,
            page_token: String::new(),
            status_filter: None,
            template_type_filter: None,
        })
        .await?
        .into_inner();

    println!("Total Products: {}", products.total_count);
    for p in &products.products {
        println!("  - {} ({}): {}", p.name, p.id, p.description);
    }
    println!();

    // =========================================================================
    // Step 7: Get Product Details
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Step 7: Get Product Details                                    │");
    println!("└─────────────────────────────────────────────────────────────────┘");

    let details = product_client
        .get_product(GetProductRequest {
            id: product.id.clone(),
        })
        .await?
        .into_inner();

    println!("Product Details:");
    println!("  ID: {}", details.id);
    println!("  Name: {}", details.name);
    println!("  Description: {}", details.description);
    println!("  Type: {}", details.template_type);
    println!("  Created: {}", details.created_at);
    println!();

    // =========================================================================
    // Step 8: Cleanup (Delete Product)
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Step 8: Cleanup                                                │");
    println!("└─────────────────────────────────────────────────────────────────┘");

    product_client
        .delete_product(DeleteProductRequest {
            id: product.id.clone(),
        })
        .await?;

    println!("Deleted product: {}", product.id);
    println!();

    // Final health check
    let health = client
        .health_check(HealthCheckRequest {
            service: "product-farm".into(),
        })
        .await?
        .into_inner();

    println!("Final Server State: {:?}", health.status);

    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("                    Example Complete!                              ");
    println!("═══════════════════════════════════════════════════════════════════");

    Ok(())
}
