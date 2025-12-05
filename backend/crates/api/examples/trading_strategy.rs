//! Example: Trading Strategy Rule Evaluation
//!
//! This example demonstrates how to use Product-FARM to build a simple
//! trading strategy with rules for entry/exit signals.
//!
//! Run with: cargo run --example trading_strategy
//!
//! Note: Start the server first: cargo run -p product-farm-api

use std::collections::HashMap;

use product_farm_api::grpc::proto::{
    product_farm_service_client::ProductFarmServiceClient,
    product_service_client::ProductServiceClient,
    rule_service_client::RuleServiceClient,
    CreateProductRequest, CreateRuleRequest, EvaluateRequest,
    Value, value,
};

fn float_value(f: f64) -> Value {
    Value { value: Some(value::Value::FloatValue(f)) }
}

fn current_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_addr = std::env::var("GRPC_SERVER")
        .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());

    println!("Connecting to Product-FARM server at {}...", server_addr);

    // Connect to all services
    let mut product_client = ProductServiceClient::connect(server_addr.clone()).await?;
    let mut rule_client = RuleServiceClient::connect(server_addr.clone()).await?;
    let mut eval_client = ProductFarmServiceClient::connect(server_addr).await?;

    println!("Connected! Creating trading strategy product...\n");

    // =========================================================================
    // Step 1: Create a Trading Strategy Product
    // =========================================================================
    let product = product_client
        .create_product(CreateProductRequest {
            id: "rsi_momentum_strategy".into(),
            name: "RSI Momentum Strategy".into(),
            template_type: "trading".into(),
            effective_from: current_timestamp(),
            expiry_at: None,
            description: Some("A simple momentum strategy using RSI indicator".into()),
        })
        .await?
        .into_inner();

    println!("Created product: {} (ID: {})", product.name, product.id);

    // =========================================================================
    // Step 2: Create Trading Rules
    // =========================================================================

    // Rule 1: RSI Signal - determine if RSI indicates oversold/overbought
    rule_client
        .create_rule(CreateRuleRequest {
            product_id: product.id.clone(),
            rule_type: "indicator-signal".into(),
            input_attributes: vec!["rsi".into()],
            output_attributes: vec!["rsi_signal".into()],
            display_expression: "OVERSOLD if RSI < 30, OVERBOUGHT if RSI > 70, else NEUTRAL".into(),
            expression_json: r#"{"if": [{">": [{"var": "rsi"}, 70]}, "OVERBOUGHT", {"<": [{"var": "rsi"}, 30]}, "OVERSOLD", "NEUTRAL"]}"#.into(),
            description: Some("Convert RSI value to a signal".into()),
            order_index: 0,
        })
        .await?;
    println!("Created rule: RSI Signal");

    // Rule 2: Trend Filter - check if price is above moving average
    rule_client
        .create_rule(CreateRuleRequest {
            product_id: product.id.clone(),
            rule_type: "trend-filter".into(),
            input_attributes: vec!["price".into(), "sma_50".into()],
            output_attributes: vec!["trend".into()],
            display_expression: "BULLISH if price > SMA50, else BEARISH".into(),
            expression_json: r#"{"if": [{">": [{"var": "price"}, {"var": "sma_50"}]}, "BULLISH", "BEARISH"]}"#.into(),
            description: Some("Determine trend based on price vs SMA".into()),
            order_index: 1,
        })
        .await?;
    println!("Created rule: Trend Filter");

    // Rule 3: Entry Signal - combine RSI signal and trend
    rule_client
        .create_rule(CreateRuleRequest {
            product_id: product.id.clone(),
            rule_type: "entry-logic".into(),
            input_attributes: vec!["rsi_signal".into(), "trend".into()],
            output_attributes: vec!["entry_signal".into()],
            display_expression: "BUY if oversold + bullish trend, SELL if overbought + bearish, else HOLD".into(),
            expression_json: r#"{"if": [{"and": [{"==": [{"var": "rsi_signal"}, "OVERSOLD"]}, {"==": [{"var": "trend"}, "BULLISH"]}]}, "BUY", {"and": [{"==": [{"var": "rsi_signal"}, "OVERBOUGHT"]}, {"==": [{"var": "trend"}, "BEARISH"]}]}, "SELL", "HOLD"]}"#.into(),
            description: Some("Generate entry signal based on RSI and trend".into()),
            order_index: 2,
        })
        .await?;
    println!("Created rule: Entry Signal");

    // Rule 4: Position Size - calculate position size based on risk
    rule_client
        .create_rule(CreateRuleRequest {
            product_id: product.id.clone(),
            rule_type: "position-sizing".into(),
            input_attributes: vec!["account_balance".into(), "risk_percent".into(), "entry_signal".into()],
            output_attributes: vec!["position_size".into()],
            display_expression: "position = balance * risk% if entry signal, else 0".into(),
            expression_json: r#"{"if": [{"==": [{"var": "entry_signal"}, "HOLD"]}, 0, {"*": [{"var": "account_balance"}, {"/": [{"var": "risk_percent"}, 100]}]}]}"#.into(),
            description: Some("Calculate position size based on risk management".into()),
            order_index: 3,
        })
        .await?;
    println!("Created rule: Position Size\n");

    // =========================================================================
    // Step 3: Evaluate Different Market Scenarios
    // =========================================================================

    println!("=== Scenario 1: Oversold in Bullish Trend (BUY signal expected) ===");
    evaluate_scenario(&mut eval_client, &product.id, HashMap::from([
        ("rsi".into(), float_value(25.0)),
        ("price".into(), float_value(105.0)),
        ("sma_50".into(), float_value(100.0)),
        ("account_balance".into(), float_value(10000.0)),
        ("risk_percent".into(), float_value(2.0)),
    ])).await?;

    println!("\n=== Scenario 2: Overbought in Bearish Trend (SELL signal expected) ===");
    evaluate_scenario(&mut eval_client, &product.id, HashMap::from([
        ("rsi".into(), float_value(75.0)),
        ("price".into(), float_value(95.0)),
        ("sma_50".into(), float_value(100.0)),
        ("account_balance".into(), float_value(10000.0)),
        ("risk_percent".into(), float_value(2.0)),
    ])).await?;

    println!("\n=== Scenario 3: Neutral RSI (HOLD signal expected) ===");
    evaluate_scenario(&mut eval_client, &product.id, HashMap::from([
        ("rsi".into(), float_value(50.0)),
        ("price".into(), float_value(100.0)),
        ("sma_50".into(), float_value(100.0)),
        ("account_balance".into(), float_value(10000.0)),
        ("risk_percent".into(), float_value(2.0)),
    ])).await?;

    println!("\n=== Scenario 4: Oversold but Bearish Trend (HOLD - conflicting signals) ===");
    evaluate_scenario(&mut eval_client, &product.id, HashMap::from([
        ("rsi".into(), float_value(25.0)),
        ("price".into(), float_value(95.0)),
        ("sma_50".into(), float_value(100.0)),
        ("account_balance".into(), float_value(10000.0)),
        ("risk_percent".into(), float_value(2.0)),
    ])).await?;

    println!("\nExample complete!");
    Ok(())
}

async fn evaluate_scenario(
    client: &mut ProductFarmServiceClient<tonic::transport::Channel>,
    product_id: &str,
    input_data: HashMap<String, Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Print inputs
    println!("Inputs:");
    for (k, v) in &input_data {
        println!("  {}: {}", k, format_value(v));
    }

    // Evaluate
    let response = client
        .evaluate(EvaluateRequest {
            product_id: product_id.to_string(),
            input_data,
            rule_ids: vec![],
            options: None,
        })
        .await?
        .into_inner();

    // Print outputs
    println!("Outputs:");
    for key in ["rsi_signal", "trend", "entry_signal", "position_size"] {
        if let Some(v) = response.outputs.get(key) {
            println!("  {}: {}", key, format_value(v));
        }
    }

    // Print metrics
    if let Some(metrics) = &response.metrics {
        println!("Metrics:");
        println!("  Rules executed: {}", metrics.rules_executed);
        println!("  Total time: {}µs", metrics.total_time_ns / 1000);
    }

    Ok(())
}

fn format_value(v: &Value) -> String {
    match &v.value {
        Some(value::Value::NullValue(_)) => "null".into(),
        Some(value::Value::BoolValue(b)) => b.to_string(),
        Some(value::Value::IntValue(i)) => i.to_string(),
        Some(value::Value::FloatValue(f)) => format!("{:.2}", f),
        Some(value::Value::StringValue(s)) => s.clone(),
        Some(value::Value::DecimalValue(d)) => d.clone(),
        Some(value::Value::ArrayValue(_)) => "[...]".into(),
        Some(value::Value::ObjectValue(_)) => "{...}".into(),
        None => "undefined".into(),
    }
}
