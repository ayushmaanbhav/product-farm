//! Comprehensive Example: Full Feature Showcase
//!
//! This example demonstrates all major features of Product-FARM:
//! - Rule builders for common patterns
//! - Rule validation before execution
//! - Batch evaluation for multiple inputs
//! - Execution statistics
//! - Trading strategy use case
//!
//! Run with: cargo run --example comprehensive

use product_farm_api::{ProductFarmService, RuleValidator, find_missing_inputs};
use product_farm_core::{
    CalcRuleBuilder, ConditionalRuleBuilder, SignalRuleBuilder,
    ProductId, Rule,
};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       Product-FARM Comprehensive Feature Showcase            ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut service = ProductFarmService::new();

    // =========================================================================
    // Part 1: Rule Builders
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Part 1: Rule Builders - Create rules with fluent API           │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    // 1.1 Calculation Rules
    println!("1.1 CalcRuleBuilder Examples:");

    let _position_value_rule = CalcRuleBuilder::new("trading", "position_value")
        .with_description("Calculate position value")
        .multiply("shares", "price");
    println!("   - Position Value: shares × price");

    let _risk_amount_rule = CalcRuleBuilder::new("trading", "risk_amount")
        .with_description("Calculate risk amount")
        .percentage("capital", "risk_pct");
    println!("   - Risk Amount: capital × (risk_pct / 100)");

    let _total_exposure_rule = CalcRuleBuilder::new("trading", "total_exposure")
        .with_description("Sum all positions")
        .sum(&["pos1_value", "pos2_value", "pos3_value"]);
    println!("   - Total Exposure: pos1 + pos2 + pos3");

    // 1.2 Conditional Rules
    println!("\n1.2 ConditionalRuleBuilder Examples:");

    let _grade_rule = ConditionalRuleBuilder::new("grading", "grade")
        .with_description("Assign letter grade")
        .tiers("score", &[(90.0, "A"), (80.0, "B"), (70.0, "C"), (60.0, "D")], "F");
    println!("   - Grade Tiers: A(>90), B(>80), C(>70), D(>60), F(else)");

    let _status_rule = ConditionalRuleBuilder::new("monitor", "status")
        .with_description("Check temperature status")
        .in_range("temperature", 36.0, 37.5, "NORMAL", "ABNORMAL");
    println!("   - Temperature: NORMAL if 36.0-37.5, else ABNORMAL");

    // 1.3 Signal Rules (Trading)
    println!("\n1.3 SignalRuleBuilder Examples:");

    let _rsi_signal = SignalRuleBuilder::new("strategy", "rsi_signal")
        .with_description("RSI oversold/overbought signal")
        .rsi_signal("rsi", 30.0, 70.0);
    println!("   - RSI Signal: BUY(<30), SELL(>70), HOLD(else)");

    let _ma_crossover = SignalRuleBuilder::new("strategy", "ma_signal")
        .with_description("Moving average crossover")
        .crossover_signal("sma_fast", "sma_slow");
    println!("   - MA Crossover: BUY(fast>slow), SELL(fast<slow)\n");

    // =========================================================================
    // Part 2: Rule Validation
    // =========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Part 2: Rule Validation - Check rules before execution         │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    // 2.1 Valid rules
    println!("2.1 Validating valid rule chain:");
    let valid_rules = vec![
        Rule::from_json_logic("test", "calc", json!({"*": [{"var": "x"}, 2]}))
            .with_inputs(["x"])
            .with_outputs(["doubled"])
            .with_order(0),
        Rule::from_json_logic("test", "calc", json!({"+": [{"var": "doubled"}, 10]}))
            .with_inputs(["doubled"])
            .with_outputs(["result"])
            .with_order(1),
    ];

    let validation = RuleValidator::validate(&valid_rules);
    println!("   Valid: {}", validation.valid);
    println!("   Errors: {}", validation.errors.len());
    println!("   Warnings: {}", validation.warnings.len());
    if let Some(ref levels) = validation.execution_levels {
        println!("   Execution Levels: {} (parallel groups)", levels.len());
    }

    // 2.2 Invalid rules (cycle detection)
    println!("\n2.2 Detecting cyclic dependencies:");
    let cyclic_rules = vec![
        Rule::from_json_logic("test", "calc", json!({"var": "b"}))
            .with_inputs(["b"])
            .with_outputs(["a"]),
        Rule::from_json_logic("test", "calc", json!({"var": "a"}))
            .with_inputs(["a"])
            .with_outputs(["b"]),
    ];

    let validation = RuleValidator::validate(&cyclic_rules);
    println!("   Valid: {}", validation.valid);
    println!("   Error: {}", validation.errors.first()
        .map(|e| e.message.as_str())
        .unwrap_or("none"));

    // 2.3 Duplicate output detection
    println!("\n2.3 Detecting duplicate outputs:");
    let duplicate_rules = vec![
        Rule::from_json_logic("test", "calc", json!({"var": "x"}))
            .with_inputs(["x"])
            .with_outputs(["result"]),
        Rule::from_json_logic("test", "calc", json!({"var": "y"}))
            .with_inputs(["y"])
            .with_outputs(["result"]), // Duplicate!
    ];

    let validation = RuleValidator::validate(&duplicate_rules);
    println!("   Valid: {}", validation.valid);
    println!("   Error: {}", validation.errors.first()
        .map(|e| e.message.as_str())
        .unwrap_or("none"));

    // 2.4 Find missing inputs
    println!("\n2.4 Finding missing inputs:");
    let rules_with_missing = vec![
        Rule::from_json_logic("test", "calc", json!({"+": [{"var": "a"}, {"var": "b"}]}))
            .with_inputs(["a", "b"])
            .with_outputs(["sum"]),
        Rule::from_json_logic("test", "calc", json!({"*": [{"var": "sum"}, {"var": "c"}]}))
            .with_inputs(["sum", "c"])
            .with_outputs(["product"]),
    ];

    let missing = find_missing_inputs(&rules_with_missing, &["a"]);
    println!("   Provided: [a]");
    println!("   Missing: {:?}", missing);

    // =========================================================================
    // Part 3: Batch Evaluation
    // =========================================================================
    println!("\n┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Part 3: Batch Evaluation - Process multiple inputs at once     │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let pricing_rules = vec![
        Rule::from_json_logic("pricing", "calc", json!({
            "if": [
                {">=": [{"var": "quantity"}, 100]}, 0.80,  // 20% discount
                {">=": [{"var": "quantity"}, 50]}, 0.90,   // 10% discount
                {">=": [{"var": "quantity"}, 20]}, 0.95,   // 5% discount
                1.0  // No discount
            ]
        }))
        .with_inputs(["quantity"])
        .with_outputs(["discount_factor"])
        .with_order(0),

        Rule::from_json_logic("pricing", "calc", json!({
            "*": [{"var": "unit_price"}, {"var": "quantity"}, {"var": "discount_factor"}]
        }))
        .with_inputs(["unit_price", "quantity", "discount_factor"])
        .with_outputs(["total_price"])
        .with_order(1),
    ];

    println!("Pricing rules: discount based on quantity, then calculate total");
    println!("\nBatch evaluation with 5 orders:");

    let orders = vec![
        json!({"unit_price": 10.0, "quantity": 10}),
        json!({"unit_price": 10.0, "quantity": 25}),
        json!({"unit_price": 10.0, "quantity": 55}),
        json!({"unit_price": 10.0, "quantity": 100}),
        json!({"unit_price": 10.0, "quantity": 200}),
    ];

    let results = service.evaluate_batch(
        &ProductId::new("pricing"),
        &pricing_rules,
        &orders,
    )?;

    println!("   ┌──────────┬──────────────────┬─────────────┐");
    println!("   │ Quantity │ Discount Factor  │ Total Price │");
    println!("   ├──────────┼──────────────────┼─────────────┤");
    for (order, result) in orders.iter().zip(results.iter()) {
        println!("   │ {:>8} │ {:>16} │ ${:>9.2} │",
            order["quantity"],
            result["discount_factor"],
            result["total_price"].as_f64().unwrap_or(0.0)
        );
    }
    println!("   └──────────┴──────────────────┴─────────────┘");

    // =========================================================================
    // Part 4: Trading Strategy Example
    // =========================================================================
    println!("\n┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Part 4: Complete Trading Strategy Example                      │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    // Build a complete momentum trading strategy
    let strategy_rules = vec![
        // Rule 1: RSI Signal
        SignalRuleBuilder::new("momentum", "rsi_signal")
            .rsi_signal("rsi", 30.0, 70.0),

        // Rule 2: Price vs SMA Signal
        SignalRuleBuilder::new("momentum", "trend_signal")
            .price_vs_ma("price", "sma_50", 0.02),

        // Rule 3: Combined entry signal
        Rule::from_json_logic("momentum", "signal", json!({
            "if": [
                {"and": [
                    {"==": [{"var": "rsi_signal"}, "BUY"]},
                    {"==": [{"var": "trend_signal"}, "BUY"]}
                ]}, "STRONG_BUY",
                {"or": [
                    {"==": [{"var": "rsi_signal"}, "BUY"]},
                    {"==": [{"var": "trend_signal"}, "BUY"]}
                ]}, "BUY",
                {"and": [
                    {"==": [{"var": "rsi_signal"}, "SELL"]},
                    {"==": [{"var": "trend_signal"}, "SELL"]}
                ]}, "STRONG_SELL",
                {"or": [
                    {"==": [{"var": "rsi_signal"}, "SELL"]},
                    {"==": [{"var": "trend_signal"}, "SELL"]}
                ]}, "SELL",
                "HOLD"
            ]
        }))
        .with_inputs(["rsi_signal", "trend_signal"])
        .with_outputs(["entry_signal"])
        .with_description("Combined entry signal"),

        // Rule 4: Position size based on signal strength
        Rule::from_json_logic("momentum", "calc", json!({
            "if": [
                {"==": [{"var": "entry_signal"}, "STRONG_BUY"]},
                {"*": [{"var": "capital"}, 0.10]},  // 10% position
                {"==": [{"var": "entry_signal"}, "BUY"]},
                {"*": [{"var": "capital"}, 0.05]},  // 5% position
                {"==": [{"var": "entry_signal"}, "STRONG_SELL"]},
                {"*": [{"var": "capital"}, -0.10]}, // -10% (short)
                {"==": [{"var": "entry_signal"}, "SELL"]},
                {"*": [{"var": "capital"}, -0.05]}, // -5% (short)
                0  // No position
            ]
        }))
        .with_inputs(["entry_signal", "capital"])
        .with_outputs(["position_value"])
        .with_description("Calculate position size"),
    ];

    // Validate before execution
    println!("Validating strategy rules...");
    let validation = service.validate(&strategy_rules);
    if !validation.valid {
        println!("Strategy validation failed!");
        for err in &validation.errors {
            println!("  Error: {}", err.message);
        }
        return Ok(());
    }
    println!("Strategy valid! {} execution levels\n",
        validation.execution_levels.as_ref().map(|l| l.len()).unwrap_or(0));

    // Simulate market data
    let market_scenarios = vec![
        ("Oversold + Uptrend", json!({
            "rsi": 25.0, "price": 105.0, "sma_50": 100.0, "capital": 100000.0
        })),
        ("Neutral", json!({
            "rsi": 50.0, "price": 100.0, "sma_50": 100.0, "capital": 100000.0
        })),
        ("Overbought + Downtrend", json!({
            "rsi": 75.0, "price": 95.0, "sma_50": 100.0, "capital": 100000.0
        })),
        ("Strong Buy (RSI + Trend)", json!({
            "rsi": 28.0, "price": 108.0, "sma_50": 100.0, "capital": 100000.0
        })),
    ];

    println!("Strategy Results:");
    println!("┌─────────────────────────┬─────────┬─────────────┬───────────────┬──────────────┐");
    println!("│ Scenario                │ RSI     │ Price/SMA50 │ Signal        │ Position     │");
    println!("├─────────────────────────┼─────────┼─────────────┼───────────────┼──────────────┤");

    for (name, data) in &market_scenarios {
        let result = service.evaluate_with_stats(
            &ProductId::new("momentum"),
            &strategy_rules,
            data,
        )?;

        println!("│ {:23} │ {:>7.1} │ {:>11.1} │ {:13} │ ${:>10.0} │",
            name,
            data["rsi"].as_f64().unwrap(),
            data["price"].as_f64().unwrap(),
            result.output["entry_signal"].as_str().unwrap_or("?"),
            result.output["position_value"].as_f64().unwrap_or(0.0)
        );
    }
    println!("└─────────────────────────┴─────────┴─────────────┴───────────────┴──────────────┘");

    // =========================================================================
    // Part 5: Execution Statistics
    // =========================================================================
    println!("\n┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Part 5: Execution Statistics                                   │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let stats = service.stats();
    println!("Executor Statistics:");
    println!("  • Compiled rules in cache: {}", stats.compiled_rules);
    println!("  • Total AST nodes: {}", stats.total_ast_nodes);
    println!("  • Rules with bytecode: {}", stats.rules_with_bytecode);

    // Run a timed evaluation
    let result = service.evaluate_with_stats(
        &ProductId::new("momentum"),
        &strategy_rules,
        &market_scenarios[0].1,
    )?;
    println!("\nLast evaluation:");
    println!("  • Rules executed: {}", result.rules_executed);
    println!("  • Execution time: {} µs", result.execution_time_us);

    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("                    Example Complete!                              ");
    println!("═══════════════════════════════════════════════════════════════════");

    Ok(())
}
