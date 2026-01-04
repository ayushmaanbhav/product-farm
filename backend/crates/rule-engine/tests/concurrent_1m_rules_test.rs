//! Concurrent 1 Million Rules Stress Test
//!
//! Tests extreme scalability with:
//! - 1,000,000 total rules
//! - 100,000-level chain (minimum chain length requirement)
//! - 10,000+ unique variables
//! - Complex interdependency patterns: chains, diamonds, trees, lattices, cascades, meshes
//!
//! Run with: cargo test -p product-farm-rule-engine --test concurrent_1m_rules_test --release -- --nocapture
//! Note: This test requires significant memory and time. Use release mode for performance.

use product_farm_core::Rule;
use product_farm_farmscript::compile as compile_farmscript;
use product_farm_rule_engine::{ExecutionContext, RuleExecutor};
use serde_json::json;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// =============================================================================
// Configuration
// =============================================================================

const TOTAL_RULES: usize = 1_000_000;
const MIN_CHAIN_LENGTH: usize = 100_000;
const MIN_VARIABLES: usize = 10_000;

// Pattern sizes
const CHAIN_LENGTH: usize = 100_000;
const DIAMOND_WIDTH: usize = 10_000;
const LATTICE_ROWS: usize = 100;
const LATTICE_COLS: usize = 1_000;
const CASCADE_COUNT: usize = 10;
const CASCADE_LENGTH: usize = 10_000;
const TREE_DEPTH: usize = 8;
const TREE_BRANCH: usize = 4;
const MESH_GROUPS: usize = 100;
const MESH_SIZE: usize = 50;

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
// Pattern Generators
// =============================================================================

/// Generate a 100,000-level chain (alternating JSON Logic and FarmScript)
fn gen_chain_100k() -> (Vec<Rule>, HashSet<String>) {
    println!("  Generating 100k chain...");
    let mut rules = Vec::with_capacity(CHAIN_LENGTH);
    let mut vars = HashSet::new();

    vars.insert("chain_input".to_string());

    // First rule
    rules.push(make_json_logic_rule(
        "chain-0",
        &["chain_input"],
        &["chain_0"],
        json!({ "+": [{"var": "chain_input"}, 1] }),
    ));
    vars.insert("chain_0".to_string());

    // Remaining chain
    for i in 1..CHAIN_LENGTH {
        let input = format!("chain_{}", i - 1);
        let output = format!("chain_{}", i);
        vars.insert(output.clone());

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

        if i % 10000 == 0 {
            println!("    Chain progress: {}/{}", i, CHAIN_LENGTH);
        }
    }

    (rules, vars)
}

/// Generate a 10,000-wide diamond pattern
fn gen_diamond_10k() -> (Vec<Rule>, HashSet<String>) {
    println!("  Generating 10k diamond...");
    let mut rules = Vec::with_capacity(DIAMOND_WIDTH + 2);
    let mut vars = HashSet::new();

    vars.insert("diam_input".to_string());
    vars.insert("diam_base".to_string());

    // Base
    rules.push(make_json_logic_rule(
        "diam-base",
        &["diam_input"],
        &["diam_base"],
        json!({"var": "diam_input"}),
    ));

    // Middle layer (10,000 parallel)
    let mut mid_outputs = Vec::with_capacity(DIAMOND_WIDTH);
    for i in 0..DIAMOND_WIDTH {
        let output = format!("diam_mid_{}", i);
        mid_outputs.push(output.clone());
        vars.insert(output.clone());

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

        if i % 2000 == 0 && i > 0 {
            println!("    Diamond middle progress: {}/{}", i, DIAMOND_WIDTH);
        }
    }

    // Final aggregation (sum first 100 to avoid massive expression)
    let sum_vars: Vec<serde_json::Value> = mid_outputs[0..100]
        .iter()
        .map(|name| json!({"var": name}))
        .collect();
    let sum_refs: Vec<&str> = mid_outputs[0..100].iter().map(|s| s.as_str()).collect();

    vars.insert("diam_final".to_string());
    rules.push(make_json_logic_rule(
        "diam-final",
        &sum_refs,
        &["diam_final"],
        json!({ "+": sum_vars }),
    ));

    (rules, vars)
}

/// Generate a 100x1000 lattice
fn gen_lattice_100x1000() -> (Vec<Rule>, HashSet<String>) {
    println!("  Generating 100x1000 lattice...");
    let mut rules = Vec::with_capacity(LATTICE_ROWS * LATTICE_COLS + 2);
    let mut vars = HashSet::new();

    vars.insert("lat_input".to_string());

    // Initialize first column
    for row in 0..LATTICE_ROWS {
        let output = format!("lat_{}_{}", row, 0);
        vars.insert(output.clone());

        if row == 0 {
            rules.push(make_json_logic_rule(
                &format!("lat-{}-0", row),
                &["lat_input"],
                &[&output],
                json!({ "+": [{"var": "lat_input"}, 1] }),
            ));
        } else {
            let above = format!("lat_{}_{}", row - 1, 0);
            rules.push(make_json_logic_rule(
                &format!("lat-{}-0", row),
                &[&above],
                &[&output],
                json!({ "+": [{"var": &above}, 1] }),
            ));
        }
    }

    // Fill lattice
    for col in 1..LATTICE_COLS {
        for row in 0..LATTICE_ROWS {
            let output = format!("lat_{}_{}", row, col);
            vars.insert(output.clone());

            let left = format!("lat_{}_{}", row, col - 1);

            if row == 0 {
                // First row: only depends on left
                if col % 2 == 0 {
                    rules.push(make_json_logic_rule(
                        &format!("lat-{}-{}", row, col),
                        &[&left],
                        &[&output],
                        json!({ "*": [{"var": &left}, 1.0001] }),
                    ));
                } else {
                    rules.push(make_farmscript_rule(
                        &format!("lat-{}-{}", row, col),
                        &[&left],
                        &[&output],
                        &format!("{} * 1.0001", left),
                    ));
                }
            } else {
                // Depends on left and above
                let above = format!("lat_{}_{}", row - 1, col);
                if col % 2 == 0 {
                    rules.push(make_json_logic_rule(
                        &format!("lat-{}-{}", row, col),
                        &[&left, &above],
                        &[&output],
                        json!({ "+": [{"var": &left}, {"var": &above}] }),
                    ));
                } else {
                    rules.push(make_farmscript_rule(
                        &format!("lat-{}-{}", row, col),
                        &[&left, &above],
                        &[&output],
                        &format!("({} + {}) / 2", left, above),
                    ));
                }
            }
        }

        if col % 200 == 0 {
            println!("    Lattice progress: col {}/{}", col, LATTICE_COLS);
        }
    }

    // Final: sum of last column
    let last_col_refs: Vec<String> = (0..LATTICE_ROWS)
        .map(|r| format!("lat_{}_{}", r, LATTICE_COLS - 1))
        .collect();
    let last_col_vars: Vec<serde_json::Value> = last_col_refs
        .iter()
        .map(|name| json!({"var": name}))
        .collect();
    let last_col_str: Vec<&str> = last_col_refs.iter().map(|s| s.as_str()).collect();

    vars.insert("lat_final".to_string());
    rules.push(make_json_logic_rule(
        "lat-final",
        &last_col_str,
        &["lat_final"],
        json!({ "+": last_col_vars }),
    ));

    (rules, vars)
}

/// Generate 10 cascades of 10,000 levels each
fn gen_cascades_10x10k() -> (Vec<Rule>, HashSet<String>) {
    println!("  Generating 10x10k cascades...");
    let mut rules = Vec::with_capacity(CASCADE_COUNT * CASCADE_LENGTH);
    let mut vars = HashSet::new();

    for cascade_id in 0..CASCADE_COUNT {
        let input_var = format!("casc_{}_input", cascade_id);
        vars.insert(input_var.clone());

        // First rule of cascade
        let first_output = format!("casc_{}_{}", cascade_id, 0);
        vars.insert(first_output.clone());
        rules.push(make_json_logic_rule(
            &format!("casc-{}-0", cascade_id),
            &[&input_var],
            &[&first_output],
            json!({ "+": [{"var": &input_var}, cascade_id + 1] }),
        ));

        // Rest of cascade with lookback
        for level in 1..CASCADE_LENGTH {
            let output = format!("casc_{}_{}", cascade_id, level);
            vars.insert(output.clone());

            // Depend on previous 1-3 levels
            let lookback = (level % 3) + 1;
            let mut inputs: Vec<String> = Vec::new();
            let mut sum_parts: Vec<serde_json::Value> = Vec::new();

            for lb in 1..=lookback {
                if level >= lb {
                    let prev = format!("casc_{}_{}", cascade_id, level - lb);
                    sum_parts.push(json!({"var": &prev}));
                    inputs.push(prev);
                }
            }

            let input_refs: Vec<&str> = inputs.iter().map(|s| s.as_str()).collect();

            if level % 2 == 0 {
                rules.push(make_json_logic_rule(
                    &format!("casc-{}-{}", cascade_id, level),
                    &input_refs,
                    &[&output],
                    json!({ "+": sum_parts }),
                ));
            } else {
                // FarmScript version - just use first input
                let first_input = &inputs[0];
                rules.push(make_farmscript_rule(
                    &format!("casc-{}-{}", cascade_id, level),
                    &[first_input],
                    &[&output],
                    &format!("{} + 1", first_input),
                ));
            }
        }

        println!("    Cascade {} complete", cascade_id);
    }

    (rules, vars)
}

/// Generate a large tree (depth 8, branch 4)
fn gen_tree_large() -> (Vec<Rule>, HashSet<String>) {
    println!("  Generating large tree (depth {}, branch {})...", TREE_DEPTH, TREE_BRANCH);
    let mut rules = Vec::new();
    let mut vars = HashSet::new();

    vars.insert("tree_input".to_string());

    // Root
    vars.insert("tree_0_0".to_string());
    rules.push(make_json_logic_rule(
        "tree-0-0",
        &["tree_input"],
        &["tree_0_0"],
        json!({"var": "tree_input"}),
    ));

    let mut current_level = vec!["tree_0_0".to_string()];
    let mut node_counter = 1;

    for level in 1..=TREE_DEPTH {
        let mut next_level = Vec::new();

        for (parent_idx, parent) in current_level.iter().enumerate() {
            for branch in 0..TREE_BRANCH {
                let output = format!("tree_{}_{}", level, node_counter);
                vars.insert(output.clone());
                node_counter += 1;

                let multiplier = ((level * TREE_BRANCH + branch) % 10) as f64 / 10.0 + 0.5;

                if (parent_idx + branch) % 2 == 0 {
                    rules.push(make_json_logic_rule(
                        &format!("tree-{}-{}", level, node_counter),
                        &[parent],
                        &[&output],
                        json!({ "*": [{"var": parent}, multiplier] }),
                    ));
                } else {
                    rules.push(make_farmscript_rule(
                        &format!("tree-{}-{}", level, node_counter),
                        &[parent],
                        &[&output],
                        &format!("{} * {}", parent, multiplier),
                    ));
                }

                next_level.push(output);
            }
        }

        println!("    Tree level {}: {} nodes", level, next_level.len());
        current_level = next_level;
    }

    // Final aggregation
    let leaf_sum: Vec<serde_json::Value> = current_level[0..100.min(current_level.len())]
        .iter()
        .map(|name| json!({"var": name}))
        .collect();
    let leaf_refs: Vec<&str> = current_level[0..100.min(current_level.len())]
        .iter()
        .map(|s| s.as_str())
        .collect();

    vars.insert("tree_final".to_string());
    rules.push(make_json_logic_rule(
        "tree-final",
        &leaf_refs,
        &["tree_final"],
        json!({ "+": leaf_sum }),
    ));

    (rules, vars)
}

/// Generate mesh groups (100 groups of 50 interconnected nodes)
fn gen_mesh_groups() -> (Vec<Rule>, HashSet<String>) {
    println!("  Generating {} mesh groups of {} nodes...", MESH_GROUPS, MESH_SIZE);
    let mut rules = Vec::new();
    let mut vars = HashSet::new();

    for group in 0..MESH_GROUPS {
        let group_input = format!("mesh_{}_input", group);
        vars.insert(group_input.clone());

        // First node
        let first = format!("mesh_{}_{}", group, 0);
        vars.insert(first.clone());
        rules.push(make_json_logic_rule(
            &format!("mesh-{}-0", group),
            &[&group_input],
            &[&first],
            json!({ "+": [{"var": &group_input}, group + 1] }),
        ));

        // Remaining nodes with complex dependencies
        for node in 1..MESH_SIZE {
            let output = format!("mesh_{}_{}", group, node);
            vars.insert(output.clone());

            // Depend on previous nodes (1-3 back)
            let mut deps: Vec<String> = Vec::new();
            for back in 1..=3.min(node) {
                deps.push(format!("mesh_{}_{}", group, node - back));
            }

            let dep_refs: Vec<&str> = deps.iter().map(|s| s.as_str()).collect();
            let sum_parts: Vec<serde_json::Value> = deps
                .iter()
                .map(|name| json!({"var": name}))
                .collect();

            if node % 2 == 0 {
                rules.push(make_json_logic_rule(
                    &format!("mesh-{}-{}", group, node),
                    &dep_refs,
                    &[&output],
                    json!({ "+": sum_parts }),
                ));
            } else {
                rules.push(make_farmscript_rule(
                    &format!("mesh-{}-{}", group, node),
                    &[&deps[0]],
                    &[&output],
                    &format!("{} + {}", deps[0], node),
                ));
            }
        }

        // Group output
        let group_output = format!("mesh_{}_out", group);
        let last_node = format!("mesh_{}_{}", group, MESH_SIZE - 1);
        vars.insert(group_output.clone());
        rules.push(make_json_logic_rule(
            &format!("mesh-{}-out", group),
            &[&last_node],
            &[&group_output],
            json!({"var": &last_node}),
        ));

        if group % 20 == 0 && group > 0 {
            println!("    Mesh groups progress: {}/{}", group, MESH_GROUPS);
        }
    }

    (rules, vars)
}

/// Generate independent rules to fill up to 1 million
fn gen_independent_rules(count: usize, start_id: usize) -> (Vec<Rule>, HashSet<String>) {
    println!("  Generating {} independent rules...", count);
    let mut rules = Vec::with_capacity(count);
    let mut vars = HashSet::new();

    // Use a set of 1000 input variables
    for i in 0..1000 {
        vars.insert(format!("ind_in_{}", i));
    }

    for i in 0..count {
        let id = start_id + i;
        let input_var = format!("ind_in_{}", i % 1000);
        let output_var = format!("ind_out_{}", id);
        vars.insert(output_var.clone());

        let n = (i % 100) + 1;

        match i % 6 {
            0 => rules.push(make_json_logic_rule(
                &format!("ind-{}", id),
                &[&input_var],
                &[&output_var],
                json!({ "+": [{"var": &input_var}, n] }),
            )),
            1 => rules.push(make_json_logic_rule(
                &format!("ind-{}", id),
                &[&input_var],
                &[&output_var],
                json!({ "*": [{"var": &input_var}, (n % 10) + 1] }),
            )),
            2 => rules.push(make_json_logic_rule(
                &format!("ind-{}", id),
                &[&input_var],
                &[&output_var],
                json!({ ">": [{"var": &input_var}, n * 10] }),
            )),
            3 => rules.push(make_farmscript_rule(
                &format!("ind-{}", id),
                &[&input_var],
                &[&output_var],
                &format!("{} + {}", input_var, n),
            )),
            4 => rules.push(make_farmscript_rule(
                &format!("ind-{}", id),
                &[&input_var],
                &[&output_var],
                &format!("{} * {}", input_var, (n % 10) + 1),
            )),
            _ => rules.push(make_farmscript_rule(
                &format!("ind-{}", id),
                &[&input_var],
                &[&output_var],
                &format!("{} > {}", input_var, n * 10),
            )),
        }

        if i % 100000 == 0 && i > 0 {
            println!("    Independent rules progress: {}/{}", i, count);
        }
    }

    (rules, vars)
}

// =============================================================================
// Test Generators
// =============================================================================

fn generate_1m_rules() -> (Vec<Rule>, HashSet<String>) {
    println!("\n=== Generating 1 Million Rules ===");
    let start = std::time::Instant::now();

    let mut all_rules = Vec::with_capacity(TOTAL_RULES);
    let mut all_vars = HashSet::new();

    // Pattern 1: 100k chain
    let (chain_rules, chain_vars) = gen_chain_100k();
    println!("    Chain: {} rules, {} vars", chain_rules.len(), chain_vars.len());
    all_rules.extend(chain_rules);
    all_vars.extend(chain_vars);

    // Pattern 2: 10k diamond
    let (diamond_rules, diamond_vars) = gen_diamond_10k();
    println!("    Diamond: {} rules, {} vars", diamond_rules.len(), diamond_vars.len());
    all_rules.extend(diamond_rules);
    all_vars.extend(diamond_vars);

    // Pattern 3: 100x1000 lattice
    let (lattice_rules, lattice_vars) = gen_lattice_100x1000();
    println!("    Lattice: {} rules, {} vars", lattice_rules.len(), lattice_vars.len());
    all_rules.extend(lattice_rules);
    all_vars.extend(lattice_vars);

    // Pattern 4: 10x10k cascades
    let (cascade_rules, cascade_vars) = gen_cascades_10x10k();
    println!("    Cascades: {} rules, {} vars", cascade_rules.len(), cascade_vars.len());
    all_rules.extend(cascade_rules);
    all_vars.extend(cascade_vars);

    // Pattern 5: Large tree
    let (tree_rules, tree_vars) = gen_tree_large();
    println!("    Tree: {} rules, {} vars", tree_rules.len(), tree_vars.len());
    all_rules.extend(tree_rules);
    all_vars.extend(tree_vars);

    // Pattern 6: Mesh groups
    let (mesh_rules, mesh_vars) = gen_mesh_groups();
    println!("    Mesh: {} rules, {} vars", mesh_rules.len(), mesh_vars.len());
    all_rules.extend(mesh_rules);
    all_vars.extend(mesh_vars);

    // Fill remaining with independent rules
    let current_count = all_rules.len();
    let remaining = TOTAL_RULES.saturating_sub(current_count);
    if remaining > 0 {
        let (ind_rules, ind_vars) = gen_independent_rules(remaining, current_count);
        println!("    Independent: {} rules, {} vars", ind_rules.len(), ind_vars.len());
        all_rules.extend(ind_rules);
        all_vars.extend(ind_vars);
    }

    let elapsed = start.elapsed();
    println!("\nGeneration complete:");
    println!("  Total rules: {}", all_rules.len());
    println!("  Total unique variables: {}", all_vars.len());
    println!("  Generation time: {:?}", elapsed);

    assert!(all_rules.len() >= TOTAL_RULES, "Should have at least {} rules", TOTAL_RULES);
    assert!(all_vars.len() >= MIN_VARIABLES, "Should have at least {} variables", MIN_VARIABLES);

    (all_rules, all_vars)
}

// =============================================================================
// Tests
// =============================================================================

#[test]
fn test_100k_chain() {
    println!("\n=== 100,000-Level Chain Test ===");

    let (chain_rules, _) = gen_chain_100k();
    println!("Chain rules: {}", chain_rules.len());

    assert_eq!(chain_rules.len(), CHAIN_LENGTH, "Chain should have exactly {} rules", CHAIN_LENGTH);

    println!("Creating executor...");
    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "chain_input": 0
    }));

    println!("Starting execution...");
    let start = std::time::Instant::now();
    let result = executor.execute(&chain_rules, &mut context).unwrap();
    let elapsed = start.elapsed();
    println!("Execution complete!");

    println!("Levels: {}", result.levels.len());
    println!("Time: {:?}", elapsed);

    // chain_99999 = 0 + 100000 * 1 = 100000
    let final_val = result.get_output(&format!("chain_{}", CHAIN_LENGTH - 1))
        .expect("Should have final chain value")
        .to_number();
    println!("Final value (chain_{}): {}", CHAIN_LENGTH - 1, final_val);

    assert_eq!(result.levels.len(), CHAIN_LENGTH, "Should have {} levels", CHAIN_LENGTH);
    assert_eq!(final_val, CHAIN_LENGTH as f64, "Final value should be {}", CHAIN_LENGTH);
}

#[test]
fn test_10k_diamond() {
    println!("\n=== 10,000-Wide Diamond Test ===");

    let (diamond_rules, _) = gen_diamond_10k();
    println!("Diamond rules: {}", diamond_rules.len());

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "diam_input": 10
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&diamond_rules, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("Levels: {}", result.levels.len());
    println!("Time: {:?}", elapsed);

    // diam_final = sum of first 100 middle nodes = sum(10+1, 10+2, ..., 10+100) = 100*10 + 5050 = 6050
    let final_val = result.get_output("diam_final")
        .expect("Should have final diamond value")
        .to_number();
    println!("Final value: {}", final_val);

    assert_eq!(result.levels.len(), 3, "Diamond should have 3 levels");
    assert_eq!(final_val, 6050.0, "Diamond final should be 6050");
}

#[test]
fn test_100x1000_lattice() {
    println!("\n=== 100x1000 Lattice Test ===");

    let (lattice_rules, _) = gen_lattice_100x1000();
    println!("Lattice rules: {}", lattice_rules.len());

    let executor = RuleExecutor::new();
    let mut context = ExecutionContext::from_json(&json!({
        "lat_input": 1
    }));

    let start = std::time::Instant::now();
    let result = executor.execute(&lattice_rules, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("Levels: {}", result.levels.len());
    println!("Time: {:?}", elapsed);

    let final_val = result.get_output("lat_final");
    println!("Final value: {:?}", final_val);

    assert!(final_val.is_some(), "Should have lattice final value");
}

#[test]
fn test_10_cascades_10k() {
    println!("\n=== 10 Cascades x 10,000 Levels Test ===");

    let (cascade_rules, _) = gen_cascades_10x10k();
    println!("Cascade rules: {}", cascade_rules.len());

    let executor = RuleExecutor::new();

    // Initialize all cascade inputs
    let mut context_data = serde_json::Map::new();
    for i in 0..CASCADE_COUNT {
        context_data.insert(format!("casc_{}_input", i), json!(1));
    }
    let mut context = ExecutionContext::from_json(&serde_json::Value::Object(context_data));

    let start = std::time::Instant::now();
    let result = executor.execute(&cascade_rules, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("Time: {:?}", elapsed);

    // Check some cascade outputs
    for i in 0..CASCADE_COUNT {
        let final_var = format!("casc_{}_{}", i, CASCADE_LENGTH - 1);
        let val = result.get_output(&final_var);
        if i < 3 {
            println!("Cascade {} final: {:?}", i, val);
        }
        assert!(val.is_some(), "Should have cascade {} final value", i);
    }
}

#[test]
fn test_variable_count() {
    println!("\n=== Variable Count Test ===");

    let (_, vars) = generate_1m_rules();

    println!("Total unique variables: {}", vars.len());
    assert!(vars.len() >= MIN_VARIABLES, "Should have at least {} variables, got {}", MIN_VARIABLES, vars.len());
}

#[test]
fn test_1m_compilation() {
    println!("\n=== 1 Million Rules Compilation Test ===");

    let (rules, vars) = generate_1m_rules();

    println!("Rules: {}", rules.len());
    println!("Variables: {}", vars.len());

    let mut executor = RuleExecutor::new();

    let compile_start = std::time::Instant::now();
    executor.compile_rules(&rules).unwrap();
    let compile_elapsed = compile_start.elapsed();

    println!("Compilation time: {:?}", compile_elapsed);
    println!("Compilation rate: {:.0} rules/sec", rules.len() as f64 / compile_elapsed.as_secs_f64());
}

#[test]
fn test_thread_safety_1m() {
    println!("\n=== Thread Safety 1M Rules ===");

    // Use a smaller subset for thread safety testing (100k rules)
    let (chain_rules, _) = gen_chain_100k();

    let mut executor = RuleExecutor::new();
    executor.compile_rules(&chain_rules).unwrap();
    let executor = Arc::new(executor);

    let completed = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    let thread_count = 4; // Limited for memory

    println!("{} threads x {} rules = {} executions", thread_count, chain_rules.len(), thread_count * chain_rules.len());

    let start = std::time::Instant::now();

    for thread_id in 0..thread_count {
        let exec = executor.clone();
        let rules = chain_rules.clone();
        let completed = completed.clone();

        let handle = std::thread::spawn(move || {
            let mut context = ExecutionContext::from_json(&json!({
                "chain_input": thread_id
            }));

            let result = exec.execute(&rules, &mut context);
            assert!(result.is_ok(), "Thread {} execution failed", thread_id);

            // Verify thread-specific result
            let final_val = result.unwrap()
                .get_output(&format!("chain_{}", CHAIN_LENGTH - 1))
                .unwrap()
                .to_number();

            // chain_99999 = thread_id + 100000
            assert_eq!(final_val, (thread_id + CHAIN_LENGTH) as f64,
                "Thread {} should get {}", thread_id, thread_id + CHAIN_LENGTH);

            completed.fetch_add(1, Ordering::SeqCst);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let elapsed = start.elapsed();
    let total_executions = thread_count * chain_rules.len();

    println!("Time: {:?}", elapsed);
    println!("Throughput: {:.0} rule executions/sec", total_executions as f64 / elapsed.as_secs_f64());
    println!("Completed: {}/{}", completed.load(Ordering::SeqCst), thread_count);

    assert_eq!(completed.load(Ordering::SeqCst), thread_count);
}

#[test]
fn test_full_1m_execution() {
    println!("\n=== Full 1 Million Rules Execution Test ===");
    println!("WARNING: This test requires significant memory and time");

    let (rules, vars) = generate_1m_rules();

    println!("Rules: {}", rules.len());
    println!("Variables: {}", vars.len());

    // Build context with all input variables
    let mut context_data = serde_json::Map::new();

    // Chain input
    context_data.insert("chain_input".to_string(), json!(0));

    // Diamond input
    context_data.insert("diam_input".to_string(), json!(10));

    // Lattice input
    context_data.insert("lat_input".to_string(), json!(1));

    // Cascade inputs
    for i in 0..CASCADE_COUNT {
        context_data.insert(format!("casc_{}_input", i), json!(1));
    }

    // Tree input
    context_data.insert("tree_input".to_string(), json!(10));

    // Mesh inputs
    for i in 0..MESH_GROUPS {
        context_data.insert(format!("mesh_{}_input", i), json!(1));
    }

    // Independent inputs
    for i in 0..1000 {
        context_data.insert(format!("ind_in_{}", i), json!(i + 1));
    }

    let mut context = ExecutionContext::from_json(&serde_json::Value::Object(context_data));

    let executor = RuleExecutor::new();

    let start = std::time::Instant::now();
    let result = executor.execute(&rules, &mut context).unwrap();
    let elapsed = start.elapsed();

    println!("\nExecution complete:");
    println!("  Time: {:?}", elapsed);
    println!("  Throughput: {:.0} rules/sec", rules.len() as f64 / elapsed.as_secs_f64());
    println!("  Levels: {}", result.levels.len());

    // Verify key outputs
    let chain_final = result.get_output(&format!("chain_{}", CHAIN_LENGTH - 1));
    println!("  Chain final: {:?}", chain_final.map(|v| v.to_number()));
    assert!(chain_final.is_some(), "Should have chain final");

    let diamond_final = result.get_output("diam_final");
    println!("  Diamond final: {:?}", diamond_final.map(|v| v.to_number()));
    assert!(diamond_final.is_some(), "Should have diamond final");

    let lattice_final = result.get_output("lat_final");
    println!("  Lattice final: {:?}", lattice_final.map(|v| v.to_number()));
    assert!(lattice_final.is_some(), "Should have lattice final");

    println!("\n✓ All 1 million rules executed successfully");
}
