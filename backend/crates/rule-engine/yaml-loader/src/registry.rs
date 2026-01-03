//! Product registry for loaded schemas.
//!
//! Manages loaded product schemas and provides access to
//! entities, functions, and layer-specific views.

use crate::error::{LoaderError, LoaderResult};
use crate::schema::{LayerVisibilityConfig, MasterSchema};
use product_farm_core::{AbstractAttribute, LlmEvaluator, NoOpLlmEvaluator, Rule};
use product_farm_llm_evaluator::{
    RuleEngineLlmConfig, ClaudeLlmEvaluator, OllamaLlmEvaluator,
};
use product_farm_rule_engine::{RuleDag, RuleExecutor};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry of loaded product schemas.
pub struct ProductRegistry {
    /// Loaded schemas by product ID.
    schemas: HashMap<String, MasterSchema>,

    /// Rule executor for JSON Logic evaluation.
    executor: RuleExecutor,

    /// Optional LLM evaluator for LLM-based rules.
    llm_evaluator: Arc<dyn LlmEvaluator>,
}

impl Default for ProductRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProductRegistry {
    /// Create a new empty registry with NoOp LLM evaluator.
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            executor: RuleExecutor::new(),
            llm_evaluator: Arc::new(NoOpLlmEvaluator::new()),
        }
    }

    /// Create a new registry with LLM evaluator auto-initialized from environment.
    ///
    /// This reads `RULE_ENGINE_LLM_PROVIDER` and other environment variables
    /// to automatically configure the appropriate LLM evaluator:
    ///
    /// - `ollama` (default): Uses local Ollama instance
    /// - `anthropic`/`claude`: Uses Anthropic Claude API (requires API key)
    ///
    /// # Environment Variables
    ///
    /// Common:
    /// - `RULE_ENGINE_LLM_PROVIDER`: Provider to use ("ollama" or "anthropic")
    /// - `RULE_ENGINE_LLM_TEMPERATURE`: Generation temperature (0.0-1.0)
    /// - `RULE_ENGINE_LLM_MAX_OUTPUT_TOKENS`: Max tokens in response
    ///
    /// Ollama:
    /// - `RULE_ENGINE_OLLAMA_BASE_URL`: Ollama API URL (default: http://localhost:11434)
    /// - `RULE_ENGINE_OLLAMA_MODEL`: Model to use (default: qwen2.5:7b)
    ///
    /// Anthropic:
    /// - `RULE_ENGINE_ANTHROPIC_API_KEY`: Anthropic API key (required)
    /// - `RULE_ENGINE_ANTHROPIC_MODEL`: Model to use (default: claude-sonnet-4-20250514)
    ///
    /// # Errors
    /// Returns an error if the LLM provider cannot be initialized (e.g., missing API key).
    pub fn from_env() -> LoaderResult<Self> {
        let env_config = RuleEngineLlmConfig::from_env();
        let provider = env_config.default_provider();

        tracing::info!(
            provider = %provider,
            "Initializing ProductRegistry with LLM evaluator from environment"
        );

        let llm_evaluator: Arc<dyn LlmEvaluator> = match provider.to_lowercase().as_str() {
            "anthropic" | "claude" => {
                let evaluator = ClaudeLlmEvaluator::from_env()
                    .map_err(|e| LoaderError::LlmInitializationFailed(format!(
                        "Failed to initialize Claude LLM evaluator: {}",
                        e
                    )))?;
                tracing::info!("Initialized Claude LLM evaluator");
                Arc::new(evaluator)
            }
            "ollama" | _ => {
                let evaluator = OllamaLlmEvaluator::from_env()
                    .map_err(|e| LoaderError::LlmInitializationFailed(format!(
                        "Failed to initialize Ollama LLM evaluator: {}",
                        e
                    )))?;
                tracing::info!(
                    model = %env_config.ollama.model,
                    base_url = %env_config.ollama.base_url,
                    "Initialized Ollama LLM evaluator"
                );
                Arc::new(evaluator)
            }
        };

        Ok(Self {
            schemas: HashMap::new(),
            executor: RuleExecutor::new(),
            llm_evaluator,
        })
    }

    /// Get the current LLM configuration summary for debugging.
    pub fn llm_config_summary(&self) -> String {
        let env_config = RuleEngineLlmConfig::from_env();
        env_config.summary()
    }

    /// Set a custom LLM evaluator.
    pub fn with_llm_evaluator<E: LlmEvaluator + 'static>(mut self, evaluator: E) -> Self {
        self.llm_evaluator = Arc::new(evaluator);
        self
    }

    /// Set the LLM evaluator.
    pub fn set_llm_evaluator<E: LlmEvaluator + 'static>(&mut self, evaluator: E) {
        self.llm_evaluator = Arc::new(evaluator);
    }

    /// Register a master schema.
    ///
    /// Validates the schema for circular dependencies before registering.
    pub fn register(&mut self, schema: MasterSchema) -> LoaderResult<()> {
        // Validate for circular dependencies at registration time
        if !schema.rules.is_empty() {
            RuleDag::from_rules(&schema.rules)
                .map_err(|e| LoaderError::CircularDependency(format!(
                    "Product '{}': {}",
                    schema.product.id, e
                )))?;
        }

        let product_id = schema.product.id.to_string();
        self.schemas.insert(product_id, schema);
        Ok(())
    }

    /// Compile all rules for efficient execution.
    pub fn compile_rules(&mut self) -> LoaderResult<()> {
        for schema in self.schemas.values() {
            self.executor.compile_rules(&schema.rules).map_err(|e| {
                LoaderError::Internal(format!("Failed to compile rules: {}", e))
            })?;
        }
        Ok(())
    }

    /// Get a schema by product ID.
    pub fn get_schema(&self, product_id: &str) -> LoaderResult<&MasterSchema> {
        self.schemas
            .get(product_id)
            .ok_or_else(|| LoaderError::ProductNotFound(product_id.to_string()))
    }

    /// Get all registered product IDs.
    pub fn product_ids(&self) -> Vec<&String> {
        self.schemas.keys().collect()
    }

    /// Check if a product is registered.
    pub fn has_product(&self, product_id: &str) -> bool {
        self.schemas.contains_key(product_id)
    }

    /// Get entities visible in a specific layer/interface.
    pub fn get_interface_entities(
        &self,
        product_id: &str,
        layer: &str,
    ) -> LoaderResult<Vec<&AbstractAttribute>> {
        let schema = self.get_schema(product_id)?;
        let layer_config = &schema.layer_config;

        if let Some(layer_def) = layer_config.layers.get(layer) {
            Ok(schema
                .attributes
                .iter()
                .filter(|attr| {
                    layer_def.visible_entities.contains(&attr.component_type)
                        || layer_def.visible_attributes.contains(&attr.abstract_path.to_string())
                })
                .collect())
        } else {
            // If layer not defined, return all attributes
            Ok(schema.attributes.iter().collect())
        }
    }

    /// Get functions visible in a specific layer/interface.
    pub fn get_interface_functions(
        &self,
        product_id: &str,
        layer: &str,
    ) -> LoaderResult<Vec<&Rule>> {
        let schema = self.get_schema(product_id)?;
        let layer_config = &schema.layer_config;

        if let Some(layer_def) = layer_config.layers.get(layer) {
            Ok(schema
                .rules
                .iter()
                .filter(|rule| layer_def.visible_functions.contains(&rule.rule_type))
                .collect())
        } else {
            // If layer not defined, return all rules
            Ok(schema.rules.iter().collect())
        }
    }

    /// Get all attributes for a product.
    pub fn get_attributes(&self, product_id: &str) -> LoaderResult<&[AbstractAttribute]> {
        Ok(&self.get_schema(product_id)?.attributes)
    }

    /// Get all rules for a product.
    pub fn get_rules(&self, product_id: &str) -> LoaderResult<&[Rule]> {
        Ok(&self.get_schema(product_id)?.rules)
    }

    /// Find a rule by name.
    pub fn find_rule(&self, product_id: &str, rule_name: &str) -> LoaderResult<&Rule> {
        let schema = self.get_schema(product_id)?;
        schema
            .find_rule(rule_name)
            .ok_or_else(|| LoaderError::FunctionNotFound(rule_name.to_string()))
    }

    /// Get the layer configuration for a product.
    pub fn get_layer_config(&self, product_id: &str) -> LoaderResult<&LayerVisibilityConfig> {
        Ok(&self.get_schema(product_id)?.layer_config)
    }

    /// Get mutable access to the rule executor.
    #[allow(dead_code)]
    pub(crate) fn executor_mut(&mut self) -> &mut RuleExecutor {
        &mut self.executor
    }

    /// Get the LLM evaluator.
    #[allow(dead_code)]
    pub(crate) fn llm_evaluator(&self) -> &Arc<dyn LlmEvaluator> {
        &self.llm_evaluator
    }

    /// Evaluate a function, computing updated state.
    ///
    /// # Arguments
    ///
    /// * `product_id` - The product to evaluate within
    /// * `state` - Current state with input values
    /// * `function_name` - Name of the function/rule to evaluate
    ///
    /// # Returns
    ///
    /// An `EvalResult` containing the updated state, outputs, and execution info.
    pub fn evaluate(
        &mut self,
        product_id: &str,
        state: crate::evaluator::State,
        function_name: &str,
    ) -> LoaderResult<crate::evaluator::EvalResult> {
        use crate::evaluator::{EvalResult, State};
        use product_farm_core::EvaluatorType;
        use product_farm_rule_engine::ExecutionContext;

        let start = std::time::Instant::now();

        // Get the schema and find the rule (clone to avoid borrow issues)
        let schema = self.schemas.get(product_id)
            .ok_or_else(|| LoaderError::ProductNotFound(product_id.to_string()))?;

        let target_rule = schema
            .find_rule(function_name)
            .ok_or_else(|| LoaderError::FunctionNotFound(function_name.to_string()))?
            .clone();

        // Build execution context from state (no conversion needed - both use hashbrown)
        let mut context = ExecutionContext::new(state.to_hashbrown());

        let mut executed_rules = Vec::new();
        let mut outputs = std::collections::HashMap::new();

        // Execute based on evaluator type
        match &target_rule.evaluator.name {
            EvaluatorType::JsonLogic => {
                // Execute using rule engine with single rule
                let exec_result = self.executor.execute(&[target_rule.clone()], &mut context)
                    .map_err(|e| LoaderError::Internal(format!("Execution error: {}", e)))?;

                // Collect outputs from execution result
                for rule_result in &exec_result.rule_results {
                    for (key, value) in &rule_result.outputs {
                        outputs.insert(key.clone(), value.clone());
                    }
                }
                executed_rules.push(function_name.to_string());
            }

            EvaluatorType::LargeLanguageModel => {
                // Check if LLM evaluator is configured
                if !self.llm_evaluator.is_ready() {
                    return Err(LoaderError::LlmNotConfigured);
                }

                // Prepare inputs from context
                let inputs = state.to_std_hashmap();

                // Get config from rule - fail fast if missing or incomplete
                let config: std::collections::HashMap<String, product_farm_core::Value> = target_rule
                    .evaluator
                    .config
                    .as_ref()
                    .ok_or_else(|| LoaderError::InvalidField {
                        function: function_name.to_string(),
                        field: "evaluator.config".to_string(),
                        reason: "LLM evaluator requires configuration (model, prompt_template)".to_string(),
                    })?
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                // Validate required config fields
                if !config.contains_key("model") {
                    return Err(LoaderError::MissingField {
                        function: function_name.to_string(),
                        field: "model".to_string(),
                    });
                }
                if !config.contains_key("prompt_template") {
                    return Err(LoaderError::MissingField {
                        function: function_name.to_string(),
                        field: "prompt_template".to_string(),
                    });
                }

                let output_names: Vec<String> = target_rule
                    .output_attributes
                    .iter()
                    .map(|a| a.path.to_string())
                    .collect();

                // Execute LLM evaluation (blocking for now)
                let rt = tokio::runtime::Handle::try_current()
                    .map_err(|_| LoaderError::Internal("No tokio runtime available".into()))?;

                let llm = self.llm_evaluator.clone();
                // Errors are automatically converted: LlmEvaluatorError -> CoreError::LlmError -> LoaderError::Core
                let llm_result = rt.block_on(async {
                    llm.evaluate(&config, &inputs, &output_names).await
                })?;

                // Store results
                for (key, value) in llm_result {
                    outputs.insert(key.clone(), value.clone());
                    context.set(key, value);
                }
                executed_rules.push(function_name.to_string());
            }

            EvaluatorType::Custom(name) => {
                return Err(LoaderError::UnsupportedEvaluator(name.clone()));
            }
        }

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(EvalResult {
            state: State::from_context(&context),
            outputs,
            executed_rules,
            execution_time_ms,
        })
    }

    /// Evaluate all enabled functions for a product.
    ///
    /// Executes rules in order based on their order_index.
    pub fn evaluate_all(
        &mut self,
        product_id: &str,
        mut state: crate::evaluator::State,
    ) -> LoaderResult<crate::evaluator::EvalResult> {
        let start = std::time::Instant::now();

        // Get rule names first to avoid borrow issues
        let rule_names: Vec<String> = {
            let schema = self.schemas.get(product_id)
                .ok_or_else(|| LoaderError::ProductNotFound(product_id.to_string()))?;

            let mut rules: Vec<_> = schema
                .rules
                .iter()
                .filter(|r| r.enabled)
                .collect();
            rules.sort_by_key(|r| r.order_index);
            rules.iter().map(|r| r.rule_type.clone()).collect()
        };

        let mut all_outputs = std::collections::HashMap::new();
        let mut all_executed = Vec::new();

        // Execute each rule by name
        for rule_name in rule_names {
            let result = self.evaluate(product_id, state.clone(), &rule_name)?;
            state.merge(result.state);
            all_outputs.extend(result.outputs);
            all_executed.extend(result.executed_rules);
        }

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(crate::evaluator::EvalResult {
            state,
            outputs: all_outputs,
            executed_rules: all_executed,
            execution_time_ms,
        })
    }

    /// Evaluate a function asynchronously with parallel LLM execution support.
    ///
    /// This is the async version of `evaluate` that properly handles LLM rules
    /// without blocking the async runtime.
    pub async fn evaluate_async(
        &mut self,
        product_id: &str,
        state: crate::evaluator::State,
        function_name: &str,
    ) -> LoaderResult<crate::evaluator::EvalResult> {
        use crate::evaluator::{EvalResult, State};
        use product_farm_core::EvaluatorType;
        use product_farm_rule_engine::ExecutionContext;

        let start = std::time::Instant::now();

        // Get the schema and find the rule
        let schema = self.schemas.get(product_id)
            .ok_or_else(|| LoaderError::ProductNotFound(product_id.to_string()))?;

        let target_rule = schema
            .find_rule(function_name)
            .ok_or_else(|| LoaderError::FunctionNotFound(function_name.to_string()))?
            .clone();

        // Build execution context (no conversion needed - both use hashbrown)
        let mut context = ExecutionContext::new(state.to_hashbrown());

        let mut executed_rules = Vec::new();
        let mut outputs = std::collections::HashMap::new();

        // Execute based on evaluator type
        match &target_rule.evaluator.name {
            EvaluatorType::JsonLogic => {
                // JSON Logic rules use the sync executor (already parallel via rayon)
                let exec_result = self.executor.execute(&[target_rule.clone()], &mut context)
                    .map_err(|e| LoaderError::Internal(format!("Execution error: {}", e)))?;

                for rule_result in &exec_result.rule_results {
                    for (key, value) in &rule_result.outputs {
                        outputs.insert(key.clone(), value.clone());
                    }
                }
                executed_rules.push(function_name.to_string());
            }

            EvaluatorType::LargeLanguageModel => {
                // Use async LLM evaluation
                if !self.llm_evaluator.is_ready() {
                    return Err(LoaderError::LlmNotConfigured);
                }

                let inputs = state.to_std_hashmap();

                let config: std::collections::HashMap<String, product_farm_core::Value> = target_rule
                    .evaluator
                    .config
                    .as_ref()
                    .ok_or_else(|| LoaderError::InvalidField {
                        function: function_name.to_string(),
                        field: "evaluator.config".to_string(),
                        reason: "LLM evaluator requires configuration".to_string(),
                    })?
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                let output_names: Vec<String> = target_rule
                    .output_attributes
                    .iter()
                    .map(|a| a.path.to_string())
                    .collect();

                // Async LLM evaluation (non-blocking)
                // Errors are automatically converted: LlmEvaluatorError -> CoreError::LlmError -> LoaderError::Core
                let llm = self.llm_evaluator.clone();
                let llm_result = llm.evaluate(&config, &inputs, &output_names).await?;

                for (key, value) in llm_result {
                    outputs.insert(key.clone(), value.clone());
                    context.set(key, value);
                }
                executed_rules.push(function_name.to_string());
            }

            EvaluatorType::Custom(name) => {
                return Err(LoaderError::UnsupportedEvaluator(name.clone()));
            }
        }

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(EvalResult {
            state: State::from_context(&context),
            outputs,
            executed_rules,
            execution_time_ms,
        })
    }

    /// Evaluate all enabled functions asynchronously with parallel execution.
    ///
    /// Uses DAG-based dependency analysis to execute independent rules in parallel:
    /// - JSON Logic rules: parallel via rayon (CPU-bound)
    /// - LLM rules: parallel via tokio (IO-bound)
    pub async fn evaluate_all_async(
        &mut self,
        product_id: &str,
        state: crate::evaluator::State,
    ) -> LoaderResult<crate::evaluator::EvalResult> {
        use crate::evaluator::{EvalResult, State};
        use product_farm_core::EvaluatorType;
        use product_farm_rule_engine::{ExecutionContext, RuleDag};

        let start = std::time::Instant::now();

        // Get schema and enabled rules
        let schema = self.schemas.get(product_id)
            .ok_or_else(|| LoaderError::ProductNotFound(product_id.to_string()))?;

        let enabled_rules: Vec<_> = schema
            .rules
            .iter()
            .filter(|r| r.enabled)
            .cloned()
            .collect();

        if enabled_rules.is_empty() {
            return Ok(EvalResult {
                state,
                outputs: std::collections::HashMap::new(),
                executed_rules: Vec::new(),
                execution_time_ms: 0,
            });
        }

        // Separate JSON Logic and LLM rules
        let json_logic_rules: Vec<_> = enabled_rules
            .iter()
            .filter(|r| matches!(r.evaluator.name, EvaluatorType::JsonLogic))
            .cloned()
            .collect();

        let llm_rules: Vec<_> = enabled_rules
            .iter()
            .filter(|r| matches!(r.evaluator.name, EvaluatorType::LargeLanguageModel))
            .cloned()
            .collect();

        let mut all_outputs = std::collections::HashMap::new();
        let mut all_executed = Vec::new();

        // Build context from state (no conversion needed - both use hashbrown)
        let mut context = ExecutionContext::new(state.to_hashbrown());

        // Execute JSON Logic rules with parallel DAG execution
        if !json_logic_rules.is_empty() {
            // Build DAG for dependency analysis
            let dag = RuleDag::from_rules(&json_logic_rules)
                .map_err(|e| LoaderError::Internal(format!("DAG error: {}", e)))?;

            let levels = dag.execution_levels()
                .map_err(|e| LoaderError::Internal(format!("Execution levels error: {}", e)))?;

            tracing::info!(
                json_logic_rules = json_logic_rules.len(),
                levels = levels.len(),
                parallel_levels = levels.iter().filter(|l| l.len() > 1).count(),
                "Executing JSON Logic rules with parallel DAG"
            );

            // Execute using the parallel rule executor
            let exec_result = self.executor.execute(&json_logic_rules, &mut context)
                .map_err(|e| LoaderError::Internal(format!("Execution error: {}", e)))?;

            for rule_result in &exec_result.rule_results {
                for (key, value) in &rule_result.outputs {
                    all_outputs.insert(key.clone(), value.clone());
                }
                all_executed.push(rule_result.rule_id.to_string());
            }
        }

        // Execute LLM rules (parallel for independent rules)
        if !llm_rules.is_empty() {
            if !self.llm_evaluator.is_ready() {
                return Err(LoaderError::LlmNotConfigured);
            }

            // Build DAG for LLM rules (errors converted via RuleEngineError -> LoaderError::RuleEngine)
            let llm_dag = RuleDag::from_rules(&llm_rules)?;
            let llm_levels = llm_dag.execution_levels()?;

            tracing::info!(
                llm_rules = llm_rules.len(),
                levels = llm_levels.len(),
                parallel_levels = llm_levels.iter().filter(|l| l.len() > 1).count(),
                "Executing LLM rules with parallel levels"
            );

            // Execute level by level (parallel within levels)
            for level in llm_levels {
                if level.len() == 1 {
                    // Single rule - execute directly
                    let rule_id = &level[0];
                    let rule = llm_rules.iter().find(|r| &r.id == rule_id)
                        .ok_or_else(|| LoaderError::FunctionNotFound(rule_id.to_string()))?;
                    let result = self.execute_llm_rule_async(rule, &context).await?;
                    for (key, value) in result {
                        context.set(key.clone(), value.clone());
                        all_outputs.insert(key, value);
                    }
                    all_executed.push(rule_id.to_string());
                } else {
                    // Multiple rules - execute in parallel
                    let rules_for_level: Result<Vec<_>, LoaderError> = level.iter().map(|rule_id| {
                        llm_rules.iter().find(|r| &r.id == rule_id)
                            .cloned()
                            .ok_or_else(|| LoaderError::FunctionNotFound(rule_id.to_string()))
                    }).collect();
                    let rules_for_level = rules_for_level?;

                    let futures: Vec<_> = rules_for_level.into_iter().map(|rule| {
                        let ctx_snapshot = context.clone();
                        let llm = self.llm_evaluator.clone();
                        async move {
                            Self::execute_llm_rule_static(&rule, &ctx_snapshot, llm).await
                                .map(|outputs| (rule.id.clone(), outputs))
                        }
                    }).collect();

                    let results = futures::future::join_all(futures).await;

                    for result in results {
                        let (rule_id, outputs) = result?;
                        for (key, value) in outputs {
                            context.set(key.clone(), value.clone());
                            all_outputs.insert(key, value);
                        }
                        all_executed.push(rule_id.to_string());
                    }
                }
            }
        }

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(EvalResult {
            state: State::from_context(&context),
            outputs: all_outputs,
            executed_rules: all_executed,
            execution_time_ms,
        })
    }

    /// Execute a single LLM rule asynchronously
    async fn execute_llm_rule_async(
        &self,
        rule: &Rule,
        context: &product_farm_rule_engine::ExecutionContext,
    ) -> LoaderResult<std::collections::HashMap<String, product_farm_core::Value>> {
        Self::execute_llm_rule_static(rule, context, self.llm_evaluator.clone()).await
    }

    /// Static version for parallel execution
    async fn execute_llm_rule_static(
        rule: &Rule,
        context: &product_farm_rule_engine::ExecutionContext,
        llm_evaluator: Arc<dyn product_farm_core::LlmEvaluator>,
    ) -> LoaderResult<std::collections::HashMap<String, product_farm_core::Value>> {
        let inputs: std::collections::HashMap<String, product_farm_core::Value> = context
            .to_json()
            .as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), product_farm_core::Value::from_json(v)))
                    .collect()
            })
            .unwrap_or_default();

        let config: std::collections::HashMap<String, product_farm_core::Value> = rule
            .evaluator
            .config
            .as_ref()
            .ok_or_else(|| LoaderError::InvalidField {
                function: rule.rule_type.clone(),
                field: "evaluator.config".to_string(),
                reason: "LLM evaluator requires configuration".to_string(),
            })?
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let output_names: Vec<String> = rule
            .output_attributes
            .iter()
            .map(|a| a.path.to_string())
            .collect();

        // Errors are automatically converted: CoreError::LlmError -> LoaderError::Core
        Ok(llm_evaluator.evaluate(&config, &inputs, &output_names).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use product_farm_core::{Product, ProductId, ProductStatus};

    fn create_test_schema() -> MasterSchema {
        use product_farm_core::TemplateType;
        let product = Product {
            id: ProductId::new("test-product"),
            name: "Test Product".to_string(),
            description: None,
            version: 1,
            status: ProductStatus::Draft,
            template_type: TemplateType::new("test"),
            parent_product_id: None,
            effective_from: chrono::Utc::now(),
            expiry_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        MasterSchema::new(product)
    }

    #[test]
    fn test_registry_register() {
        let mut registry = ProductRegistry::new();
        let schema = create_test_schema();

        registry.register(schema).unwrap();

        assert!(registry.has_product("test-product"));
        assert!(!registry.has_product("other-product"));
    }

    #[test]
    fn test_registry_get_schema() {
        let mut registry = ProductRegistry::new();
        let schema = create_test_schema();

        registry.register(schema).unwrap();

        let retrieved = registry.get_schema("test-product").unwrap();
        assert_eq!(retrieved.product.name, "Test Product");
    }

    #[test]
    fn test_registry_product_not_found() {
        let registry = ProductRegistry::new();

        let result = registry.get_schema("nonexistent");
        assert!(matches!(result, Err(LoaderError::ProductNotFound(_))));
    }
}
