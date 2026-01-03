//! Error types for rule engine operations

use product_farm_core::RuleId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuleEngineError {
    #[error("Cyclic dependency detected: {0}")]
    CyclicDependency(String),

    #[error("Missing dependency: rule '{rule}' requires '{dependency}' which is not available")]
    MissingDependency { rule: String, dependency: String },

    #[error("Missing dependencies: {}", format_missing_deps(.0))]
    MissingDependencies(Vec<(String, String)>),

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

    #[error("Multiple rule failures during parallel execution: {}", format_rule_failures(.0))]
    MultipleRuleFailures(Vec<(RuleId, RuleEngineError)>),
}

/// Format multiple rule failures for display
fn format_rule_failures(failures: &[(RuleId, RuleEngineError)]) -> String {
    failures
        .iter()
        .map(|(id, err)| format!("{:?}: {}", id, err))
        .collect::<Vec<_>>()
        .join("; ")
}

/// Format missing dependencies for display
fn format_missing_deps(deps: &[(String, String)]) -> String {
    deps.iter()
        .map(|(rule, dep)| format!("rule '{}' requires '{}'", rule, dep))
        .collect::<Vec<_>>()
        .join("; ")
}

pub type RuleEngineResult<T> = Result<T, RuleEngineError>;
