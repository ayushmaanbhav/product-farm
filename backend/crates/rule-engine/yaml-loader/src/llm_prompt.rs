//! LLM Prompt Preprocessing
//!
//! Handles prompt validation, optimization, and deduplication at YAML load time.
//! This ensures prompts are processed once during loading, not at runtime.

use crate::error::{LoaderError, LoaderResult};
use crate::schema::YamlFunction;
use std::collections::HashMap;

// =============================================================================
// Prompt Configuration
// =============================================================================

/// Preprocessed LLM configuration ready for runtime use.
#[derive(Debug, Clone)]
pub struct PreprocessedLlmConfig {
    /// The model to use (e.g., "claude-3-5-sonnet", "qwen2.5:7b")
    pub model: String,
    /// The final preprocessed prompt template
    pub prompt_template: String,
    /// Optional system prompt
    pub system_prompt: Option<String>,
    /// Temperature for generation (0.0 - 1.0)
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Output format: "json", "text", "boolean", "number"
    pub output_format: String,
    /// Provider identifier: "anthropic", "ollama", "openai"
    pub provider: String,
    /// Additional provider-specific options (renamed from extra_options for consistency)
    pub options: HashMap<String, serde_json::Value>,
}

impl Default for PreprocessedLlmConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            prompt_template: String::new(),
            system_prompt: None,
            temperature: 0.0,
            max_tokens: 1024,
            output_format: "json".to_string(),
            provider: "anthropic".to_string(),
            options: HashMap::new(),
        }
    }
}

// =============================================================================
// Instruction Deduplication
// =============================================================================

/// Detects if user's prompt already contains specific instructions.
/// Only returns true for 100% certain matches - conservative approach.
#[derive(Debug)]
struct InstructionDetector {
    /// User's prompt text (lowercased for matching)
    prompt_lower: String,
}

impl InstructionDetector {
    fn new(prompt: &str) -> Self {
        Self {
            prompt_lower: prompt.to_lowercase(),
        }
    }

    /// Check if user explicitly specified JSON output format with complete instructions.
    /// Must contain "json" AND some indication of structure or keys.
    fn has_json_format_instruction(&self) -> bool {
        let has_json = self.prompt_lower.contains("json");
        let has_structure = self.prompt_lower.contains("```json")
            || (self.prompt_lower.contains("{") && self.prompt_lower.contains("}"))
            || self.prompt_lower.contains("json object")
            || self.prompt_lower.contains("json format")
            || self.prompt_lower.contains("json with keys")
            || self.prompt_lower.contains("as json");

        has_json && has_structure
    }

    /// Check if user mentioned "only" output restrictions.
    fn has_output_only_instruction(&self) -> bool {
        let patterns = [
            "return only",
            "respond only",
            "output only",
            "return just",
            "only return",
            "only respond",
            "no explanation",
            "no additional text",
            "nothing else",
        ];
        patterns.iter().any(|p| self.prompt_lower.contains(p))
    }

    /// Check if user mentioned constraints with specific values.
    fn has_constraint_instruction(&self, constraint_values: &[&str]) -> bool {
        let mentions_constraints = self.prompt_lower.contains("constraint")
            || self.prompt_lower.contains("validation")
            || self.prompt_lower.contains("valid values")
            || self.prompt_lower.contains("allowed values")
            || self.prompt_lower.contains("must be one of");

        if !mentions_constraints {
            return false;
        }

        // Check if ALL constraint values are mentioned
        constraint_values.iter().all(|v| self.prompt_lower.contains(&v.to_lowercase()))
    }

    /// Check if user mentioned data types with specific type names.
    fn has_datatype_instruction(&self, type_names: &[&str]) -> bool {
        let mentions_types = self.prompt_lower.contains("type:")
            || self.prompt_lower.contains("type :")
            || self.prompt_lower.contains("data type")
            || self.prompt_lower.contains("datatype")
            || self.prompt_lower.contains("expected type")
            || self.prompt_lower.contains("of type")
            || self.prompt_lower.contains("must be");

        if !mentions_types {
            return false;
        }

        // Check if ALL type names are mentioned
        type_names.iter().all(|t| self.prompt_lower.contains(&t.to_lowercase()))
    }
}

// =============================================================================
// Supplementary Instructions Builder
// =============================================================================

/// Builds supplementary instructions that complement user's prompt.
/// Only adds instructions that are missing from user's prompt.
struct SupplementaryInstructionsBuilder {
    detector: InstructionDetector,
    instructions: Vec<String>,
}

impl SupplementaryInstructionsBuilder {
    fn new(user_prompt: &str) -> Self {
        Self {
            detector: InstructionDetector::new(user_prompt),
            instructions: Vec::new(),
        }
    }

    /// Add constraint/validation instruction if not already present.
    fn add_constraint_instruction_if_needed(
        &mut self,
        constraint_values: &[&str],
    ) -> &mut Self {
        if !constraint_values.is_empty()
            && !self.detector.has_constraint_instruction(constraint_values)
        {
            self.instructions.push(format!(
                "Valid values: {}",
                constraint_values.join(", ")
            ));
        }
        self
    }

    /// Add datatype instruction if not already present.
    fn add_datatype_instruction_if_needed(
        &mut self,
        output_name: &str,
        type_name: &str,
    ) -> &mut Self {
        if !self.detector.has_datatype_instruction(&[type_name]) {
            self.instructions.push(format!(
                "Output '{}' must be of type: {}",
                output_name, type_name
            ));
        }
        self
    }

    /// Add JSON format instruction if not already present.
    fn add_json_format_if_needed(&mut self, output_keys: &[&str]) -> &mut Self {
        if !self.detector.has_json_format_instruction() && !output_keys.is_empty() {
            self.instructions.push(format!(
                "Return JSON with keys: {}",
                output_keys.join(", ")
            ));
        }
        self
    }

    /// Build the final supplementary instructions string.
    fn build(&self) -> Option<String> {
        if self.instructions.is_empty() {
            None
        } else {
            Some(self.instructions.join("\n"))
        }
    }
}

// =============================================================================
// Prompt Preprocessor
// =============================================================================

/// Metadata about inputs/outputs for prompt preprocessing.
#[derive(Debug, Clone, Default)]
pub struct FunctionMetadata {
    /// Input attribute names
    pub inputs: Vec<String>,
    /// Output attribute names with their types
    pub outputs: Vec<OutputMetadata>,
    /// Function description
    pub description: Option<String>,
}

/// Metadata about a single output.
#[derive(Debug, Clone)]
pub struct OutputMetadata {
    /// Output attribute name
    pub name: String,
    /// Data type (e.g., "string", "number", "boolean", "enum[a,b,c]")
    pub data_type: Option<String>,
    /// Enum values if applicable
    pub enum_values: Option<Vec<String>>,
    /// Constraints description
    pub constraints: Option<String>,
}

/// Preprocesses LLM configuration from YAML at load time.
pub struct LlmPromptPreprocessor;

impl LlmPromptPreprocessor {
    /// Preprocess LLM configuration from a YAML function definition.
    ///
    /// If `prompt_template` is not provided, generates a default one based on
    /// the function's inputs, outputs, and description.
    pub fn preprocess(
        func: &YamlFunction,
        func_name: &str,
        metadata: &FunctionMetadata,
    ) -> LoaderResult<PreprocessedLlmConfig> {
        let config = func.evaluator_config.as_ref();

        // Extract model (required)
        let model = config
            .and_then(|c| c.get("model"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| LoaderError::MissingField {
                function: func_name.to_string(),
                field: "model".to_string(),
            })?;

        // Extract prompt_template (optional - will generate default if missing)
        let user_prompt = config
            .and_then(|c| c.get("prompt_template"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| Self::generate_default_prompt(func_name, metadata));

        // Build supplementary instructions (deduplication happens here)
        let final_prompt = Self::build_final_prompt(&user_prompt, metadata);

        // Extract optional fields
        let system_prompt = config
            .and_then(|c| c.get("system_prompt"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let temperature = config
            .and_then(|c| c.get("temperature"))
            .and_then(|v| v.as_f64())
            .map(|f| f as f32)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);

        let max_tokens = config
            .and_then(|c| c.get("max_tokens"))
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .unwrap_or(1024);

        let output_format = config
            .and_then(|c| c.get("output_format"))
            .and_then(|v| v.as_str())
            .unwrap_or("json")
            .to_string();

        // Extract provider - infer from model name if not explicitly set
        let provider = config
            .and_then(|c| c.get("provider"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Self::infer_provider_from_model(&model));

        // Collect extra options (anything not already extracted)
        let known_keys = ["model", "prompt_template", "system_prompt", "temperature", "max_tokens", "output_format", "provider"];
        let options: HashMap<String, serde_json::Value> = config
            .map(|c| {
                c.iter()
                    .filter(|(k, _)| !known_keys.contains(&k.as_str()))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(PreprocessedLlmConfig {
            model,
            prompt_template: final_prompt,
            system_prompt,
            temperature,
            max_tokens,
            output_format,
            provider,
            options,
        })
    }

    /// Build the final prompt by combining user prompt with supplementary instructions.
    fn build_final_prompt(user_prompt: &str, metadata: &FunctionMetadata) -> String {
        let mut builder = SupplementaryInstructionsBuilder::new(user_prompt);

        // Collect output keys for JSON format check
        let output_keys: Vec<&str> = metadata.outputs.iter().map(|o| o.name.as_str()).collect();

        // Add JSON format instruction if needed
        builder.add_json_format_if_needed(&output_keys);

        // Add type and constraint instructions for each output
        for output in &metadata.outputs {
            // Add datatype instruction if specified and not covered
            if let Some(ref dtype) = output.data_type {
                builder.add_datatype_instruction_if_needed(&output.name, dtype);
            }

            // Add constraint/enum values if specified and not covered
            if let Some(ref enum_vals) = output.enum_values {
                let vals: Vec<&str> = enum_vals.iter().map(|s| s.as_str()).collect();
                builder.add_constraint_instruction_if_needed(&vals);
            }
        }

        // Combine user prompt with supplementary instructions
        if let Some(supplements) = builder.build() {
            format!("{}\n\n---\n{}", user_prompt.trim(), supplements)
        } else {
            user_prompt.to_string()
        }
    }

    /// Generate a default prompt based on function metadata.
    /// Used when no prompt_template is provided in the YAML.
    fn generate_default_prompt(func_name: &str, metadata: &FunctionMetadata) -> String {
        let mut prompt = String::new();

        // Add function description if available
        if let Some(ref desc) = metadata.description {
            prompt.push_str(desc);
            prompt.push_str("\n\n");
        } else {
            prompt.push_str(&format!("Evaluate the '{}' function.\n\n", func_name));
        }

        // List inputs
        if !metadata.inputs.is_empty() {
            prompt.push_str("## Inputs\n");
            for input in &metadata.inputs {
                prompt.push_str(&format!("- {}: {{{{{}}}}}\n", input, input));
            }
            prompt.push('\n');
        }

        // List expected outputs with types
        if !metadata.outputs.is_empty() {
            prompt.push_str("## Expected Outputs\n");
            for output in &metadata.outputs {
                let mut output_desc = format!("- {}", output.name);
                if let Some(ref dtype) = output.data_type {
                    output_desc.push_str(&format!(" ({})", dtype));
                }
                if let Some(ref enums) = output.enum_values {
                    output_desc.push_str(&format!(": one of [{}]", enums.join(", ")));
                }
                output_desc.push('\n');
                prompt.push_str(&output_desc);
            }
            prompt.push('\n');
        }

        // Add output format instruction
        let output_names: Vec<&str> = metadata.outputs.iter().map(|o| o.name.as_str()).collect();
        if !output_names.is_empty() {
            prompt.push_str(&format!("Return JSON with keys: {}", output_names.join(", ")));
        }

        prompt
    }

    /// Infer provider from model name if not explicitly set.
    /// This provides sensible defaults based on common model naming conventions.
    fn infer_provider_from_model(model: &str) -> String {
        let model_lower = model.to_lowercase();

        if model_lower.contains("claude") || model_lower.contains("anthropic") {
            "anthropic".to_string()
        } else if model_lower.contains("gpt") || model_lower.contains("openai") || model_lower.contains("o1") {
            "openai".to_string()
        } else if model_lower.contains("llama") || model_lower.contains("qwen") || model_lower.contains("mistral")
            || model_lower.contains("gemma") || model_lower.contains("phi") || model_lower.contains("codellama")
            || model_lower.contains(":") // Ollama uses model:tag format
        {
            "ollama".to_string()
        } else {
            // Default to anthropic for unknown models
            "anthropic".to_string()
        }
    }

    /// Convert preprocessed config to HashMap<String, Value> for RuleEvaluator.
    pub fn to_evaluator_config(config: &PreprocessedLlmConfig) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();

        map.insert("model".to_string(), serde_json::Value::String(config.model.clone()));
        map.insert("prompt_template".to_string(), serde_json::Value::String(config.prompt_template.clone()));
        map.insert("temperature".to_string(), serde_json::json!(config.temperature));
        map.insert("max_tokens".to_string(), serde_json::json!(config.max_tokens));
        map.insert("output_format".to_string(), serde_json::Value::String(config.output_format.clone()));
        map.insert("provider".to_string(), serde_json::Value::String(config.provider.clone()));

        if let Some(ref sys) = config.system_prompt {
            map.insert("system_prompt".to_string(), serde_json::Value::String(sys.clone()));
        }

        // Add provider-specific options
        for (k, v) in &config.options {
            map.insert(k.clone(), v.clone());
        }

        map
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_detector_json_format() {
        let detector = InstructionDetector::new("Return the result as a JSON object with {\"score\": value}");
        assert!(detector.has_json_format_instruction());

        let detector = InstructionDetector::new("Calculate the score");
        assert!(!detector.has_json_format_instruction());

        let detector = InstructionDetector::new("Format response as:\n```json\n{}\n```");
        assert!(detector.has_json_format_instruction());
    }

    #[test]
    fn test_instruction_detector_output_only() {
        let detector = InstructionDetector::new("Return only the number, no explanation");
        assert!(detector.has_output_only_instruction());

        let detector = InstructionDetector::new("Calculate and explain your reasoning");
        assert!(!detector.has_output_only_instruction());
    }

    #[test]
    fn test_instruction_detector_constraints() {
        let detector = InstructionDetector::new("Valid values are: easy, medium, hard");
        assert!(detector.has_constraint_instruction(&["easy", "medium", "hard"]));

        // Incomplete - missing "hard"
        let detector = InstructionDetector::new("Valid values are: easy, medium");
        assert!(!detector.has_constraint_instruction(&["easy", "medium", "hard"]));
    }

    #[test]
    fn test_supplementary_builder_adds_missing() {
        let user_prompt = "Calculate the premium based on age";
        let mut builder = SupplementaryInstructionsBuilder::new(user_prompt);

        builder.add_json_format_if_needed(&["premium"]);
        builder.add_datatype_instruction_if_needed("premium", "number");

        let result = builder.build().unwrap();
        assert!(result.contains("JSON with keys: premium"));
        assert!(result.contains("type: number"));
    }

    #[test]
    fn test_supplementary_builder_skips_existing() {
        let user_prompt = "Calculate and return JSON with keys: premium. Output must be of type: number.";
        let mut builder = SupplementaryInstructionsBuilder::new(user_prompt);

        builder.add_json_format_if_needed(&["premium"]);
        builder.add_datatype_instruction_if_needed("premium", "number");

        // Should not add anything since user already has these
        assert!(builder.build().is_none());
    }

    #[test]
    fn test_build_final_prompt_no_supplements() {
        let user_prompt = "Calculate premium. Return JSON with keys: premium. Type: number.";
        let metadata = FunctionMetadata {
            inputs: vec!["age".to_string()],
            outputs: vec![OutputMetadata {
                name: "premium".to_string(),
                data_type: Some("number".to_string()),
                enum_values: None,
                constraints: None,
            }],
            description: None,
        };

        let final_prompt = LlmPromptPreprocessor::build_final_prompt(user_prompt, &metadata);

        // Should be unchanged since user covered everything
        assert_eq!(final_prompt, user_prompt);
    }

    #[test]
    fn test_build_final_prompt_with_supplements() {
        let user_prompt = "Calculate the premium based on age.";
        let metadata = FunctionMetadata {
            inputs: vec!["age".to_string()],
            outputs: vec![OutputMetadata {
                name: "premium".to_string(),
                data_type: Some("number".to_string()),
                enum_values: None,
                constraints: None,
            }],
            description: None,
        };

        let final_prompt = LlmPromptPreprocessor::build_final_prompt(user_prompt, &metadata);

        // Should add JSON and type instructions
        assert!(final_prompt.contains("Calculate the premium"));
        assert!(final_prompt.contains("---"));
        assert!(final_prompt.contains("JSON with keys: premium"));
        assert!(final_prompt.contains("type: number"));
    }

    #[test]
    fn test_build_final_prompt_with_enum() {
        let user_prompt = "Classify the difficulty level.";
        let metadata = FunctionMetadata {
            inputs: vec!["score".to_string()],
            outputs: vec![OutputMetadata {
                name: "difficulty".to_string(),
                data_type: Some("enum".to_string()),
                enum_values: Some(vec!["easy".to_string(), "medium".to_string(), "hard".to_string()]),
                constraints: None,
            }],
            description: None,
        };

        let final_prompt = LlmPromptPreprocessor::build_final_prompt(user_prompt, &metadata);

        // Should add enum values
        assert!(final_prompt.contains("easy, medium, hard"));
    }
}
