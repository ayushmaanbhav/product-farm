//! Service layer for Product-FARM API
//!
//! Provides the business logic for API operations.
//! Note: The gRPC service implementations are in the `grpc` module.
//! This module provides a simpler synchronous API for direct library usage.

use product_farm_core::{ProductId, Rule};
use product_farm_rule_engine::{ExecutionContext, RuleExecutor};
use crate::error::{ApiError, ApiResult};
use crate::validation::{RuleValidator, ValidationResult};

/// Main service for Product-FARM operations (synchronous API)
pub struct ProductFarmService {
    executor: RuleExecutor,
}

impl ProductFarmService {
    pub fn new() -> Self {
        Self {
            executor: RuleExecutor::new(),
        }
    }

    /// Evaluate rules for a product with given input data
    pub fn evaluate(
        &mut self,
        _product_id: &ProductId,
        rules: &[Rule],
        input: &serde_json::Value,
    ) -> ApiResult<serde_json::Value> {
        let mut context = ExecutionContext::from_json(input);

        let result = self.executor
            .execute(rules, &mut context)
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        Ok(result.context.to_json())
    }

    /// Evaluate rules with multiple inputs (batch evaluation)
    pub fn evaluate_batch(
        &mut self,
        _product_id: &ProductId,
        rules: &[Rule],
        inputs: &[serde_json::Value],
    ) -> ApiResult<Vec<serde_json::Value>> {
        let mut results = Vec::with_capacity(inputs.len());

        for input in inputs {
            let mut context = ExecutionContext::from_json(input);
            let result = self.executor
                .execute(rules, &mut context)
                .map_err(|e| ApiError::Internal(e.to_string()))?;
            results.push(result.context.to_json());
        }

        Ok(results)
    }

    /// Validate rules without executing them
    pub fn validate(&self, rules: &[Rule]) -> ValidationResult {
        RuleValidator::validate(rules)
    }

    /// Evaluate rules and return detailed execution result
    pub fn evaluate_with_stats(
        &mut self,
        _product_id: &ProductId,
        rules: &[Rule],
        input: &serde_json::Value,
    ) -> ApiResult<EvaluationResult> {
        let mut context = ExecutionContext::from_json(input);

        let result = self.executor
            .execute(rules, &mut context)
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        Ok(EvaluationResult {
            output: result.context.to_json(),
            rules_executed: result.rule_results.len(),
            execution_time_us: result.total_time_ns / 1000,
        })
    }

    /// Get executor statistics
    pub fn stats(&self) -> ExecutorStats {
        let stats = self.executor.stats();
        ExecutorStats {
            compiled_rules: stats.compiled_rules,
            total_ast_nodes: stats.total_ast_nodes,
            rules_with_bytecode: stats.rules_with_bytecode,
        }
    }

    /// Clear the executor cache
    pub fn clear_cache(&mut self) {
        self.executor.clear_cache();
    }
}

impl Default for ProductFarmService {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of rule evaluation with statistics
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// The output data after rule execution
    pub output: serde_json::Value,
    /// Number of rules executed
    pub rules_executed: usize,
    /// Execution time in microseconds
    pub execution_time_us: u64,
}

/// Statistics about the service executor
#[derive(Debug, Clone)]
pub struct ExecutorStats {
    /// Number of compiled rules in cache
    pub compiled_rules: usize,
    /// Total AST nodes across all rules
    pub total_ast_nodes: usize,
    /// Number of rules with bytecode compilation
    pub rules_with_bytecode: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use product_farm_core::Rule;
    use serde_json::json;

    #[test]
    fn test_service_evaluate() {
        let mut service = ProductFarmService::new();

        let rules = vec![
            Rule::from_json_logic("test-product", "calc", json!({
                "*": [{"var": "x"}, 2]
            }))
            .with_inputs(["x"])
            .with_outputs(["result"]),
        ];

        let input = json!({"x": 21});
        let result = service.evaluate(&ProductId::new("test-product"), &rules, &input).unwrap();

        assert_eq!(result["result"], 42.0);
    }

    #[test]
    fn test_service_batch_evaluate() {
        let mut service = ProductFarmService::new();

        let rules = vec![
            Rule::from_json_logic("test-product", "calc", json!({
                "*": [{"var": "x"}, 2]
            }))
            .with_inputs(["x"])
            .with_outputs(["result"]),
        ];

        let inputs = vec![
            json!({"x": 1}),
            json!({"x": 2}),
            json!({"x": 3}),
        ];

        let results = service.evaluate_batch(&ProductId::new("test-product"), &rules, &inputs).unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0]["result"], 2.0);
        assert_eq!(results[1]["result"], 4.0);
        assert_eq!(results[2]["result"], 6.0);
    }

    #[test]
    fn test_service_validate() {
        let service = ProductFarmService::new();

        let rules = vec![
            Rule::from_json_logic("test-product", "calc", json!({"*": [{"var": "x"}, 2]}))
                .with_inputs(["x"])
                .with_outputs(["doubled"]),
            Rule::from_json_logic("test-product", "calc", json!({"+": [{"var": "doubled"}, 10]}))
                .with_inputs(["doubled"])
                .with_outputs(["result"]),
        ];

        let result = service.validate(&rules);
        assert!(result.valid);
    }

    #[test]
    fn test_service_evaluate_with_stats() {
        let mut service = ProductFarmService::new();

        let rules = vec![
            Rule::from_json_logic("test-product", "calc", json!({"*": [{"var": "x"}, 2]}))
                .with_inputs(["x"])
                .with_outputs(["result"]),
        ];

        let input = json!({"x": 21});
        let result = service.evaluate_with_stats(&ProductId::new("test-product"), &rules, &input).unwrap();

        assert_eq!(result.output["result"], 42.0);
        assert_eq!(result.rules_executed, 1);
    }

    #[test]
    fn test_service_stats() {
        let service = ProductFarmService::new();
        let stats = service.stats();
        assert_eq!(stats.compiled_rules, 0);
    }
}
