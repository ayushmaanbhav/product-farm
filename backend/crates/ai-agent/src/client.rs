//! Claude API Client
//!
//! HTTP client for interacting with Anthropic's Claude API with tool use support.

#[cfg(feature = "anthropic")]
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::error::{AgentError, AgentResult};
use crate::tools::ToolMetadata;

/// Claude model versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum ClaudeModel {
    /// Claude 3.5 Sonnet - best balance of speed and capability
    #[default]
    Claude35Sonnet,
    /// Claude 3.5 Haiku - fastest, most cost-effective
    Claude35Haiku,
    /// Claude 3 Opus - most capable
    Claude3Opus,
}

impl ClaudeModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClaudeModel::Claude35Sonnet => "claude-3-5-sonnet-20241022",
            ClaudeModel::Claude35Haiku => "claude-3-5-haiku-20241022",
            ClaudeModel::Claude3Opus => "claude-3-opus-20240229",
        }
    }
}


/// Configuration for the Claude client
#[derive(Debug, Clone)]
pub struct ClaudeConfig {
    /// API key for authentication
    pub api_key: String,
    /// Model to use
    pub model: ClaudeModel,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Temperature for response generation (0.0 - 1.0)
    pub temperature: f32,
    /// System prompt
    pub system_prompt: Option<String>,
}

impl ClaudeConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: ClaudeModel::default(),
            max_tokens: 4096,
            temperature: 0.0,
            system_prompt: None,
        }
    }

    pub fn with_model(mut self, model: ClaudeModel) -> Self {
        self.model = model;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

impl Message {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: vec![ContentBlock::Text { text: text.into() }],
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: vec![ContentBlock::Text { text: text.into() }],
        }
    }

    pub fn assistant_with_tool_use(tool_use: ToolUse) -> Self {
        Self {
            role: "assistant".to_string(),
            content: vec![ContentBlock::ToolUse {
                id: tool_use.id,
                name: tool_use.name,
                input: tool_use.input,
            }],
        }
    }

    pub fn tool_result(tool_use_id: impl Into<String>, result: serde_json::Value) -> Self {
        Self {
            role: "user".to_string(),
            content: vec![ContentBlock::ToolResult {
                tool_use_id: tool_use_id.into(),
                content: serde_json::to_string(&result).unwrap_or_default(),
            }],
        }
    }
}

/// A content block in a message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

/// A tool use request from Claude
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUse {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

/// Tool definition for Claude API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

impl From<ToolMetadata> for ToolDefinition {
    fn from(meta: ToolMetadata) -> Self {
        Self {
            name: meta.name,
            description: meta.description,
            input_schema: meta.input_schema,
        }
    }
}

/// Request to the Claude API
#[derive(Debug, Clone, Serialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

/// Response from the Claude API
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeResponse {
    pub id: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: Usage,
}

/// Token usage information
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl ClaudeResponse {
    /// Extract text content from the response
    pub fn text(&self) -> Option<String> {
        self.content.iter().find_map(|block| {
            if let ContentBlock::Text { text } = block {
                Some(text.clone())
            } else {
                None
            }
        })
    }

    /// Extract tool use from the response
    pub fn tool_uses(&self) -> Vec<ToolUse> {
        self.content
            .iter()
            .filter_map(|block| {
                if let ContentBlock::ToolUse { id, name, input } = block {
                    Some(ToolUse {
                        id: id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if the response requires tool use
    pub fn needs_tool_use(&self) -> bool {
        self.stop_reason.as_deref() == Some("tool_use")
    }
}

/// Claude API client
#[cfg(feature = "anthropic")]
pub struct ClaudeClient {
    client: Client,
    config: ClaudeConfig,
}

#[cfg(feature = "anthropic")]
impl ClaudeClient {
    const API_URL: &'static str = "https://api.anthropic.com/v1/messages";
    const API_VERSION: &'static str = "2023-06-01";

    /// Create a new Claude client
    pub fn new(config: ClaudeConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Send a message to Claude
    pub async fn send(&self, request: ClaudeRequest) -> AgentResult<ClaudeResponse> {
        let response = self
            .client
            .post(Self::API_URL)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", Self::API_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AgentError::ApiError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentError::ApiError(format!(
                "API returned {}: {}",
                status, error_body
            )));
        }

        response
            .json::<ClaudeResponse>()
            .await
            .map_err(|e| AgentError::ApiError(format!("Failed to parse response: {}", e)))
    }

    /// Send a simple message without tools
    pub async fn chat(&self, messages: Vec<Message>) -> AgentResult<ClaudeResponse> {
        let request = ClaudeRequest {
            model: self.config.model.as_str().to_string(),
            max_tokens: self.config.max_tokens,
            messages,
            system: self.config.system_prompt.clone(),
            tools: vec![],
            temperature: Some(self.config.temperature),
        };
        self.send(request).await
    }

    /// Send a message with tools available
    pub async fn chat_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> AgentResult<ClaudeResponse> {
        let request = ClaudeRequest {
            model: self.config.model.as_str().to_string(),
            max_tokens: self.config.max_tokens,
            messages,
            system: self.config.system_prompt.clone(),
            tools,
            temperature: Some(self.config.temperature),
        };
        self.send(request).await
    }

    /// Get the current configuration
    pub fn config(&self) -> &ClaudeConfig {
        &self.config
    }
}

/// Stub client when anthropic feature is not enabled
#[cfg(not(feature = "anthropic"))]
pub struct ClaudeClient {
    _config: ClaudeConfig,
}

#[cfg(not(feature = "anthropic"))]
impl ClaudeClient {
    pub fn new(config: ClaudeConfig) -> Self {
        Self { _config: config }
    }

    pub async fn send(&self, _request: ClaudeRequest) -> AgentResult<ClaudeResponse> {
        Err(AgentError::FeatureNotEnabled(
            "anthropic feature is required for Claude API calls".to_string(),
        ))
    }

    pub async fn chat(&self, _messages: Vec<Message>) -> AgentResult<ClaudeResponse> {
        Err(AgentError::FeatureNotEnabled(
            "anthropic feature is required for Claude API calls".to_string(),
        ))
    }

    pub async fn chat_with_tools(
        &self,
        _messages: Vec<Message>,
        _tools: Vec<ToolDefinition>,
    ) -> AgentResult<ClaudeResponse> {
        Err(AgentError::FeatureNotEnabled(
            "anthropic feature is required for Claude API calls".to_string(),
        ))
    }

    pub fn config(&self) -> &ClaudeConfig {
        &self._config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let user_msg = Message::user("Hello");
        assert_eq!(user_msg.role, "user");

        let assistant_msg = Message::assistant("Hi there!");
        assert_eq!(assistant_msg.role, "assistant");
    }

    #[test]
    fn test_tool_definition_from_metadata() {
        let meta = ToolMetadata {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
        };

        let def: ToolDefinition = meta.into();
        assert_eq!(def.name, "test_tool");
    }
}
