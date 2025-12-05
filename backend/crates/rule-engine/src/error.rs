//! Error types for rule engine operations

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuleEngineError {
    #[error("Cyclic dependency detected: {0}")]
    CyclicDependency(String),

    #[error("Missing dependency: rule '{rule}' requires '{dependency}' which is not available")]
    MissingDependency { rule: String, dependency: String },

    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("Evaluation error: {0}")]
    EvaluationError(String),

    #[error("Invalid rule configuration: {0}")]
    InvalidConfiguration(String),

    #[error("JSON Logic error: {0}")]
    JsonLogicError(#[from] product_farm_json_logic::JsonLogicError),

    #[error("Context error: {0}")]
    ContextError(String),
}

pub type RuleEngineResult<T> = Result<T, RuleEngineError>;
