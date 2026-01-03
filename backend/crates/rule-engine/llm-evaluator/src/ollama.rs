//! Ollama LLM Evaluator Implementation
//!
//! Local Ollama-based implementation of the LlmEvaluator trait.
//! Useful for development, testing, and self-hosted deployments.

use crate::config::{LlmEvaluatorConfig, OutputFormat};
use crate::error::{LlmEvaluatorError, LlmEvaluatorResult};
use async_trait::async_trait;
use product_farm_core::{CoreError, CoreResult, LlmEvaluator, Value};
use std::collections::HashMap;
use tracing::{debug, info, warn};

// =============================================================================
// Ollama Client
// =============================================================================

#[cfg(feature = "ollama")]
mod client {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize)]
    pub struct OllamaRequest {
        pub model: String,
        pub prompt: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub system: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub temperature: Option<f32>,
        pub stream: bool,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct OllamaResponse {
        pub response: String,
        #[serde(default)]
        pub done: bool,
        #[serde(default)]
        pub total_duration: Option<u64>,
        #[serde(default)]
        pub eval_count: Option<u64>,
    }

    pub struct OllamaClient {
        client: reqwest::Client,
        base_url: String,
    }

    impl OllamaClient {
        pub fn new(base_url: String) -> Self {
            Self {
                client: reqwest::Client::new(),
                base_url,
            }
        }

        pub async fn generate(&self, request: OllamaRequest) -> LlmEvaluatorResult<OllamaResponse> {
            let url = format!("{}/api/generate", self.base_url);

            let response = self
                .client
                .post(&url)
                .json(&request)
                .send()
                .await
                .map_err(|e| LlmEvaluatorError::ApiError(format!("Ollama request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(LlmEvaluatorError::ApiError(format!(
                    "Ollama API returned {}: {}",
                    status, error_body
                )));
            }

            response
                .json::<OllamaResponse>()
                .await
                .map_err(|e| LlmEvaluatorError::ApiError(format!("Failed to parse Ollama response: {}", e)))
        }

        /// Check if Ollama is available and the model is loaded
        pub async fn health_check(&self, model: &str) -> bool {
            let request = OllamaRequest {
                model: model.to_string(),
                prompt: "Hi".to_string(),
                system: None,
                temperature: None,
                stream: false,
            };

            match self.generate(request).await {
                Ok(_) => true,
                Err(e) => {
                    warn!(error = %e, "Ollama health check failed");
                    false
                }
            }
        }
    }
}

// =============================================================================
// Ollama LLM Evaluator
// =============================================================================

/// Ollama-based LLM evaluator for local rule evaluation.
///
/// This evaluator uses a local Ollama instance for LLM inference,
/// making it ideal for development, testing, and privacy-sensitive deployments.
#[cfg(feature = "ollama")]
pub struct OllamaLlmEvaluator {
    base_url: String,
    default_model: String,
}

#[cfg(feature = "ollama")]
impl OllamaLlmEvaluator {
    /// Create a new Ollama LLM evaluator.
    ///
    /// # Arguments
    /// * `base_url` - Ollama API base URL (e.g., "http://localhost:11434")
    /// * `model` - Default model to use (e.g., "qwen2.5:7b", "llama3.2")
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            default_model: model.into(),
        }
    }

    /// Create with default localhost URL from environment config
    pub fn localhost(model: impl Into<String>) -> Self {
        let env_config = crate::env_config::RuleEngineLlmConfig::from_env();
        Self::new(env_config.ollama.base_url, model)
    }

    /// Create from environment configuration.
    ///
    /// Reads from `RULE_ENGINE_OLLAMA_BASE_URL` and `RULE_ENGINE_OLLAMA_MODEL`.
    pub fn from_env() -> LlmEvaluatorResult<Self> {
        let env_config = crate::env_config::RuleEngineLlmConfig::from_env();
        Ok(Self::new(env_config.ollama.base_url, env_config.ollama.model))
    }

    /// Create with default configuration (Ollama is the default provider)
    pub fn default_from_env() -> Self {
        let env_config = crate::env_config::RuleEngineLlmConfig::from_env();
        Self::new(env_config.ollama.base_url, env_config.ollama.model)
    }

    /// Check if Ollama is available
    pub async fn is_available(&self) -> bool {
        let client = client::OllamaClient::new(self.base_url.clone());
        client.health_check(&self.default_model).await
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
                            // Try to find JSON object in response
                            if let Some(start) = trimmed.find('{') {
                                if let Some(end) = trimmed.rfind('}') {
                                    return serde_json::from_str(&trimmed[start..=end]);
                                }
                            }
                            Err(serde_json::Error::io(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Not valid JSON",
                            )))
                        }
                    })
                    .map_err(|e| LlmEvaluatorError::ParseError(format!(
                        "Failed to parse JSON: {}. Response: {}",
                        e, response_text
                    )))?;

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
                let text = response_text.trim().to_lowercase();
                let value = text.contains("true") || text.contains("yes") || text == "1";
                if let Some(name) = output_names.first() {
                    outputs.insert(name.clone(), Value::Bool(value));
                }
            }

            OutputFormat::Number => {
                // Extract first number from response
                let num: f64 = response_text
                    .chars()
                    .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                    .collect::<String>()
                    .parse()
                    .map_err(|e| LlmEvaluatorError::ParseError(format!("Failed to parse number: {}", e)))?;

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

#[cfg(feature = "ollama")]
#[async_trait]
impl LlmEvaluator for OllamaLlmEvaluator {
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

        // Use model from config or default
        let model = if config.model.is_empty() {
            self.default_model.clone()
        } else {
            config.model.clone()
        };

        info!(
            model = %model,
            outputs = ?output_names,
            "Evaluating rule with Ollama"
        );

        // Build the prompt
        let prompt = config.render_prompt_with_outputs(inputs, output_names);
        debug!(prompt = %prompt, "Rendered prompt");

        // Create request
        let request = OllamaRequest {
            model,
            prompt,
            system: config.system_prompt.clone(),
            temperature: Some(config.temperature),
            stream: false,
        };

        // Send request
        let client = OllamaClient::new(self.base_url.clone());
        let response = client
            .generate(request)
            .await
            .map_err(|e| CoreError::Internal(format!("Ollama API error: {}", e)))?;

        let response_text = response.response;
        debug!(response = %response_text, "Ollama response");

        // Parse response based on output format
        let outputs = self
            .parse_response(&response_text, &config.output_format, output_names)
            .map_err(|e| CoreError::Internal(format!("Failed to parse Ollama response: {}", e)))?;

        info!(outputs = ?outputs.keys().collect::<Vec<_>>(), "Ollama evaluation complete");
        Ok(outputs)
    }

    fn name(&self) -> &str {
        "ollama-llm-evaluator"
    }

    fn is_ready(&self) -> bool {
        !self.base_url.is_empty() && !self.default_model.is_empty()
    }
}

// =============================================================================
// Stub when ollama feature is not enabled
// =============================================================================

#[cfg(not(feature = "ollama"))]
pub struct OllamaLlmEvaluator;

#[cfg(not(feature = "ollama"))]
impl OllamaLlmEvaluator {
    pub fn new(_base_url: impl Into<String>, _model: impl Into<String>) -> Self {
        Self
    }

    pub fn localhost(_model: impl Into<String>) -> Self {
        Self
    }

    pub fn from_env() -> LlmEvaluatorResult<Self> {
        Err(LlmEvaluatorError::FeatureNotEnabled(
            "ollama feature required for OllamaLlmEvaluator".to_string(),
        ))
    }

    pub fn default_from_env() -> Self {
        Self
    }
}

#[cfg(not(feature = "ollama"))]
#[async_trait]
impl LlmEvaluator for OllamaLlmEvaluator {
    async fn evaluate(
        &self,
        _config: &HashMap<String, Value>,
        _inputs: &HashMap<String, Value>,
        _output_names: &[String],
    ) -> CoreResult<HashMap<String, Value>> {
        Err(CoreError::Internal(
            "ollama feature required for OllamaLlmEvaluator".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "ollama-llm-evaluator-stub"
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
    fn test_ollama_evaluator_creation() {
        let evaluator = OllamaLlmEvaluator::new("http://localhost:11434", "qwen2.5:7b");
        assert_eq!(evaluator.name(), if cfg!(feature = "ollama") { "ollama-llm-evaluator" } else { "ollama-llm-evaluator-stub" });
    }

    #[test]
    fn test_ollama_localhost() {
        let evaluator = OllamaLlmEvaluator::localhost("llama3.2");
        #[cfg(feature = "ollama")]
        {
            assert!(evaluator.is_ready());
        }
    }

    #[cfg(feature = "ollama")]
    mod ollama_tests {
        use super::*;

        #[test]
        fn test_parse_response_json() {
            let evaluator = OllamaLlmEvaluator::localhost("test");
            let outputs = vec!["result".to_string()];

            let result = evaluator
                .parse_response(r#"{"result": 42}"#, &OutputFormat::Json, &outputs)
                .unwrap();
            assert_eq!(result.get("result"), Some(&Value::Int(42)));
        }

        #[test]
        fn test_parse_response_json_in_markdown() {
            let evaluator = OllamaLlmEvaluator::localhost("test");
            let outputs = vec!["score".to_string()];

            let response = r#"Here is the result:
```json
{"score": 85}
```"#;

            let result = evaluator
                .parse_response(response, &OutputFormat::Json, &outputs)
                .unwrap();
            assert_eq!(result.get("score"), Some(&Value::Int(85)));
        }

        #[test]
        fn test_parse_response_boolean() {
            let evaluator = OllamaLlmEvaluator::localhost("test");
            let outputs = vec!["approved".to_string()];

            let result = evaluator
                .parse_response("Yes, this is approved", &OutputFormat::Boolean, &outputs)
                .unwrap();
            assert_eq!(result.get("approved"), Some(&Value::Bool(true)));
        }

        #[test]
        fn test_parse_response_number() {
            let evaluator = OllamaLlmEvaluator::localhost("test");
            let outputs = vec!["premium".to_string()];

            let result = evaluator
                .parse_response("The premium is $150.50 per month", &OutputFormat::Number, &outputs)
                .unwrap();
            assert_eq!(result.get("premium"), Some(&Value::Float(150.50)));
        }
    }
}
