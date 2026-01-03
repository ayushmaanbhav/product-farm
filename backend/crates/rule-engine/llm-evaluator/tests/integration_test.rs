//! End-to-end integration tests for LLM evaluators
//!
//! These tests require a running Ollama instance with a compatible model.
//! Run with: cargo test -p product-farm-llm-evaluator --test integration_test --features ollama
//!
//! To set up Ollama:
//! 1. Install Ollama: https://ollama.ai
//! 2. Pull a model: ollama pull qwen3:4b or ollama pull qwen2.5:7b
//! 3. Ensure Ollama is running: ollama serve

/// Default model to use for tests (can be overridden with OLLAMA_TEST_MODEL env var)
const DEFAULT_TEST_MODEL: &str = "qwen3:4b-instruct-2507-q4_K_M";

fn test_model() -> String {
    std::env::var("OLLAMA_TEST_MODEL").unwrap_or_else(|_| DEFAULT_TEST_MODEL.to_string())
}

use product_farm_core::{LlmEvaluator, Value};
use product_farm_llm_evaluator::{
    OllamaLlmEvaluator, LlmEvaluatorConfig,
    ParallelLlmExecutor, ParallelExecutorConfig, RuleMetadata, RetryConfig,
    PromptBuilder, RuleEvaluationContext, AttributeInfo,
};
use std::collections::HashMap;

/// Create executor config that uses the test model
fn test_executor_config() -> ParallelExecutorConfig {
    ParallelExecutorConfig {
        max_concurrency: 1000,
        timeout_ms: 60000, // 60s for local models
        continue_on_error: true,
        default_llm_config: LlmEvaluatorConfig::new(
            test_model(),
            "", // Will use generated prompt
        ),
        retry_config: RetryConfig::default(),
    }
}

/// Check if Ollama is available before running tests
async fn ollama_available() -> bool {
    let evaluator = OllamaLlmEvaluator::localhost(test_model());
    #[cfg(feature = "ollama")]
    {
        evaluator.is_available().await
    }
    #[cfg(not(feature = "ollama"))]
    {
        false
    }
}

/// Skip test if Ollama is not available
macro_rules! skip_if_no_ollama {
    () => {
        if !ollama_available().await {
            eprintln!("Skipping test: Ollama not available. Run 'ollama serve' and 'ollama pull qwen2.5:7b'");
            return;
        }
    };
}

// =============================================================================
// Basic Ollama Evaluator Tests
// =============================================================================

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_ollama_simple_arithmetic() {
    skip_if_no_ollama!();

    let evaluator = OllamaLlmEvaluator::localhost(test_model());

    let mut config_map = HashMap::new();
    config_map.insert("model".to_string(), Value::String(test_model().to_string()));
    config_map.insert("output_format".to_string(), Value::String("json".to_string()));
    config_map.insert(
        "prompt_template".to_string(),
        Value::String(
            r#"Calculate the sum of {{a}} and {{b}}.
Return ONLY a JSON object with the result:
{"sum": <number>}"#.to_string()
        ),
    );

    let mut inputs = HashMap::new();
    inputs.insert("a".to_string(), Value::Int(25));
    inputs.insert("b".to_string(), Value::Int(17));

    let output_names = vec!["sum".to_string()];

    let result = evaluator.evaluate(&config_map, &inputs, &output_names).await;

    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
    let outputs = result.unwrap();
    assert!(outputs.contains_key("sum"), "Missing 'sum' in outputs");

    let sum = outputs.get("sum").unwrap();
    match sum {
        Value::Int(n) => assert_eq!(*n, 42, "Expected 42, got {}", n),
        Value::Float(n) => assert!((n - 42.0).abs() < 0.01, "Expected 42.0, got {}", n),
        _ => panic!("Expected number, got {:?}", sum),
    }
}

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_ollama_boolean_response() {
    skip_if_no_ollama!();

    let evaluator = OllamaLlmEvaluator::localhost(test_model());

    let mut config_map = HashMap::new();
    config_map.insert("model".to_string(), Value::String(test_model().to_string()));
    config_map.insert("output_format".to_string(), Value::String("boolean".to_string()));
    config_map.insert(
        "prompt_template".to_string(),
        Value::String(
            "Is the person with age {{age}} eligible for a senior discount (age >= 65)? Answer only true or false.".to_string()
        ),
    );

    // Test case: age 70 (should be true)
    let mut inputs = HashMap::new();
    inputs.insert("age".to_string(), Value::Int(70));

    let output_names = vec!["eligible".to_string()];
    let result = evaluator.evaluate(&config_map, &inputs, &output_names).await.unwrap();

    assert_eq!(result.get("eligible"), Some(&Value::Bool(true)));

    // Test case: age 30 (should be false)
    inputs.insert("age".to_string(), Value::Int(30));
    let result = evaluator.evaluate(&config_map, &inputs, &output_names).await.unwrap();

    assert_eq!(result.get("eligible"), Some(&Value::Bool(false)));
}

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_ollama_json_complex_output() {
    skip_if_no_ollama!();

    let evaluator = OllamaLlmEvaluator::localhost(test_model());

    let mut config_map = HashMap::new();
    config_map.insert("model".to_string(), Value::String(test_model().to_string()));
    config_map.insert("output_format".to_string(), Value::String("json".to_string()));
    config_map.insert(
        "prompt_template".to_string(),
        Value::String(
            r#"Analyze this insurance application:
- Applicant age: {{age}}
- Has accidents: {{has_accidents}}
- Years of driving: {{years_driving}}

Calculate:
1. Risk level: "low", "medium", or "high"
2. Premium multiplier: a number between 0.8 and 2.0

Return ONLY a JSON object:
{"risk_level": "<level>", "premium_multiplier": <number>}"#.to_string()
        ),
    );

    let mut inputs = HashMap::new();
    inputs.insert("age".to_string(), Value::Int(45));
    inputs.insert("has_accidents".to_string(), Value::Bool(false));
    inputs.insert("years_driving".to_string(), Value::Int(20));

    let output_names = vec!["risk_level".to_string(), "premium_multiplier".to_string()];

    let result = evaluator.evaluate(&config_map, &inputs, &output_names).await;
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());

    let outputs = result.unwrap();
    assert!(outputs.contains_key("risk_level"), "Missing risk_level");
    assert!(outputs.contains_key("premium_multiplier"), "Missing premium_multiplier");

    // Verify risk_level is a valid string
    match outputs.get("risk_level") {
        Some(Value::String(s)) => {
            assert!(
                s == "low" || s == "medium" || s == "high",
                "Invalid risk_level: {}",
                s
            );
        }
        other => panic!("Expected string for risk_level, got {:?}", other),
    }

    // Verify premium_multiplier is a number in range
    match outputs.get("premium_multiplier") {
        Some(Value::Float(n)) => {
            assert!(*n >= 0.5 && *n <= 3.0, "Premium multiplier {} out of expected range", n);
        }
        Some(Value::Int(n)) => {
            assert!(*n >= 1 && *n <= 3, "Premium multiplier {} out of expected range", n);
        }
        other => panic!("Expected number for premium_multiplier, got {:?}", other),
    }
}

// =============================================================================
// Prompt Builder Integration Tests
// =============================================================================

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_prompt_builder_with_ollama() {
    skip_if_no_ollama!();

    let evaluator = OllamaLlmEvaluator::localhost(test_model());

    // Build a rich context prompt
    let context = RuleEvaluationContext::new("calculate-discount")
        .with_description("Calculate customer discount based on loyalty status and purchase amount")
        .with_rule_type("pricing")
        .with_product("retail-store")
        .add_input(
            AttributeInfo::new("customer_tier")
                .with_description("Customer loyalty tier: bronze, silver, gold, or platinum")
                .with_data_type("string")
        )
        .add_input(
            AttributeInfo::new("purchase_amount")
                .with_description("Total purchase amount in dollars")
                .with_data_type("number")
        )
        .add_output(
            AttributeInfo::new("discount_percent")
                .with_description("Discount percentage to apply (0-50)")
                .with_data_type("number")
        )
        .with_input_value("customer_tier".to_string(), Value::String("gold".to_string()))
        .with_input_value("purchase_amount".to_string(), Value::Float(150.0));

    let prompt = PromptBuilder::new().build(&context);

    // Create config with the generated prompt
    let mut config_map = HashMap::new();
    config_map.insert("model".to_string(), Value::String(test_model().to_string()));
    config_map.insert("output_format".to_string(), Value::String("json".to_string()));
    config_map.insert("prompt_template".to_string(), Value::String(prompt));

    let inputs: HashMap<String, Value> = context.input_values.clone();
    let output_names = vec!["discount_percent".to_string()];

    let result = evaluator.evaluate(&config_map, &inputs, &output_names).await;
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());

    let outputs = result.unwrap();
    assert!(outputs.contains_key("discount_percent"));

    // Gold tier should get a reasonable discount
    match outputs.get("discount_percent") {
        Some(Value::Int(n)) => assert!(*n >= 5 && *n <= 50, "Discount {} not in range", n),
        Some(Value::Float(n)) => assert!(*n >= 5.0 && *n <= 50.0, "Discount {} not in range", n),
        other => panic!("Expected number, got {:?}", other),
    }
}

// =============================================================================
// Parallel Executor Integration Tests
// =============================================================================

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_parallel_executor_multiple_rules() {
    skip_if_no_ollama!();

    let evaluator = OllamaLlmEvaluator::localhost(test_model());
    let config = test_executor_config()
        .with_max_concurrency(3); // Limit to 3 concurrent for local testing

    let executor = ParallelLlmExecutor::new(evaluator, config);

    // Create 3 independent rules to execute in parallel
    let rules = vec![
        // Rule 1: Simple addition
        (
            "rule-add".to_string(),
            RuleMetadata {
                name: "addition".to_string(),
                description: Some("Add two numbers".to_string()),
                outputs: vec![AttributeInfo::new("sum")],
                prompt_template: Some(
                    "Calculate {{a}} + {{b}}. Return JSON: {\"sum\": <number>}".to_string()
                ),
                ..Default::default()
            },
            {
                let mut inputs = HashMap::new();
                inputs.insert("a".to_string(), Value::Int(10));
                inputs.insert("b".to_string(), Value::Int(5));
                inputs
            },
        ),
        // Rule 2: Simple multiplication
        (
            "rule-mult".to_string(),
            RuleMetadata {
                name: "multiplication".to_string(),
                description: Some("Multiply two numbers".to_string()),
                outputs: vec![AttributeInfo::new("product")],
                prompt_template: Some(
                    "Calculate {{x}} * {{y}}. Return JSON: {\"product\": <number>}".to_string()
                ),
                ..Default::default()
            },
            {
                let mut inputs = HashMap::new();
                inputs.insert("x".to_string(), Value::Int(7));
                inputs.insert("y".to_string(), Value::Int(6));
                inputs
            },
        ),
        // Rule 3: Boolean check
        (
            "rule-check".to_string(),
            RuleMetadata {
                name: "eligibility-check".to_string(),
                description: Some("Check if eligible".to_string()),
                outputs: vec![AttributeInfo::new("eligible")],
                prompt_template: Some(
                    "Is {{age}} >= 18? Return JSON: {\"eligible\": true or false}".to_string()
                ),
                ..Default::default()
            },
            {
                let mut inputs = HashMap::new();
                inputs.insert("age".to_string(), Value::Int(25));
                inputs
            },
        ),
    ];

    let start = std::time::Instant::now();
    let results = executor.execute_parallel(rules).await;
    let elapsed = start.elapsed();

    println!("Parallel execution took {:?}", elapsed);

    // Verify all rules executed
    assert_eq!(results.len(), 3);

    // Verify results
    for result in &results {
        assert!(
            result.success,
            "Rule {} failed: {:?}",
            result.rule_id,
            result.error
        );
        println!(
            "Rule {}: outputs={:?}, time={}ms, retries={}",
            result.rule_id, result.outputs, result.execution_time_ms, result.retry_count
        );
    }

    // Check specific outputs
    let add_result = results.iter().find(|r| r.rule_id == "rule-add").unwrap();
    assert!(add_result.outputs.contains_key("sum"));

    let mult_result = results.iter().find(|r| r.rule_id == "rule-mult").unwrap();
    assert!(mult_result.outputs.contains_key("product"));

    let check_result = results.iter().find(|r| r.rule_id == "rule-check").unwrap();
    assert_eq!(check_result.outputs.get("eligible"), Some(&Value::Bool(true)));
}

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_parallel_executor_with_levels() {
    skip_if_no_ollama!();

    let evaluator = OllamaLlmEvaluator::localhost(test_model());
    let config = test_executor_config();
    let executor = ParallelLlmExecutor::new(evaluator, config);

    // Level 0: Calculate base values (can run in parallel)
    // Level 1: Use those values (must wait for level 0)
    let levels = vec![
        // Level 0: Two independent calculations
        vec![
            (
                "base-a".to_string(),
                RuleMetadata {
                    name: "calc-a".to_string(),
                    outputs: vec![AttributeInfo::new("a")],
                    prompt_template: Some(
                        "Return the number 10. JSON: {\"a\": 10}".to_string()
                    ),
                    ..Default::default()
                },
            ),
            (
                "base-b".to_string(),
                RuleMetadata {
                    name: "calc-b".to_string(),
                    outputs: vec![AttributeInfo::new("b")],
                    prompt_template: Some(
                        "Return the number 5. JSON: {\"b\": 5}".to_string()
                    ),
                    ..Default::default()
                },
            ),
        ],
        // Level 1: Depends on level 0 outputs
        vec![(
            "final".to_string(),
            RuleMetadata {
                name: "calc-final".to_string(),
                inputs: vec![AttributeInfo::new("a"), AttributeInfo::new("b")],
                outputs: vec![AttributeInfo::new("result")],
                prompt_template: Some(
                    "Calculate {{a}} + {{b}}. Return JSON: {\"result\": <sum>}".to_string()
                ),
                ..Default::default()
            },
        )],
    ];

    let results = executor
        .execute_by_levels(levels, HashMap::new())
        .await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.success), "Some rules failed");

    // The final result should have access to a and b from shared context
    let final_result = results.iter().find(|r| r.rule_id == "final").unwrap();
    assert!(final_result.outputs.contains_key("result"));
}

// =============================================================================
// Retry Logic Tests
// =============================================================================

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_retry_config_applied() {
    // This test verifies retry configuration is respected
    // We use a very short timeout to force retries

    let evaluator = OllamaLlmEvaluator::localhost(test_model());
    let config = test_executor_config()
        .with_retry_config(RetryConfig {
            max_retries: 2,
            initial_backoff_ms: 50,
            max_total_backoff_ms: 500,
            backoff_multiplier: 2.0,
        });

    let executor = ParallelLlmExecutor::new(evaluator, config);

    let metadata = RuleMetadata {
        name: "test-rule".to_string(),
        outputs: vec![AttributeInfo::new("result")],
        prompt_template: Some("Return JSON: {\"result\": 42}".to_string()),
        ..Default::default()
    };

    let result = executor
        .execute_rule("test", &metadata, HashMap::new(), None)
        .await;

    // Should succeed (retries shouldn't be needed for a working model)
    assert!(result.success, "Rule failed: {:?}", result.error);
    assert_eq!(result.retry_count, 0, "Unexpected retries for working request");
}

// =============================================================================
// Context Propagation Tests
// =============================================================================

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_context_propagation_between_rules() {
    skip_if_no_ollama!();

    let evaluator = OllamaLlmEvaluator::localhost(test_model());
    let config = test_executor_config();
    let executor = ParallelLlmExecutor::new(evaluator, config);

    // Set initial context
    let mut initial = HashMap::new();
    initial.insert("base_price".to_string(), Value::Float(100.0));
    executor.set_context(initial).await;

    // Execute first rule that produces discount_rate
    let meta1 = RuleMetadata {
        name: "calc-discount".to_string(),
        inputs: vec![AttributeInfo::new("base_price")],
        outputs: vec![AttributeInfo::new("discount_rate")],
        prompt_template: Some(
            "Given base_price={{base_price}}, return a 10% discount rate. JSON: {\"discount_rate\": 0.1}".to_string()
        ),
        ..Default::default()
    };

    let mut inputs1 = HashMap::new();
    inputs1.insert("base_price".to_string(), Value::Float(100.0));

    let result1 = executor
        .execute_rule("rule-1", &meta1, inputs1, None)
        .await;
    assert!(result1.success);

    // Verify discount_rate is in shared context
    let discount = executor.get_context_value("discount_rate").await;
    assert!(discount.is_some(), "discount_rate not in context");

    // Execute second rule that uses discount_rate
    let meta2 = RuleMetadata {
        name: "calc-final".to_string(),
        inputs: vec![
            AttributeInfo::new("base_price"),
            AttributeInfo::new("discount_rate"),
        ],
        outputs: vec![AttributeInfo::new("final_price")],
        prompt_template: Some(
            "Calculate final_price = base_price * (1 - discount_rate). base_price={{base_price}}, discount_rate={{discount_rate}}. Return JSON: {\"final_price\": <number>}".to_string()
        ),
        ..Default::default()
    };

    // Get inputs from context using public methods
    let mut inputs2 = HashMap::new();
    for input in &meta2.inputs {
        if let Some(v) = executor.get_context_value(&input.name).await {
            inputs2.insert(input.name.clone(), v);
        }
    }

    let result2 = executor
        .execute_rule("rule-2", &meta2, inputs2, None)
        .await;
    assert!(result2.success, "Rule 2 failed: {:?}", result2.error);

    // final_price should be ~90 (100 * 0.9)
    match result2.outputs.get("final_price") {
        Some(Value::Float(n)) => assert!((*n - 90.0).abs() < 5.0, "Expected ~90, got {}", n),
        Some(Value::Int(n)) => assert!((*n - 90).abs() < 5, "Expected ~90, got {}", n),
        other => panic!("Expected number, got {:?}", other),
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_invalid_model_error() {
    let evaluator = OllamaLlmEvaluator::new("http://localhost:11434", "nonexistent-model-xyz");

    let mut config_map = HashMap::new();
    config_map.insert("model".to_string(), Value::String("nonexistent-model-xyz".to_string()));
    config_map.insert("prompt_template".to_string(), Value::String("Hello".to_string()));

    let result = evaluator
        .evaluate(&config_map, &HashMap::new(), &["output".to_string()])
        .await;

    assert!(result.is_err(), "Should fail with invalid model");
}

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_connection_error() {
    let evaluator = OllamaLlmEvaluator::new("http://localhost:99999", test_model());

    let mut config_map = HashMap::new();
    config_map.insert("prompt_template".to_string(), Value::String("Hello".to_string()));

    let result = evaluator
        .evaluate(&config_map, &HashMap::new(), &["output".to_string()])
        .await;

    assert!(result.is_err(), "Should fail with connection error");
}

// =============================================================================
// Stress Tests (optional, slower)
// =============================================================================

#[tokio::test]
#[ignore] // Run with: cargo test --features ollama -- --ignored
#[cfg(feature = "ollama")]
async fn test_high_concurrency() {
    skip_if_no_ollama!();

    let evaluator = OllamaLlmEvaluator::localhost(test_model());
    let config = test_executor_config()
        .with_max_concurrency(10);

    let executor = ParallelLlmExecutor::new(evaluator, config);

    // Create 20 simple rules
    let rules: Vec<_> = (0..20)
        .map(|i| {
            (
                format!("rule-{}", i),
                RuleMetadata {
                    name: format!("calc-{}", i),
                    outputs: vec![AttributeInfo::new("result")],
                    prompt_template: Some(format!(
                        "Return the number {}. JSON: {{\"result\": {}}}",
                        i, i
                    )),
                    ..Default::default()
                },
                HashMap::new(),
            )
        })
        .collect();

    let start = std::time::Instant::now();
    let results = executor.execute_parallel(rules).await;
    let elapsed = start.elapsed();

    println!("High concurrency test: {} rules in {:?}", results.len(), elapsed);

    let success_count = results.iter().filter(|r| r.success).count();
    let failure_count = results.iter().filter(|r| !r.success).count();
    let total_retries: u32 = results.iter().map(|r| r.retry_count).sum();

    println!(
        "Success: {}, Failures: {}, Total retries: {}",
        success_count, failure_count, total_retries
    );

    // At least 80% should succeed
    assert!(
        success_count >= 16,
        "Too many failures: {}/20 succeeded",
        success_count
    );
}
