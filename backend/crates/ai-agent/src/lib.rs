//! AI Agent Tools for Product-FARM Rule Management
//!
//! This crate provides tools that can be used by AI agents (like Claude) to:
//! - Create and modify rules using natural language
//! - Explain existing rules in plain English
//! - Validate rule integrity (cycles, types)
//! - Visualize dependency graphs
//! - Test rules with sample inputs
//!
//! ## Tool Architecture
//!
//! Tools are designed to be called by an LLM that acts as an orchestrator.
//! Each tool has a well-defined input/output schema for structured interaction.
//!
//! ## Usage with Claude API
//!
//! Enable the `anthropic` feature to use the Claude API client:
//!
//! ```toml
//! [dependencies]
//! product-farm-ai-agent = { version = "0.1", features = ["anthropic"] }
//! ```

pub mod tools;
pub mod explainer;
pub mod translator;
pub mod validator;
pub mod visualizer;
pub mod error;
pub mod client;
pub mod agent;

pub use tools::*;
pub use explainer::*;
pub use translator::*;
pub use validator::*;
pub use visualizer::*;
pub use error::*;
pub use client::*;
pub use agent::{RuleAgent, RuleAgentBuilder, TranslationResult};
