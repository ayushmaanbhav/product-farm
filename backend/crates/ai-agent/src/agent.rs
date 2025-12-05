//! AI Agent for Rule Management
//!
//! High-level agent that orchestrates tool calls with Claude API
//! for natural language rule management.

use crate::client::{
    ClaudeClient, ClaudeConfig, ClaudeResponse, Message, ToolDefinition, ToolUse,
};
use crate::error::{AgentError, AgentResult};
use crate::explainer::{RuleExplainer, Verbosity};
use crate::tools::{
    get_tool_definitions, CreateRuleInput, CreateRuleOutput, ExplainRuleInput,
    ExplainRuleOutput, TestRuleInput, TestRuleOutput, ValidateRuleInput, ValidateRuleOutput,
};
use crate::translator::{RuleTranslator, TranslationContext};
use crate::validator::RuleValidator;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Maximum number of tool call iterations
const MAX_TOOL_ITERATIONS: usize = 10;

/// AI Agent for rule management
pub struct RuleAgent {
    client: ClaudeClient,
    translator: RuleTranslator,
    validator: RuleValidator,
    /// Conversation history
    messages: Vec<Message>,
    /// Tool definitions for Claude
    tools: Vec<ToolDefinition>,
}

impl RuleAgent {
    /// Create a new rule agent with the given Claude configuration
    pub fn new(config: ClaudeConfig) -> Self {
        let system_prompt = Self::build_system_prompt();
        let config = config.with_system_prompt(system_prompt);

        let tools: Vec<ToolDefinition> = get_tool_definitions()
            .into_iter()
            .map(ToolDefinition::from)
            .collect();

        Self {
            client: ClaudeClient::new(config),
            translator: RuleTranslator::new(TranslationContext::new()),
            validator: RuleValidator::new(),
            messages: Vec::new(),
            tools,
        }
    }

    /// Create agent with custom translation context
    pub fn with_context(config: ClaudeConfig, context: TranslationContext) -> Self {
        let system_prompt = Self::build_system_prompt_with_context(&context);
        let config = config.with_system_prompt(system_prompt);

        let tools: Vec<ToolDefinition> = get_tool_definitions()
            .into_iter()
            .map(ToolDefinition::from)
            .collect();

        Self {
            client: ClaudeClient::new(config),
            translator: RuleTranslator::new(context),
            validator: RuleValidator::new(),
            messages: Vec::new(),
            tools,
        }
    }

    fn build_system_prompt() -> String {
        r#"You are an AI assistant specialized in creating and managing business rules for Product-FARM, a financial product rule engine.

Your capabilities:
1. Create rules from natural language descriptions
2. Validate rule syntax and logic
3. Explain existing rules in plain English
4. Test rules with sample inputs
5. Visualize rule dependencies

When creating rules:
- Use JSON Logic format for rule expressions
- Ensure proper variable references with {"var": "path"}
- Consider type safety (numeric vs string comparisons)
- Add appropriate validation for edge cases

Always validate rules before confirming creation.
"#.to_string()
    }

    fn build_system_prompt_with_context(context: &TranslationContext) -> String {
        let mut prompt = Self::build_system_prompt();
        prompt.push('\n');
        prompt.push_str(&context.to_system_prompt());
        prompt
    }

    /// Process a user message and return the response
    pub async fn process(&mut self, user_message: impl Into<String>) -> AgentResult<String> {
        let user_msg = Message::user(user_message);
        self.messages.push(user_msg);

        let mut iterations = 0;
        loop {
            if iterations >= MAX_TOOL_ITERATIONS {
                return Err(AgentError::MaxIterationsReached(MAX_TOOL_ITERATIONS));
            }

            let response = self
                .client
                .chat_with_tools(self.messages.clone(), self.tools.clone())
                .await?;

            debug!(
                "Claude response: stop_reason={:?}, tool_uses={}",
                response.stop_reason,
                response.tool_uses().len()
            );

            if response.needs_tool_use() {
                self.handle_tool_calls(&response).await?;
                iterations += 1;
            } else {
                // Conversation complete
                if let Some(text) = response.text() {
                    self.messages.push(Message::assistant(&text));
                    return Ok(text);
                } else {
                    return Ok("I processed your request but have no additional response.".to_string());
                }
            }
        }
    }

    /// Handle tool calls from Claude's response
    async fn handle_tool_calls(&mut self, response: &ClaudeResponse) -> AgentResult<()> {
        let tool_uses = response.tool_uses();

        // Add assistant message with tool uses
        for tool_use in &tool_uses {
            self.messages
                .push(Message::assistant_with_tool_use(tool_use.clone()));
        }

        // Execute each tool and add results
        for tool_use in tool_uses {
            info!("Executing tool: {}", tool_use.name);
            let result = self.execute_tool(&tool_use).await;

            match result {
                Ok(output) => {
                    self.messages
                        .push(Message::tool_result(&tool_use.id, output));
                }
                Err(e) => {
                    warn!("Tool {} failed: {}", tool_use.name, e);
                    self.messages.push(Message::tool_result(
                        &tool_use.id,
                        serde_json::json!({
                            "error": e.to_string()
                        }),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Execute a single tool
    async fn execute_tool(&self, tool_use: &ToolUse) -> AgentResult<Value> {
        match tool_use.name.as_str() {
            "create_rule" => {
                let input: CreateRuleInput = serde_json::from_value(tool_use.input.clone())
                    .map_err(|e| AgentError::ToolError(format!("Invalid input: {}", e)))?;
                let output = self.create_rule(input)?;
                Ok(serde_json::to_value(output).unwrap())
            }
            "validate_rule" => {
                let input: ValidateRuleInput = serde_json::from_value(tool_use.input.clone())
                    .map_err(|e| AgentError::ToolError(format!("Invalid input: {}", e)))?;
                let output = self.validate_rule(input)?;
                Ok(serde_json::to_value(output).unwrap())
            }
            "explain_rule" => {
                let input: ExplainRuleInput = serde_json::from_value(tool_use.input.clone())
                    .map_err(|e| AgentError::ToolError(format!("Invalid input: {}", e)))?;
                let output = self.explain_rule(input)?;
                Ok(serde_json::to_value(output).unwrap())
            }
            "test_rule" => {
                let input: TestRuleInput = serde_json::from_value(tool_use.input.clone())
                    .map_err(|e| AgentError::ToolError(format!("Invalid input: {}", e)))?;
                let output = self.test_rule(input)?;
                Ok(serde_json::to_value(output).unwrap())
            }
            "list_rules" | "visualize_graph" | "analyze_impact" | "suggest_optimizations" => {
                // These tools require persistence layer - return placeholder
                Ok(serde_json::json!({
                    "message": format!("Tool '{}' requires product context from persistence layer", tool_use.name),
                    "status": "not_implemented"
                }))
            }
            _ => Err(AgentError::ToolError(format!(
                "Unknown tool: {}",
                tool_use.name
            ))),
        }
    }

    /// Create a rule from natural language or JSON Logic
    fn create_rule(&self, input: CreateRuleInput) -> AgentResult<CreateRuleOutput> {
        // For now, we expect the LLM to have translated the description to JSON Logic
        // In a full implementation, we would call back to Claude for translation

        // The description should contain or reference a JSON Logic expression
        // This is a simplified implementation
        let expression = self.parse_expression_from_description(&input.description)?;

        let input_attrs = input.input_attributes.unwrap_or_default();
        let output_attrs = input.output_attributes.unwrap_or_default();

        self.translator
            .validate_expression(&expression, &input_attrs, &output_attrs)
            .map(|mut output| {
                output.rule.rule_type = input.rule_type;
                output.rule.description = input.description;
                output
            })
    }

    fn parse_expression_from_description(&self, description: &str) -> AgentResult<Value> {
        // Try to extract JSON from the description
        // Look for JSON-like content between { and }
        if let Some(start) = description.find('{') {
            if let Some(end) = description.rfind('}') {
                let json_str = &description[start..=end];
                return serde_json::from_str(json_str)
                    .map_err(|e| AgentError::JsonLogicParseError(e.to_string()));
            }
        }

        Err(AgentError::ToolError(
            "Could not extract JSON Logic expression from description".to_string(),
        ))
    }

    /// Validate a rule
    fn validate_rule(&self, input: ValidateRuleInput) -> AgentResult<ValidateRuleOutput> {
        self.validator
            .validate(&input.expression, &input.input_attributes, &input.output_attributes)
    }

    /// Explain a rule
    fn explain_rule(&self, input: ExplainRuleInput) -> AgentResult<ExplainRuleOutput> {
        let expression = input
            .expression
            .ok_or_else(|| AgentError::ToolError("Expression required".to_string()))?;

        let verbosity = match input.verbosity.as_str() {
            "brief" => Verbosity::Brief,
            "technical" => Verbosity::Technical,
            _ => Verbosity::Detailed,
        };

        let explainer = RuleExplainer::new(verbosity);
        explainer.explain(&expression)
    }

    /// Test a rule with sample inputs
    fn test_rule(&self, input: TestRuleInput) -> AgentResult<TestRuleOutput> {
        let expression = input
            .expression
            .ok_or_else(|| AgentError::ToolError("Expression required".to_string()))?;

        let start = std::time::Instant::now();

        let data = serde_json::to_value(&input.inputs)
            .map_err(|e| AgentError::ToolError(format!("Invalid inputs: {}", e)))?;

        match product_farm_json_logic::evaluate(&expression, &data) {
            Ok(result) => {
                let execution_time_ns = start.elapsed().as_nanos() as u64;
                // Convert Value to serde_json::Value
                let result_json = serde_json::to_value(&result)
                    .map_err(|e| AgentError::ToolError(format!("Failed to serialize result: {}", e)))?;
                Ok(TestRuleOutput {
                    success: true,
                    result: Some(result_json),
                    error: None,
                    execution_time_ns,
                    intermediate_values: HashMap::new(),
                })
            }
            Err(e) => {
                let execution_time_ns = start.elapsed().as_nanos() as u64;
                Ok(TestRuleOutput {
                    success: false,
                    result: None,
                    error: Some(e.to_string()),
                    execution_time_ns,
                    intermediate_values: HashMap::new(),
                })
            }
        }
    }

    /// Translate natural language to JSON Logic expression
    ///
    /// This method calls Claude API to translate a natural language description
    /// into a valid JSON Logic expression.
    ///
    /// # Example
    /// ```ignore
    /// let result = agent.translate_to_json_logic(
    ///     "If age is greater than 60, multiply base_rate by 1.2, otherwise use base_rate",
    ///     &["age", "base_rate"],
    ///     &["adjusted_rate"]
    /// ).await?;
    /// ```
    pub async fn translate_to_json_logic(
        &self,
        natural_language: &str,
        input_attributes: &[&str],
        output_attributes: &[&str],
    ) -> AgentResult<TranslationResult> {
        let prompt = format!(
            r#"Convert this natural language rule description to JSON Logic.

Rule Description: "{}"

Available Input Attributes: {}
Output Attributes: {}

{}

Respond with ONLY a JSON object in this exact format (no markdown, no explanation):
{{
  "expression": <the JSON Logic expression>,
  "display_expression": "<human readable version>",
  "input_attributes": [<list of input attributes used>],
  "output_attributes": [<list of output attributes>]
}}"#,
            natural_language,
            input_attributes.join(", "),
            output_attributes.join(", "),
            self.translator.get_context().to_system_prompt()
        );

        let messages = vec![Message::user(prompt)];
        let response = self.client.chat(messages).await?;

        let text = response.text().ok_or_else(|| {
            AgentError::ApiError("No response from Claude".to_string())
        })?;

        // Parse the JSON response
        let parsed: TranslationResult = serde_json::from_str(&text)
            .map_err(|e| AgentError::JsonLogicParseError(format!(
                "Failed to parse Claude response: {}. Response was: {}", e, text
            )))?;

        // Validate the generated expression
        let input_attrs: Vec<String> = parsed.input_attributes.iter().map(|s| s.to_string()).collect();
        let output_attrs: Vec<String> = parsed.output_attributes.iter().map(|s| s.to_string()).collect();

        let validation = self.validator.validate(&parsed.expression, &input_attrs, &output_attrs)?;

        if !validation.is_valid {
            let errors: Vec<String> = validation.errors.iter().map(|e| e.message.clone()).collect();
            return Err(AgentError::ValidationError(errors.join("; ")));
        }

        Ok(parsed)
    }

    /// Create a rule from natural language description
    ///
    /// This is the high-level method that translates NL to JSON Logic,
    /// validates it, and returns a complete rule output.
    pub async fn create_rule_from_nl(
        &self,
        description: &str,
        rule_type: &str,
        input_hints: Option<&[&str]>,
        output_hints: Option<&[&str]>,
    ) -> AgentResult<CreateRuleOutput> {
        let inputs = input_hints.unwrap_or(&[]);
        let outputs = output_hints.unwrap_or(&[]);

        let translation = self.translate_to_json_logic(description, inputs, outputs).await?;

        let display_expr = self.translator.generate_display_expression(&translation.expression);

        Ok(CreateRuleOutput {
            rule: crate::tools::GeneratedRule {
                rule_type: rule_type.to_string(),
                expression: translation.expression,
                display_expression: translation.display_expression.unwrap_or(display_expr),
                description: description.to_string(),
                input_attributes: translation.input_attributes,
                output_attributes: translation.output_attributes,
            },
            explanation: "Rule created from natural language description".to_string(),
            warnings: vec![],
        })
    }

    /// Clear conversation history
    pub fn clear_history(&mut self) {
        self.messages.clear();
    }

    /// Get conversation history
    pub fn history(&self) -> &[Message] {
        &self.messages
    }
}

/// Result of translating natural language to JSON Logic
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranslationResult {
    /// The JSON Logic expression
    pub expression: Value,
    /// Human-readable display expression
    pub display_expression: Option<String>,
    /// Input attributes used
    pub input_attributes: Vec<String>,
    /// Output attributes
    pub output_attributes: Vec<String>,
}

/// Builder for RuleAgent
pub struct RuleAgentBuilder {
    api_key: Option<String>,
    context: Option<TranslationContext>,
}

impl RuleAgentBuilder {
    pub fn new() -> Self {
        Self {
            api_key: None,
            context: None,
        }
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn with_context(mut self, context: TranslationContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn build(self) -> AgentResult<RuleAgent> {
        let api_key = self
            .api_key
            .ok_or_else(|| AgentError::ConfigError("API key required".to_string()))?;

        let config = ClaudeConfig::new(api_key);

        Ok(match self.context {
            Some(ctx) => RuleAgent::with_context(config, ctx),
            None => RuleAgent::new(config),
        })
    }
}

impl Default for RuleAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_builder() {
        let result = RuleAgentBuilder::new().build();
        assert!(result.is_err()); // No API key

        let result = RuleAgentBuilder::new()
            .with_api_key("test-key")
            .build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_rule() {
        let agent = RuleAgentBuilder::new()
            .with_api_key("test-key")
            .build()
            .unwrap();

        let input = ValidateRuleInput {
            expression: serde_json::json!({">": [{"var": "age"}, 18]}),
            product_id: "test".to_string(),
            input_attributes: vec!["age".to_string()],
            output_attributes: vec!["is_adult".to_string()],
        };

        let result = agent.validate_rule(input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.is_valid);
    }

    #[test]
    fn test_explain_rule() {
        let agent = RuleAgentBuilder::new()
            .with_api_key("test-key")
            .build()
            .unwrap();

        let input = ExplainRuleInput {
            rule_id: None,
            expression: Some(serde_json::json!({">": [{"var": "age"}, 18]})),
            verbosity: "detailed".to_string(),
        };

        let result = agent.explain_rule(input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.explanation.is_empty());
    }

    #[test]
    fn test_test_rule() {
        let agent = RuleAgentBuilder::new()
            .with_api_key("test-key")
            .build()
            .unwrap();

        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), serde_json::json!(25));

        let input = TestRuleInput {
            rule_id: None,
            expression: Some(serde_json::json!({">": [{"var": "age"}, 18]})),
            inputs,
        };

        let result = agent.test_rule(input);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result, Some(serde_json::json!(true)));
    }
}
