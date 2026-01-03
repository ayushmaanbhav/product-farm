//! Async Parallel Executor for LLM Rules
//!
//! Executes LLM-based rules in parallel with:
//! - No artificial concurrency limits (very relaxed default)
//! - Exponential backoff retry on failures (max 3s)
//! - Rich tracing and metrics for observability

use crate::config::LlmEvaluatorConfig;
use crate::prompt::{AttributeInfo, PromptBuilder, RuleEvaluationContext, default_system_prompt};
use product_farm_core::{CoreError, LlmEvaluator, Rule, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error, instrument};

/// Check if a CoreError indicates a retryable condition.
///
/// Retryable conditions (from LLM error patterns):
/// - Network errors: connection failures, DNS issues
/// - Rate limit errors: 429 status codes
/// - Server errors: 5xx status codes
/// - Timeout errors
///
/// Non-retryable conditions:
/// - Parse errors: malformed response
/// - Config errors: invalid configuration
/// - API errors: 4xx (except 429)
fn is_error_retryable(err: &CoreError) -> bool {
    match err {
        CoreError::Internal(msg) => {
            let msg_lower = msg.to_lowercase();
            // Check for retryable patterns
            msg_lower.contains("network")
                || msg_lower.contains("rate limit")
                || msg_lower.contains("429")
                || msg_lower.contains("server error")
                || msg_lower.contains("500")
                || msg_lower.contains("502")
                || msg_lower.contains("503")
                || msg_lower.contains("504")
                || msg_lower.contains("timeout")
                || msg_lower.contains("connection")
                || msg_lower.contains("dns")
        }
        // Other CoreError variants are generally not retryable
        _ => false,
    }
}

/// Result of a single LLM rule evaluation
#[derive(Debug, Clone)]
pub struct LlmRuleResult {
    /// Rule identifier
    pub rule_id: String,
    /// Computed output values
    pub outputs: HashMap<String, Value>,
    /// Total execution time in milliseconds (including retries)
    pub execution_time_ms: u64,
    /// Whether evaluation succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Number of retry attempts made
    pub retry_count: u32,
}

/// Retry configuration with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,
    /// Maximum total backoff time in milliseconds
    pub max_total_backoff_ms: u64,
    /// Backoff multiplier (e.g., 2.0 for doubling)
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        // Load from env_config defaults
        let env_retry = crate::env_config::RetryConfig::default();
        Self {
            max_retries: env_retry.max_retries,
            initial_backoff_ms: env_retry.initial_backoff.as_millis() as u64,
            max_total_backoff_ms: env_retry.max_total_backoff.as_millis() as u64,
            backoff_multiplier: env_retry.multiplier,
        }
    }
}

impl RetryConfig {
    /// Create from environment configuration
    pub fn from_env() -> Self {
        let env_retry = crate::env_config::RetryConfig::from_env();
        Self {
            max_retries: env_retry.max_retries,
            initial_backoff_ms: env_retry.initial_backoff.as_millis() as u64,
            max_total_backoff_ms: env_retry.max_total_backoff.as_millis() as u64,
            backoff_multiplier: env_retry.multiplier,
        }
    }

    /// Calculate backoff duration for a given attempt (0-indexed)
    pub fn backoff_for_attempt(&self, attempt: u32) -> Duration {
        let backoff_ms = self.initial_backoff_ms as f64
            * self.backoff_multiplier.powi(attempt as i32);
        Duration::from_millis(backoff_ms.min(self.max_total_backoff_ms as f64) as u64)
    }
}

/// Metadata for rule evaluation
#[derive(Debug, Clone, Default)]
pub struct RuleMetadata {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: Option<String>,
    /// Rule type
    pub rule_type: Option<String>,
    /// Product name
    pub product_name: Option<String>,
    /// Input attribute metadata
    pub inputs: Vec<AttributeInfo>,
    /// Output attribute metadata
    pub outputs: Vec<AttributeInfo>,
    /// Custom prompt template (overrides default)
    pub prompt_template: Option<String>,
    /// Custom system prompt
    pub system_prompt: Option<String>,
}

impl RuleMetadata {
    pub fn from_rule(rule: &Rule) -> Self {
        let inputs: Vec<AttributeInfo> = rule
            .input_attributes
            .iter()
            .map(|attr| AttributeInfo::new(attr.path.as_str()))
            .collect();

        let outputs: Vec<AttributeInfo> = rule
            .output_attributes
            .iter()
            .map(|attr| AttributeInfo::new(attr.path.as_str()))
            .collect();

        Self {
            name: rule.rule_type.clone(),
            description: rule.description.clone(),
            rule_type: Some(rule.rule_type.clone()),
            product_name: Some(rule.product_id.0.clone()),
            inputs,
            outputs,
            prompt_template: None,
            system_prompt: None,
        }
    }

    pub fn with_prompt_template(mut self, template: impl Into<String>) -> Self {
        self.prompt_template = Some(template.into());
        self
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn with_input_descriptions(mut self, descriptions: HashMap<String, String>) -> Self {
        for input in &mut self.inputs {
            if let Some(desc) = descriptions.get(&input.name) {
                input.description = Some(desc.clone());
            }
        }
        self
    }

    pub fn with_output_descriptions(mut self, descriptions: HashMap<String, String>) -> Self {
        for output in &mut self.outputs {
            if let Some(desc) = descriptions.get(&output.name) {
                output.description = Some(desc.clone());
            }
        }
        self
    }
}

/// Configuration for the parallel executor
#[derive(Debug, Clone)]
pub struct ParallelExecutorConfig {
    /// Maximum concurrent LLM requests (very relaxed by default)
    pub max_concurrency: usize,
    /// Timeout per rule evaluation in milliseconds
    pub timeout_ms: u64,
    /// Whether to continue on individual rule failures
    pub continue_on_error: bool,
    /// Default LLM configuration
    pub default_llm_config: LlmEvaluatorConfig,
    /// Retry configuration with exponential backoff
    pub retry_config: RetryConfig,
}

impl Default for ParallelExecutorConfig {
    fn default() -> Self {
        // Load from environment config
        let env_config = crate::env_config::RuleEngineLlmConfig::from_env();
        let provider = env_config.default_provider();

        Self {
            // Use provider-specific concurrency
            max_concurrency: env_config.max_concurrency_for(provider),
            timeout_ms: env_config.timeout_for(provider).as_millis() as u64,
            continue_on_error: true,
            default_llm_config: LlmEvaluatorConfig::new(
                Self::default_model_for_provider(provider),
                "", // Will use generated prompt
            )
            .with_temperature(env_config.temperature_for(provider))
            .with_max_tokens(env_config.max_output_tokens_for(provider))
            .with_provider(provider),
            retry_config: RetryConfig::from_env(),
        }
    }
}

impl ParallelExecutorConfig {
    /// Get default model for a provider
    fn default_model_for_provider(provider: &str) -> String {
        let env_config = crate::env_config::RuleEngineLlmConfig::from_env();
        match provider.to_lowercase().as_str() {
            "anthropic" | "claude" => env_config.anthropic.model.clone(),
            "ollama" => env_config.ollama.model.clone(),
            _ => env_config.ollama.model.clone(),
        }
    }

    /// Create config for a specific provider from environment
    pub fn for_provider(provider: &str) -> Self {
        let env_config = crate::env_config::RuleEngineLlmConfig::from_env();

        Self {
            max_concurrency: env_config.max_concurrency_for(provider),
            timeout_ms: env_config.timeout_for(provider).as_millis() as u64,
            continue_on_error: true,
            default_llm_config: LlmEvaluatorConfig::new(
                Self::default_model_for_provider(provider),
                "",
            )
            .with_temperature(env_config.temperature_for(provider))
            .with_max_tokens(env_config.max_output_tokens_for(provider))
            .with_provider(provider),
            retry_config: RetryConfig::from_env(),
        }
    }

    /// Create with unlimited concurrency
    pub fn unlimited() -> Self {
        Self {
            max_concurrency: usize::MAX,
            ..Default::default()
        }
    }

    /// Set max concurrency
    pub fn with_max_concurrency(mut self, max: usize) -> Self {
        self.max_concurrency = max;
        self
    }

    /// Set retry configuration
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Disable retries
    pub fn without_retries(mut self) -> Self {
        self.retry_config.max_retries = 0;
        self
    }
}

/// Async parallel executor for LLM rules
pub struct ParallelLlmExecutor<E: LlmEvaluator> {
    evaluator: Arc<E>,
    config: ParallelExecutorConfig,
    /// Shared context for all evaluations (thread-safe)
    shared_context: Arc<RwLock<HashMap<String, Value>>>,
}

impl<E: LlmEvaluator + 'static> ParallelLlmExecutor<E> {
    /// Create a new parallel executor
    pub fn new(evaluator: E, config: ParallelExecutorConfig) -> Self {
        Self {
            evaluator: Arc::new(evaluator),
            config,
            shared_context: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set initial context values
    pub async fn set_context(&self, values: HashMap<String, Value>) {
        let mut ctx = self.shared_context.write().await;
        ctx.extend(values);
    }

    /// Get a value from the shared context
    pub async fn get_context_value(&self, key: &str) -> Option<Value> {
        let ctx = self.shared_context.read().await;
        ctx.get(key).cloned()
    }

    /// Execute a single rule with full context and retry logic
    #[instrument(
        skip(self, metadata, input_values, llm_config),
        fields(
            rule_id = %rule_id,
            rule_name = %metadata.name,
            input_count = metadata.inputs.len(),
            output_count = metadata.outputs.len(),
        )
    )]
    pub async fn execute_rule(
        &self,
        rule_id: &str,
        metadata: &RuleMetadata,
        input_values: HashMap<String, Value>,
        llm_config: Option<&LlmEvaluatorConfig>,
    ) -> LlmRuleResult {
        let start = std::time::Instant::now();
        let retry_config = &self.config.retry_config;
        let mut last_error: Option<String> = None;
        let mut retry_count = 0u32;
        let mut total_backoff_ms = 0u64;

        // Build evaluation context (done once, reused across retries)
        let eval_context = self.build_eval_context(metadata, &input_values);
        let prompt = self.build_prompt(metadata, &eval_context);
        let config_map = self.build_config_map(metadata, llm_config, &prompt);
        let output_names: Vec<String> = metadata.outputs.iter().map(|o| o.name.clone()).collect();

        // Retry loop with exponential backoff
        for attempt in 0..=retry_config.max_retries {
            if attempt > 0 {
                // Calculate backoff
                let backoff = retry_config.backoff_for_attempt(attempt - 1);
                let backoff_ms = backoff.as_millis() as u64;

                // Check if we'd exceed max total backoff
                if total_backoff_ms + backoff_ms > retry_config.max_total_backoff_ms {
                    warn!(
                        rule_id = %rule_id,
                        attempt = attempt,
                        total_backoff_ms = total_backoff_ms,
                        max_backoff_ms = retry_config.max_total_backoff_ms,
                        "Max backoff exceeded, stopping retries"
                    );
                    break;
                }

                info!(
                    rule_id = %rule_id,
                    attempt = attempt,
                    backoff_ms = backoff_ms,
                    "Retrying LLM evaluation after backoff"
                );

                tokio::time::sleep(backoff).await;
                total_backoff_ms += backoff_ms;
                retry_count = attempt;
            }

            // Execute with timeout
            let result = tokio::time::timeout(
                Duration::from_millis(self.config.timeout_ms),
                self.evaluator.evaluate(&config_map, &input_values, &output_names),
            )
            .await;

            match result {
                Ok(Ok(outputs)) => {
                    let execution_time_ms = start.elapsed().as_millis() as u64;

                    info!(
                        rule_id = %rule_id,
                        outputs = ?outputs.keys().collect::<Vec<_>>(),
                        time_ms = execution_time_ms,
                        retries = retry_count,
                        "LLM rule evaluation succeeded"
                    );

                    // Update shared context with outputs
                    {
                        let mut ctx = self.shared_context.write().await;
                        for (k, v) in &outputs {
                            ctx.insert(k.clone(), v.clone());
                        }
                    }

                    return LlmRuleResult {
                        rule_id: rule_id.to_string(),
                        outputs,
                        execution_time_ms,
                        success: true,
                        error: None,
                        retry_count,
                    };
                }
                Ok(Err(e)) => {
                    let err_msg = e.to_string();
                    let retryable = is_error_retryable(&e);

                    if retryable {
                        warn!(
                            rule_id = %rule_id,
                            attempt = attempt,
                            error = %err_msg,
                            retryable = true,
                            "LLM rule evaluation failed (will retry)"
                        );
                    } else {
                        // Non-retryable error - don't waste API calls
                        error!(
                            rule_id = %rule_id,
                            attempt = attempt,
                            error = %err_msg,
                            retryable = false,
                            "LLM rule evaluation failed with non-retryable error"
                        );
                        // Return immediately without further retries
                        let execution_time_ms = start.elapsed().as_millis() as u64;
                        return LlmRuleResult {
                            rule_id: rule_id.to_string(),
                            outputs: HashMap::new(),
                            execution_time_ms,
                            success: false,
                            error: Some(err_msg),
                            retry_count,
                        };
                    }
                    last_error = Some(err_msg);
                }
                Err(_) => {
                    // Timeout is retryable
                    let err_msg = format!("Timeout after {}ms", self.config.timeout_ms);
                    warn!(
                        rule_id = %rule_id,
                        attempt = attempt,
                        timeout_ms = self.config.timeout_ms,
                        "LLM rule evaluation timed out (will retry)"
                    );
                    last_error = Some(err_msg);
                }
            }
        }

        // All retries exhausted
        let execution_time_ms = start.elapsed().as_millis() as u64;
        error!(
            rule_id = %rule_id,
            retries = retry_count,
            time_ms = execution_time_ms,
            error = ?last_error,
            "LLM rule evaluation failed after all retries"
        );

        LlmRuleResult {
            rule_id: rule_id.to_string(),
            outputs: HashMap::new(),
            execution_time_ms,
            success: false,
            error: last_error,
            retry_count,
        }
    }

    /// Build evaluation context from metadata and inputs
    fn build_eval_context(
        &self,
        metadata: &RuleMetadata,
        input_values: &HashMap<String, Value>,
    ) -> RuleEvaluationContext {
        let mut eval_context = RuleEvaluationContext::new(&metadata.name);

        if let Some(desc) = &metadata.description {
            eval_context = eval_context.with_description(desc);
        }

        if let Some(rt) = &metadata.rule_type {
            eval_context = eval_context.with_rule_type(rt);
        }

        if let Some(pn) = &metadata.product_name {
            eval_context = eval_context.with_product(pn);
        }

        for input in &metadata.inputs {
            eval_context = eval_context.add_input(input.clone());
        }

        for output in &metadata.outputs {
            eval_context = eval_context.add_output(output.clone());
        }

        eval_context.with_input_values(input_values.clone())
    }

    /// Build prompt from metadata and context
    fn build_prompt(&self, metadata: &RuleMetadata, context: &RuleEvaluationContext) -> String {
        let prompt_builder = if metadata.prompt_template.is_some() {
            PromptBuilder::new().with_template(metadata.prompt_template.as_ref().unwrap())
        } else {
            PromptBuilder::new()
        };
        prompt_builder.build(context)
    }

    /// Build config map for LLM evaluator
    fn build_config_map(
        &self,
        metadata: &RuleMetadata,
        llm_config: Option<&LlmEvaluatorConfig>,
        prompt: &str,
    ) -> HashMap<String, Value> {
        let config = llm_config.unwrap_or(&self.config.default_llm_config);
        let mut config_map = config.to_config_map();
        config_map.insert("prompt_template".to_string(), Value::String(prompt.to_string()));

        let system_prompt = metadata
            .system_prompt
            .clone()
            .or_else(|| config.system_prompt.clone())
            .unwrap_or_else(default_system_prompt);
        config_map.insert("system_prompt".to_string(), Value::String(system_prompt));

        config_map
    }

    /// Execute multiple rules in parallel
    ///
    /// Rules at the same dependency level can execute concurrently.
    /// Uses a very relaxed semaphore to allow natural API rate limiting.
    #[instrument(skip(self, rules), fields(rule_count = rules.len()))]
    pub async fn execute_parallel(
        &self,
        rules: Vec<(String, RuleMetadata, HashMap<String, Value>)>,
    ) -> Vec<LlmRuleResult> {
        use futures::stream::{self, StreamExt};

        let rule_count = rules.len();
        info!(rule_count = rule_count, max_concurrency = self.config.max_concurrency, "Starting parallel LLM execution");

        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.max_concurrency));
        let start = std::time::Instant::now();

        let futures: Vec<_> = rules
            .into_iter()
            .map(|(rule_id, metadata, inputs)| {
                let sem = semaphore.clone();
                let evaluator = self.evaluator.clone();
                let config = self.config.clone();
                let shared_ctx = self.shared_context.clone();

                async move {
                    // Acquire semaphore permit (very relaxed, mostly for safety)
                    let _permit = sem.acquire().await.unwrap();

                    // Create a temporary executor for this rule
                    let temp_executor = ParallelLlmExecutor {
                        evaluator,
                        config,
                        shared_context: shared_ctx,
                    };

                    temp_executor
                        .execute_rule(&rule_id, &metadata, inputs, None)
                        .await
                }
            })
            .collect();

        // Execute all futures concurrently
        let results: Vec<LlmRuleResult> = stream::iter(futures)
            .buffer_unordered(self.config.max_concurrency)
            .collect()
            .await;

        let elapsed_ms = start.elapsed().as_millis() as u64;
        let success_count = results.iter().filter(|r| r.success).count();
        let failure_count = results.iter().filter(|r| !r.success).count();
        let total_retries: u32 = results.iter().map(|r| r.retry_count).sum();

        info!(
            rule_count = rule_count,
            success_count = success_count,
            failure_count = failure_count,
            total_retries = total_retries,
            elapsed_ms = elapsed_ms,
            "Parallel LLM execution completed"
        );

        results
    }

    /// Execute rules in dependency levels (sequential between levels, parallel within)
    pub async fn execute_by_levels(
        &self,
        levels: Vec<Vec<(String, RuleMetadata)>>,
        initial_inputs: HashMap<String, Value>,
    ) -> Vec<LlmRuleResult> {
        let mut all_results = Vec::new();

        // Set initial context
        self.set_context(initial_inputs.clone()).await;

        for (level_idx, level) in levels.into_iter().enumerate() {
            debug!(level = level_idx, rules = level.len(), "Executing level");

            // Get current context for this level
            let current_context = self.shared_context.read().await.clone();

            // Prepare rules for parallel execution
            let rules_with_inputs: Vec<_> = level
                .into_iter()
                .map(|(id, meta)| {
                    // Gather inputs for this rule from current context
                    let inputs: HashMap<String, Value> = meta
                        .inputs
                        .iter()
                        .filter_map(|input| {
                            current_context
                                .get(&input.name)
                                .map(|v| (input.name.clone(), v.clone()))
                        })
                        .collect();
                    (id, meta, inputs)
                })
                .collect();

            // Execute this level in parallel
            let level_results = self.execute_parallel(rules_with_inputs).await;

            // Check for failures if not continuing on error
            if !self.config.continue_on_error {
                let has_failure = level_results.iter().any(|r| !r.success);
                if has_failure {
                    let failed_id = level_results
                        .iter()
                        .find(|r| !r.success)
                        .map(|r| r.rule_id.clone())
                        .unwrap_or_default();
                    all_results.extend(level_results);
                    warn!(
                        rule_id = %failed_id,
                        "Stopping execution due to rule failure"
                    );
                    break;
                }
            }

            all_results.extend(level_results);
        }

        all_results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use product_farm_core::NoOpLlmEvaluator;

    #[tokio::test]
    async fn test_rule_metadata_from_inputs() {
        let metadata = RuleMetadata {
            name: "test-rule".to_string(),
            description: Some("A test rule".to_string()),
            inputs: vec![
                AttributeInfo::new("x").with_description("Input X"),
                AttributeInfo::new("y").with_description("Input Y"),
            ],
            outputs: vec![AttributeInfo::new("result").with_description("The result")],
            ..Default::default()
        };

        assert_eq!(metadata.inputs.len(), 2);
        assert_eq!(metadata.outputs.len(), 1);
    }

    #[tokio::test]
    async fn test_parallel_executor_config_defaults() {
        let config = ParallelExecutorConfig::default();
        // Default provider is ollama with max_concurrency=10
        assert_eq!(config.max_concurrency, 10);
        assert_eq!(config.timeout_ms, 30000);
        assert!(config.continue_on_error);
        // Retry config defaults
        assert_eq!(config.retry_config.max_retries, 3);
        assert_eq!(config.retry_config.max_total_backoff_ms, 3000);
    }

    #[tokio::test]
    async fn test_parallel_executor_config_unlimited() {
        let config = ParallelExecutorConfig::unlimited();
        assert_eq!(config.max_concurrency, usize::MAX);
    }

    #[tokio::test]
    async fn test_retry_config_backoff() {
        let config = RetryConfig::default();

        // First retry: 100ms
        assert_eq!(config.backoff_for_attempt(0).as_millis(), 100);
        // Second retry: 200ms
        assert_eq!(config.backoff_for_attempt(1).as_millis(), 200);
        // Third retry: 400ms
        assert_eq!(config.backoff_for_attempt(2).as_millis(), 400);
        // Fourth retry: 800ms
        assert_eq!(config.backoff_for_attempt(3).as_millis(), 800);
        // Fifth retry: capped at 3000ms
        assert_eq!(config.backoff_for_attempt(10).as_millis(), 3000);
    }

    #[tokio::test]
    async fn test_context_sharing() {
        let evaluator = NoOpLlmEvaluator::new();
        let config = ParallelExecutorConfig::default();
        let executor = ParallelLlmExecutor::new(evaluator, config);

        let mut values = HashMap::new();
        values.insert("test_key".to_string(), Value::Int(42));
        executor.set_context(values).await;

        let result = executor.get_context_value("test_key").await;
        assert_eq!(result, Some(Value::Int(42)));
    }

    #[tokio::test]
    async fn test_config_builder_methods() {
        let config = ParallelExecutorConfig::default()
            .with_max_concurrency(50)
            .with_retry_config(RetryConfig {
                max_retries: 5,
                initial_backoff_ms: 50,
                max_total_backoff_ms: 5000,
                backoff_multiplier: 1.5,
            });

        assert_eq!(config.max_concurrency, 50);
        assert_eq!(config.retry_config.max_retries, 5);
        assert_eq!(config.retry_config.initial_backoff_ms, 50);
    }

    #[tokio::test]
    async fn test_config_without_retries() {
        let config = ParallelExecutorConfig::default().without_retries();
        assert_eq!(config.retry_config.max_retries, 0);
    }
}
