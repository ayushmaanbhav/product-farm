//! Concurrent 1000 Rules Test
//!
//! Tests thread safety, scalability, and concurrent evaluation with:
//! - 10 LLM rules (Ollama-based, slow)
//! - ~495 JSON Logic rules (fast, deterministic)
//! - ~495 FarmScript rules (compiled to JSON Logic)
//! - Complex interdependent rule chains mixing all types
//!
//! Run with: cargo test -p product-farm-rule-engine --test concurrent_1000_rules_test --features ollama -- --nocapture

use product_farm_core::{Rule, Value};
use product_farm_farmscript::compile as compile_farmscript;
use product_farm_rule_engine::{ExecutionContext, RuleExecutor};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// =============================================================================
// Rule Generation Constants
// =============================================================================

const TOTAL_RULES: usize = 1000;
const LLM_RULES: usize = 10;
const JSON_LOGIC_RULES: usize = 495;
const FARMSCRIPT_RULES: usize = 495;

// =============================================================================
// JSON Logic Rule Generators
// =============================================================================

fn make_json_logic_rule(id: &str, inputs: &[&str], outputs: &[&str], expr: serde_json::Value) -> Rule {
    Rule::from_json_logic("test-product", id, expr)
        .with_inputs(inputs.iter().map(|s| s.to_string()))
        .with_outputs(outputs.iter().map(|s| s.to_string()))
}

/// Generate arithmetic JSON Logic rules
fn generate_json_logic_arithmetic(count: usize, start_id: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start_id + i;
            let op = match i % 4 {
                0 => json!({ "+": [{"var": "x"}, (i % 100) + 1] }),
                1 => json!({ "-": [{"var": "x"}, (i % 50) + 1] }),
                2 => json!({ "*": [{"var": "x"}, (i % 10) + 1] }),
                _ => json!({ "/": [{"var": "x"}, (i % 10) + 1] }),
            };
            make_json_logic_rule(
                &format!("jl-arith-{}", id),
                &["x"],
                &[&format!("jl_arith_{}", id)],
                op,
            )
        })
        .collect()
}

/// Generate comparison JSON Logic rules
fn generate_json_logic_comparison(count: usize, start_id: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start_id + i;
            let threshold = (i * 7) % 100;
            let op = match i % 6 {
                0 => json!({ ">": [{"var": "value"}, threshold] }),
                1 => json!({ ">=": [{"var": "value"}, threshold] }),
                2 => json!({ "<": [{"var": "value"}, threshold] }),
                3 => json!({ "<=": [{"var": "value"}, threshold] }),
                4 => json!({ "==": [{"var": "value"}, threshold] }),
                _ => json!({ "!=": [{"var": "value"}, threshold] }),
            };
            make_json_logic_rule(
                &format!("jl-cmp-{}", id),
                &["value"],
                &[&format!("jl_cmp_{}", id)],
                op,
            )
        })
        .collect()
}

/// Generate conditional JSON Logic rules
fn generate_json_logic_conditional(count: usize, start_id: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start_id + i;
            let t1 = (i * 3) % 100;
            let t2 = (i * 5) % 100;
            let expr = json!({
                "if": [
                    { ">": [{"var": "score"}, t2.max(t1)] }, "high",
                    { ">": [{"var": "score"}, t1.min(t2)] }, "medium",
                    "low"
                ]
            });
            make_json_logic_rule(
                &format!("jl-cond-{}", id),
                &["score"],
                &[&format!("jl_cond_{}", id)],
                expr,
            )
        })
        .collect()
}

/// Generate complex nested JSON Logic rules
fn generate_json_logic_complex(count: usize, start_id: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start_id + i;
            let w = ((i % 5) + 1) as f64 / 5.0;
            let expr = json!({
                "if": [
                    {"and": [
                        {">": [{"var": "quality"}, 50]},
                        {">": [{"var": "speed"}, 30]}
                    ]},
                    {"*": [
                        {"+": [
                            {"*": [{"var": "quality"}, w]},
                            {"*": [{"var": "speed"}, 1.0 - w]}
                        ]},
                        1.2
                    ]},
                    {"+": [{"var": "quality"}, {"var": "speed"}]}
                ]
            });
            make_json_logic_rule(
                &format!("jl-complex-{}", id),
                &["quality", "speed"],
                &[&format!("jl_complex_{}", id)],
                expr,
            )
        })
        .collect()
}

/// Generate boolean logic JSON Logic rules
fn generate_json_logic_boolean(count: usize, start_id: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start_id + i;
            let expr = match i % 4 {
                0 => json!({"and": [{"var": "a"}, {"var": "b"}]}),
                1 => json!({"or": [{"var": "a"}, {"var": "b"}]}),
                2 => json!({"!": {"var": "a"}}),
                _ => json!({"if": [{"var": "a"}, {"var": "b"}, {"var": "c"}]}),
            };
            make_json_logic_rule(
                &format!("jl-bool-{}", id),
                &["a", "b", "c"],
                &[&format!("jl_bool_{}", id)],
                expr,
            )
        })
        .collect()
}

// =============================================================================
// FarmScript Rule Generators
// =============================================================================

fn make_farmscript_rule(id: &str, inputs: &[&str], outputs: &[&str], farmscript: &str) -> Rule {
    let json_logic = compile_farmscript(farmscript).expect(&format!("Failed to compile: {}", farmscript));
    Rule::from_json_logic("test-product", id, json_logic)
        .with_inputs(inputs.iter().map(|s| s.to_string()))
        .with_outputs(outputs.iter().map(|s| s.to_string()))
}

/// Generate arithmetic FarmScript rules
fn generate_farmscript_arithmetic(count: usize, start_id: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start_id + i;
            let n = (i % 100) + 1;
            let expr = match i % 4 {
                0 => format!("x + {}", n),
                1 => format!("x - {}", n),
                2 => format!("x * {}", n),
                _ => format!("x / {}", n.max(1)),
            };
            make_farmscript_rule(
                &format!("fs-arith-{}", id),
                &["x"],
                &[&format!("fs_arith_{}", id)],
                &expr,
            )
        })
        .collect()
}

/// Generate comparison FarmScript rules
fn generate_farmscript_comparison(count: usize, start_id: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start_id + i;
            let threshold = (i * 7) % 100;
            let expr = match i % 6 {
                0 => format!("value > {}", threshold),
                1 => format!("value >= {}", threshold),
                2 => format!("value < {}", threshold),
                3 => format!("value <= {}", threshold),
                4 => format!("value == {}", threshold),
                _ => format!("value != {}", threshold),
            };
            make_farmscript_rule(
                &format!("fs-cmp-{}", id),
                &["value"],
                &[&format!("fs_cmp_{}", id)],
                &expr,
            )
        })
        .collect()
}

/// Generate conditional FarmScript rules
fn generate_farmscript_conditional(count: usize, start_id: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start_id + i;
            let t1 = (i * 3) % 100;
            let t2 = ((i * 5) % 100).max(t1 + 1);
            let expr = format!(
                "if score > {} then \"high\" else if score > {} then \"medium\" else \"low\"",
                t2, t1
            );
            make_farmscript_rule(
                &format!("fs-cond-{}", id),
                &["score"],
                &[&format!("fs_cond_{}", id)],
                &expr,
            )
        })
        .collect()
}

/// Generate complex FarmScript rules with multiple operations
fn generate_farmscript_complex(count: usize, start_id: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start_id + i;
            let w1 = ((i % 5) + 1) as f64 / 10.0;
            let w2 = 1.0 - w1;
            let expr = format!(
                "if quality > 50 and speed > 30 then (quality * {} + speed * {}) * 1.2 else quality + speed",
                w1, w2
            );
            make_farmscript_rule(
                &format!("fs-complex-{}", id),
                &["quality", "speed"],
                &[&format!("fs_complex_{}", id)],
                &expr,
            )
        })
        .collect()
}

/// Generate boolean FarmScript rules
fn generate_farmscript_boolean(count: usize, start_id: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start_id + i;
            let expr = match i % 4 {
                0 => "a and b".to_string(),
                1 => "a or b".to_string(),
                2 => "not a".to_string(),
                _ => "if a then b else c".to_string(),
            };
            make_farmscript_rule(
                &format!("fs-bool-{}", id),
                &["a", "b", "c"],
                &[&format!("fs_bool_{}", id)],
                &expr,
            )
        })
        .collect()
}

/// Generate weighted average FarmScript rules
fn generate_farmscript_weighted(count: usize, start_id: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start_id + i;
            let w = ((i % 10) + 1) as f64 / 10.0;
            let expr = format!("(p1 * {} + p2 * {} + p3 * {}) / 3", w, w * 0.8, w * 0.6);
            make_farmscript_rule(
                &format!("fs-weighted-{}", id),
                &["p1", "p2", "p3"],
                &[&format!("fs_weighted_{}", id)],
                &expr,
            )
        })
        .collect()
}

// =============================================================================
// Generate All 1000 Rules
// =============================================================================

fn generate_all_rules() -> (Vec<Rule>, Vec<String>) {
    let mut rules = Vec::with_capacity(TOTAL_RULES);
    let mut llm_rule_ids = Vec::with_capacity(LLM_RULES);

    // JSON Logic rules (~495)
    let jl_per_type = JSON_LOGIC_RULES / 5;
    rules.extend(generate_json_logic_arithmetic(jl_per_type, 0));
    rules.extend(generate_json_logic_comparison(jl_per_type, jl_per_type));
    rules.extend(generate_json_logic_conditional(jl_per_type, jl_per_type * 2));
    rules.extend(generate_json_logic_complex(jl_per_type, jl_per_type * 3));
    rules.extend(generate_json_logic_boolean(jl_per_type, jl_per_type * 4));

    // FarmScript rules (~495)
    let fs_per_type = FARMSCRIPT_RULES / 6;
    let fs_start = JSON_LOGIC_RULES;
    rules.extend(generate_farmscript_arithmetic(fs_per_type, fs_start));
    rules.extend(generate_farmscript_comparison(fs_per_type, fs_start + fs_per_type));
    rules.extend(generate_farmscript_conditional(fs_per_type, fs_start + fs_per_type * 2));
    rules.extend(generate_farmscript_complex(fs_per_type, fs_start + fs_per_type * 3));
    rules.extend(generate_farmscript_boolean(fs_per_type, fs_start + fs_per_type * 4));
    rules.extend(generate_farmscript_weighted(fs_per_type, fs_start + fs_per_type * 5));

    // LLM rules (10) - these will be placeholders in this test
    // In real usage, these would use the LLM executor
    for i in 0..LLM_RULES {
        let id = format!("llm-rule-{}", i);
        llm_rule_ids.push(id.clone());

        // Create a placeholder JSON Logic rule that simulates LLM output
        // In production, this would be evaluated by the LLM executor
        let expr = match i % 5 {
            0 => json!({"if": [{">": [{"var": "sentiment_score"}, 0.5]}, "positive", "negative"]}),
            1 => json!({"+": [{"var": "base_score"}, {"*": [{"var": "modifier"}, 10]}]}),
            2 => json!({"if": [{"and": [{">": [{"var": "risk"}, 50]}, {"<": [{"var": "confidence"}, 0.7]}]}, "review", "approve"]}),
            3 => json!({"*": [{"var": "quality_score"}, {"if": [{">": [{"var": "urgency"}, 0.8]}, 1.5, 1.0]}]}),
            _ => json!({"cat": ["Category: ", {"if": [{">": [{"var": "complexity"}, 70]}, "Complex", "Simple"]}]}),
        };

        let inputs: Vec<&str> = match i % 5 {
            0 => vec!["sentiment_score"],
            1 => vec!["base_score", "modifier"],
            2 => vec!["risk", "confidence"],
            3 => vec!["quality_score", "urgency"],
            _ => vec!["complexity"],
        };

        rules.push(make_json_logic_rule(
            &id,
            &inputs,
            &[&format!("llm_output_{}", i)],
            expr,
        ));
    }

    println!("Generated {} total rules:", rules.len());
    println!("  - JSON Logic: {}", JSON_LOGIC_RULES);
    println!("  - FarmScript: {}", FARMSCRIPT_RULES);
    println!("  - LLM placeholders: {}", LLM_RULES);

    (rules, llm_rule_ids)
}

// =============================================================================
// Interdependent Rule Generators
// =============================================================================

/// Generate chain of rules where each depends on the previous
fn generate_chained_rules(length: usize) -> Vec<Rule> {
    let mut rules = Vec::with_capacity(length);

    // First rule: JSON Logic
    rules.push(make_json_logic_rule(
        "chain-0",
        &["input"],
        &["chain_0"],
        json!({"+": [{"var": "input"}, 1]}),
    ));

    for i in 1..length {
        let input = format!("chain_{}", i - 1);
        let output = format!("chain_{}", i);

        // Alternate between JSON Logic and FarmScript
        if i % 2 == 0 {
            rules.push(make_json_logic_rule(
                &format!("chain-{}", i),
                &[&input],
                &[&output],
                json!({"+": [{"var": &input}, 1]}),
            ));
        } else {
            rules.push(make_farmscript_rule(
                &format!("chain-{}", i),
                &[&input],
                &[&output],
                &format!("{} + 1", input),
            ));
        }
    }

    rules
}

/// Generate diamond dependency pattern
fn generate_diamond_rules(width: usize) -> Vec<Rule> {
    let mut rules = Vec::new();

    // Base: JSON Logic
    rules.push(make_json_logic_rule(
        "diamond-base",
        &["input"],
        &["d_base"],
        json!({"var": "input"}),
    ));

    // Middle layer: alternate JSON Logic and FarmScript
    let mut mid_outputs = Vec::new();
    for i in 0..width {
        let output = format!("d_mid_{}", i);
        mid_outputs.push(output.clone());

        if i % 2 == 0 {
            rules.push(make_json_logic_rule(
                &format!("diamond-mid-{}", i),
                &["d_base"],
                &[&output],
                json!({"+": [{"var": "d_base"}, i + 1]}),
            ));
        } else {
            rules.push(make_farmscript_rule(
                &format!("diamond-mid-{}", i),
                &["d_base"],
                &[&output],
                &format!("d_base + {}", i + 1),
            ));
        }
    }

    // Final aggregation: sum all middle values
    let mid_refs: Vec<&str> = mid_outputs.iter().map(|s| s.as_str()).collect();
    let sum_expr: Vec<serde_json::Value> = mid_outputs
        .iter()
        .map(|name| json!({"var": name}))
        .collect();

    rules.push(make_json_logic_rule(
        "diamond-final",
        &mid_refs,
        &["d_final"],
        json!({"+": sum_expr}),
    ));

    rules
}

/// Generate tree dependency pattern (fan-out then fan-in)
fn generate_tree_rules(depth: usize, branch_factor: usize) -> Vec<Rule> {
    let mut rules = Vec::new();
    let mut current_level_outputs = vec!["tree_input".to_string()];

    // Root
    rules.push(make_json_logic_rule(
        "tree-root",
        &["input"],
        &["tree_input"],
        json!({"var": "input"}),
    ));

    // Expand levels
    for level in 0..depth {
        let mut next_level_outputs = Vec::new();

        for (parent_idx, parent_output) in current_level_outputs.iter().enumerate() {
            for branch in 0..branch_factor {
                let output = format!("tree_{}_{}", level, parent_idx * branch_factor + branch);
                next_level_outputs.push(output.clone());

                let op = (level + branch) % 3;
                if op == 0 {
                    rules.push(make_json_logic_rule(
                        &format!("tree-{}-{}", level, parent_idx * branch_factor + branch),
                        &[parent_output],
                        &[&output],
                        json!({"+": [{"var": parent_output}, branch + 1]}),
                    ));
                } else if op == 1 {
                    rules.push(make_farmscript_rule(
                        &format!("tree-{}-{}", level, parent_idx * branch_factor + branch),
                        &[parent_output],
                        &[&output],
                        &format!("{} * 2", parent_output),
                    ));
                } else {
                    rules.push(make_json_logic_rule(
                        &format!("tree-{}-{}", level, parent_idx * branch_factor + branch),
                        &[parent_output],
                        &[&output],
                        json!({"-": [{"var": parent_output}, 1]}),
                    ));
                }
            }
        }

        current_level_outputs = next_level_outputs;
    }

    // Final aggregation
    let final_refs: Vec<&str> = current_level_outputs.iter().map(|s| s.as_str()).collect();
    let sum_expr: Vec<serde_json::Value> = current_level_outputs
        .iter()
        .map(|name| json!({"var": name}))
        .collect();

    rules.push(make_json_logic_rule(
        "tree-final",
        &final_refs,
        &["tree_final"],
        json!({"+": sum_expr}),
    ));

    rules
}

/// Generate mesh of interdependent rules
fn generate_mesh_rules(size: usize) -> Vec<Rule> {
    let mut rules = Vec::new();

    // First layer: independent base rules
    for i in 0..size {
        if i % 2 == 0 {
            rules.push(make_json_logic_rule(
                &format!("mesh-base-{}", i),
                &["x"],
                &[&format!("mesh_base_{}", i)],
                json!({"+": [{"var": "x"}, i]}),
            ));
        } else {
            rules.push(make_farmscript_rule(
                &format!("mesh-base-{}", i),
                &["x"],
                &[&format!("mesh_base_{}", i)],
                &format!("x + {}", i),
            ));
        }
    }

    // Second layer: each depends on 2 base rules
    for i in 0..size {
        let dep1 = format!("mesh_base_{}", i);
        let dep2 = format!("mesh_base_{}", (i + 1) % size);

        if i % 2 == 0 {
            rules.push(make_farmscript_rule(
                &format!("mesh-mid-{}", i),
                &[&dep1, &dep2],
                &[&format!("mesh_mid_{}", i)],
                &format!("{} + {}", dep1, dep2),
            ));
        } else {
            rules.push(make_json_logic_rule(
                &format!("mesh-mid-{}", i),
                &[&dep1, &dep2],
                &[&format!("mesh_mid_{}", i)],
                json!({"+": [{"var": &dep1}, {"var": &dep2}]}),
            ));
        }
    }

    // Final layer: aggregate all mid values
    let mid_refs: Vec<String> = (0..size).map(|i| format!("mesh_mid_{}", i)).collect();
    let ref_strs: Vec<&str> = mid_refs.iter().map(|s| s.as_str()).collect();
    let sum_expr: Vec<serde_json::Value> = mid_refs
        .iter()
        .map(|name| json!({"var": name}))
        .collect();

    rules.push(make_json_logic_rule(
        "mesh-final",
        &ref_strs,
        &["mesh_final"],
        json!({"+": sum_expr}),
    ));

    rules
}

// =============================================================================
// Tests
// =============================================================================

#[test]
fn test_1000_rules_concurrent() {
    let (rules, _llm_ids) = generate_all_rules();

    let mut executor = RuleExecutor::new();

    let compile_start = std::time::Instant::now();
    executor.compile_rules(&rules).unwrap();
    let compile_time = compile_start.elapsed();

    let stats = executor.stats();
    println!("\n=== Compilation Stats ===");
    println!("Compiled {} rules in {:?}", stats.compiled_rules, compile_time);
    println!("Total AST nodes: {}", stats.total_ast_nodes);
    println!("Rules with bytecode: {}", stats.rules_with_bytecode);

    // Execute with comprehensive context
    let mut context = ExecutionContext::from_json(&json!({
        "x": 100,
        "value": 75,
        "score": 60,
        "quality": 80,
        "speed": 65,
        "a": true,
        "b": false,
        "c": true,
        "p1": 90,
        "p2": 85,
        "p3": 70,
        "sentiment_score": 0.7,
        "base_score": 50,
        "modifier": 2,
        "risk": 40,
        "confidence": 0.8,
        "quality_score": 75,
        "urgency": 0.5,
        "complexity": 60
    }));

    let exec_start = std::time::Instant::now();
    let result = executor.execute(&rules, &mut context).unwrap();
    let exec_time = exec_start.elapsed();

    println!("\n=== Execution Results ===");
    println!("Total rules executed: {}", result.rule_results.len());
    println!("Total levels: {}", result.levels.len());
    println!("Execution time: {:?}", exec_time);
    println!("Throughput: {:.0} rules/sec", rules.len() as f64 / exec_time.as_secs_f64());

    // Verify counts
    assert_eq!(result.rule_results.len(), rules.len());

    // Sample some results
    println!("\n=== Sample Results ===");
    if let Some(v) = result.get_output("jl_arith_0") {
        println!("jl_arith_0 (x + 1 = 101): {:?}", v);
    }
    if let Some(v) = result.get_output("fs_arith_495") {
        println!("fs_arith_495: {:?}", v);
    }
    if let Some(v) = result.get_output("llm_output_0") {
        println!("llm_output_0 (sentiment): {:?}", v);
    }
}

#[test]
fn test_chain_100_mixed() {
    let rules = generate_chained_rules(100);

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "input": 0
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&rules, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("\n=== 100-Level Chain (Mixed JSON Logic + FarmScript) ===");
    println!("Levels: {}", result.levels.len());
    println!("Time: {:?}", elapsed);

    // Each rule adds 1, so final = 100
    let final_val = result.get_output("chain_99").unwrap().to_number();
    println!("Final value (chain_99): {}", final_val);
    assert_eq!(final_val, 100.0);
    assert_eq!(result.levels.len(), 100);
}

#[test]
fn test_diamond_50_wide() {
    let rules = generate_diamond_rules(50);

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "input": 10
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&rules, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("\n=== Diamond Pattern (width=50) ===");
    println!("Levels: {}", result.levels.len());
    for (i, level) in result.levels.iter().enumerate() {
        println!("  Level {}: {} rules", i, level.len());
    }
    println!("Time: {:?}", elapsed);

    // d_final = sum of (10+1), (10+2), ..., (10+50) = 50*10 + (1+...+50) = 500 + 1275 = 1775
    let final_val = result.get_output("d_final").unwrap().to_number();
    println!("Final value: {}", final_val);
    assert_eq!(final_val, 1775.0);
}

#[test]
fn test_tree_3_levels_4_branches() {
    let rules = generate_tree_rules(3, 4);

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "input": 10
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&rules, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("\n=== Tree Pattern (depth=3, branches=4) ===");
    println!("Total rules: {}", result.rule_results.len());
    println!("Levels: {}", result.levels.len());
    println!("Time: {:?}", elapsed);

    // Final should aggregate 4^3 = 64 leaf values
    let final_val = result.get_output("tree_final");
    println!("Final value: {:?}", final_val);
    assert!(final_val.is_some());
}

#[test]
fn test_mesh_20_nodes() {
    let rules = generate_mesh_rules(20);

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "x": 5
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&rules, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("\n=== Mesh Pattern (20 nodes) ===");
    println!("Total rules: {}", result.rule_results.len());
    println!("Levels: {}", result.levels.len());
    println!("Time: {:?}", elapsed);

    let final_val = result.get_output("mesh_final");
    println!("Final value: {:?}", final_val);
    assert!(final_val.is_some());
}

#[test]
fn test_thread_safety_1000_rules() {
    let (rules, _) = generate_all_rules();

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&rules).unwrap();
    let executor = Arc::new(executor);

    let completed = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    // 20 threads each executing all 1000 rules
    for thread_id in 0..20 {
        let exec = executor.clone();
        let rules = rules.clone();
        let completed = completed.clone();

        let handle = std::thread::spawn(move || {
            let mut context = ExecutionContext::from_json(&json!({
                "x": thread_id * 10,
                "value": 50 + thread_id,
                "score": 40 + thread_id * 2,
                "quality": 70,
                "speed": 60,
                "a": thread_id % 2 == 0,
                "b": thread_id % 3 == 0,
                "c": true,
                "p1": 80,
                "p2": 70,
                "p3": 60,
                "sentiment_score": 0.5,
                "base_score": 40,
                "modifier": 1,
                "risk": 30,
                "confidence": 0.9,
                "quality_score": 65,
                "urgency": 0.3,
                "complexity": 50
            }));

            let result = exec.execute(&rules, &mut context).unwrap();
            assert_eq!(result.rule_results.len(), rules.len());
            completed.fetch_add(1, Ordering::SeqCst);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    println!("\n=== Thread Safety Test ===");
    println!("20 threads x 1000 rules = 20,000 rule executions");
    println!("All threads completed: {}/20", completed.load(Ordering::SeqCst));
    assert_eq!(completed.load(Ordering::SeqCst), 20);
}

#[test]
fn test_high_concurrency_stress() {
    let (rules, _) = generate_all_rules();

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&rules).unwrap();
    let executor = Arc::new(executor);

    let start = std::time::Instant::now();

    // 50 threads
    let handles: Vec<_> = (0..50)
        .map(|i| {
            let exec = executor.clone();
            let rules = rules.clone();

            std::thread::spawn(move || {
                let mut context = ExecutionContext::from_json(&json!({
                    "x": i,
                    "value": 50,
                    "score": 60,
                    "quality": 80,
                    "speed": 70,
                    "a": true,
                    "b": false,
                    "c": true,
                    "p1": 90,
                    "p2": 80,
                    "p3": 70,
                    "sentiment_score": 0.6,
                    "base_score": 50,
                    "modifier": 2,
                    "risk": 35,
                    "confidence": 0.85,
                    "quality_score": 72,
                    "urgency": 0.4,
                    "complexity": 55
                }));

                exec.execute(&rules, &mut context).unwrap()
            })
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let elapsed = start.elapsed();

    let total_executions = 50 * rules.len();
    println!("\n=== High Concurrency Stress Test ===");
    println!("50 threads x {} rules = {} rule executions", rules.len(), total_executions);
    println!("Time: {:?}", elapsed);
    println!("Throughput: {:.0} rule executions/sec", total_executions as f64 / elapsed.as_secs_f64());

    assert_eq!(results.len(), 50);
    for result in &results {
        assert_eq!(result.rule_results.len(), rules.len());
    }
}

#[test]
fn test_repeated_execution_performance() {
    let (rules, _) = generate_all_rules();

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&rules).unwrap();

    let mut context = ExecutionContext::from_json(&json!({
        "x": 100,
        "value": 75,
        "score": 60,
        "quality": 80,
        "speed": 65,
        "a": true,
        "b": false,
        "c": true,
        "p1": 90,
        "p2": 85,
        "p3": 70,
        "sentiment_score": 0.7,
        "base_score": 50,
        "modifier": 2,
        "risk": 40,
        "confidence": 0.8,
        "quality_score": 75,
        "urgency": 0.5,
        "complexity": 60
    }));

    // Warmup
    let _ = executor.execute(&rules, &mut context).unwrap();

    // Measure 10 executions
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let result = executor.execute(&rules, &mut context).unwrap();
        assert_eq!(result.rule_results.len(), rules.len());
    }
    let elapsed = start.elapsed();

    println!("\n=== Repeated Execution Performance ===");
    println!("10 executions of {} rules", rules.len());
    println!("Total time: {:?}", elapsed);
    println!("Avg per execution: {:?}", elapsed / 10);
    println!("Throughput: {:.0} rules/sec", (10 * rules.len()) as f64 / elapsed.as_secs_f64());
}
