//! Claude LLM Evaluator Implementation
//!
//! Anthropic Claude-based implementation of the LlmEvaluator trait.

use crate::config::{LlmEvaluatorConfig, OutputFormat};
use crate::error::{LlmEvaluatorError, LlmEvaluatorResult};
use crate::parsing;
use async_trait::async_trait;
use product_farm_core::{CoreError, CoreResult, LlmEvaluator, Value};
use std::collections::HashMap;
use tracing::{debug, info};

// =============================================================================
// Claude Client (embedded, minimal implementation)
// =============================================================================

#[cfg(feature = "anthropic")]
mod client {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// Claude model name resolver.
    ///
    /// Resolves shorthand model names to full API model identifiers.
    /// If the model name is already a full identifier, it is returned as-is.
    pub fn resolve_model_name(model: &str) -> String {
        // If user provides full model name (contains version date), use as-is
        if model.contains("-202") {
            return model.to_string();
        }

        // Default fallback mappings for convenience shorthands
        // These are only used if user doesn't provide a full model name
        // Using -latest suffix allows automatic updates without code changes
        match model.to_lowercase().as_str() {
            "opus" | "claude-opus" | "claude-3-opus" | "claude-4-opus" => "claude-opus-4-latest".to_string(),
            "sonnet" | "claude-sonnet" | "claude-3-sonnet" | "claude-3-5-sonnet" | "claude-4-sonnet" => "claude-sonnet-4-latest".to_string(),
            "haiku" | "claude-haiku" | "claude-3-haiku" | "claude-3-5-haiku" => "claude-3-5-haiku-latest".to_string(),
            // Otherwise use as-is (allows new models without code changes)
            _ => model.to_string(),
        }
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct ClaudeRequest {
        pub model: String,
        pub max_tokens: u32,
        pub messages: Vec<Message>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub system: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub temperature: Option<f32>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Message {
        pub role: String,
        pub content: String,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct ClaudeResponse {
        pub content: Vec<ContentBlock>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct ContentBlock {
        #[serde(rename = "type")]
        #[allow(dead_code)]
        pub block_type: String,
        pub text: Option<String>,
    }

    impl ClaudeResponse {
        pub fn text(&self) -> Option<String> {
            self.content.iter().find_map(|block| block.text.clone())
        }
    }

    pub struct ClaudeClient {
        client: reqwest::Client,
        api_key: String,
        api_url: String,
        api_version: String,
    }

    impl ClaudeClient {
        /// Default Anthropic API URL (can be overridden for proxies)
        pub const DEFAULT_API_URL: &'static str = "https://api.anthropic.com/v1/messages";
        /// Default API version
        pub const DEFAULT_API_VERSION: &'static str = "2023-06-01";

        pub fn new(api_key: String) -> Self {
            Self {
                client: reqwest::Client::new(),
                api_key,
                api_url: Self::DEFAULT_API_URL.to_string(),
                api_version: Self::DEFAULT_API_VERSION.to_string(),
            }
        }

        /// Create with custom API URL (for proxies like AWS Bedrock, Azure, etc.)
        pub fn with_api_url(mut self, url: impl Into<String>) -> Self {
            self.api_url = url.into();
            self
        }

        /// Create with custom API version
        pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
            self.api_version = version.into();
            self
        }

        pub async fn send(&self, request: ClaudeRequest) -> LlmEvaluatorResult<ClaudeResponse> {
            let response = self
                .client
                .post(&self.api_url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", &self.api_version)
                .header("content-type", "application/json")
                .json(&request)
                .send()
                .await
                .map_err(|e| {
                    // Network errors are retryable
                    if e.is_connect() || e.is_timeout() {
                        LlmEvaluatorError::network(e.to_string())
                    } else {
                        LlmEvaluatorError::ApiError(e.to_string())
                    }
                })?;

            if !response.status().is_success() {
                let status = response.status();
                let status_code = status.as_u16();
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());

                // Categorize error by status code
                return Err(match status_code {
                    429 => LlmEvaluatorError::rate_limit(format!(
                        "Rate limit exceeded: {}", error_body
                    )),
                    500..=599 => LlmEvaluatorError::server_error(status_code, error_body),
                    _ => LlmEvaluatorError::ApiError(format!(
                        "API returned {}: {}",
                        status, error_body
                    )),
                });
            }

            response
                .json::<ClaudeResponse>()
                .await
                .map_err(|e| LlmEvaluatorError::ParseError(format!("Failed to parse response: {}", e)))
        }
    }
}

// =============================================================================
// Claude LLM Evaluator
// =============================================================================

/// Claude-based LLM evaluator for rule evaluation.
///
/// This evaluator uses Anthropic's Claude API to evaluate rules
/// that require AI-based reasoning rather than deterministic logic.
#[cfg(feature = "anthropic")]
pub struct ClaudeLlmEvaluator {
    api_key: String,
    api_url: String,
    api_version: String,
}

#[cfg(feature = "anthropic")]
impl ClaudeLlmEvaluator {
    /// Create a new Claude LLM evaluator with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        let env_config = crate::env_config::RuleEngineLlmConfig::from_env();
        Self {
            api_key: api_key.into(),
            api_url: env_config.anthropic.api_url,
            api_version: env_config.anthropic.api_version,
        }
    }

    /// Create with custom API URL (for proxies like AWS Bedrock, Azure, etc.)
    pub fn with_api_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = url.into();
        self
    }

    /// Create with custom API version
    pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = version.into();
        self
    }

    /// Create a new evaluator from environment configuration.
    ///
    /// Reads from `RULE_ENGINE_ANTHROPIC_API_KEY` (or `ANTHROPIC_API_KEY` as fallback).
    pub fn from_env() -> LlmEvaluatorResult<Self> {
        let env_config = crate::env_config::RuleEngineLlmConfig::from_env();
        let api_key = env_config.anthropic.api_key
            .ok_or_else(|| LlmEvaluatorError::ConfigError(
                format!("{}_ANTHROPIC_API_KEY not set", crate::env_config::ENV_PREFIX)
            ))?;
        Ok(Self {
            api_key,
            api_url: env_config.anthropic.api_url,
            api_version: env_config.anthropic.api_version,
        })
    }

    /// Parse the LLM response based on the expected output format.
    fn parse_response(
        &self,
        response_text: &str,
        output_format: &OutputFormat,
        output_names: &[String],
    ) -> LlmEvaluatorResult<HashMap<String, Value>> {
        let mut outputs = HashMap::new();

        match output_format {
            OutputFormat::Json => {
                // Try to parse as JSON object
                let json: serde_json::Value = serde_json::from_str(response_text)
                    .or_else(|_| {
                        // Try to extract JSON from markdown code block
                        let trimmed = response_text.trim();
                        if trimmed.starts_with("```json") {
                            let json_str = trimmed
                                .strip_prefix("```json")
                                .and_then(|s| s.strip_suffix("```"))
                                .unwrap_or(trimmed);
                            serde_json::from_str(json_str.trim())
                        } else if trimmed.starts_with("```") {
                            let json_str = trimmed
                                .strip_prefix("```")
                                .and_then(|s| s.strip_suffix("```"))
                                .unwrap_or(trimmed);
                            serde_json::from_str(json_str.trim())
                        } else {
                            Err(serde_json::Error::io(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Not valid JSON",
                            )))
                        }
                    })
                    .map_err(|e| LlmEvaluatorError::ParseError(format!("Failed to parse JSON: {}", e)))?;

                if let serde_json::Value::Object(map) = json {
                    for name in output_names {
                        if let Some(val) = map.get(name) {
                            outputs.insert(name.clone(), json_to_value(val.clone()));
                        }
                    }
                } else {
                    // Single value - assign to first output
                    if let Some(name) = output_names.first() {
                        outputs.insert(name.clone(), json_to_value(json));
                    }
                }
            }

            OutputFormat::Boolean => {
                let value = parsing::parse_boolean(response_text)?;
                if let Some(name) = output_names.first() {
                    outputs.insert(name.clone(), Value::Bool(value));
                }
            }

            OutputFormat::Number => {
                let num = parsing::parse_number(response_text)?;
                if let Some(name) = output_names.first() {
                    if num.fract() == 0.0 {
                        outputs.insert(name.clone(), Value::Int(num as i64));
                    } else {
                        outputs.insert(name.clone(), Value::Float(num));
                    }
                }
            }

            OutputFormat::Text => {
                if let Some(name) = output_names.first() {
                    outputs.insert(name.clone(), Value::String(response_text.trim().to_string()));
                }
            }

            OutputFormat::Array => {
                let json: serde_json::Value = serde_json::from_str(response_text)
                    .map_err(|e| LlmEvaluatorError::ParseError(format!("Failed to parse array: {}", e)))?;

                if let serde_json::Value::Array(arr) = json {
                    if let Some(name) = output_names.first() {
                        let values: Vec<Value> = arr.into_iter().map(json_to_value).collect();
                        outputs.insert(name.clone(), Value::Array(values));
                    }
                }
            }
        }

        Ok(outputs)
    }
}

#[cfg(feature = "anthropic")]
#[async_trait]
impl LlmEvaluator for ClaudeLlmEvaluator {
    async fn evaluate(
        &self,
        config_map: &HashMap<String, Value>,
        inputs: &HashMap<String, Value>,
        output_names: &[String],
    ) -> CoreResult<HashMap<String, Value>> {
        use client::*;

        // Parse config from map
        let config = LlmEvaluatorConfig::from_config_map(config_map)
            .ok_or_else(|| CoreError::Internal("Invalid LLM evaluator config".to_string()))?;

        info!(
            model = %config.model,
            outputs = ?output_names,
            "Evaluating rule with LLM"
        );

        // Build the prompt
        let prompt = config.render_prompt_with_outputs(inputs, output_names);
        debug!(prompt = %prompt, "Rendered prompt");

        // Resolve model name (supports both shorthands and full model names)
        let model = resolve_model_name(&config.model);

        // Create request
        let request = ClaudeRequest {
            model,
            max_tokens: config.max_tokens,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            system: config.system_prompt.clone(),
            temperature: Some(config.temperature),
        };

        // Send request
        let client = ClaudeClient::new(self.api_key.clone())
            .with_api_url(&self.api_url)
            .with_api_version(&self.api_version);
        let response = client
            .send(request)
            .await
            .map_err(|e| CoreError::Internal(format!("LLM API error: {}", e)))?;

        let response_text = response
            .text()
            .ok_or_else(|| CoreError::Internal("Empty response from LLM".to_string()))?;

        debug!(response = %response_text, "LLM response");

        // Parse response based on output format
        let outputs = self
            .parse_response(&response_text, &config.output_format, output_names)
            .map_err(|e| CoreError::Internal(format!("Failed to parse LLM response: {}", e)))?;

        info!(outputs = ?outputs.keys().collect::<Vec<_>>(), "LLM evaluation complete");
        Ok(outputs)
    }

    fn name(&self) -> &str {
        "claude-llm-evaluator"
    }

    fn is_ready(&self) -> bool {
        !self.api_key.is_empty()
    }
}

// =============================================================================
// Stub when anthropic feature is not enabled
// =============================================================================

#[cfg(not(feature = "anthropic"))]
pub struct ClaudeLlmEvaluator;

#[cfg(not(feature = "anthropic"))]
impl ClaudeLlmEvaluator {
    pub fn new(_api_key: impl Into<String>) -> Self {
        Self
    }

    pub fn from_env() -> LlmEvaluatorResult<Self> {
        Err(LlmEvaluatorError::FeatureNotEnabled(
            "anthropic feature required for ClaudeLlmEvaluator".to_string(),
        ))
    }
}

#[cfg(not(feature = "anthropic"))]
#[async_trait]
impl LlmEvaluator for ClaudeLlmEvaluator {
    async fn evaluate(
        &self,
        _config: &HashMap<String, Value>,
        _inputs: &HashMap<String, Value>,
        _output_names: &[String],
    ) -> CoreResult<HashMap<String, Value>> {
        Err(CoreError::Internal(
            "anthropic feature required for ClaudeLlmEvaluator".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "claude-llm-evaluator-stub"
    }

    fn is_ready(&self) -> bool {
        false
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn json_to_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            Value::Array(arr.into_iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(obj) => {
            Value::Object(obj.into_iter().map(|(k, v)| (k, json_to_value(v))).collect())
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_value_primitives() {
        assert_eq!(json_to_value(serde_json::json!(null)), Value::Null);
        assert_eq!(json_to_value(serde_json::json!(true)), Value::Bool(true));
        assert_eq!(json_to_value(serde_json::json!(42)), Value::Int(42));
        assert_eq!(json_to_value(serde_json::json!(3.14)), Value::Float(3.14));
        assert_eq!(
            json_to_value(serde_json::json!("hello")),
            Value::String("hello".to_string())
        );
    }

    #[cfg(feature = "anthropic")]
    mod anthropic_tests {
        use super::*;

        #[test]
        fn test_evaluator_creation() {
            let evaluator = ClaudeLlmEvaluator::new("test-key");
            assert_eq!(evaluator.name(), "claude-llm-evaluator");
            assert!(evaluator.is_ready());
        }

        #[test]
        fn test_evaluator_empty_key_not_ready() {
            let evaluator = ClaudeLlmEvaluator::new("");
            assert!(!evaluator.is_ready());
        }

        #[test]
        fn test_parse_response_boolean() {
            let evaluator = ClaudeLlmEvaluator::new("test");
            let outputs = vec!["result".to_string()];

            let result = evaluator
                .parse_response("true", &OutputFormat::Boolean, &outputs)
                .unwrap();
            assert_eq!(result.get("result"), Some(&Value::Bool(true)));

            let result = evaluator
                .parse_response("The answer is YES", &OutputFormat::Boolean, &outputs)
                .unwrap();
            assert_eq!(result.get("result"), Some(&Value::Bool(true)));

            let result = evaluator
                .parse_response("false", &OutputFormat::Boolean, &outputs)
                .unwrap();
            assert_eq!(result.get("result"), Some(&Value::Bool(false)));
        }

        #[test]
        fn test_parse_response_number() {
            let evaluator = ClaudeLlmEvaluator::new("test");
            let outputs = vec!["score".to_string()];

            let result = evaluator
                .parse_response("The score is 95", &OutputFormat::Number, &outputs)
                .unwrap();
            assert_eq!(result.get("score"), Some(&Value::Int(95)));
        }

        #[test]
        fn test_parse_response_text() {
            let evaluator = ClaudeLlmEvaluator::new("test");
            let outputs = vec!["message".to_string()];

            let result = evaluator
                .parse_response("  Hello World  ", &OutputFormat::Text, &outputs)
                .unwrap();
            assert_eq!(
                result.get("message"),
                Some(&Value::String("Hello World".to_string()))
            );
        }

        #[test]
        fn test_parse_response_json() {
            let evaluator = ClaudeLlmEvaluator::new("test");
            let outputs = vec!["name".to_string(), "age".to_string()];

            let result = evaluator
                .parse_response(r#"{"name": "Alice", "age": 30}"#, &OutputFormat::Json, &outputs)
                .unwrap();

            assert_eq!(
                result.get("name"),
                Some(&Value::String("Alice".to_string()))
            );
            assert_eq!(result.get("age"), Some(&Value::Int(30)));
        }
    }
}
