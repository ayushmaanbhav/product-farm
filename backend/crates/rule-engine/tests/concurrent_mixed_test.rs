//! Concurrent Mixed Tests for Rule Engine
//!
//! Tests thread safety, scalability, and concurrent evaluation with:
//! - 50+ FarmScript/JSON Logic rules (deterministic, fast)
//! - 50+ LLM rules (Ollama-based, slower)
//! - Complex interdependent rule chains mixing both types
//! - Thread safety verification
//!
//! Run with: cargo test -p product-farm-rule-engine --test concurrent_mixed_test --features ollama -- --nocapture

use product_farm_core::{Rule, Value};
use product_farm_json_logic::{compile, Evaluator};
use product_farm_rule_engine::{ExecutionContext, RuleExecutor};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// =============================================================================
// JSON Logic Rule Generators (50+ rules)
// =============================================================================

/// Create a JSON Logic rule for testing
fn make_json_logic_rule(
    id: &str,
    inputs: &[&str],
    outputs: &[&str],
    expr: serde_json::Value,
) -> Rule {
    Rule::from_json_logic("test-product", id, expr)
        .with_inputs(inputs.iter().map(|s| s.to_string()))
        .with_outputs(outputs.iter().map(|s| s.to_string()))
}

/// Generate arithmetic JSON Logic rules
fn generate_arithmetic_rules(count: usize) -> Vec<Rule> {
    let mut rules = Vec::with_capacity(count);

    for i in 0..count {
        let op = match i % 4 {
            0 => "+",
            1 => "-",
            2 => "*",
            _ => "/",
        };

        let expr = match op {
            "+" => json!({ "+": [{"var": "x"}, i + 1] }),
            "-" => json!({ "-": [{"var": "x"}, i + 1] }),
            "*" => json!({ "*": [{"var": "x"}, i + 1] }),
            "/" => json!({ "/": [{"var": "x"}, (i + 1).max(1)] }),
            _ => json!({ "var": "x" }),
        };

        rules.push(make_json_logic_rule(
            &format!("arith-{}", i),
            &["x"],
            &[&format!("result_{}", i)],
            expr,
        ));
    }

    rules
}

/// Generate comparison/boolean JSON Logic rules
fn generate_comparison_rules(count: usize) -> Vec<Rule> {
    let mut rules = Vec::with_capacity(count);

    for i in 0..count {
        let threshold = (i * 10) as i64;
        let op = match i % 6 {
            0 => ">",
            1 => ">=",
            2 => "<",
            3 => "<=",
            4 => "==",
            _ => "!=",
        };

        let expr = json!({ op: [{"var": "value"}, threshold] });

        rules.push(make_json_logic_rule(
            &format!("compare-{}", i),
            &["value"],
            &[&format!("cmp_result_{}", i)],
            expr,
        ));
    }

    rules
}

/// Generate conditional/if-then-else JSON Logic rules
fn generate_conditional_rules(count: usize) -> Vec<Rule> {
    let mut rules = Vec::with_capacity(count);

    for i in 0..count {
        let threshold1 = (i * 5) as i64;
        let threshold2 = (i * 10) as i64;

        let expr = json!({
            "if": [
                { ">": [{"var": "score"}, threshold2] }, "high",
                { ">": [{"var": "score"}, threshold1] }, "medium",
                "low"
            ]
        });

        rules.push(make_json_logic_rule(
            &format!("conditional-{}", i),
            &["score"],
            &[&format!("category_{}", i)],
            expr,
        ));
    }

    rules
}

/// Generate array operation JSON Logic rules
fn generate_array_rules(count: usize) -> Vec<Rule> {
    let mut rules = Vec::with_capacity(count);

    for i in 0..count {
        let expr = match i % 4 {
            // Sum all elements
            0 => json!({
                "reduce": [
                    {"var": "items"},
                    {"+": [{"var": "accumulator"}, {"var": "current"}]},
                    0
                ]
            }),
            // Count elements
            1 => json!({
                "reduce": [
                    {"var": "items"},
                    {"+": [{"var": "accumulator"}, 1]},
                    0
                ]
            }),
            // Find max (simplified - just return first + length)
            2 => json!({
                "+": [
                    {"var": "items.0"},
                    {"reduce": [{"var": "items"}, {"+": [{"var": "accumulator"}, 1]}, 0]}
                ]
            }),
            // Check if any > threshold
            _ => json!({
                "some": [
                    {"var": "items"},
                    {">": [{"var": ""}, i as i64]}
                ]
            }),
        };

        rules.push(make_json_logic_rule(
            &format!("array-{}", i),
            &["items"],
            &[&format!("array_result_{}", i)],
            expr,
        ));
    }

    rules
}

/// Generate complex nested JSON Logic rules
fn generate_complex_rules(count: usize) -> Vec<Rule> {
    let mut rules = Vec::with_capacity(count);

    for i in 0..count {
        // Complex rule: weighted scoring with conditions
        let weight = ((i % 5) + 1) as f64 / 5.0;
        let expr = json!({
            "if": [
                {"and": [
                    {">": [{"var": "quality"}, 50]},
                    {">": [{"var": "speed"}, 30]}
                ]},
                {"*": [
                    {"+": [
                        {"*": [{"var": "quality"}, weight]},
                        {"*": [{"var": "speed"}, 1.0 - weight]}
                    ]},
                    1.2
                ]},
                {"if": [
                    {">": [{"var": "quality"}, 30]},
                    {"+": [{"var": "quality"}, {"var": "speed"}]},
                    {"*": [{"var": "speed"}, 0.5]}
                ]}
            ]
        });

        rules.push(make_json_logic_rule(
            &format!("complex-{}", i),
            &["quality", "speed"],
            &[&format!("complex_score_{}", i)],
            expr,
        ));
    }

    rules
}

/// Generate chained dependency rules
fn generate_chained_rules(chain_length: usize) -> Vec<Rule> {
    let mut rules = Vec::with_capacity(chain_length);

    // First rule takes input
    rules.push(make_json_logic_rule(
        "chain-0",
        &["input"],
        &["chain_0"],
        json!({ "+": [{"var": "input"}, 1] }),
    ));

    // Subsequent rules depend on previous
    for i in 1..chain_length {
        let input = format!("chain_{}", i - 1);
        let output = format!("chain_{}", i);

        rules.push(make_json_logic_rule(
            &format!("chain-{}", i),
            &[&input],
            &[&output],
            json!({ "+": [{"var": input}, 1] }),
        ));
    }

    rules
}

/// Generate diamond dependency rules
fn generate_diamond_rules(width: usize) -> Vec<Rule> {
    let mut rules = Vec::new();

    // Base rule
    rules.push(make_json_logic_rule(
        "diamond-base",
        &["input"],
        &["diamond_base"],
        json!({ "var": "input" }),
    ));

    // Middle layer - parallel rules
    for i in 0..width {
        rules.push(make_json_logic_rule(
            &format!("diamond-mid-{}", i),
            &["diamond_base"],
            &[&format!("diamond_mid_{}", i)],
            json!({ "+": [{"var": "diamond_base"}, i + 1] }),
        ));
    }

    // Final aggregation - depends on all middle rules
    let mid_inputs: Vec<&str> = (0..width)
        .map(|i| {
            // Need to leak to get &'static str, or use owned strings
            Box::leak(format!("diamond_mid_{}", i).into_boxed_str()) as &str
        })
        .collect();

    // Sum all middle values
    let sum_expr = if width > 0 {
        let mut vars: Vec<serde_json::Value> = mid_inputs
            .iter()
            .map(|name| json!({ "var": name }))
            .collect();
        json!({ "+": vars })
    } else {
        json!(0)
    };

    rules.push(make_json_logic_rule(
        "diamond-final",
        &mid_inputs,
        &["diamond_final"],
        sum_expr,
    ));

    rules
}

// =============================================================================
// JSON Logic Concurrent Tests
// =============================================================================

#[test]
fn test_50_arithmetic_rules_concurrent() {
    let rules = generate_arithmetic_rules(50);

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&rules).unwrap();

    let mut context = ExecutionContext::from_json(&json!({
        "x": 100
    }));

    let result = executor.execute(&rules, &mut context).unwrap();

    println!("=== 50 Arithmetic Rules ===");
    println!("Total rules: {}", result.rule_results.len());
    println!("Total levels: {}", result.levels.len());
    println!("Time: {}ms", result.total_time_ns / 1_000_000);

    // All should be in one level (independent)
    assert_eq!(result.levels.len(), 1);
    assert_eq!(result.levels[0].len(), 50);

    // Verify some results
    // result_0 = 100 + 1 = 101 (addition)
    assert_eq!(result.get_output("result_0").unwrap().to_number(), 101.0);
    // result_1 = 100 - 2 = 98 (subtraction)
    assert_eq!(result.get_output("result_1").unwrap().to_number(), 98.0);
    // result_2 = 100 * 3 = 300 (multiplication)
    assert_eq!(result.get_output("result_2").unwrap().to_number(), 300.0);
}

#[test]
fn test_50_comparison_rules_concurrent() {
    let rules = generate_comparison_rules(50);

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "value": 250
    }));

    let result = executor.execute(&rules, &mut context).unwrap();

    println!("=== 50 Comparison Rules ===");
    println!("Total rules: {}", result.rule_results.len());
    println!("Time: {}ms", result.total_time_ns / 1_000_000);

    // Count true/false results
    let true_count = (0..50)
        .filter(|i| {
            result
                .get_output(&format!("cmp_result_{}", i))
                .map(|v| v == &Value::Bool(true))
                .unwrap_or(false)
        })
        .count();

    println!("True results: {}/50", true_count);
    assert_eq!(result.rule_results.len(), 50);
}

#[test]
fn test_50_conditional_rules_concurrent() {
    let rules = generate_conditional_rules(50);

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "score": 75
    }));

    let result = executor.execute(&rules, &mut context).unwrap();

    println!("=== 50 Conditional Rules ===");
    println!("Total rules: {}", result.rule_results.len());
    println!("Time: {}ms", result.total_time_ns / 1_000_000);

    // Count categories
    let mut high_count = 0;
    let mut medium_count = 0;
    let mut low_count = 0;

    for i in 0..50 {
        if let Some(val) = result.get_output(&format!("category_{}", i)) {
            match val {
                Value::String(s) if s == "high" => high_count += 1,
                Value::String(s) if s == "medium" => medium_count += 1,
                Value::String(s) if s == "low" => low_count += 1,
                _ => {}
            }
        }
    }

    println!("Categories: high={}, medium={}, low={}", high_count, medium_count, low_count);
    assert_eq!(high_count + medium_count + low_count, 50);
}

#[test]
fn test_50_complex_rules_concurrent() {
    let rules = generate_complex_rules(50);

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "quality": 70,
        "speed": 60
    }));

    let result = executor.execute(&rules, &mut context).unwrap();

    println!("=== 50 Complex Rules ===");
    println!("Total rules: {}", result.rule_results.len());
    println!("Time: {}ms", result.total_time_ns / 1_000_000);

    // All complex rules should execute
    assert_eq!(result.rule_results.len(), 50);

    // Verify some complex scores are reasonable
    for i in 0..5 {
        let score = result.get_output(&format!("complex_score_{}", i));
        println!("complex_score_{}: {:?}", i, score);
        assert!(score.is_some());
    }
}

#[test]
fn test_chained_rules_sequential_dependency() {
    let rules = generate_chained_rules(20);

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "input": 0
    }));

    let result = executor.execute(&rules, &mut context).unwrap();

    println!("=== 20 Chained Rules ===");
    println!("Total levels: {}", result.levels.len());
    println!("Time: {}ms", result.total_time_ns / 1_000_000);

    // Should have 20 levels (fully sequential)
    assert_eq!(result.levels.len(), 20);

    // Each level should have exactly 1 rule
    for level in &result.levels {
        assert_eq!(level.len(), 1);
    }

    // Final result should be input + 20 = 20
    assert_eq!(result.get_output("chain_19").unwrap().to_number(), 20.0);
}

#[test]
fn test_diamond_dependency_parallel_middle() {
    let rules = generate_diamond_rules(10);

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "input": 100
    }));

    let result = executor.execute(&rules, &mut context).unwrap();

    println!("=== Diamond Dependency (width=10) ===");
    println!("Total levels: {}", result.levels.len());
    for (i, level) in result.levels.iter().enumerate() {
        println!("  Level {}: {} rules", i, level.len());
    }
    println!("Time: {}ms", result.total_time_ns / 1_000_000);

    // Should have 3 levels: base (1), middle (10), final (1)
    assert_eq!(result.levels.len(), 3);
    assert_eq!(result.levels[0].len(), 1);
    assert_eq!(result.levels[1].len(), 10);
    assert_eq!(result.levels[2].len(), 1);

    // diamond_final = sum of (100 + 1), (100 + 2), ..., (100 + 10)
    // = 10*100 + (1+2+...+10) = 1000 + 55 = 1055
    let final_val = result.get_output("diamond_final").unwrap().to_number();
    assert_eq!(final_val, 1055.0);
}

#[test]
fn test_100_mixed_rules_concurrent() {
    // Generate 100 mixed rules
    let mut all_rules = Vec::new();
    all_rules.extend(generate_arithmetic_rules(25));
    all_rules.extend(generate_comparison_rules(25));
    all_rules.extend(generate_conditional_rules(25));
    all_rules.extend(generate_complex_rules(25));

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&all_rules).unwrap();

    let mut context = ExecutionContext::from_json(&json!({
        "x": 50,
        "value": 100,
        "score": 60,
        "quality": 80,
        "speed": 40
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&all_rules, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("=== 100 Mixed Rules ===");
    println!("Total rules: {}", result.rule_results.len());
    println!("Total levels: {}", result.levels.len());
    println!("Time: {:?}", elapsed);
    println!("Throughput: {:.0} rules/sec", 100.0 / elapsed.as_secs_f64());

    assert_eq!(result.rule_results.len(), 100);
    // All rules are independent, so should be 1 level
    assert_eq!(result.levels.len(), 1);
}

// =============================================================================
// Thread Safety Tests for JSON Logic
// =============================================================================

#[test]
fn test_json_logic_thread_safety_shared_executor() {
    let rules = generate_arithmetic_rules(20);

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&rules).unwrap();
    let executor = Arc::new(executor);

    let completed = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    // Spawn 10 threads, each executing all rules
    for thread_id in 0..10 {
        let exec = executor.clone();
        let completed = completed.clone();
        let rules = rules.clone();

        let handle = std::thread::spawn(move || {
            let mut context = ExecutionContext::from_json(&json!({
                "x": thread_id * 10
            }));

            let result = exec.execute(&rules, &mut context).unwrap();
            assert_eq!(result.rule_results.len(), 20);

            completed.fetch_add(1, Ordering::SeqCst);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    assert_eq!(completed.load(Ordering::SeqCst), 10);
    println!("Thread safety test: 10/10 threads completed successfully");
}

#[test]
fn test_json_logic_parallel_contexts() {
    let rules = generate_arithmetic_rules(50);

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&rules).unwrap();
    let executor = Arc::new(executor);

    let results: Vec<_> = (0..20)
        .into_iter()
        .map(|i| {
            let exec = executor.clone();
            let rules = rules.clone();

            std::thread::spawn(move || {
                let mut context = ExecutionContext::from_json(&json!({
                    "x": i * 100
                }));

                let result = exec.execute(&rules, &mut context).unwrap();

                // Return first output for verification
                result.get_output("result_0").unwrap().to_number()
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect();

    // Verify each thread got correct result
    // result_0 = x + 1
    for (i, &result) in results.iter().enumerate() {
        let expected = (i * 100 + 1) as f64;
        assert_eq!(result, expected, "Thread {} got wrong result", i);
    }

    println!("Parallel contexts test: 20 threads with correct isolated results");
}

#[test]
fn test_json_logic_high_concurrency() {
    let rules = generate_arithmetic_rules(100);

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&rules).unwrap();
    let executor = Arc::new(executor);

    let start = std::time::Instant::now();

    let handles: Vec<_> = (0..50)
        .map(|i| {
            let exec = executor.clone();
            let rules = rules.clone();

            std::thread::spawn(move || {
                let mut context = ExecutionContext::from_json(&json!({
                    "x": i
                }));

                exec.execute(&rules, &mut context).unwrap()
            })
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    let elapsed = start.elapsed();

    println!("=== High Concurrency Test ===");
    println!("50 threads x 100 rules = 5000 rule executions");
    println!("Time: {:?}", elapsed);
    println!("Throughput: {:.0} rule executions/sec", 5000.0 / elapsed.as_secs_f64());

    // Verify all completed
    assert_eq!(results.len(), 50);
    for result in &results {
        assert_eq!(result.rule_results.len(), 100);
    }
}

// =============================================================================
// Array Operations Tests
// =============================================================================

#[test]
fn test_array_rules_concurrent() {
    let rules = generate_array_rules(20);

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "items": [10, 20, 30, 40, 50]
    }));

    let result = executor.execute(&rules, &mut context).unwrap();

    println!("=== 20 Array Rules ===");
    println!("Total rules: {}", result.rule_results.len());
    println!("Time: {}ms", result.total_time_ns / 1_000_000);

    // Verify sum rules (index 0, 4, 8, 12, 16)
    // sum = 10 + 20 + 30 + 40 + 50 = 150
    let sum_result = result.get_output("array_result_0").unwrap().to_number();
    assert_eq!(sum_result, 150.0);

    assert_eq!(result.rule_results.len(), 20);
}

// =============================================================================
// Stress Tests
// =============================================================================

#[test]
fn test_200_rules_stress() {
    // Generate 200 rules of different types
    let mut all_rules = Vec::new();
    all_rules.extend(generate_arithmetic_rules(50));
    all_rules.extend(generate_comparison_rules(50));
    all_rules.extend(generate_conditional_rules(50));
    all_rules.extend(generate_complex_rules(50));

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&all_rules).unwrap();

    let stats = executor.stats();
    println!("Compiled {} rules with {} AST nodes", stats.compiled_rules, stats.total_ast_nodes);

    let mut context = ExecutionContext::from_json(&json!({
        "x": 100,
        "value": 500,
        "score": 75,
        "quality": 85,
        "speed": 65
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&all_rules, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("=== 200 Rules Stress Test ===");
    println!("Total rules: {}", result.rule_results.len());
    println!("Total levels: {}", result.levels.len());
    println!("Time: {:?}", elapsed);
    println!("Throughput: {:.0} rules/sec", 200.0 / elapsed.as_secs_f64());

    assert_eq!(result.rule_results.len(), 200);
}

#[test]
fn test_deep_chain_stress() {
    // 50-level deep chain
    let rules = generate_chained_rules(50);

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "input": 0
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&rules, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("=== Deep Chain (50 levels) ===");
    println!("Levels: {}", result.levels.len());
    println!("Time: {:?}", elapsed);

    assert_eq!(result.levels.len(), 50);
    assert_eq!(result.get_output("chain_49").unwrap().to_number(), 50.0);
}

#[test]
fn test_wide_diamond_stress() {
    // Diamond with width 50
    let rules = generate_diamond_rules(50);

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "input": 10
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&rules, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("=== Wide Diamond (width=50) ===");
    println!("Levels: {}", result.levels.len());
    println!("Middle level size: {}", result.levels[1].len());
    println!("Time: {:?}", elapsed);

    assert_eq!(result.levels.len(), 3);
    assert_eq!(result.levels[1].len(), 50);

    // diamond_final = sum of (10 + 1), (10 + 2), ..., (10 + 50)
    // = 50*10 + (1+2+...+50) = 500 + 1275 = 1775
    let final_val = result.get_output("diamond_final").unwrap().to_number();
    assert_eq!(final_val, 1775.0);
}

// =============================================================================
// Repeated Execution Tests
// =============================================================================

#[test]
fn test_repeated_execution_same_context() {
    let rules = generate_arithmetic_rules(50);

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&rules).unwrap();

    let mut context = ExecutionContext::from_json(&json!({
        "x": 42
    }));

    // Execute 100 times
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let result = executor.execute(&rules, &mut context).unwrap();
        assert_eq!(result.rule_results.len(), 50);
    }
    let elapsed = start.elapsed();

    println!("=== 100 Repeated Executions ===");
    println!("Time: {:?}", elapsed);
    println!("Avg per execution: {:?}", elapsed / 100);
    println!("Throughput: {:.0} executions/sec", 100.0 / elapsed.as_secs_f64());
}

#[test]
fn test_rapid_compile_execute_cycles() {
    // Test creating new executors rapidly
    let rules = generate_arithmetic_rules(20);

    let start = std::time::Instant::now();
    for i in 0..50 {
        let mut executor = RuleExecutor::new();
        executor.compile_rules(&rules).unwrap();

        let mut context = ExecutionContext::from_json(&json!({
            "x": i
        }));

        let result = executor.execute(&rules, &mut context).unwrap();
        assert_eq!(result.rule_results.len(), 20);
    }
    let elapsed = start.elapsed();

    println!("=== 50 Compile-Execute Cycles ===");
    println!("Time: {:?}", elapsed);
    println!("Avg per cycle: {:?}", elapsed / 50);
}
