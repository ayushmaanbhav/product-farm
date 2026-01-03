//! Concurrent Stress Tests for LLM Rule Evaluation
//!
//! Tests thread safety, scalability, and concurrent evaluation with:
//! - 10 LLM-based rules (Ollama)
//! - Complex interdependent rule chains
//! - Multiple concurrent execution contexts
//! - Thread safety verification
//!
//! Run with: cargo test -p product-farm-llm-evaluator --test concurrent_stress_test --features ollama -- --nocapture
//!
//! NOTE: For JSON Logic + FarmScript concurrent tests (1000 rules), see:
//!       cargo test -p product-farm-rule-engine --test concurrent_1000_rules_test

use product_farm_core::Value;
use product_farm_llm_evaluator::{
    AttributeInfo, LlmEvaluatorConfig, OllamaLlmEvaluator, ParallelExecutorConfig,
    ParallelLlmExecutor, RetryConfig, RuleMetadata,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Default model for tests (can override with OLLAMA_TEST_MODEL env var)
const DEFAULT_TEST_MODEL: &str = "qwen3:4b-instruct-2507-q4_K_M";

fn test_model() -> String {
    std::env::var("OLLAMA_TEST_MODEL").unwrap_or_else(|_| DEFAULT_TEST_MODEL.to_string())
}

/// Check if Ollama is available
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

macro_rules! skip_if_no_ollama {
    () => {
        if !ollama_available().await {
            eprintln!(
                "Skipping test: Ollama not available. Run 'ollama serve' and 'ollama pull {}'",
                test_model()
            );
            return;
        }
    };
}

/// Create executor config with specified concurrency
fn test_executor_config(max_concurrency: usize) -> ParallelExecutorConfig {
    ParallelExecutorConfig {
        max_concurrency,
        timeout_ms: 60000, // 60s for LLM calls
        continue_on_error: true,
        default_llm_config: LlmEvaluatorConfig::new(test_model(), ""),
        retry_config: RetryConfig {
            max_retries: 2,
            initial_backoff_ms: 100,
            max_total_backoff_ms: 2000,
            backoff_multiplier: 2.0,
        },
    }
}

// =============================================================================
// 10 LLM Rule Definitions
// =============================================================================

/// Generate 10 diverse LLM rules for testing
fn generate_10_llm_rules() -> Vec<(String, RuleMetadata, HashMap<String, Value>)> {
    vec![
        // Rule 1: Simple arithmetic
        (
            "llm-add".to_string(),
            RuleMetadata {
                name: "addition".to_string(),
                description: Some("Add two numbers".to_string()),
                outputs: vec![AttributeInfo::new("sum")],
                prompt_template: Some(
                    "Calculate 25 + 17. Return ONLY JSON: {\"sum\": <number>}".to_string()
                ),
                ..Default::default()
            },
            HashMap::new(),
        ),
        // Rule 2: Boolean classification
        (
            "llm-classify".to_string(),
            RuleMetadata {
                name: "classifier".to_string(),
                description: Some("Classify age eligibility".to_string()),
                inputs: vec![AttributeInfo::new("age")],
                outputs: vec![AttributeInfo::new("eligible")],
                prompt_template: Some(
                    "Is age {{age}} >= 18 for adult status? Return JSON: {\"eligible\": true or false}".to_string()
                ),
                ..Default::default()
            },
            {
                let mut m = HashMap::new();
                m.insert("age".to_string(), Value::Int(25));
                m
            },
        ),
        // Rule 3: Category assignment
        (
            "llm-category".to_string(),
            RuleMetadata {
                name: "categorizer".to_string(),
                description: Some("Assign priority category".to_string()),
                inputs: vec![AttributeInfo::new("score")],
                outputs: vec![AttributeInfo::new("priority")],
                prompt_template: Some(
                    "Score is {{score}}. If > 80: high, if > 50: medium, else: low. Return JSON: {\"priority\": \"<level>\"}".to_string()
                ),
                ..Default::default()
            },
            {
                let mut m = HashMap::new();
                m.insert("score".to_string(), Value::Int(75));
                m
            },
        ),
        // Rule 4: Risk assessment
        (
            "llm-risk".to_string(),
            RuleMetadata {
                name: "risk-assessor".to_string(),
                description: Some("Assess risk level".to_string()),
                inputs: vec![
                    AttributeInfo::new("value"),
                    AttributeInfo::new("threshold"),
                ],
                outputs: vec![AttributeInfo::new("risk_level")],
                prompt_template: Some(
                    "Value={{value}}, Threshold={{threshold}}. If value > threshold*1.5: critical, if > threshold: elevated, else: normal. Return JSON: {\"risk_level\": \"<level>\"}".to_string()
                ),
                ..Default::default()
            },
            {
                let mut m = HashMap::new();
                m.insert("value".to_string(), Value::Int(120));
                m.insert("threshold".to_string(), Value::Int(100));
                m
            },
        ),
        // Rule 5: Sentiment analysis (simulated)
        (
            "llm-sentiment".to_string(),
            RuleMetadata {
                name: "sentiment-analyzer".to_string(),
                description: Some("Analyze sentiment".to_string()),
                inputs: vec![AttributeInfo::new("text")],
                outputs: vec![
                    AttributeInfo::new("sentiment"),
                    AttributeInfo::new("confidence"),
                ],
                prompt_template: Some(
                    "Analyze sentiment of: \"{{text}}\". Return JSON: {\"sentiment\": \"positive\"|\"negative\"|\"neutral\", \"confidence\": 0.0-1.0}".to_string()
                ),
                ..Default::default()
            },
            {
                let mut m = HashMap::new();
                m.insert("text".to_string(), Value::String("This product is excellent!".to_string()));
                m
            },
        ),
        // Rule 6: Numeric scoring
        (
            "llm-score".to_string(),
            RuleMetadata {
                name: "scorer".to_string(),
                description: Some("Calculate composite score".to_string()),
                inputs: vec![
                    AttributeInfo::new("quality"),
                    AttributeInfo::new("speed"),
                ],
                outputs: vec![AttributeInfo::new("total_score")],
                prompt_template: Some(
                    "Quality={{quality}}, Speed={{speed}}. Calculate weighted score: (quality*0.6 + speed*0.4). Return JSON: {\"total_score\": <number>}".to_string()
                ),
                ..Default::default()
            },
            {
                let mut m = HashMap::new();
                m.insert("quality".to_string(), Value::Int(85));
                m.insert("speed".to_string(), Value::Int(70));
                m
            },
        ),
        // Rule 7: Decision making
        (
            "llm-decision".to_string(),
            RuleMetadata {
                name: "decision-maker".to_string(),
                description: Some("Make approval decision".to_string()),
                inputs: vec![
                    AttributeInfo::new("amount"),
                    AttributeInfo::new("credit_score"),
                ],
                outputs: vec![
                    AttributeInfo::new("approved"),
                    AttributeInfo::new("reason"),
                ],
                prompt_template: Some(
                    "Loan: amount={{amount}}, credit_score={{credit_score}}. Approve if credit_score >= 650 and amount <= 50000. Return JSON: {\"approved\": true/false, \"reason\": \"<brief reason>\"}".to_string()
                ),
                ..Default::default()
            },
            {
                let mut m = HashMap::new();
                m.insert("amount".to_string(), Value::Int(25000));
                m.insert("credit_score".to_string(), Value::Int(720));
                m
            },
        ),
        // Rule 8: Comparison
        (
            "llm-compare".to_string(),
            RuleMetadata {
                name: "comparator".to_string(),
                description: Some("Compare two values".to_string()),
                inputs: vec![
                    AttributeInfo::new("a"),
                    AttributeInfo::new("b"),
                ],
                outputs: vec![
                    AttributeInfo::new("greater"),
                    AttributeInfo::new("difference"),
                ],
                prompt_template: Some(
                    "Compare a={{a}} and b={{b}}. Return JSON: {\"greater\": \"a\"|\"b\"|\"equal\", \"difference\": <absolute difference>}".to_string()
                ),
                ..Default::default()
            },
            {
                let mut m = HashMap::new();
                m.insert("a".to_string(), Value::Int(45));
                m.insert("b".to_string(), Value::Int(38));
                m
            },
        ),
        // Rule 9: Text extraction
        (
            "llm-extract".to_string(),
            RuleMetadata {
                name: "extractor".to_string(),
                description: Some("Extract key information".to_string()),
                inputs: vec![AttributeInfo::new("description")],
                outputs: vec![
                    AttributeInfo::new("category"),
                    AttributeInfo::new("urgency"),
                ],
                prompt_template: Some(
                    "From description: \"{{description}}\", extract category (bug/feature/question) and urgency (1-5). Return JSON: {\"category\": \"<type>\", \"urgency\": <1-5>}".to_string()
                ),
                ..Default::default()
            },
            {
                let mut m = HashMap::new();
                m.insert("description".to_string(), Value::String("Critical bug: system crashes on login".to_string()));
                m
            },
        ),
        // Rule 10: Multi-factor evaluation
        (
            "llm-evaluate".to_string(),
            RuleMetadata {
                name: "evaluator".to_string(),
                description: Some("Multi-factor evaluation".to_string()),
                inputs: vec![
                    AttributeInfo::new("experience"),
                    AttributeInfo::new("skills"),
                    AttributeInfo::new("interview_score"),
                ],
                outputs: vec![
                    AttributeInfo::new("recommendation"),
                    AttributeInfo::new("overall_score"),
                ],
                prompt_template: Some(
                    "Candidate: experience={{experience}}yrs, skills={{skills}}/10, interview={{interview_score}}/100. Recommend: strong_hire (>85), hire (>70), no_hire (>50), strong_no_hire. Calculate overall = experience*5 + skills*3 + interview_score*0.5. Return JSON: {\"recommendation\": \"<rec>\", \"overall_score\": <number>}".to_string()
                ),
                ..Default::default()
            },
            {
                let mut m = HashMap::new();
                m.insert("experience".to_string(), Value::Int(5));
                m.insert("skills".to_string(), Value::Int(8));
                m.insert("interview_score".to_string(), Value::Int(85));
                m
            },
        ),
    ]
}

// =============================================================================
// Concurrent LLM Tests
// =============================================================================

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_10_llm_rules_concurrent() {
    skip_if_no_ollama!();

    let evaluator = OllamaLlmEvaluator::localhost(test_model());
    let config = test_executor_config(10); // All 10 concurrent
    let executor = ParallelLlmExecutor::new(evaluator, config);

    let rules = generate_10_llm_rules();

    println!("\n=== 10 LLM Rules Concurrent Execution ===");
    println!("Model: {}", test_model());
    println!("Max concurrency: 10");

    let start = std::time::Instant::now();
    let results = executor.execute_parallel(rules).await;
    let elapsed = start.elapsed();

    let success_count = results.iter().filter(|r| r.success).count();
    let failure_count = results.iter().filter(|r| !r.success).count();
    let total_retries: u32 = results.iter().map(|r| r.retry_count).sum();

    println!("\nResults:");
    println!("  Successes: {}/10", success_count);
    println!("  Failures: {}/10", failure_count);
    println!("  Total retries: {}", total_retries);
    println!("  Total time: {:?}", elapsed);
    println!("  Avg per rule: {:?}", elapsed / 10);

    // Print individual results
    println!("\nIndividual rule results:");
    for result in &results {
        if result.success {
            println!("  {} ✓ ({}ms): {:?}",
                result.rule_id,
                result.execution_time_ms,
                result.outputs.keys().collect::<Vec<_>>()
            );
        } else {
            println!("  {} ✗ ({}ms): {:?}",
                result.rule_id,
                result.execution_time_ms,
                result.error
            );
        }
    }

    // At least 80% should succeed
    assert!(
        success_count >= 8,
        "Too many failures: only {}/10 succeeded",
        success_count
    );
}

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_llm_rules_sequential_vs_parallel() {
    skip_if_no_ollama!();

    let rules = generate_10_llm_rules();

    // Sequential (concurrency = 1)
    let evaluator_seq = OllamaLlmEvaluator::localhost(test_model());
    let config_seq = test_executor_config(1);
    let executor_seq = ParallelLlmExecutor::new(evaluator_seq, config_seq);

    println!("\n=== Sequential vs Parallel Comparison ===");

    let seq_start = std::time::Instant::now();
    let seq_results = executor_seq.execute_parallel(rules.clone()).await;
    let seq_elapsed = seq_start.elapsed();

    let seq_success = seq_results.iter().filter(|r| r.success).count();
    println!("Sequential (concurrency=1): {}/10 in {:?}", seq_success, seq_elapsed);

    // Parallel (concurrency = 5)
    let evaluator_par = OllamaLlmEvaluator::localhost(test_model());
    let config_par = test_executor_config(5);
    let executor_par = ParallelLlmExecutor::new(evaluator_par, config_par);

    let par_start = std::time::Instant::now();
    let par_results = executor_par.execute_parallel(rules).await;
    let par_elapsed = par_start.elapsed();

    let par_success = par_results.iter().filter(|r| r.success).count();
    println!("Parallel (concurrency=5): {}/10 in {:?}", par_success, par_elapsed);

    // Parallel should be faster (unless Ollama is resource-constrained)
    if seq_elapsed > par_elapsed {
        println!("Speedup: {:.2}x", seq_elapsed.as_secs_f64() / par_elapsed.as_secs_f64());
    }
}

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_llm_thread_safety_shared_executor() {
    skip_if_no_ollama!();

    let evaluator = OllamaLlmEvaluator::localhost(test_model());
    let config = test_executor_config(10);
    let executor = Arc::new(ParallelLlmExecutor::new(evaluator, config));

    let completed = Arc::new(AtomicUsize::new(0));
    let success = Arc::new(AtomicUsize::new(0));

    // Spawn 5 tasks, each executing 2 LLM rules
    let mut handles = Vec::new();

    for task_id in 0..5 {
        let exec = executor.clone();
        let completed = completed.clone();
        let success = success.clone();

        let handle = tokio::spawn(async move {
            let rules: Vec<_> = vec![
                (
                    format!("task-{}-add", task_id),
                    RuleMetadata {
                        name: "addition".to_string(),
                        outputs: vec![AttributeInfo::new("sum")],
                        prompt_template: Some(format!(
                            "Calculate {} + 10. Return JSON: {{\"sum\": <number>}}",
                            task_id * 5
                        )),
                        ..Default::default()
                    },
                    HashMap::new(),
                ),
                (
                    format!("task-{}-check", task_id),
                    RuleMetadata {
                        name: "checker".to_string(),
                        outputs: vec![AttributeInfo::new("valid")],
                        prompt_template: Some(
                            "Is 42 the answer? Return JSON: {\"valid\": true}".to_string()
                        ),
                        ..Default::default()
                    },
                    HashMap::new(),
                ),
            ];

            let results = exec.execute_parallel(rules).await;

            for result in results {
                completed.fetch_add(1, Ordering::SeqCst);
                if result.success {
                    success.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Task panicked");
    }

    let total_completed = completed.load(Ordering::SeqCst);
    let total_success = success.load(Ordering::SeqCst);

    println!("\n=== Thread Safety Test ===");
    println!("5 tasks x 2 rules = 10 total executions");
    println!("Completed: {}/10", total_completed);
    println!("Successes: {}/10", total_success);

    assert_eq!(total_completed, 10);
    assert!(total_success >= 7, "Too many failures: {}/10", total_success);
}

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_llm_context_isolation() {
    skip_if_no_ollama!();

    // Two executors with different contexts
    let evaluator1 = OllamaLlmEvaluator::localhost(test_model());
    let evaluator2 = OllamaLlmEvaluator::localhost(test_model());

    let config = test_executor_config(5);
    let executor1 = Arc::new(ParallelLlmExecutor::new(evaluator1, config.clone()));
    let executor2 = Arc::new(ParallelLlmExecutor::new(evaluator2, config));

    // Set different contexts
    let mut ctx1 = HashMap::new();
    ctx1.insert("source".to_string(), Value::String("executor1".to_string()));
    executor1.set_context(ctx1).await;

    let mut ctx2 = HashMap::new();
    ctx2.insert("source".to_string(), Value::String("executor2".to_string()));
    executor2.set_context(ctx2).await;

    // Execute in parallel
    let exec1 = executor1.clone();
    let exec2 = executor2.clone();

    let handle1 = tokio::spawn(async move {
        let rules = vec![(
            "ctx1-rule".to_string(),
            RuleMetadata {
                name: "test".to_string(),
                outputs: vec![AttributeInfo::new("result")],
                prompt_template: Some("Return JSON: {\"result\": 1}".to_string()),
                ..Default::default()
            },
            HashMap::new(),
        )];
        exec1.execute_parallel(rules).await
    });

    let handle2 = tokio::spawn(async move {
        let rules = vec![(
            "ctx2-rule".to_string(),
            RuleMetadata {
                name: "test".to_string(),
                outputs: vec![AttributeInfo::new("result")],
                prompt_template: Some("Return JSON: {\"result\": 2}".to_string()),
                ..Default::default()
            },
            HashMap::new(),
        )];
        exec2.execute_parallel(rules).await
    });

    let _ = handle1.await;
    let _ = handle2.await;

    // Verify contexts are isolated
    let src1 = executor1.get_context_value("source").await;
    let src2 = executor2.get_context_value("source").await;

    println!("\n=== Context Isolation Test ===");
    println!("Executor1 source: {:?}", src1);
    println!("Executor2 source: {:?}", src2);

    assert_eq!(src1, Some(Value::String("executor1".to_string())));
    assert_eq!(src2, Some(Value::String("executor2".to_string())));
}

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_llm_interdependent_levels() {
    skip_if_no_ollama!();

    let evaluator = OllamaLlmEvaluator::localhost(test_model());
    let config = test_executor_config(5);
    let executor = ParallelLlmExecutor::new(evaluator, config);

    // Level 0: Two independent calculations
    // Level 1: One rule that uses both outputs

    let levels = vec![
        // Level 0
        vec![
            (
                "base-a".to_string(),
                RuleMetadata {
                    name: "calc-a".to_string(),
                    outputs: vec![AttributeInfo::new("a")],
                    prompt_template: Some("Return JSON: {\"a\": 10}".to_string()),
                    ..Default::default()
                },
            ),
            (
                "base-b".to_string(),
                RuleMetadata {
                    name: "calc-b".to_string(),
                    outputs: vec![AttributeInfo::new("b")],
                    prompt_template: Some("Return JSON: {\"b\": 20}".to_string()),
                    ..Default::default()
                },
            ),
        ],
        // Level 1
        vec![(
            "final".to_string(),
            RuleMetadata {
                name: "calc-final".to_string(),
                inputs: vec![AttributeInfo::new("a"), AttributeInfo::new("b")],
                outputs: vec![AttributeInfo::new("sum")],
                prompt_template: Some(
                    "Calculate {{a}} + {{b}}. Return JSON: {\"sum\": <number>}".to_string(),
                ),
                ..Default::default()
            },
        )],
    ];

    println!("\n=== Interdependent Levels Test ===");
    let start = std::time::Instant::now();

    let results = executor.execute_by_levels(levels, HashMap::new()).await;

    let elapsed = start.elapsed();
    let success_count = results.iter().filter(|r| r.success).count();

    println!("Total results: {}", results.len());
    println!("Successes: {}/3", success_count);
    println!("Time: {:?}", elapsed);

    for result in &results {
        println!("  {}: {:?}", result.rule_id, result.outputs);
    }

    assert!(success_count >= 2, "Expected at least 2 successes");
}

// =============================================================================
// Retry and Error Handling
// =============================================================================

#[tokio::test]
#[cfg(feature = "ollama")]
async fn test_retry_behavior() {
    skip_if_no_ollama!();

    let evaluator = OllamaLlmEvaluator::localhost(test_model());
    let config = test_executor_config(5)
        .with_retry_config(RetryConfig {
            max_retries: 2,
            initial_backoff_ms: 50,
            max_total_backoff_ms: 500,
            backoff_multiplier: 2.0,
        });

    let executor = ParallelLlmExecutor::new(evaluator, config);

    let meta = RuleMetadata {
        name: "test-retry".to_string(),
        outputs: vec![AttributeInfo::new("result")],
        prompt_template: Some("Return JSON: {\"result\": 42}".to_string()),
        ..Default::default()
    };

    let result = executor
        .execute_rule("test", &meta, HashMap::new(), None)
        .await;

    println!("\n=== Retry Behavior Test ===");
    println!("Success: {}", result.success);
    println!("Retry count: {}", result.retry_count);
    println!("Time: {}ms", result.execution_time_ms);

    // Should succeed without retries for a valid request
    assert!(result.success);
    assert_eq!(result.retry_count, 0);
}
