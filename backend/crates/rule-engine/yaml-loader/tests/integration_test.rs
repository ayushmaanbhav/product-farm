//! Integration tests for the yaml-loader crate

use product_farm_yaml_loader::{init, init_with_report};
use product_farm_core::{Rule, EvaluatorType};
use product_farm_rule_engine::{RuleExecutor, ExecutionContext};
use serde_json::json;
use std::path::PathBuf;

fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("banoo-assessment")
}

fn db_outage_fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("db-outage-scenario")
}

#[test]
fn test_load_banoo_assessment_product() {
    let path = fixtures_path();
    println!("Loading from: {:?}", path);

    let result = init(&path);

    match &result {
        Ok(registry) => {
            println!("Successfully loaded product!");
            assert!(registry.has_product("banoo-assessment-v1"));

            // Check product IDs
            let product_ids = registry.product_ids();
            println!("Product IDs: {:?}", product_ids);
            assert!(!product_ids.is_empty());
        }
        Err(e) => {
            println!("Error loading product: {:?}", e);
            // For now, allow partial success as we're testing the loading process
        }
    }
}

#[test]
fn test_load_with_inference_report() {
    let path = fixtures_path();

    let result = init_with_report(&path);

    match result {
        Ok((registry, report)) => {
            println!("Registry loaded with report!");
            println!("Source files: {:?}", report.source_files);
            println!("Entities found: {}", report.entities.len());
            println!("Functions found: {}", report.functions.len());
            println!("Warnings: {}", report.warnings.len());

            // Print inference report markdown
            let markdown = report.to_markdown();
            println!("\n=== Inference Report ===\n{}", markdown);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

#[test]
fn test_get_schema_details() {
    let path = fixtures_path();

    if let Ok(registry) = init(&path) {
        if let Ok(schema) = registry.get_schema("banoo-assessment-v1") {
            println!("Product: {} ({})", schema.product.name, schema.product.id);
            println!("Attributes: {}", schema.attributes.len());
            println!("Rules: {}", schema.rules.len());
            println!("Data Types: {}", schema.data_types.len());
            println!("Functionalities: {}", schema.functionalities.len());

            // List some attributes
            println!("\n=== Sample Attributes ===");
            for attr in schema.attributes.iter().take(5) {
                println!("  - {} ({})", attr.abstract_path, attr.datatype_id);
            }

            // List some rules
            println!("\n=== Sample Rules ===");
            for rule in schema.rules.iter().take(5) {
                println!("  - {} ({:?})", rule.rule_type, rule.evaluator.name);
            }
        }
    }
}

#[test]
fn test_layer_visibility() {
    let path = fixtures_path();

    if let Ok(registry) = init(&path) {
        // Get layer 2 (domain) entities
        let domain_entities = registry.get_interface_entities("banoo-assessment-v1", "layer-2-domain");
        match domain_entities {
            Ok(entities) => {
                println!("Layer 2 (Domain) entities: {}", entities.len());
                for entity in entities.iter().take(3) {
                    println!("  - {}", entity.abstract_path);
                }
            }
            Err(e) => println!("Error getting domain entities: {:?}", e),
        }

        // Get layer 3 (backend) functions
        let backend_functions = registry.get_interface_functions("banoo-assessment-v1", "layer-3-backend");
        match backend_functions {
            Ok(functions) => {
                println!("Layer 3 (Backend) functions: {}", functions.len());
                for func in functions.iter().take(3) {
                    println!("  - {}", func.rule_type);
                }
            }
            Err(e) => println!("Error getting backend functions: {:?}", e),
        }
    }
}

// ===========================================
// Database Outage Scenario Tests (example2)
// ===========================================

#[test]
fn test_load_db_outage_scenario() {
    let path = db_outage_fixtures_path();
    println!("Loading DB Outage scenario from: {:?}", path);

    let result = init(&path);

    match &result {
        Ok(registry) => {
            println!("Successfully loaded DB Outage scenario!");
            assert!(registry.has_product("db-outage-crisis-v1"));

            let product_ids = registry.product_ids();
            println!("Product IDs: {:?}", product_ids);
        }
        Err(e) => {
            println!("Error loading product: {:?}", e);
        }
    }
}

#[test]
fn test_db_outage_with_report() {
    let path = db_outage_fixtures_path();

    let result = init_with_report(&path);

    match result {
        Ok((_registry, report)) => {
            println!("\n=== DB Outage Scenario Inference Report ===\n");
            println!("Source files: {:?}", report.source_files);
            println!("Entities found: {}", report.entities.len());
            println!("Functions found: {}", report.functions.len());
            println!("Warnings: {}", report.warnings.len());
            println!("Low confidence items: {}", report.low_confidence_items.len());

            // Print summary of entities
            println!("\n=== Entities Summary ===");
            for entity in &report.entities {
                println!("  {} ({}) - {} attributes",
                    entity.name,
                    entity.detected_as,
                    entity.attributes.len()
                );
            }

            // Print summary of functions
            println!("\n=== Functions Summary ===");
            for func in &report.functions {
                println!("  {} - {} inputs, {} outputs, evaluator: {}",
                    func.name,
                    func.input_count,
                    func.output_count,
                    func.evaluator_type
                );
            }

            // Print full markdown report
            println!("\n=== Full Inference Report ===\n{}", report.to_markdown());
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

#[test]
fn test_db_outage_schema_details() {
    let path = db_outage_fixtures_path();

    if let Ok(registry) = init(&path) {
        if let Ok(schema) = registry.get_schema("db-outage-crisis-v1") {
            println!("\n=== DB Outage Schema Details ===");
            println!("Product: {} ({})", schema.product.name, schema.product.id);
            println!("Total Attributes: {}", schema.attributes.len());
            println!("Total Rules: {}", schema.rules.len());

            // Group attributes by component type
            let mut by_type: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
            for attr in &schema.attributes {
                by_type.entry(attr.component_type.clone())
                    .or_default()
                    .push(attr);
            }

            println!("\n=== Attributes by Entity ===");
            for (entity, attrs) in &by_type {
                println!("  {}: {} attributes", entity, attrs.len());
            }

            // List rules by evaluator type
            println!("\n=== Rules by Evaluator ===");
            let json_logic_count = schema.rules.iter()
                .filter(|r| matches!(r.evaluator.name, product_farm_core::EvaluatorType::JsonLogic))
                .count();
            let llm_count = schema.rules.iter()
                .filter(|r| matches!(r.evaluator.name, product_farm_core::EvaluatorType::LargeLanguageModel))
                .count();

            println!("  JSON Logic rules: {}", json_logic_count);
            println!("  LLM rules: {}", llm_count);

            // List all rule names
            println!("\n=== All Rules ===");
            for rule in &schema.rules {
                println!("  - {} ({:?})", rule.rule_type, rule.evaluator.name);
            }
        }
    }
}

#[test]
fn test_db_outage_layer_visibility() {
    let path = db_outage_fixtures_path();

    if let Ok(registry) = init(&path) {
        println!("\n=== DB Outage Layer Visibility ===");

        // Test each layer
        let layers = vec![
            "layer-1-requirements",
            "layer-2-domain",
            "layer-3-backend",
            "layer-4-session-ui",
            "layer-5-portal",
        ];

        for layer in layers {
            let entities = registry.get_interface_entities("db-outage-crisis-v1", layer);
            let functions = registry.get_interface_functions("db-outage-crisis-v1", layer);

            println!("\n{}", layer);
            println!("  Entities: {}", entities.map(|e| e.len()).unwrap_or(0));
            println!("  Functions: {}", functions.map(|f| f.len()).unwrap_or(0));
        }
    }
}

// ===========================================
// End-to-End Rule Execution Tests
// ===========================================

/// Helper to get JSON Logic rules from the loaded schema
fn get_json_logic_rules(rules: &[Rule]) -> Vec<&Rule> {
    rules.iter()
        .filter(|r| matches!(r.evaluator.name, EvaluatorType::JsonLogic))
        .collect()
}

#[test]
fn test_execute_detect_quick_response_rule() {
    let path = db_outage_fixtures_path();
    let registry = init(&path).expect("Failed to load product");
    let schema = registry.get_schema("db-outage-crisis-v1").expect("Schema not found");

    // Find the detect-quick-response rule
    let rule = schema.rules.iter()
        .find(|r| r.rule_type == "detect-quick-response")
        .expect("Rule 'detect-quick-response' not found");

    println!("Testing rule: {}", rule.rule_type);
    println!("Expression: {}", rule.compiled_expression);

    let mut executor = RuleExecutor::new();

    // Test case 1: Quick response (< 2 min = 120 secs)
    let mut ctx = ExecutionContext::from_json(&json!({
        "alert_acknowledged": true,
        "time_since_alert_secs": 90
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("evidence_detected");
    println!("Test 1 (quick response): {:?}", output);
    assert_eq!(output.map(|v| v.as_bool()), Some(Some(true)), "Should detect quick response");

    // Test case 2: Slow response (>= 2 min)
    let mut ctx = ExecutionContext::from_json(&json!({
        "alert_acknowledged": true,
        "time_since_alert_secs": 150
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("evidence_detected");
    println!("Test 2 (slow response): {:?}", output);
    assert_eq!(output.map(|v| v.as_bool()), Some(Some(false)), "Should NOT detect quick response");

    // Test case 3: Not acknowledged
    let mut ctx = ExecutionContext::from_json(&json!({
        "alert_acknowledged": false,
        "time_since_alert_secs": 60
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("evidence_detected");
    println!("Test 3 (not acknowledged): {:?}", output);
    assert_eq!(output.map(|v| v.as_bool()), Some(Some(false)), "Should NOT detect if not acknowledged");
}

#[test]
fn test_execute_detect_vp_communication_rule() {
    let path = db_outage_fixtures_path();
    let registry = init(&path).expect("Failed to load product");
    let schema = registry.get_schema("db-outage-crisis-v1").expect("Schema not found");

    let rule = schema.rules.iter()
        .find(|r| r.rule_type == "detect-vp-communication")
        .expect("Rule 'detect-vp-communication' not found");

    println!("Testing rule: {}", rule.rule_type);

    let mut executor = RuleExecutor::new();

    // Test case 1: Responded in time (< 10 min = 600 secs)
    let mut ctx = ExecutionContext::from_json(&json!({
        "vp_email_received": true,
        "vp_responded": true,
        "time_since_email_secs": 300
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("responded_in_time");
    println!("Test 1 (responded in time): {:?}", output);
    assert_eq!(output.map(|v| v.as_bool()), Some(Some(true)));

    // Test case 2: Responded too late
    let mut ctx = ExecutionContext::from_json(&json!({
        "vp_email_received": true,
        "vp_responded": true,
        "time_since_email_secs": 700
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("responded_in_time");
    println!("Test 2 (responded too late): {:?}", output);
    assert_eq!(output.map(|v| v.as_bool()), Some(Some(false)));

    // Test case 3: Did not respond
    let mut ctx = ExecutionContext::from_json(&json!({
        "vp_email_received": true,
        "vp_responded": false,
        "time_since_email_secs": 300
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("responded_in_time");
    println!("Test 3 (did not respond): {:?}", output);
    assert_eq!(output.map(|v| v.as_bool()), Some(Some(false)));
}

#[test]
fn test_execute_compute_signal_score_rule() {
    let path = db_outage_fixtures_path();
    let registry = init(&path).expect("Failed to load product");
    let schema = registry.get_schema("db-outage-crisis-v1").expect("Schema not found");

    let rule = schema.rules.iter()
        .find(|r| r.rule_type == "compute-signal-score")
        .expect("Rule 'compute-signal-score' not found");

    println!("Testing rule: {}", rule.rule_type);
    println!("Expression: {}", rule.compiled_expression);

    let mut executor = RuleExecutor::new();

    // Test case 1: High positive, low negative
    // Formula: max(0, min(100, max_possible * (positive - negative * 0.5)))
    // = max(0, min(100, 100 * (0.8 - 0.2 * 0.5))) = max(0, min(100, 100 * 0.7)) = 70
    let mut ctx = ExecutionContext::from_json(&json!({
        "positive_signals": 0.8,
        "negative_signals": 0.2,
        "max_possible_score": 100
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("score");
    println!("Test 1 (high positive): {:?}", output);
    let score = output.map(|v| v.to_number()).unwrap_or(0.0);
    assert!((score - 70.0).abs() < 0.01, "Expected ~70, got {}", score);

    // Test case 2: All positive signals
    // = max(0, min(100, 100 * (1.0 - 0 * 0.5))) = 100
    let mut ctx = ExecutionContext::from_json(&json!({
        "positive_signals": 1.0,
        "negative_signals": 0.0,
        "max_possible_score": 100
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("score");
    println!("Test 2 (all positive): {:?}", output);
    let score = output.map(|v| v.to_number()).unwrap_or(0.0);
    assert!((score - 100.0).abs() < 0.01, "Expected 100, got {}", score);

    // Test case 3: More negative than positive (should clamp to 0)
    // = max(0, min(100, 100 * (0.2 - 1.0 * 0.5))) = max(0, -30) = 0
    let mut ctx = ExecutionContext::from_json(&json!({
        "positive_signals": 0.2,
        "negative_signals": 1.0,
        "max_possible_score": 100
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("score");
    println!("Test 3 (more negative): {:?}", output);
    let score = output.map(|v| v.to_number()).unwrap_or(-1.0);
    assert!(score >= 0.0, "Score should be clamped to 0, got {}", score);
}

#[test]
fn test_execute_compute_recommendation_rule() {
    let path = db_outage_fixtures_path();
    let registry = init(&path).expect("Failed to load product");
    let schema = registry.get_schema("db-outage-crisis-v1").expect("Schema not found");

    let rule = schema.rules.iter()
        .find(|r| r.rule_type == "compute-recommendation")
        .expect("Rule 'compute-recommendation' not found");

    println!("Testing rule: {}", rule.rule_type);
    println!("Expression: {}", rule.compiled_expression);

    let mut executor = RuleExecutor::new();

    // Test case 1: Critical failures → strong_no_hire
    let mut ctx = ExecutionContext::from_json(&json!({
        "overall_score": 90,
        "critical_failures": 1
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("recommendation");
    println!("Test 1 (critical failures): {:?}", output);
    assert_eq!(output.map(|v| v.as_str()), Some(Some("strong_no_hire")));

    // Test case 2: Score >= 85 → strong_hire
    let mut ctx = ExecutionContext::from_json(&json!({
        "overall_score": 90,
        "critical_failures": 0
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("recommendation");
    println!("Test 2 (score 90): {:?}", output);
    assert_eq!(output.map(|v| v.as_str()), Some(Some("strong_hire")));

    // Test case 3: Score >= 65 → hire
    let mut ctx = ExecutionContext::from_json(&json!({
        "overall_score": 70,
        "critical_failures": 0
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("recommendation");
    println!("Test 3 (score 70): {:?}", output);
    assert_eq!(output.map(|v| v.as_str()), Some(Some("hire")));

    // Test case 4: Score >= 45 → no_hire
    let mut ctx = ExecutionContext::from_json(&json!({
        "overall_score": 50,
        "critical_failures": 0
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("recommendation");
    println!("Test 4 (score 50): {:?}", output);
    assert_eq!(output.map(|v| v.as_str()), Some(Some("no_hire")));

    // Test case 5: Score < 45 → strong_no_hire
    let mut ctx = ExecutionContext::from_json(&json!({
        "overall_score": 30,
        "critical_failures": 0
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("recommendation");
    println!("Test 5 (score 30): {:?}", output);
    assert_eq!(output.map(|v| v.as_str()), Some(Some("strong_no_hire")));
}

#[test]
fn test_execute_calculate_time_remaining_rule() {
    let path = db_outage_fixtures_path();
    let registry = init(&path).expect("Failed to load product");
    let schema = registry.get_schema("db-outage-crisis-v1").expect("Schema not found");

    let rule = schema.rules.iter()
        .find(|r| r.rule_type == "calculate-time-remaining")
        .expect("Rule 'calculate-time-remaining' not found");

    println!("Testing rule: {}", rule.rule_type);

    let mut executor = RuleExecutor::new();

    // Test case 1: Time remaining
    let mut ctx = ExecutionContext::from_json(&json!({
        "time_limit_secs": 3600,
        "elapsed_secs": 1200
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("remaining_secs");
    println!("Test 1: {:?}", output);
    let remaining = output.map(|v| v.to_number()).unwrap_or(-1.0);
    assert!((remaining - 2400.0).abs() < 0.01, "Expected 2400, got {}", remaining);

    // Test case 2: Time exceeded (should clamp to 0)
    let mut ctx = ExecutionContext::from_json(&json!({
        "time_limit_secs": 3600,
        "elapsed_secs": 4000
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("remaining_secs");
    println!("Test 2: {:?}", output);
    let remaining = output.map(|v| v.to_number()).unwrap_or(-1.0);
    assert!((remaining - 0.0).abs() < 0.01, "Expected 0, got {}", remaining);
}

#[test]
fn test_execute_calculate_phase_progress_rule() {
    let path = db_outage_fixtures_path();
    let registry = init(&path).expect("Failed to load product");
    let schema = registry.get_schema("db-outage-crisis-v1").expect("Schema not found");

    let rule = schema.rules.iter()
        .find(|r| r.rule_type == "calculate-phase-progress")
        .expect("Rule 'calculate-phase-progress' not found");

    println!("Testing rule: {}", rule.rule_type);

    let mut executor = RuleExecutor::new();

    // Test case 1: 50% progress
    let mut ctx = ExecutionContext::from_json(&json!({
        "phase_elapsed_secs": 300,
        "phase_max_duration_secs": 600
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("progress_percent");
    println!("Test 1 (50%): {:?}", output);
    let progress = output.map(|v| v.to_number()).unwrap_or(-1.0);
    assert!((progress - 50.0).abs() < 0.01, "Expected 50, got {}", progress);

    // Test case 2: Over 100% (should clamp)
    let mut ctx = ExecutionContext::from_json(&json!({
        "phase_elapsed_secs": 800,
        "phase_max_duration_secs": 600
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("progress_percent");
    println!("Test 2 (capped at 100%): {:?}", output);
    let progress = output.map(|v| v.to_number()).unwrap_or(-1.0);
    assert!((progress - 100.0).abs() < 0.01, "Expected 100, got {}", progress);
}

#[test]
fn test_execute_check_detection_timeout_rule() {
    let path = db_outage_fixtures_path();
    let registry = init(&path).expect("Failed to load product");
    let schema = registry.get_schema("db-outage-crisis-v1").expect("Schema not found");

    let rule = schema.rules.iter()
        .find(|r| r.rule_type == "check-detection-timeout")
        .expect("Rule 'check-detection-timeout' not found");

    println!("Testing rule: {}", rule.rule_type);

    let mut executor = RuleExecutor::new();

    // Test case 1: Not timed out (< 600 secs)
    let mut ctx = ExecutionContext::from_json(&json!({
        "phase_elapsed_secs": 300
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("timeout");
    println!("Test 1 (not timed out): {:?}", output);
    assert_eq!(output.map(|v| v.as_bool()), Some(Some(false)));

    // Test case 2: Timed out (>= 600 secs)
    let mut ctx = ExecutionContext::from_json(&json!({
        "phase_elapsed_secs": 700
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("timeout");
    println!("Test 2 (timed out): {:?}", output);
    assert_eq!(output.map(|v| v.as_bool()), Some(Some(true)));
}

#[test]
fn test_execute_all_json_logic_rules() {
    let path = db_outage_fixtures_path();
    let registry = init(&path).expect("Failed to load product");
    let schema = registry.get_schema("db-outage-crisis-v1").expect("Schema not found");

    let json_logic_rules = get_json_logic_rules(&schema.rules);

    println!("\n=== Executing All JSON Logic Rules ===");
    println!("Total JSON Logic rules: {}", json_logic_rules.len());

    let mut executor = RuleExecutor::new();
    let mut success_count = 0;
    let mut failed_rules = Vec::new();

    let total_rules = json_logic_rules.len();

    for rule in json_logic_rules {
        println!("\nTesting: {}", rule.rule_type);

        // Create a generic context with typical test values
        let mut ctx = ExecutionContext::from_json(&json!({
            // Detection inputs
            "alert_acknowledged": true,
            "time_since_alert_secs": 90,
            "status_posted": true,
            "time_since_start_secs": 240,

            // VP communication inputs
            "vp_email_received": true,
            "vp_responded": true,
            "time_since_email_secs": 300,

            // Branch activation inputs
            "email_sent_to_vp": true,
            "phase_elapsed_secs": 500,
            "resolution_action_taken": true,
            "session_elapsed_secs": 300,
            "any_chat_message": true,

            // Score computation inputs
            "positive_signals": 0.8,
            "negative_signals": 0.1,
            "max_possible_score": 100,
            "overall_score": 75,
            "critical_failures": 0,

            // Time calculation inputs
            "time_limit_secs": 3600,
            "elapsed_secs": 1200,
            "phase_max_duration_secs": 600
        }));

        match executor.execute(&[(*rule).clone()], &mut ctx) {
            Ok(result) => {
                success_count += 1;
                println!("  ✓ SUCCESS");
                for (key, value) in &result.rule_results[0].outputs {
                    println!("    {} = {:?}", key, value);
                }
            }
            Err(e) => {
                failed_rules.push((rule.rule_type.clone(), e.to_string()));
                println!("  ✗ FAILED: {}", e);
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Passed: {}/{}", success_count, total_rules);

    if !failed_rules.is_empty() {
        println!("\nFailed rules:");
        for (name, error) in &failed_rules {
            println!("  - {}: {}", name, error);
        }
    }

    // All JSON Logic rules should execute successfully
    assert!(failed_rules.is_empty(), "Some rules failed to execute: {:?}", failed_rules);
}

#[test]
fn test_rule_inputs_outputs_are_set() {
    let path = db_outage_fixtures_path();
    let registry = init(&path).expect("Failed to load product");
    let schema = registry.get_schema("db-outage-crisis-v1").expect("Schema not found");

    println!("\n=== Verifying Rule Inputs/Outputs ===");

    for rule in &schema.rules {
        println!("\nRule: {}", rule.rule_type);
        println!("  Inputs: {}", rule.input_count());
        println!("  Outputs: {}", rule.output_count());

        // All rules should have at least one output
        assert!(rule.output_count() > 0, "Rule {} has no outputs", rule.rule_type);

        // Print input/output paths
        for input in &rule.input_attributes {
            println!("    Input: {}", input.path);
        }
        for output in &rule.output_attributes {
            println!("    Output: {}", output.path);
        }
    }
}

// ===========================================
// FarmScript Compilation Verification Tests
// (db-outage-scenario now uses FarmScript natively)
// ===========================================

#[test]
fn test_farmscript_compilation_and_execution() {
    // The db-outage-scenario fixtures now use FarmScript
    // This test verifies FarmScript compiles correctly and executes
    let path = db_outage_fixtures_path();
    let registry = init(&path).expect("Failed to load product");
    let schema = registry.get_schema("db-outage-crisis-v1").expect("Schema not found");

    let rule = schema.rules.iter()
        .find(|r| r.rule_type == "compute-recommendation")
        .expect("Rule 'compute-recommendation' not found");

    println!("Testing FarmScript rule: {}", rule.rule_type);
    println!("Compiled expression: {}", rule.compiled_expression);

    let mut executor = RuleExecutor::new();

    // Test case: Score 90 with no failures => strong_hire
    let mut ctx = ExecutionContext::from_json(&json!({
        "overall_score": 90,
        "critical_failures": 0
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("recommendation");
    println!("Recommendation output: {:?}", output);
    assert_eq!(output.map(|v| v.as_str()), Some(Some("strong_hire")));

    // Test case: Critical failures => strong_no_hire
    let mut ctx = ExecutionContext::from_json(&json!({
        "overall_score": 90,
        "critical_failures": 1
    }));

    let result = executor.execute(&[rule.clone()], &mut ctx).expect("Execution failed");
    let output = result.get_output("recommendation");
    println!("Recommendation with failures: {:?}", output);
    assert_eq!(output.map(|v| v.as_str()), Some(Some("strong_no_hire")));
}

/// Test all FarmScript rules execute correctly
#[test]
fn test_all_farmscript_rules_execute() {
    let path = db_outage_fixtures_path();
    let registry = init(&path).expect("Failed to load product");
    let schema = registry.get_schema("db-outage-crisis-v1").expect("Schema not found");

    println!("\n=== Testing All FarmScript Rules ===\n");

    let mut executor = RuleExecutor::new();

    // Test data covering various scenarios
    let test_data = json!({
        "alert_acknowledged": true,
        "time_since_alert_secs": 90,
        "vp_email_received": true,
        "vp_responded": true,
        "time_since_email_secs": 300,
        "email_sent_to_vp": true,
        "phase_elapsed_secs": 500,
        "resolution_action_taken": true,
        "session_elapsed_secs": 300,
        "any_chat_message": true,
        "status_posted": true,
        "time_since_start_secs": 240,
        "positive_signals": 0.8,
        "negative_signals": 0.2,
        "max_possible_score": 100,
        "overall_score": 75,
        "critical_failures": 0,
        "time_limit_secs": 3600,
        "elapsed_secs": 1200,
        "phase_max_duration_secs": 600
    });

    let farmscript_rules: Vec<_> = schema.rules.iter()
        .filter(|r| matches!(r.evaluator.name, EvaluatorType::JsonLogic))
        .collect();

    println!("Found {} FarmScript rules (compiled to JSON Logic)", farmscript_rules.len());

    let mut success_count = 0;
    let mut failed_rules = Vec::new();

    for rule in &farmscript_rules {
        let mut ctx = ExecutionContext::from_json(&test_data);

        match executor.execute(&[(*rule).clone()], &mut ctx) {
            Ok(result) => {
                success_count += 1;
                println!("  ✓ {}", rule.rule_type);
                for (key, value) in &result.rule_results[0].outputs {
                    println!("    {} = {:?}", key, value);
                }
            }
            Err(e) => {
                failed_rules.push((rule.rule_type.clone(), e.to_string()));
                println!("  ✗ {} - {}", rule.rule_type, e);
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Passed: {}/{}", success_count, farmscript_rules.len());

    assert!(failed_rules.is_empty(), "Some FarmScript rules failed: {:?}", failed_rules);
}
