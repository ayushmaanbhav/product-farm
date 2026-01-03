//! Environment-based configuration for LLM evaluators.
//!
//! All configuration can be loaded from environment variables with the
//! `RULE_ENGINE_` prefix to avoid conflicts with other packages.
//!
//! # Environment Variables
//!
//! ## Global Settings
//! - `RULE_ENGINE_LLM_PROVIDER` - Default provider: "ollama" (default), "anthropic"
//! - `RULE_ENGINE_LLM_TEMPERATURE` - Default temperature (0.0-1.0), default: 0.0
//! - `RULE_ENGINE_LLM_MAX_OUTPUT_TOKENS` - Max tokens in LLM response, default: 1024
//! - `RULE_ENGINE_LLM_TIMEOUT_MS` - Request timeout in milliseconds, default: 30000
//! - `RULE_ENGINE_LLM_MAX_RETRIES` - Maximum retry attempts, default: 3
//! - `RULE_ENGINE_LLM_RETRY_INITIAL_MS` - Initial retry backoff, default: 100
//! - `RULE_ENGINE_LLM_RETRY_MAX_MS` - Maximum total retry backoff, default: 3000
//! - `RULE_ENGINE_LLM_RETRY_MULTIPLIER` - Backoff multiplier, default: 2.0
//!
//! ## Anthropic-Specific (override globals)
//! - `RULE_ENGINE_ANTHROPIC_API_KEY` - Anthropic API key (required for anthropic)
//! - `RULE_ENGINE_ANTHROPIC_API_URL` - API endpoint URL
//! - `RULE_ENGINE_ANTHROPIC_API_VERSION` - API version string
//! - `RULE_ENGINE_ANTHROPIC_MODEL` - Default model name
//! - `RULE_ENGINE_ANTHROPIC_MAX_CONCURRENCY` - Max concurrent requests, default: 5
//! - `RULE_ENGINE_ANTHROPIC_TEMPERATURE` - Override global temperature
//! - `RULE_ENGINE_ANTHROPIC_MAX_OUTPUT_TOKENS` - Override global max tokens
//! - `RULE_ENGINE_ANTHROPIC_TIMEOUT_MS` - Override global timeout
//!
//! ## Ollama-Specific (override globals)
//! - `RULE_ENGINE_OLLAMA_BASE_URL` - Ollama server URL, default: http://localhost:11434
//! - `RULE_ENGINE_OLLAMA_MODEL` - Default model name, default: qwen2.5:7b
//! - `RULE_ENGINE_OLLAMA_MAX_CONCURRENCY` - Max concurrent requests, default: 10
//! - `RULE_ENGINE_OLLAMA_TEMPERATURE` - Override global temperature
//! - `RULE_ENGINE_OLLAMA_MAX_OUTPUT_TOKENS` - Override global max tokens
//! - `RULE_ENGINE_OLLAMA_TIMEOUT_MS` - Override global timeout

use std::env;
use std::time::Duration;

/// Environment variable prefix for all rule engine config
pub const ENV_PREFIX: &str = "RULE_ENGINE";

// =============================================================================
// Global Defaults (can be overridden per-provider)
// =============================================================================

/// Global LLM configuration defaults
#[derive(Debug, Clone)]
pub struct GlobalLlmConfig {
    /// Default provider: "ollama" or "anthropic"
    pub provider: String,
    /// Default temperature (0.0 = deterministic, 1.0 = creative)
    pub temperature: f32,
    /// Maximum tokens in LLM response (not input, this is output limit)
    pub max_output_tokens: u32,
    /// Request timeout
    pub timeout: Duration,
    /// Retry configuration
    pub retry: RetryConfig,
}

impl Default for GlobalLlmConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".to_string(),  // Ollama as default (local, free)
            temperature: 0.0,
            max_output_tokens: 1024,
            timeout: Duration::from_millis(30000),
            retry: RetryConfig::default(),
        }
    }
}

impl GlobalLlmConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(v) = env::var(format!("{}_LLM_PROVIDER", ENV_PREFIX)) {
            config.provider = v.to_lowercase();
        }

        if let Ok(v) = env::var(format!("{}_LLM_TEMPERATURE", ENV_PREFIX)) {
            if let Ok(t) = v.parse::<f32>() {
                config.temperature = t.clamp(0.0, 1.0);
            }
        }

        if let Ok(v) = env::var(format!("{}_LLM_MAX_OUTPUT_TOKENS", ENV_PREFIX)) {
            if let Ok(t) = v.parse::<u32>() {
                config.max_output_tokens = t;
            }
        }

        if let Ok(v) = env::var(format!("{}_LLM_TIMEOUT_MS", ENV_PREFIX)) {
            if let Ok(t) = v.parse::<u64>() {
                config.timeout = Duration::from_millis(t);
            }
        }

        config.retry = RetryConfig::from_env();

        config
    }
}

// =============================================================================
// Retry Configuration
// =============================================================================

/// Retry configuration with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum total backoff time
    pub max_total_backoff: Duration,
    /// Backoff multiplier (e.g., 2.0 for doubling)
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            max_total_backoff: Duration::from_millis(3000),
            multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Load from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(v) = env::var(format!("{}_LLM_MAX_RETRIES", ENV_PREFIX)) {
            if let Ok(r) = v.parse::<u32>() {
                config.max_retries = r;
            }
        }

        if let Ok(v) = env::var(format!("{}_LLM_RETRY_INITIAL_MS", ENV_PREFIX)) {
            if let Ok(ms) = v.parse::<u64>() {
                config.initial_backoff = Duration::from_millis(ms);
            }
        }

        if let Ok(v) = env::var(format!("{}_LLM_RETRY_MAX_MS", ENV_PREFIX)) {
            if let Ok(ms) = v.parse::<u64>() {
                config.max_total_backoff = Duration::from_millis(ms);
            }
        }

        if let Ok(v) = env::var(format!("{}_LLM_RETRY_MULTIPLIER", ENV_PREFIX)) {
            if let Ok(m) = v.parse::<f64>() {
                if m >= 1.0 {
                    config.multiplier = m;
                }
            }
        }

        config
    }

    /// Calculate backoff duration for a given attempt (0-indexed)
    pub fn backoff_for_attempt(&self, attempt: u32) -> Duration {
        let backoff_ms = self.initial_backoff.as_millis() as f64
            * self.multiplier.powi(attempt as i32);
        let capped = backoff_ms.min(self.max_total_backoff.as_millis() as f64);
        Duration::from_millis(capped as u64)
    }
}

// =============================================================================
// Anthropic Configuration
// =============================================================================

/// Anthropic (Claude) provider configuration
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    /// API key (required)
    pub api_key: Option<String>,
    /// API endpoint URL
    pub api_url: String,
    /// API version string
    pub api_version: String,
    /// Default model name
    pub model: String,
    /// Maximum concurrent requests (conservative for rate limits)
    pub max_concurrency: usize,
    /// Temperature (provider-specific override)
    pub temperature: Option<f32>,
    /// Max output tokens (provider-specific override)
    pub max_output_tokens: Option<u32>,
    /// Timeout (provider-specific override)
    pub timeout: Option<Duration>,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_url: "https://api.anthropic.com/v1/messages".to_string(),
            api_version: "2023-06-01".to_string(),
            model: "claude-sonnet-4-latest".to_string(),
            max_concurrency: 5,  // Conservative for API rate limits
            temperature: None,
            max_output_tokens: None,
            timeout: None,
        }
    }
}

impl AnthropicConfig {
    /// Load from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // API key (also check legacy ANTHROPIC_API_KEY)
        config.api_key = env::var(format!("{}_ANTHROPIC_API_KEY", ENV_PREFIX))
            .ok()
            .or_else(|| env::var("ANTHROPIC_API_KEY").ok());

        if let Ok(v) = env::var(format!("{}_ANTHROPIC_API_URL", ENV_PREFIX)) {
            config.api_url = v;
        }

        if let Ok(v) = env::var(format!("{}_ANTHROPIC_API_VERSION", ENV_PREFIX)) {
            config.api_version = v;
        }

        if let Ok(v) = env::var(format!("{}_ANTHROPIC_MODEL", ENV_PREFIX)) {
            config.model = v;
        }

        if let Ok(v) = env::var(format!("{}_ANTHROPIC_MAX_CONCURRENCY", ENV_PREFIX)) {
            if let Ok(c) = v.parse::<usize>() {
                config.max_concurrency = c;
            }
        }

        if let Ok(v) = env::var(format!("{}_ANTHROPIC_TEMPERATURE", ENV_PREFIX)) {
            if let Ok(t) = v.parse::<f32>() {
                config.temperature = Some(t.clamp(0.0, 1.0));
            }
        }

        if let Ok(v) = env::var(format!("{}_ANTHROPIC_MAX_OUTPUT_TOKENS", ENV_PREFIX)) {
            if let Ok(t) = v.parse::<u32>() {
                config.max_output_tokens = Some(t);
            }
        }

        if let Ok(v) = env::var(format!("{}_ANTHROPIC_TIMEOUT_MS", ENV_PREFIX)) {
            if let Ok(ms) = v.parse::<u64>() {
                config.timeout = Some(Duration::from_millis(ms));
            }
        }

        config
    }

    /// Check if API key is available
    pub fn is_configured(&self) -> bool {
        self.api_key.is_some() && !self.api_key.as_ref().unwrap().is_empty()
    }

    /// Get effective temperature (provider override or global)
    pub fn effective_temperature(&self, global: &GlobalLlmConfig) -> f32 {
        self.temperature.unwrap_or(global.temperature)
    }

    /// Get effective max output tokens
    pub fn effective_max_output_tokens(&self, global: &GlobalLlmConfig) -> u32 {
        self.max_output_tokens.unwrap_or(global.max_output_tokens)
    }

    /// Get effective timeout
    pub fn effective_timeout(&self, global: &GlobalLlmConfig) -> Duration {
        self.timeout.unwrap_or(global.timeout)
    }
}

// =============================================================================
// Ollama Configuration
// =============================================================================

/// Ollama provider configuration
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    /// Ollama server URL
    pub base_url: String,
    /// Default model name
    pub model: String,
    /// Maximum concurrent requests (higher for local)
    pub max_concurrency: usize,
    /// Temperature (provider-specific override)
    pub temperature: Option<f32>,
    /// Max output tokens (provider-specific override)
    pub max_output_tokens: Option<u32>,
    /// Timeout (provider-specific override)
    pub timeout: Option<Duration>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model: "qwen2.5:7b".to_string(),
            max_concurrency: 10,  // Higher for local execution
            temperature: None,
            max_output_tokens: None,
            timeout: None,
        }
    }
}

impl OllamaConfig {
    /// Load from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Also check legacy OLLAMA_* variables
        if let Ok(v) = env::var(format!("{}_OLLAMA_BASE_URL", ENV_PREFIX)) {
            config.base_url = v;
        } else if let Ok(v) = env::var("OLLAMA_BASE_URL") {
            config.base_url = v;
        }

        if let Ok(v) = env::var(format!("{}_OLLAMA_MODEL", ENV_PREFIX)) {
            config.model = v;
        } else if let Ok(v) = env::var("OLLAMA_MODEL") {
            config.model = v;
        }

        if let Ok(v) = env::var(format!("{}_OLLAMA_MAX_CONCURRENCY", ENV_PREFIX)) {
            if let Ok(c) = v.parse::<usize>() {
                config.max_concurrency = c;
            }
        }

        if let Ok(v) = env::var(format!("{}_OLLAMA_TEMPERATURE", ENV_PREFIX)) {
            if let Ok(t) = v.parse::<f32>() {
                config.temperature = Some(t.clamp(0.0, 1.0));
            }
        }

        if let Ok(v) = env::var(format!("{}_OLLAMA_MAX_OUTPUT_TOKENS", ENV_PREFIX)) {
            if let Ok(t) = v.parse::<u32>() {
                config.max_output_tokens = Some(t);
            }
        }

        if let Ok(v) = env::var(format!("{}_OLLAMA_TIMEOUT_MS", ENV_PREFIX)) {
            if let Ok(ms) = v.parse::<u64>() {
                config.timeout = Some(Duration::from_millis(ms));
            }
        }

        config
    }

    /// Get effective temperature (provider override or global)
    pub fn effective_temperature(&self, global: &GlobalLlmConfig) -> f32 {
        self.temperature.unwrap_or(global.temperature)
    }

    /// Get effective max output tokens
    pub fn effective_max_output_tokens(&self, global: &GlobalLlmConfig) -> u32 {
        self.max_output_tokens.unwrap_or(global.max_output_tokens)
    }

    /// Get effective timeout
    pub fn effective_timeout(&self, global: &GlobalLlmConfig) -> Duration {
        self.timeout.unwrap_or(global.timeout)
    }
}

// =============================================================================
// Combined Configuration
// =============================================================================

/// Complete LLM evaluator configuration loaded from environment
#[derive(Debug, Clone)]
pub struct RuleEngineLlmConfig {
    /// Global settings (defaults)
    pub global: GlobalLlmConfig,
    /// Anthropic-specific settings
    pub anthropic: AnthropicConfig,
    /// Ollama-specific settings
    pub ollama: OllamaConfig,
}

impl Default for RuleEngineLlmConfig {
    fn default() -> Self {
        Self {
            global: GlobalLlmConfig::default(),
            anthropic: AnthropicConfig::default(),
            ollama: OllamaConfig::default(),
        }
    }
}

impl RuleEngineLlmConfig {
    /// Load complete configuration from environment
    pub fn from_env() -> Self {
        Self {
            global: GlobalLlmConfig::from_env(),
            anthropic: AnthropicConfig::from_env(),
            ollama: OllamaConfig::from_env(),
        }
    }

    /// Get the configured default provider name
    pub fn default_provider(&self) -> &str {
        &self.global.provider
    }

    /// Check if Anthropic is configured (has API key)
    pub fn is_anthropic_configured(&self) -> bool {
        self.anthropic.is_configured()
    }

    /// Get max concurrency for the specified provider
    pub fn max_concurrency_for(&self, provider: &str) -> usize {
        match provider.to_lowercase().as_str() {
            "anthropic" | "claude" => self.anthropic.max_concurrency,
            "ollama" => self.ollama.max_concurrency,
            _ => self.ollama.max_concurrency,  // Default to ollama's setting
        }
    }

    /// Get effective timeout for the specified provider
    pub fn timeout_for(&self, provider: &str) -> Duration {
        match provider.to_lowercase().as_str() {
            "anthropic" | "claude" => self.anthropic.effective_timeout(&self.global),
            "ollama" => self.ollama.effective_timeout(&self.global),
            _ => self.global.timeout,
        }
    }

    /// Get effective temperature for the specified provider
    pub fn temperature_for(&self, provider: &str) -> f32 {
        match provider.to_lowercase().as_str() {
            "anthropic" | "claude" => self.anthropic.effective_temperature(&self.global),
            "ollama" => self.ollama.effective_temperature(&self.global),
            _ => self.global.temperature,
        }
    }

    /// Get effective max output tokens for the specified provider
    pub fn max_output_tokens_for(&self, provider: &str) -> u32 {
        match provider.to_lowercase().as_str() {
            "anthropic" | "claude" => self.anthropic.effective_max_output_tokens(&self.global),
            "ollama" => self.ollama.effective_max_output_tokens(&self.global),
            _ => self.global.max_output_tokens,
        }
    }

    /// Print configuration summary (useful for debugging)
    pub fn summary(&self) -> String {
        format!(
            r#"Rule Engine LLM Configuration:
  Default Provider: {}
  Global Settings:
    Temperature: {}
    Max Output Tokens: {}
    Timeout: {:?}
    Retry: max={}, initial={:?}, max_backoff={:?}, multiplier={}
  Anthropic:
    Configured: {}
    Model: {}
    Max Concurrency: {}
    API URL: {}
  Ollama:
    Base URL: {}
    Model: {}
    Max Concurrency: {}"#,
            self.global.provider,
            self.global.temperature,
            self.global.max_output_tokens,
            self.global.timeout,
            self.global.retry.max_retries,
            self.global.retry.initial_backoff,
            self.global.retry.max_total_backoff,
            self.global.retry.multiplier,
            self.anthropic.is_configured(),
            self.anthropic.model,
            self.anthropic.max_concurrency,
            self.anthropic.api_url,
            self.ollama.base_url,
            self.ollama.model,
            self.ollama.max_concurrency,
        )
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_provider_is_ollama() {
        let config = GlobalLlmConfig::default();
        assert_eq!(config.provider, "ollama");
    }

    #[test]
    fn test_default_concurrency() {
        let config = RuleEngineLlmConfig::default();
        assert_eq!(config.anthropic.max_concurrency, 5);
        assert_eq!(config.ollama.max_concurrency, 10);
    }

    #[test]
    fn test_retry_backoff_calculation() {
        let retry = RetryConfig::default();

        // First retry: 100ms
        assert_eq!(retry.backoff_for_attempt(0).as_millis(), 100);
        // Second retry: 200ms
        assert_eq!(retry.backoff_for_attempt(1).as_millis(), 200);
        // Third retry: 400ms
        assert_eq!(retry.backoff_for_attempt(2).as_millis(), 400);
        // Capped at max
        assert_eq!(retry.backoff_for_attempt(10).as_millis(), 3000);
    }

    #[test]
    fn test_effective_temperature_override() {
        let global = GlobalLlmConfig {
            temperature: 0.5,
            ..Default::default()
        };

        let anthropic_with_override = AnthropicConfig {
            temperature: Some(0.8),
            ..Default::default()
        };

        let anthropic_without_override = AnthropicConfig::default();

        assert_eq!(anthropic_with_override.effective_temperature(&global), 0.8);
        assert_eq!(anthropic_without_override.effective_temperature(&global), 0.5);
    }

    #[test]
    fn test_max_concurrency_for_provider() {
        let config = RuleEngineLlmConfig::default();

        assert_eq!(config.max_concurrency_for("anthropic"), 5);
        assert_eq!(config.max_concurrency_for("claude"), 5);
        assert_eq!(config.max_concurrency_for("ollama"), 10);
        assert_eq!(config.max_concurrency_for("unknown"), 10);  // Defaults to ollama
    }

    #[test]
    fn test_config_summary() {
        let config = RuleEngineLlmConfig::default();
        let summary = config.summary();

        assert!(summary.contains("Default Provider: ollama"));
        assert!(summary.contains("Max Concurrency: 5"));  // anthropic
        assert!(summary.contains("Max Concurrency: 10")); // ollama
    }
}
