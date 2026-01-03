//! Configuration for JSON Logic evaluation
//!
//! All settings can be configured via environment variables with the prefix `RULE_ENGINE_`.
//! If not set, sensible defaults are used.

use std::sync::OnceLock;

/// Global configuration singleton
static CONFIG: OnceLock<Config> = OnceLock::new();

/// JSON Logic configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// How many times a rule must be evaluated before compiling to bytecode.
    /// Rules evaluated fewer times stay as AST (interpreted). Hot rules get compiled.
    /// Environment variable: RULE_ENGINE_BYTECODE_PROMOTION_THRESHOLD
    /// Default: 100
    pub bytecode_promotion_threshold: u64,

    /// Minimum expression complexity (AST node count) to consider for bytecode compilation.
    /// Simple expressions like `{"var": "x"}` stay as AST since compilation overhead isn't worth it.
    /// Environment variable: RULE_ENGINE_BYTECODE_MIN_COMPLEXITY
    /// Default: 5
    pub bytecode_min_complexity: usize,

    /// Maximum operand stack size for the bytecode VM.
    /// The VM pushes/pops values during evaluation. Limits memory for deeply nested expressions.
    /// Environment variable: RULE_ENGINE_BYTECODE_STACK_LIMIT
    /// Default: 65536
    pub bytecode_stack_limit: usize,

    /// Maximum pending operations in the evaluator work queue.
    /// The iterative evaluator uses a work queue instead of recursion. Limits memory usage.
    /// Environment variable: RULE_ENGINE_EVAL_WORK_QUEUE_LIMIT
    /// Default: 1000000
    pub eval_work_queue_limit: usize,

    /// Maximum evaluation steps (loop iterations) before aborting.
    /// Prevents infinite loops or runaway expressions from hanging.
    /// Environment variable: RULE_ENGINE_EVAL_MAX_STEPS
    /// Default: 1000000
    pub eval_max_steps: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bytecode_promotion_threshold: 100,
            bytecode_min_complexity: 5,
            bytecode_stack_limit: 65_536,
            eval_work_queue_limit: 1_000_000,
            eval_max_steps: 1_000_000,
        }
    }
}

impl Config {
    /// Load configuration from environment variables with defaults
    pub fn from_env() -> Self {
        Self {
            bytecode_promotion_threshold: parse_env("RULE_ENGINE_BYTECODE_PROMOTION_THRESHOLD", 100),
            bytecode_min_complexity: parse_env("RULE_ENGINE_BYTECODE_MIN_COMPLEXITY", 5),
            bytecode_stack_limit: parse_env("RULE_ENGINE_BYTECODE_STACK_LIMIT", 65_536),
            eval_work_queue_limit: parse_env("RULE_ENGINE_EVAL_WORK_QUEUE_LIMIT", 1_000_000),
            eval_max_steps: parse_env("RULE_ENGINE_EVAL_MAX_STEPS", 1_000_000),
        }
    }

    /// Get the global configuration (lazily initialized from environment)
    pub fn global() -> &'static Config {
        CONFIG.get_or_init(Config::from_env)
    }
}

/// Parse an environment variable with a default value
fn parse_env<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.bytecode_promotion_threshold, 100);
        assert_eq!(config.bytecode_min_complexity, 5);
        assert_eq!(config.bytecode_stack_limit, 65_536);
        assert_eq!(config.eval_work_queue_limit, 1_000_000);
        assert_eq!(config.eval_max_steps, 1_000_000);
    }

    #[test]
    fn test_global_config() {
        // Global config should be accessible
        let config = Config::global();
        assert!(config.bytecode_promotion_threshold > 0);
    }
}
