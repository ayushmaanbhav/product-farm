//! LLM Evaluator Trait
//!
//! This module defines the trait for LLM-based rule evaluation.
//! Implementations can integrate with various LLM providers (OpenAI, Anthropic, etc.).
//!
//! For the full evaluator configuration and implementations, see the
//! `product-farm-llm-evaluator` crate.

use crate::error::CoreResult;
use crate::Value;
use async_trait::async_trait;
use std::collections::HashMap;

/// Trait for LLM-based rule evaluation.
///
/// This trait allows integration with Large Language Model providers
/// for rules that require AI-based evaluation rather than deterministic
/// JSON Logic expressions.
///
/// # Example
///
/// ```ignore
/// use product_farm_core::{LlmEvaluator, CoreResult, Value};
/// use std::collections::HashMap;
/// use async_trait::async_trait;
///
/// struct OpenAIEvaluator {
///     api_key: String,
/// }
///
/// #[async_trait]
/// impl LlmEvaluator for OpenAIEvaluator {
///     async fn evaluate(
///         &self,
///         config: &HashMap<String, Value>,
///         inputs: &HashMap<String, Value>,
///         output_names: &[String],
///     ) -> CoreResult<HashMap<String, Value>> {
///         // Call OpenAI API with the provided inputs
///         // Parse response and return outputs
///         todo!()
///     }
/// }
/// ```
#[async_trait]
pub trait LlmEvaluator: Send + Sync {
    /// Evaluate inputs using an LLM and produce output values.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the LLM call (model name, temperature, prompts, etc.)
    /// * `inputs` - Input values to be provided to the LLM
    /// * `output_names` - Names of the expected output attributes
    ///
    /// # Returns
    ///
    /// A HashMap mapping output attribute names to their computed values.
    async fn evaluate(
        &self,
        config: &HashMap<String, Value>,
        inputs: &HashMap<String, Value>,
        output_names: &[String],
    ) -> CoreResult<HashMap<String, Value>>;

    /// Optional: Get the name/identifier of this evaluator implementation.
    fn name(&self) -> &str {
        "llm-evaluator"
    }

    /// Optional: Check if the evaluator is properly configured and ready.
    fn is_ready(&self) -> bool {
        true
    }
}

/// No-operation LLM evaluator placeholder.
///
/// This implementation always returns an error, indicating that
/// an actual LLM evaluator has not been configured. Use this
/// as a default when LLM evaluation is not available.
#[derive(Debug, Clone, Default)]
pub struct NoOpLlmEvaluator;

impl NoOpLlmEvaluator {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LlmEvaluator for NoOpLlmEvaluator {
    async fn evaluate(
        &self,
        _config: &HashMap<String, Value>,
        _inputs: &HashMap<String, Value>,
        _output_names: &[String],
    ) -> CoreResult<HashMap<String, Value>> {
        Err(crate::error::CoreError::Internal(
            "LLM evaluator not configured. Please provide an LlmEvaluator implementation.".into(),
        ))
    }

    fn name(&self) -> &str {
        "no-op"
    }

    fn is_ready(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_evaluator_returns_error() {
        let evaluator = NoOpLlmEvaluator::new();
        let config = HashMap::new();
        let inputs = HashMap::new();
        let outputs = vec!["result".to_string()];

        let result = evaluator.evaluate(&config, &inputs, &outputs).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_noop_evaluator_not_ready() {
        let evaluator = NoOpLlmEvaluator::new();
        assert!(!evaluator.is_ready());
        assert_eq!(evaluator.name(), "no-op");
    }
}
