//! Concurrent 10,000 Rules Stress Test
//!
//! Tests thread safety, scalability, and concurrent evaluation with:
//! - 10 LLM rules (placeholders for Ollama)
//! - ~4995 JSON Logic rules
//! - ~4995 FarmScript rules
//! - Complex interdependencies with 1000+ level chains
//! - Diamond, tree, mesh, and lattice patterns
//!
//! Run with: cargo test -p product-farm-rule-engine --test concurrent_10k_rules_test -- --nocapture
//! For release mode: cargo test -p product-farm-rule-engine --test concurrent_10k_rules_test --release -- --nocapture

use product_farm_core::Rule;
use product_farm_farmscript::compile as compile_farmscript;
use product_farm_rule_engine::{ExecutionContext, RuleExecutor};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// =============================================================================
// Configuration
// =============================================================================

const TOTAL_RULES: usize = 10_000;
const LLM_RULES: usize = 10;
const JSON_LOGIC_RULES: usize = 4995;
const FARMSCRIPT_RULES: usize = 4995;
const MIN_CHAIN_LENGTH: usize = 1000;

// =============================================================================
// Rule Builders
// =============================================================================

fn make_json_logic_rule(id: &str, inputs: &[&str], outputs: &[&str], expr: serde_json::Value) -> Rule {
    Rule::from_json_logic("test-product", id, expr)
        .with_inputs(inputs.iter().map(|s| s.to_string()))
        .with_outputs(outputs.iter().map(|s| s.to_string()))
}

fn make_farmscript_rule(id: &str, inputs: &[&str], outputs: &[&str], farmscript: &str) -> Rule {
    let json_logic = compile_farmscript(farmscript)
        .unwrap_or_else(|e| panic!("Failed to compile FarmScript '{}': {:?}", farmscript, e));
    Rule::from_json_logic("test-product", id, json_logic)
        .with_inputs(inputs.iter().map(|s| s.to_string()))
        .with_outputs(outputs.iter().map(|s| s.to_string()))
}

// =============================================================================
// JSON Logic Generators (4995 rules)
// =============================================================================

/// Generate arithmetic rules: +, -, *, /, %
fn gen_jl_arithmetic(count: usize, start: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start + i;
            let n = (i % 100) + 1;
            let expr = match i % 5 {
                0 => json!({ "+": [{"var": "x"}, n] }),
                1 => json!({ "-": [{"var": "x"}, n] }),
                2 => json!({ "*": [{"var": "x"}, (n % 10) + 1] }),
                3 => json!({ "/": [{"var": "x"}, (n % 10) + 1] }),
                _ => json!({ "%": [{"var": "x"}, (n % 10) + 1] }),
            };
            make_json_logic_rule(
                &format!("jl-arith-{}", id),
                &["x"],
                &[&format!("jl_a_{}", id)],
                expr,
            )
        })
        .collect()
}

/// Generate comparison rules: >, >=, <, <=, ==, !=
fn gen_jl_comparison(count: usize, start: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start + i;
            let threshold = (i * 7) % 1000;
            let expr = match i % 6 {
                0 => json!({ ">": [{"var": "v"}, threshold] }),
                1 => json!({ ">=": [{"var": "v"}, threshold] }),
                2 => json!({ "<": [{"var": "v"}, threshold] }),
                3 => json!({ "<=": [{"var": "v"}, threshold] }),
                4 => json!({ "==": [{"var": "v"}, threshold % 100] }),
                _ => json!({ "!=": [{"var": "v"}, threshold % 100] }),
            };
            make_json_logic_rule(
                &format!("jl-cmp-{}", id),
                &["v"],
                &[&format!("jl_c_{}", id)],
                expr,
            )
        })
        .collect()
}

/// Generate conditional rules with nested if-then-else
fn gen_jl_conditional(count: usize, start: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start + i;
            let t1 = (i * 3) % 100;
            let t2 = ((i * 5) % 100).max(t1 + 1);
            let t3 = ((i * 7) % 100).max(t2 + 1);
            let expr = json!({
                "if": [
                    { ">": [{"var": "s"}, t3] }, "critical",
                    { ">": [{"var": "s"}, t2] }, "high",
                    { ">": [{"var": "s"}, t1] }, "medium",
                    "low"
                ]
            });
            make_json_logic_rule(
                &format!("jl-cond-{}", id),
                &["s"],
                &[&format!("jl_cat_{}", id)],
                expr,
            )
        })
        .collect()
}

/// Generate boolean logic rules: and, or, not, combinations
fn gen_jl_boolean(count: usize, start: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start + i;
            let expr = match i % 8 {
                0 => json!({"and": [{"var": "a"}, {"var": "b"}]}),
                1 => json!({"or": [{"var": "a"}, {"var": "b"}]}),
                2 => json!({"!": {"var": "a"}}),
                3 => json!({"and": [{"var": "a"}, {"or": [{"var": "b"}, {"var": "c"}]}]}),
                4 => json!({"or": [{"and": [{"var": "a"}, {"var": "b"}]}, {"var": "c"}]}),
                5 => json!({"!": {"and": [{"var": "a"}, {"var": "b"}]}}),
                6 => json!({"!": {"or": [{"var": "a"}, {"var": "b"}]}}),
                _ => json!({"if": [{"var": "a"}, {"var": "b"}, {"var": "c"}]}),
            };
            make_json_logic_rule(
                &format!("jl-bool-{}", id),
                &["a", "b", "c"],
                &[&format!("jl_b_{}", id)],
                expr,
            )
        })
        .collect()
}

/// Generate complex nested rules with multiple operations
fn gen_jl_complex(count: usize, start: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start + i;
            let w1 = ((i % 10) + 1) as f64 / 10.0;
            let w2 = 1.0 - w1;
            let threshold = (i % 50) + 30;
            let expr = json!({
                "if": [
                    {"and": [
                        {">": [{"var": "q"}, threshold]},
                        {">": [{"var": "sp"}, threshold / 2]}
                    ]},
                    {"*": [
                        {"+": [
                            {"*": [{"var": "q"}, w1]},
                            {"*": [{"var": "sp"}, w2]}
                        ]},
                        1.2
                    ]},
                    {"if": [
                        {">": [{"var": "q"}, threshold / 2]},
                        {"+": [
                            {"*": [{"var": "q"}, 0.8]},
                            {"*": [{"var": "sp"}, 0.2]}
                        ]},
                        {"*": [
                            {"+": [{"var": "q"}, {"var": "sp"}]},
                            0.5
                        ]}
                    ]}
                ]
            });
            make_json_logic_rule(
                &format!("jl-cmplx-{}", id),
                &["q", "sp"],
                &[&format!("jl_cx_{}", id)],
                expr,
            )
        })
        .collect()
}

/// Generate string/data manipulation rules
fn gen_jl_data(count: usize, start: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start + i;
            let expr = match i % 4 {
                0 => json!({"cat": ["prefix_", {"var": "name"}]}),
                1 => json!({"substr": [{"var": "name"}, 0, 5]}),
                2 => json!({"in": [{"var": "item"}, {"var": "list"}]}),
                _ => json!({"merge": [{"var": "arr1"}, {"var": "arr2"}]}),
            };
            let inputs = match i % 4 {
                0 | 1 => vec!["name"],
                2 => vec!["item", "list"],
                _ => vec!["arr1", "arr2"],
            };
            make_json_logic_rule(
                &format!("jl-data-{}", id),
                &inputs,
                &[&format!("jl_d_{}", id)],
                expr,
            )
        })
        .collect()
}

// =============================================================================
// FarmScript Generators (4995 rules)
// =============================================================================

/// Generate arithmetic FarmScript rules
fn gen_fs_arithmetic(count: usize, start: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start + i;
            let n = (i % 100) + 1;
            let expr = match i % 5 {
                0 => format!("x + {}", n),
                1 => format!("x - {}", n),
                2 => format!("x * {}", (n % 10) + 1),
                3 => format!("x / {}", (n % 10) + 1),
                _ => format!("x % {}", (n % 10) + 1),
            };
            make_farmscript_rule(
                &format!("fs-arith-{}", id),
                &["x"],
                &[&format!("fs_a_{}", id)],
                &expr,
            )
        })
        .collect()
}

/// Generate comparison FarmScript rules
fn gen_fs_comparison(count: usize, start: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start + i;
            let threshold = (i * 7) % 1000;
            let expr = match i % 6 {
                0 => format!("v > {}", threshold),
                1 => format!("v >= {}", threshold),
                2 => format!("v < {}", threshold),
                3 => format!("v <= {}", threshold),
                4 => format!("v == {}", threshold % 100),
                _ => format!("v != {}", threshold % 100),
            };
            make_farmscript_rule(
                &format!("fs-cmp-{}", id),
                &["v"],
                &[&format!("fs_c_{}", id)],
                &expr,
            )
        })
        .collect()
}

/// Generate conditional FarmScript rules
fn gen_fs_conditional(count: usize, start: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start + i;
            let t1 = (i * 3) % 100;
            let t2 = ((i * 5) % 100).max(t1 + 1);
            let t3 = ((i * 7) % 100).max(t2 + 1);
            let expr = format!(
                "if s > {} then \"critical\" else if s > {} then \"high\" else if s > {} then \"medium\" else \"low\"",
                t3, t2, t1
            );
            make_farmscript_rule(
                &format!("fs-cond-{}", id),
                &["s"],
                &[&format!("fs_cat_{}", id)],
                &expr,
            )
        })
        .collect()
}

/// Generate boolean FarmScript rules
fn gen_fs_boolean(count: usize, start: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start + i;
            let expr = match i % 8 {
                0 => "a and b".to_string(),
                1 => "a or b".to_string(),
                2 => "not a".to_string(),
                3 => "a and (b or c)".to_string(),
                4 => "(a and b) or c".to_string(),
                5 => "not (a and b)".to_string(),
                6 => "not (a or b)".to_string(),
                _ => "if a then b else c".to_string(),
            };
            make_farmscript_rule(
                &format!("fs-bool-{}", id),
                &["a", "b", "c"],
                &[&format!("fs_b_{}", id)],
                &expr,
            )
        })
        .collect()
}

/// Generate complex FarmScript rules
fn gen_fs_complex(count: usize, start: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start + i;
            let w1 = ((i % 10) + 1) as f64 / 10.0;
            let w2 = 1.0 - w1;
            let t = (i % 50) + 30;
            let expr = format!(
                "if q > {} and sp > {} then (q * {} + sp * {}) * 1.2 else if q > {} then q * 0.8 + sp * 0.2 else (q + sp) * 0.5",
                t, t / 2, w1, w2, t / 2
            );
            make_farmscript_rule(
                &format!("fs-cmplx-{}", id),
                &["q", "sp"],
                &[&format!("fs_cx_{}", id)],
                &expr,
            )
        })
        .collect()
}

/// Generate weighted calculation FarmScript rules
fn gen_fs_weighted(count: usize, start: usize) -> Vec<Rule> {
    (0..count)
        .map(|i| {
            let id = start + i;
            let w1 = ((i % 5) + 1) as f64 / 10.0;
            let w2 = ((i % 3) + 1) as f64 / 10.0;
            let w3 = 1.0 - w1 - w2;
            let expr = format!("p1 * {} + p2 * {} + p3 * {}", w1, w2, w3.max(0.1));
            make_farmscript_rule(
                &format!("fs-wgt-{}", id),
                &["p1", "p2", "p3"],
                &[&format!("fs_w_{}", id)],
                &expr,
            )
        })
        .collect()
}

// =============================================================================
// Interdependent Chain Generators (1000+ levels)
// =============================================================================

/// Generate a 1000-level chain with alternating JSON Logic and FarmScript
fn gen_chain_1000() -> Vec<Rule> {
    let mut rules = Vec::with_capacity(MIN_CHAIN_LENGTH);

    // First rule
    rules.push(make_json_logic_rule(
        "chain-0",
        &["chain_input"],
        &["chain_0"],
        json!({ "+": [{"var": "chain_input"}, 1] }),
    ));

    // Generate 999 more rules
    for i in 1..MIN_CHAIN_LENGTH {
        let input = format!("chain_{}", i - 1);
        let output = format!("chain_{}", i);

        if i % 2 == 0 {
            rules.push(make_json_logic_rule(
                &format!("chain-{}", i),
                &[&input],
                &[&output],
                json!({ "+": [{"var": &input}, 1] }),
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

/// Generate a wide diamond pattern (100 wide, 3 levels)
fn gen_diamond_100() -> Vec<Rule> {
    let mut rules = Vec::new();
    let width = 100;

    // Base
    rules.push(make_json_logic_rule(
        "diam-base",
        &["diam_input"],
        &["diam_base"],
        json!({"var": "diam_input"}),
    ));

    // Middle layer (100 parallel)
    let mut mid_outputs = Vec::new();
    for i in 0..width {
        let output = format!("diam_mid_{}", i);
        mid_outputs.push(output.clone());

        if i % 2 == 0 {
            rules.push(make_json_logic_rule(
                &format!("diam-mid-{}", i),
                &["diam_base"],
                &[&output],
                json!({ "+": [{"var": "diam_base"}, i + 1] }),
            ));
        } else {
            rules.push(make_farmscript_rule(
                &format!("diam-mid-{}", i),
                &["diam_base"],
                &[&output],
                &format!("diam_base + {}", i + 1),
            ));
        }
    }

    // Final aggregation
    let sum_expr: Vec<serde_json::Value> = mid_outputs
        .iter()
        .map(|name| json!({"var": name}))
        .collect();
    let mid_refs: Vec<&str> = mid_outputs.iter().map(|s| s.as_str()).collect();

    rules.push(make_json_logic_rule(
        "diam-final",
        &mid_refs,
        &["diam_final"],
        json!({ "+": sum_expr }),
    ));

    rules
}

/// Generate a deep tree (depth 10, branch 3 = 3^10 = ~59k, but we limit)
fn gen_tree_deep() -> Vec<Rule> {
    let mut rules = Vec::new();
    let depth = 6;
    let branch = 3;

    // Root
    rules.push(make_json_logic_rule(
        "tree-root",
        &["tree_input"],
        &["tree_0_0"],
        json!({"var": "tree_input"}),
    ));

    let mut current_level = vec!["tree_0_0".to_string()];

    for level in 1..=depth {
        let mut next_level = Vec::new();

        for (parent_idx, parent) in current_level.iter().enumerate() {
            for branch_idx in 0..branch {
                let output = format!("tree_{}_{}", level, parent_idx * branch + branch_idx);
                next_level.push(output.clone());

                let op = (level + branch_idx) % 3;
                if op == 0 {
                    rules.push(make_json_logic_rule(
                        &format!("tree-{}-{}", level, parent_idx * branch + branch_idx),
                        &[parent],
                        &[&output],
                        json!({ "+": [{"var": parent}, branch_idx + 1] }),
                    ));
                } else if op == 1 {
                    rules.push(make_farmscript_rule(
                        &format!("tree-{}-{}", level, parent_idx * branch + branch_idx),
                        &[parent],
                        &[&output],
                        &format!("{} * 2", parent),
                    ));
                } else {
                    rules.push(make_json_logic_rule(
                        &format!("tree-{}-{}", level, parent_idx * branch + branch_idx),
                        &[parent],
                        &[&output],
                        json!({ "-": [{"var": parent}, 1] }),
                    ));
                }
            }
        }

        current_level = next_level;

        // Limit total rules in tree
        if rules.len() > 1000 {
            break;
        }
    }

    // Final aggregation of leaves
    let leaf_refs: Vec<&str> = current_level.iter().map(|s| s.as_str()).collect();
    let sum_expr: Vec<serde_json::Value> = current_level
        .iter()
        .map(|name| json!({"var": name}))
        .collect();

    rules.push(make_json_logic_rule(
        "tree-final",
        &leaf_refs,
        &["tree_final"],
        json!({ "+": sum_expr }),
    ));

    rules
}

/// Generate a lattice pattern (grid with dependencies)
fn gen_lattice(width: usize, height: usize) -> Vec<Rule> {
    let mut rules = Vec::new();

    // First row depends on input
    for col in 0..width {
        let output = format!("lat_0_{}", col);
        if col == 0 {
            rules.push(make_json_logic_rule(
                &format!("lat-0-{}", col),
                &["lat_input"],
                &[&output],
                json!({ "+": [{"var": "lat_input"}, col + 1] }),
            ));
        } else {
            let prev = format!("lat_0_{}", col - 1);
            rules.push(make_farmscript_rule(
                &format!("lat-0-{}", col),
                &["lat_input", &prev],
                &[&output],
                &format!("lat_input + {} + 1", prev),
            ));
        }
    }

    // Subsequent rows depend on row above and left neighbor
    for row in 1..height {
        for col in 0..width {
            let output = format!("lat_{}_{}", row, col);
            let above = format!("lat_{}_{}", row - 1, col);

            if col == 0 {
                if row % 2 == 0 {
                    rules.push(make_json_logic_rule(
                        &format!("lat-{}-{}", row, col),
                        &[&above],
                        &[&output],
                        json!({ "+": [{"var": &above}, 1] }),
                    ));
                } else {
                    rules.push(make_farmscript_rule(
                        &format!("lat-{}-{}", row, col),
                        &[&above],
                        &[&output],
                        &format!("{} + 1", above),
                    ));
                }
            } else {
                let left = format!("lat_{}_{}", row, col - 1);
                if (row + col) % 2 == 0 {
                    rules.push(make_json_logic_rule(
                        &format!("lat-{}-{}", row, col),
                        &[&above, &left],
                        &[&output],
                        json!({ "+": [{"var": &above}, {"var": &left}] }),
                    ));
                } else {
                    rules.push(make_farmscript_rule(
                        &format!("lat-{}-{}", row, col),
                        &[&above, &left],
                        &[&output],
                        &format!("{} + {}", above, left),
                    ));
                }
            }
        }
    }

    // Final: sum of last row
    let last_row: Vec<String> = (0..width).map(|col| format!("lat_{}_{}", height - 1, col)).collect();
    let refs: Vec<&str> = last_row.iter().map(|s| s.as_str()).collect();
    let sum_expr: Vec<serde_json::Value> = last_row.iter().map(|n| json!({"var": n})).collect();

    rules.push(make_json_logic_rule(
        "lat-final",
        &refs,
        &["lat_final"],
        json!({ "+": sum_expr }),
    ));

    rules
}

/// Generate cascade pattern (each rule depends on multiple previous)
fn gen_cascade(length: usize, lookback: usize) -> Vec<Rule> {
    let mut rules = Vec::new();

    // First `lookback` rules depend on input
    for i in 0..lookback.min(length) {
        let output = format!("casc_{}", i);
        rules.push(make_json_logic_rule(
            &format!("casc-{}", i),
            &["casc_input"],
            &[&output],
            json!({ "+": [{"var": "casc_input"}, i + 1] }),
        ));
    }

    // Remaining rules depend on previous `lookback` rules
    for i in lookback..length {
        let output = format!("casc_{}", i);
        let deps: Vec<String> = (0..lookback).map(|j| format!("casc_{}", i - j - 1)).collect();
        let dep_refs: Vec<&str> = deps.iter().map(|s| s.as_str()).collect();

        let sum_expr: Vec<serde_json::Value> = deps.iter().map(|d| json!({"var": d})).collect();

        if i % 2 == 0 {
            rules.push(make_json_logic_rule(
                &format!("casc-{}", i),
                &dep_refs,
                &[&output],
                json!({ "+": sum_expr }),
            ));
        } else {
            // For FarmScript, just use first two deps
            let expr = format!("{} + {}", deps[0], deps[1]);
            rules.push(make_farmscript_rule(
                &format!("casc-{}", i),
                &dep_refs[..2],
                &[&output],
                &expr,
            ));
        }
    }

    rules
}

// =============================================================================
// Generate All 10,000 Rules
// =============================================================================

fn generate_10k_rules() -> Vec<Rule> {
    let mut rules = Vec::with_capacity(TOTAL_RULES);
    let mut id_counter = 0;

    println!("Generating 10,000 rules...");

    // === Independent Rules (fast, parallelizable) ===

    // JSON Logic rules (~4995 split across types)
    let jl_per_type = JSON_LOGIC_RULES / 6;
    println!("  JSON Logic arithmetic: {}", jl_per_type);
    rules.extend(gen_jl_arithmetic(jl_per_type, id_counter));
    id_counter += jl_per_type;

    println!("  JSON Logic comparison: {}", jl_per_type);
    rules.extend(gen_jl_comparison(jl_per_type, id_counter));
    id_counter += jl_per_type;

    println!("  JSON Logic conditional: {}", jl_per_type);
    rules.extend(gen_jl_conditional(jl_per_type, id_counter));
    id_counter += jl_per_type;

    println!("  JSON Logic boolean: {}", jl_per_type);
    rules.extend(gen_jl_boolean(jl_per_type, id_counter));
    id_counter += jl_per_type;

    println!("  JSON Logic complex: {}", jl_per_type);
    rules.extend(gen_jl_complex(jl_per_type, id_counter));
    id_counter += jl_per_type;

    println!("  JSON Logic data: {}", jl_per_type);
    rules.extend(gen_jl_data(jl_per_type, id_counter));
    id_counter += jl_per_type;

    // FarmScript rules (~4995 split across types)
    let fs_per_type = FARMSCRIPT_RULES / 6;
    println!("  FarmScript arithmetic: {}", fs_per_type);
    rules.extend(gen_fs_arithmetic(fs_per_type, id_counter));
    id_counter += fs_per_type;

    println!("  FarmScript comparison: {}", fs_per_type);
    rules.extend(gen_fs_comparison(fs_per_type, id_counter));
    id_counter += fs_per_type;

    println!("  FarmScript conditional: {}", fs_per_type);
    rules.extend(gen_fs_conditional(fs_per_type, id_counter));
    id_counter += fs_per_type;

    println!("  FarmScript boolean: {}", fs_per_type);
    rules.extend(gen_fs_boolean(fs_per_type, id_counter));
    id_counter += fs_per_type;

    println!("  FarmScript complex: {}", fs_per_type);
    rules.extend(gen_fs_complex(fs_per_type, id_counter));
    id_counter += fs_per_type;

    println!("  FarmScript weighted: {}", fs_per_type);
    rules.extend(gen_fs_weighted(fs_per_type, id_counter));

    // LLM placeholder rules (10)
    println!("  LLM placeholders: {}", LLM_RULES);
    for i in 0..LLM_RULES {
        let expr = match i % 5 {
            0 => json!({"if": [{">": [{"var": "llm_score"}, 50]}, "positive", "negative"]}),
            1 => json!({"+": [{"var": "llm_base"}, {"*": [{"var": "llm_mod"}, 10]}]}),
            2 => json!({"if": [{">": [{"var": "llm_risk"}, 70]}, "high", "low"]}),
            3 => json!({"*": [{"var": "llm_quality"}, 1.5]}),
            _ => json!({"cat": ["Result: ", {"if": [{">": [{"var": "llm_val"}, 50]}, "Pass", "Fail"]}]}),
        };
        rules.push(make_json_logic_rule(
            &format!("llm-{}", i),
            &["llm_score", "llm_base", "llm_mod", "llm_risk", "llm_quality", "llm_val"],
            &[&format!("llm_out_{}", i)],
            expr,
        ));
    }

    let independent_count = rules.len();
    println!("  Total independent rules: {}", independent_count);

    rules
}

/// Generate interdependent rules separately (for specific tests)
fn generate_interdependent_rules() -> (Vec<Rule>, Vec<Rule>, Vec<Rule>, Vec<Rule>, Vec<Rule>) {
    println!("Generating interdependent rules...");

    println!("  1000-level chain...");
    let chain = gen_chain_1000();
    println!("    Chain rules: {}", chain.len());

    println!("  100-wide diamond...");
    let diamond = gen_diamond_100();
    println!("    Diamond rules: {}", diamond.len());

    println!("  Deep tree...");
    let tree = gen_tree_deep();
    println!("    Tree rules: {}", tree.len());

    println!("  20x50 lattice...");
    let lattice = gen_lattice(20, 50);
    println!("    Lattice rules: {}", lattice.len());

    println!("  500-level cascade (lookback 5)...");
    let cascade = gen_cascade(500, 5);
    println!("    Cascade rules: {}", cascade.len());

    (chain, diamond, tree, lattice, cascade)
}

// =============================================================================
// Tests
// =============================================================================

#[test]
fn test_10k_independent_rules() {
    let rules = generate_10k_rules();

    println!("\n=== 10,000 Independent Rules Test ===");
    println!("Total rules: {}", rules.len());

    let mut executor = RuleExecutor::new();

    let compile_start = std::time::Instant::now();
    executor.compile_rules(&rules).unwrap();
    let compile_time = compile_start.elapsed();

    let stats = executor.stats();
    println!("Compilation: {:?}", compile_time);
    println!("AST nodes: {}", stats.total_ast_nodes);
    println!("Bytecode rules: {}", stats.rules_with_bytecode);

    let mut context = ExecutionContext::from_json(&json!({
        "x": 100, "v": 500, "s": 75,
        "a": true, "b": false, "c": true,
        "q": 80, "sp": 60,
        "name": "test_string", "item": "a", "list": ["a", "b", "c"],
        "arr1": [1, 2], "arr2": [3, 4],
        "p1": 90, "p2": 85, "p3": 70,
        "llm_score": 60, "llm_base": 50, "llm_mod": 2,
        "llm_risk": 40, "llm_quality": 80, "llm_val": 55
    }));

    let exec_start = std::time::Instant::now();
    let result = executor.execute(&rules, &mut context).unwrap();
    let exec_time = exec_start.elapsed();

    println!("Execution: {:?}", exec_time);
    println!("Rules executed: {}", result.rule_results.len());
    println!("Levels: {}", result.levels.len());
    println!("Throughput: {:.0} rules/sec", rules.len() as f64 / exec_time.as_secs_f64());

    assert_eq!(result.rule_results.len(), rules.len());
}

#[test]
fn test_1000_level_chain() {
    let (chain, _, _, _, _) = generate_interdependent_rules();

    println!("\n=== 1000-Level Chain Test ===");
    println!("Chain rules: {}", chain.len());

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "chain_input": 0
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&chain, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("Levels: {}", result.levels.len());
    println!("Time: {:?}", elapsed);

    let final_val = result.get_output(&format!("chain_{}", MIN_CHAIN_LENGTH - 1))
        .expect("Missing final chain output")
        .to_number();

    println!("Final value (chain_{}): {}", MIN_CHAIN_LENGTH - 1, final_val);

    assert_eq!(result.levels.len(), MIN_CHAIN_LENGTH);
    assert_eq!(final_val, MIN_CHAIN_LENGTH as f64);
}

#[test]
fn test_100_wide_diamond() {
    let (_, diamond, _, _, _) = generate_interdependent_rules();

    println!("\n=== 100-Wide Diamond Test ===");
    println!("Diamond rules: {}", diamond.len());

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "diam_input": 10
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&diamond, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("Levels: {}", result.levels.len());
    println!("Middle level size: {}", result.levels.get(1).map(|l| l.len()).unwrap_or(0));
    println!("Time: {:?}", elapsed);

    // diam_final = sum of (10+1), (10+2), ..., (10+100) = 100*10 + (1+2+...+100) = 1000 + 5050 = 6050
    let final_val = result.get_output("diam_final").unwrap().to_number();
    println!("Final value: {}", final_val);

    assert_eq!(result.levels.len(), 3);
    assert_eq!(final_val, 6050.0);
}

#[test]
fn test_deep_tree() {
    let (_, _, tree, _, _) = generate_interdependent_rules();

    println!("\n=== Deep Tree Test ===");
    println!("Tree rules: {}", tree.len());

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "tree_input": 10
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&tree, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("Levels: {}", result.levels.len());
    println!("Time: {:?}", elapsed);

    let final_val = result.get_output("tree_final");
    println!("Final value: {:?}", final_val);

    assert!(final_val.is_some());
}

#[test]
fn test_lattice_20x50() {
    let (_, _, _, lattice, _) = generate_interdependent_rules();

    println!("\n=== 20x50 Lattice Test ===");
    println!("Lattice rules: {}", lattice.len());

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "lat_input": 1
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&lattice, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("Levels: {}", result.levels.len());
    println!("Time: {:?}", elapsed);

    let final_val = result.get_output("lat_final");
    println!("Final value: {:?}", final_val);

    assert!(final_val.is_some());
}

#[test]
fn test_cascade_500() {
    let (_, _, _, _, cascade) = generate_interdependent_rules();

    println!("\n=== 500-Level Cascade Test ===");
    println!("Cascade rules: {}", cascade.len());

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "casc_input": 1
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&cascade, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("Levels: {}", result.levels.len());
    println!("Time: {:?}", elapsed);

    let final_val = result.get_output("casc_499");
    println!("Final value (casc_499): {:?}", final_val);

    assert!(final_val.is_some());
}

#[test]
fn test_thread_safety_10k() {
    let rules = generate_10k_rules();

    println!("\n=== Thread Safety 10k Rules ===");

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&rules).unwrap();
    let executor = Arc::new(executor);

    let completed = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    // 10 threads each executing all 10k rules
    for thread_id in 0..10 {
        let exec = executor.clone();
        let rules = rules.clone();
        let completed = completed.clone();

        let handle = std::thread::spawn(move || {
            let mut context = ExecutionContext::from_json(&json!({
                "x": thread_id * 10, "v": 500, "s": 75,
                "a": thread_id % 2 == 0, "b": thread_id % 3 == 0, "c": true,
                "q": 80, "sp": 60,
                "name": "test", "item": "a", "list": ["a", "b"],
                "arr1": [1], "arr2": [2],
                "p1": 90, "p2": 85, "p3": 70,
                "llm_score": 60, "llm_base": 50, "llm_mod": 2,
                "llm_risk": 40, "llm_quality": 80, "llm_val": 55
            }));

            let result = exec.execute(&rules, &mut context).unwrap();
            assert_eq!(result.rule_results.len(), rules.len());
            completed.fetch_add(1, Ordering::SeqCst);
        });

        handles.push(handle);
    }

    let start = std::time::Instant::now();
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
    let elapsed = start.elapsed();

    println!("10 threads x {} rules = {} executions", rules.len(), 10 * rules.len());
    println!("Time: {:?}", elapsed);
    println!("Throughput: {:.0} rule executions/sec", (10 * rules.len()) as f64 / elapsed.as_secs_f64());
    println!("Completed: {}/10", completed.load(Ordering::SeqCst));

    assert_eq!(completed.load(Ordering::SeqCst), 10);
}

#[test]
fn test_high_concurrency_10k() {
    let rules = generate_10k_rules();

    println!("\n=== High Concurrency 10k Rules ===");

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
                    "x": i, "v": 500, "s": 75,
                    "a": true, "b": false, "c": true,
                    "q": 80, "sp": 60,
                    "name": "t", "item": "a", "list": ["a"],
                    "arr1": [1], "arr2": [2],
                    "p1": 90, "p2": 85, "p3": 70,
                    "llm_score": 60, "llm_base": 50, "llm_mod": 2,
                    "llm_risk": 40, "llm_quality": 80, "llm_val": 55
                }));

                exec.execute(&rules, &mut context).unwrap()
            })
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let elapsed = start.elapsed();

    let total = 50 * rules.len();
    println!("50 threads x {} rules = {} executions", rules.len(), total);
    println!("Time: {:?}", elapsed);
    println!("Throughput: {:.0} rule executions/sec", total as f64 / elapsed.as_secs_f64());

    assert_eq!(results.len(), 50);
    for r in &results {
        assert_eq!(r.rule_results.len(), rules.len());
    }
}
