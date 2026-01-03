//! Error types for the YAML loader.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for loader operations.
pub type LoaderResult<T> = Result<T, LoaderError>;

/// Errors that can occur during YAML loading and parsing.
#[derive(Debug, Error)]
pub enum LoaderError {
    // =========================================================================
    // File Errors
    // =========================================================================
    /// Path does not exist.
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    /// No YAML files found in the specified folder.
    #[error("No YAML files found in {0}")]
    NoYamlFiles(PathBuf),

    /// Failed to read a file.
    #[error("Failed to read file {file}: {message}")]
    FileRead { file: PathBuf, message: String },

    // =========================================================================
    // Parse Errors
    // =========================================================================
    /// YAML parsing error.
    #[error("YAML parse error in {file}: {message}")]
    YamlParse { file: PathBuf, message: String },

    /// Invalid YAML structure.
    #[error("Invalid YAML structure in {file}: {message}")]
    InvalidStructure { file: PathBuf, message: String },

    /// Invalid expression (FarmScript or JSON Logic).
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),

    // =========================================================================
    // Critical Validation Errors
    // =========================================================================
    /// Missing product ID - cannot determine from folder or YAML.
    #[error("Missing product ID - cannot determine from folder name or YAML content")]
    MissingProductId,

    /// No functions/rules defined - product has no evaluatable rules.
    #[error("No functions defined - product has no evaluatable rules")]
    NoFunctions,

    /// Circular dependency detected in function definitions.
    #[error("Circular dependency in functions: {0}")]
    CircularDependency(String),

    // =========================================================================
    // Runtime Errors
    // =========================================================================
    /// Product not found in registry.
    #[error("Product not found: {0}")]
    ProductNotFound(String),

    /// Function not found in product.
    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    /// LLM evaluator not configured.
    #[error("LLM evaluator not configured. Provide an LlmEvaluator implementation.")]
    LlmNotConfigured,

    /// Unsupported evaluator type.
    #[error("Unsupported evaluator type: {0}")]
    UnsupportedEvaluator(String),

    // =========================================================================
    // LLM Configuration Errors
    // =========================================================================
    /// Missing required field in LLM function configuration.
    #[error("Function '{function}' is missing required field '{field}' for LLM evaluator")]
    MissingField { function: String, field: String },

    /// Invalid field value in LLM function configuration.
    #[error("Function '{function}' has invalid '{field}': {reason}")]
    InvalidField {
        function: String,
        field: String,
        reason: String,
    },

    // =========================================================================
    // Wrapped Errors
    // =========================================================================
    /// Core library error.
    #[error("Core error: {0}")]
    Core(#[from] product_farm_core::CoreError),

    /// Rule engine error.
    #[error("Rule engine error: {0}")]
    RuleEngine(#[from] product_farm_rule_engine::RuleEngineError),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl LoaderError {
    /// Create a YAML parse error.
    pub fn yaml_parse(file: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::YamlParse {
            file: file.into(),
            message: message.into(),
        }
    }

    /// Create an invalid structure error.
    pub fn invalid_structure(file: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::InvalidStructure {
            file: file.into(),
            message: message.into(),
        }
    }

    /// Create a file read error.
    pub fn file_read(file: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::FileRead {
            file: file.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = LoaderError::PathNotFound(PathBuf::from("/test/path"));
        assert!(err.to_string().contains("/test/path"));

        let err = LoaderError::MissingProductId;
        assert!(err.to_string().contains("product ID"));
    }
}
