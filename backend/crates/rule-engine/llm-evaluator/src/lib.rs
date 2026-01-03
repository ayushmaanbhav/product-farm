//! LLM-based Rule Evaluation for Product-FARM
//!
//! This crate provides LLM (Large Language Model) based rule evaluation,
//! allowing rules to use AI reasoning instead of deterministic JSON Logic.
//!
//! # Features
//!
//! - `LlmEvaluatorConfig` - Configuration for LLM calls (model, temperature, prompts)
//! - `ClaudeLlmEvaluator` - Anthropic Claude implementation (requires `anthropic` feature)
//! - `OllamaLlmEvaluator` - Ollama local LLM implementation (requires `ollama` feature)
//! - `PromptBuilder` - Build context-rich prompts with rule metadata
//! - `ParallelLlmExecutor` - Execute LLM rules in parallel
//! - `RuleEngineLlmConfig` - Environment-based configuration
//!
//! # Environment Variables
//!
//! All configuration can be loaded from environment variables with the
//! `RULE_ENGINE_` prefix. See `env_config` module for full list.
//!
//! ## Quick Start
//!
//! ```bash
//! # Use Ollama (default)
//! export RULE_ENGINE_LLM_PROVIDER=ollama
//! export RULE_ENGINE_OLLAMA_MODEL=qwen2.5:7b
//!
//! # Or use Anthropic
//! export RULE_ENGINE_LLM_PROVIDER=anthropic
//! export RULE_ENGINE_ANTHROPIC_API_KEY=your-key
//! ```
//!
//! # Example
//!
//! ```ignore
//! use product_farm_llm_evaluator::{
//!     LlmEvaluatorConfig, RuleEngineLlmConfig,
//!     PromptBuilder, RuleEvaluationContext, AttributeInfo,
//!     ParallelLlmExecutor, ParallelExecutorConfig,
//! };
//!
//! // Load config from environment
//! let config = RuleEngineLlmConfig::from_env();
//! println!("{}", config.summary());
//!
//! // Build a context-rich prompt
//! let context = RuleEvaluationContext::new("calculate-premium")
//!     .with_description("Calculate insurance premium")
//!     .add_input(AttributeInfo::new("age").with_description("Driver's age"))
//!     .add_output(AttributeInfo::new("premium").with_description("Monthly premium"));
//!
//! let prompt = PromptBuilder::new().build(&context);
//! ```

mod config;
mod error;
mod claude;
mod ollama;
mod prompt;
mod executor;
mod parsing;
pub mod env_config;

pub use config::{LlmEvaluatorConfig, OutputFormat};
pub use error::{LlmEvaluatorError, LlmEvaluatorResult};
pub use claude::ClaudeLlmEvaluator;
pub use ollama::OllamaLlmEvaluator;
pub use prompt::{
    PromptBuilder, RuleEvaluationContext, AttributeInfo,
    OutputFormatInstructions, default_system_prompt,
};
pub use executor::{
    ParallelLlmExecutor, ParallelExecutorConfig,
    LlmRuleResult, RuleMetadata, RetryConfig,
};
pub use env_config::{
    RuleEngineLlmConfig, GlobalLlmConfig,
    AnthropicConfig, OllamaConfig,
    RetryConfig as EnvRetryConfig,
    ENV_PREFIX,
};
