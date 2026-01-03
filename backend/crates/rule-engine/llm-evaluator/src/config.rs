//! LLM Evaluator Configuration
//!
//! Configuration types for LLM-based rule evaluation.

use product_farm_core::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for LLM-based rule evaluation.
///
/// This config specifies how an LLM should evaluate a rule, including
/// model selection, generation parameters, and prompt templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmEvaluatorConfig {
    /// The model identifier (e.g., "claude-3-5-sonnet-20241022", "gpt-4o")
    pub model: String,

    /// Temperature for response generation (0.0 - 1.0)
    /// Lower values = more deterministic, higher = more creative
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// The prompt template with placeholders for inputs.
    /// Use `{{input_name}}` for variable substitution.
    /// Use `{{__inputs_json__}}` for all inputs as JSON.
    /// Use `{{__output_names__}}` for expected output names.
    pub prompt_template: String,

    /// Optional system prompt to set context for the LLM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,

    /// Maximum tokens to generate in the response
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Optional: expected output format ("json", "text", "boolean", "number")
    #[serde(default)]
    pub output_format: OutputFormat,

    /// Provider identifier (e.g., "anthropic", "openai", "ollama")
    #[serde(default = "default_provider")]
    pub provider: String,

    /// Additional provider-specific options
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub options: HashMap<String, Value>,
}

fn default_temperature() -> f32 {
    crate::env_config::GlobalLlmConfig::default().temperature
}

fn default_max_tokens() -> u32 {
    crate::env_config::GlobalLlmConfig::default().max_output_tokens
}

fn default_provider() -> String {
    crate::env_config::GlobalLlmConfig::default().provider
}

/// Output format expected from the LLM
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// JSON object with named outputs
    #[default]
    Json,
    /// Plain text response
    Text,
    /// Boolean true/false
    Boolean,
    /// Numeric value
    Number,
    /// Array of values
    Array,
}

impl LlmEvaluatorConfig {
    /// Create a new config with required fields
    pub fn new(model: impl Into<String>, prompt_template: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            temperature: default_temperature(),
            prompt_template: prompt_template.into(),
            system_prompt: None,
            max_tokens: default_max_tokens(),
            output_format: OutputFormat::default(),
            provider: default_provider(),
            options: HashMap::new(),
        }
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature.clamp(0.0, 1.0);
        self
    }

    /// Set the system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set the max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set the output format
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Set the provider
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = provider.into();
        self
    }

    /// Add a provider-specific option
    pub fn with_option(mut self, key: impl Into<String>, value: Value) -> Self {
        self.options.insert(key.into(), value);
        self
    }

    /// Render the prompt template with the given inputs
    pub fn render_prompt(&self, inputs: &HashMap<String, Value>) -> String {
        let mut prompt = self.prompt_template.clone();

        // Replace {{__inputs_json__}} with all inputs as JSON
        if prompt.contains("{{__inputs_json__}}") {
            let json = serde_json::to_string_pretty(&inputs).unwrap_or_default();
            prompt = prompt.replace("{{__inputs_json__}}", &json);
        }

        // Replace individual {{input_name}} placeholders
        for (key, value) in inputs {
            let placeholder = format!("{{{{{}}}}}", key);
            let value_str = match value {
                Value::String(s) => s.clone(),
                other => serde_json::to_string(other).unwrap_or_default(),
            };
            prompt = prompt.replace(&placeholder, &value_str);
        }

        prompt
    }

    /// Render the prompt with output names included
    pub fn render_prompt_with_outputs(
        &self,
        inputs: &HashMap<String, Value>,
        output_names: &[String],
    ) -> String {
        let mut prompt = self.render_prompt(inputs);

        // Replace {{__output_names__}} with the list of expected outputs
        if prompt.contains("{{__output_names__}}") {
            let names = output_names.join(", ");
            prompt = prompt.replace("{{__output_names__}}", &names);
        }

        prompt
    }

    /// Convert config to a HashMap for the evaluator trait
    pub fn to_config_map(&self) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        map.insert("model".to_string(), Value::String(self.model.clone()));
        map.insert("temperature".to_string(), Value::Float(self.temperature as f64));
        map.insert("prompt_template".to_string(), Value::String(self.prompt_template.clone()));
        map.insert("max_tokens".to_string(), Value::Int(self.max_tokens as i64));
        map.insert("provider".to_string(), Value::String(self.provider.clone()));

        if let Some(ref sys) = self.system_prompt {
            map.insert("system_prompt".to_string(), Value::String(sys.clone()));
        }

        let format_str = match self.output_format {
            OutputFormat::Json => "json",
            OutputFormat::Text => "text",
            OutputFormat::Boolean => "boolean",
            OutputFormat::Number => "number",
            OutputFormat::Array => "array",
        };
        map.insert("output_format".to_string(), Value::String(format_str.to_string()));

        map
    }

    /// Create config from a HashMap (inverse of to_config_map)
    pub fn from_config_map(map: &HashMap<String, Value>) -> Option<Self> {
        let model = match map.get("model")? {
            Value::String(s) => s.clone(),
            _ => return None,
        };

        let prompt_template = match map.get("prompt_template")? {
            Value::String(s) => s.clone(),
            _ => return None,
        };

        let temperature = match map.get("temperature") {
            Some(Value::Float(f)) => *f as f32,
            Some(Value::Int(i)) => *i as f32,
            _ => default_temperature(),
        };

        let max_tokens = match map.get("max_tokens") {
            Some(Value::Int(i)) => *i as u32,
            _ => default_max_tokens(),
        };

        let provider = match map.get("provider") {
            Some(Value::String(s)) => s.clone(),
            _ => default_provider(),
        };

        let system_prompt = match map.get("system_prompt") {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        };

        let output_format = match map.get("output_format") {
            Some(Value::String(s)) => match s.as_str() {
                "json" => OutputFormat::Json,
                "text" => OutputFormat::Text,
                "boolean" => OutputFormat::Boolean,
                "number" => OutputFormat::Number,
                "array" => OutputFormat::Array,
                _ => OutputFormat::default(),
            },
            _ => OutputFormat::default(),
        };

        Some(Self {
            model,
            temperature,
            prompt_template,
            system_prompt,
            max_tokens,
            output_format,
            provider,
            options: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = LlmEvaluatorConfig::new("claude-3-5-sonnet", "Evaluate: {{input}}");
        assert_eq!(config.model, "claude-3-5-sonnet");
        assert_eq!(config.prompt_template, "Evaluate: {{input}}");
        assert_eq!(config.temperature, 0.0);
        assert_eq!(config.max_tokens, 1024);
        // Default provider is now ollama (from env_config)
        assert_eq!(config.provider, "ollama");
        assert_eq!(config.output_format, OutputFormat::Json);
    }

    #[test]
    fn test_config_builder() {
        let config = LlmEvaluatorConfig::new("gpt-4o", "Test prompt")
            .with_temperature(0.7)
            .with_system_prompt("You are a helpful assistant")
            .with_max_tokens(2048)
            .with_output_format(OutputFormat::Boolean)
            .with_provider("openai");

        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.system_prompt, Some("You are a helpful assistant".to_string()));
        assert_eq!(config.max_tokens, 2048);
        assert_eq!(config.output_format, OutputFormat::Boolean);
        assert_eq!(config.provider, "openai");
    }

    #[test]
    fn test_config_temperature_clamped() {
        let config1 = LlmEvaluatorConfig::new("m", "p").with_temperature(1.5);
        assert_eq!(config1.temperature, 1.0);

        let config2 = LlmEvaluatorConfig::new("m", "p").with_temperature(-0.5);
        assert_eq!(config2.temperature, 0.0);
    }

    #[test]
    fn test_render_prompt_simple() {
        let config = LlmEvaluatorConfig::new("model", "Hello {{name}}, your score is {{score}}");
        let mut inputs = HashMap::new();
        inputs.insert("name".to_string(), Value::String("Alice".to_string()));
        inputs.insert("score".to_string(), Value::Int(95));

        let rendered = config.render_prompt(&inputs);
        assert_eq!(rendered, "Hello Alice, your score is 95");
    }

    #[test]
    fn test_render_prompt_with_json() {
        let config = LlmEvaluatorConfig::new("model", "Inputs: {{__inputs_json__}}");
        let mut inputs = HashMap::new();
        inputs.insert("x".to_string(), Value::Int(1));

        let rendered = config.render_prompt(&inputs);
        assert!(rendered.contains("\"x\": 1"));
    }

    #[test]
    fn test_render_prompt_with_outputs() {
        let config = LlmEvaluatorConfig::new("model", "Return: {{__output_names__}}");
        let inputs = HashMap::new();
        let outputs = vec!["result".to_string(), "confidence".to_string()];

        let rendered = config.render_prompt_with_outputs(&inputs, &outputs);
        assert_eq!(rendered, "Return: result, confidence");
    }

    #[test]
    fn test_config_map_roundtrip() {
        let config = LlmEvaluatorConfig::new("claude-3", "Test {{input}}")
            .with_temperature(0.5)
            .with_system_prompt("System")
            .with_max_tokens(512)
            .with_output_format(OutputFormat::Text)
            .with_provider("anthropic");

        let map = config.to_config_map();
        let restored = LlmEvaluatorConfig::from_config_map(&map).unwrap();

        assert_eq!(restored.model, config.model);
        assert_eq!(restored.temperature, config.temperature);
        assert_eq!(restored.prompt_template, config.prompt_template);
        assert_eq!(restored.system_prompt, config.system_prompt);
        assert_eq!(restored.max_tokens, config.max_tokens);
        assert_eq!(restored.output_format, config.output_format);
        assert_eq!(restored.provider, config.provider);
    }

    #[test]
    fn test_config_serialization() {
        let config = LlmEvaluatorConfig::new("model", "prompt")
            .with_temperature(0.3);

        let json = serde_json::to_string(&config).unwrap();
        let restored: LlmEvaluatorConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.model, "model");
        assert_eq!(restored.temperature, 0.3);
    }

    #[test]
    fn test_output_format_serialization() {
        assert_eq!(
            serde_json::to_string(&OutputFormat::Json).unwrap(),
            "\"json\""
        );
        assert_eq!(
            serde_json::to_string(&OutputFormat::Boolean).unwrap(),
            "\"boolean\""
        );
    }
}
