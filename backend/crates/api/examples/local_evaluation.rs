//! Example: Local Rule Evaluation (No Server)
//!
//! This example shows how to use Product-FARM as a library without the gRPC server.
//! Useful for embedding the rule engine directly in your application.
//!
//! Run with: cargo run --example local_evaluation

use product_farm_core::{ProductId, Rule};
use product_farm_api::ProductFarmService;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Product-FARM Local Evaluation Example ===\n");

    // Create the service (no server needed)
    let mut service = ProductFarmService::new();

    // =========================================================================
    // Example 1: Simple Calculation
    // =========================================================================
    println!("--- Example 1: Simple Calculation ---");

    let rules = vec![
        Rule::from_json_logic("calc", "math", json!({
            "*": [{"var": "x"}, 2]
        }))
        .with_inputs(["x"])
        .with_outputs(["doubled"]),
    ];

    let result = service.evaluate(
        &ProductId::new("calc"),
        &rules,
        &json!({"x": 21}),
    )?;

    println!("Input: x = 21");
    println!("Output: doubled = {}", result["doubled"]);
    println!();

    // =========================================================================
    // Example 2: Chained Calculations
    // =========================================================================
    println!("--- Example 2: Chained Calculations ---");

    let rules = vec![
        Rule::from_json_logic("chain", "calc", json!({
            "+": [{"var": "input"}, 10]
        }))
        .with_inputs(["input"])
        .with_outputs(["step1"])
        .with_order(0),

        Rule::from_json_logic("chain", "calc", json!({
            "*": [{"var": "step1"}, 2]
        }))
        .with_inputs(["step1"])
        .with_outputs(["step2"])
        .with_order(1),

        Rule::from_json_logic("chain", "calc", json!({
            "-": [{"var": "step2"}, 5]
        }))
        .with_inputs(["step2"])
        .with_outputs(["final"])
        .with_order(2),
    ];

    let result = service.evaluate(
        &ProductId::new("chain"),
        &rules,
        &json!({"input": 5}),
    )?;

    println!("Input: input = 5");
    println!("Chain: input + 10 = step1 ({}) -> step1 * 2 = step2 ({}) -> step2 - 5 = final ({})",
        result["step1"], result["step2"], result["final"]);
    println!();

    // =========================================================================
    // Example 3: Conditional Logic (Insurance Premium)
    // =========================================================================
    println!("--- Example 3: Conditional Logic (Insurance Premium) ---");

    let rules = vec![
        // Rule 1: Determine age category
        Rule::from_json_logic("insurance", "classify", json!({
            "if": [
                {">": [{"var": "age"}, 60]}, "senior",
                {">": [{"var": "age"}, 25]}, "adult",
                "young"
            ]
        }))
        .with_inputs(["age"])
        .with_outputs(["age_category"])
        .with_order(0),

        // Rule 2: Calculate age factor
        Rule::from_json_logic("insurance", "calc", json!({
            "if": [
                {"==": [{"var": "age_category"}, "senior"]}, 1.5,
                {"==": [{"var": "age_category"}, "young"]}, 1.2,
                1.0
            ]
        }))
        .with_inputs(["age_category"])
        .with_outputs(["age_factor"])
        .with_order(1),

        // Rule 3: Calculate final premium
        Rule::from_json_logic("insurance", "calc", json!({
            "*": [{"var": "base_premium"}, {"var": "age_factor"}]
        }))
        .with_inputs(["base_premium", "age_factor"])
        .with_outputs(["final_premium"])
        .with_order(2),
    ];

    // Test with different ages
    for age in [22, 35, 65] {
        let result = service.evaluate(
            &ProductId::new("insurance"),
            &rules,
            &json!({"age": age, "base_premium": 100}),
        )?;

        println!("Age {}: category={}, factor={}, premium=${}",
            age,
            result["age_category"],
            result["age_factor"],
            result["final_premium"]
        );
    }
    println!();

    // =========================================================================
    // Example 4: Array Operations
    // =========================================================================
    println!("--- Example 4: Array Operations ---");

    let rules = vec![
        // Sum of array
        Rule::from_json_logic("arrays", "calc", json!({
            "reduce": [
                {"var": "numbers"},
                {"+": [{"var": "accumulator"}, {"var": "current"}]},
                0
            ]
        }))
        .with_inputs(["numbers"])
        .with_outputs(["total"]),

        // Max value in array
        Rule::from_json_logic("arrays", "calc", json!({
            "reduce": [
                {"var": "numbers"},
                {"if": [
                    {">": [{"var": "current"}, {"var": "accumulator"}]},
                    {"var": "current"},
                    {"var": "accumulator"}
                ]},
                0
            ]
        }))
        .with_inputs(["numbers"])
        .with_outputs(["max_value"]),
    ];

    let result = service.evaluate(
        &ProductId::new("arrays"),
        &rules,
        &json!({
            "numbers": [1, 5, 10, 15, 20, 25]
        }),
    )?;

    println!("Input: [1, 5, 10, 15, 20, 25]");
    println!("Total: {}", result["total"]);
    println!("Max value: {}", result["max_value"]);
    println!();

    // =========================================================================
    // Example 5: String Operations
    // =========================================================================
    println!("--- Example 5: String and Logic Operations ---");

    let rules = vec![
        // Check if all conditions are met
        Rule::from_json_logic("validation", "check", json!({
            "and": [
                {">=": [{"var": "age"}, 18]},
                {"in": [{"var": "country"}, ["US", "UK", "CA"]]},
                {"!": [{"var": "is_blocked"}]}
            ]
        }))
        .with_inputs(["age", "country", "is_blocked"])
        .with_outputs(["is_eligible"]),
    ];

    let result = service.evaluate(
        &ProductId::new("validation"),
        &rules,
        &json!({
            "age": 25,
            "country": "US",
            "is_blocked": false
        }),
    )?;

    println!("User: age=25, country=US, is_blocked=false");
    println!("Is eligible: {}", result["is_eligible"]);

    let result = service.evaluate(
        &ProductId::new("validation"),
        &rules,
        &json!({
            "age": 16,
            "country": "US",
            "is_blocked": false
        }),
    )?;

    println!("User: age=16, country=US, is_blocked=false");
    println!("Is eligible: {}", result["is_eligible"]);
    println!();

    // Print statistics
    let stats = service.stats();
    println!("=== Executor Statistics ===");
    println!("Compiled rules: {}", stats.compiled_rules);
    println!("Total AST nodes: {}", stats.total_ast_nodes);
    println!("Rules with bytecode: {}", stats.rules_with_bytecode);

    Ok(())
}
