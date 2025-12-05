//! Rule executor with DAG-based dependency resolution
//!
//! Provides:
//! - Sequential execution following topological order
//! - Parallel execution within dependency levels
//! - Caching of compiled expressions

use hashbrown::HashMap;
use product_farm_core::{Rule, RuleId, Value};
use product_farm_json_logic::{CachedExpression, Evaluator, compile};
use crate::context::ExecutionContext;
use crate::dag::RuleDag;
use crate::error::{RuleEngineError, RuleEngineResult};
use std::sync::Arc;
use tracing::{debug, instrument};

/// Compiled rule with cached expression
#[derive(Debug, Clone)]
pub struct CompiledRule {
    /// Original rule
    pub rule: Arc<Rule>,
    /// Compiled JSON Logic expression
    pub expression: Arc<CachedExpression>,
}

impl CompiledRule {
    /// Compile a rule
    pub fn compile(rule: Rule) -> RuleEngineResult<Self> {
        let json_expr: serde_json::Value = serde_json::from_str(&rule.compiled_expression)
            .map_err(|e| RuleEngineError::EvaluationError(format!("Invalid expression JSON: {}", e)))?;

        let expression = compile(&json_expr)
            .map_err(|e| RuleEngineError::EvaluationError(e.to_string()))?;

        Ok(Self {
            rule: Arc::new(rule),
            expression: Arc::new(expression),
        })
    }
}

/// Result of executing a rule
#[derive(Debug, Clone)]
pub struct RuleResult {
    /// The rule ID
    pub rule_id: RuleId,
    /// The output attributes and their values
    pub outputs: Vec<(String, Value)>,
    /// Execution time in nanoseconds
    pub execution_time_ns: u64,
}

/// Result of executing all rules
#[derive(Debug)]
pub struct ExecutionResult {
    /// Results for each rule
    pub rule_results: Vec<RuleResult>,
    /// Final context with all computed values
    pub context: ExecutionContext,
    /// Total execution time in nanoseconds
    pub total_time_ns: u64,
    /// Execution levels (for debugging/analysis)
    pub levels: Vec<Vec<RuleId>>,
}

impl ExecutionResult {
    /// Get the result for a specific rule
    pub fn get_result(&self, rule_id: &RuleId) -> Option<&RuleResult> {
        self.rule_results.iter().find(|r| &r.rule_id == rule_id)
    }

    /// Get the value of a specific output
    pub fn get_output(&self, output: &str) -> Option<&Value> {
        self.context.get(output)
    }
}

/// The main rule executor
#[derive(Debug, Default)]
pub struct RuleExecutor {
    /// JSON Logic evaluator
    evaluator: Evaluator,
    /// Compiled rules cache
    compiled_rules: HashMap<RuleId, CompiledRule>,
}

impl RuleExecutor {
    /// Create a new executor
    pub fn new() -> Self {
        Self {
            evaluator: Evaluator::new(),
            compiled_rules: HashMap::new(),
        }
    }

    /// Pre-compile a set of rules
    pub fn compile_rules(&mut self, rules: &[Rule]) -> RuleEngineResult<()> {
        for rule in rules {
            if !self.compiled_rules.contains_key(&rule.id) {
                let compiled = CompiledRule::compile(rule.clone())?;
                self.compiled_rules.insert(rule.id.clone(), compiled);
            }
        }
        Ok(())
    }

    /// Execute rules in topological order
    #[instrument(skip(self, rules, context))]
    pub fn execute(
        &mut self,
        rules: &[Rule],
        context: &mut ExecutionContext,
    ) -> RuleEngineResult<ExecutionResult> {
        let start = std::time::Instant::now();

        // Build DAG
        let dag = RuleDag::from_rules(rules)?;
        let levels = dag.execution_levels()?;

        // Pre-compile all rules
        self.compile_rules(rules)?;

        let mut rule_results = Vec::with_capacity(rules.len());

        // Execute level by level
        for level in &levels {
            for rule_id in level {
                let result = self.execute_rule(rule_id, context)?;
                rule_results.push(result);
            }
        }

        let total_time_ns = start.elapsed().as_nanos() as u64;

        Ok(ExecutionResult {
            rule_results,
            context: context.clone(),
            total_time_ns,
            levels,
        })
    }

    /// Execute a single rule
    fn execute_rule(
        &mut self,
        rule_id: &RuleId,
        context: &mut ExecutionContext,
    ) -> RuleEngineResult<RuleResult> {
        let compiled = self.compiled_rules.get(rule_id)
            .ok_or_else(|| RuleEngineError::RuleNotFound(format!("{:?}", rule_id)))?;

        let start = std::time::Instant::now();

        // Build data JSON from context
        let data = context.to_json();

        // Evaluate the expression
        let value = self.evaluator
            .evaluate_cached(&compiled.expression, &data)
            .map_err(|e| RuleEngineError::EvaluationError(format!(
                "Rule '{:?}' evaluation failed: {}",
                rule_id, e
            )))?;

        let execution_time_ns = start.elapsed().as_nanos() as u64;

        // Store results for each output attribute
        let mut outputs = Vec::new();
        for output_path in &compiled.rule.output_attributes {
            let output_str = output_path.path.as_str().to_string();
            context.set(output_str.clone(), value.clone());
            outputs.push((output_str, value.clone()));
        }

        debug!(
            rule_id = ?rule_id,
            outputs = ?outputs,
            execution_time_ns = execution_time_ns,
            "Rule executed"
        );

        Ok(RuleResult {
            rule_id: rule_id.clone(),
            outputs,
            execution_time_ns,
        })
    }

    /// Get statistics about compiled rules
    pub fn stats(&self) -> ExecutorStats {
        let total_nodes: usize = self.compiled_rules.values()
            .map(|r| r.expression.node_count)
            .sum();

        ExecutorStats {
            compiled_rules: self.compiled_rules.len(),
            total_ast_nodes: total_nodes,
            rules_with_bytecode: self.compiled_rules.values()
                .filter(|r| r.expression.has_bytecode())
                .count(),
        }
    }

    /// Clear the compiled rules cache
    pub fn clear_cache(&mut self) {
        self.compiled_rules.clear();
    }
}

/// Statistics about the executor
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
    use serde_json::json;

    fn make_rule(product: &str, inputs: &[&str], outputs: &[&str], expr: serde_json::Value) -> Rule {
        Rule::from_json_logic(product, "test", expr)
            .with_inputs(inputs.iter().map(|s| s.to_string()))
            .with_outputs(outputs.iter().map(|s| s.to_string()))
    }

    #[test]
    fn test_simple_execution() {
        let rules = vec![
            make_rule("p", &["input"], &["doubled"], json!({
                "*": [{"var": "input"}, 2]
            })),
        ];

        let mut executor = RuleExecutor::new();
        let mut context = ExecutionContext::from_json(&json!({
            "input": 21
        }));

        let result = executor.execute(&rules, &mut context).unwrap();

        assert_eq!(result.rule_results.len(), 1);
        assert_eq!(result.get_output("doubled").unwrap().to_number(), 42.0);
    }

    #[test]
    fn test_chained_execution() {
        let rules = vec![
            make_rule("p", &["input"], &["a"], json!({
                "+": [{"var": "input"}, 10]
            })),
            make_rule("p", &["a"], &["b"], json!({
                "*": [{"var": "a"}, 2]
            })),
            make_rule("p", &["b"], &["c"], json!({
                "-": [{"var": "b"}, 5]
            })),
        ];

        let mut executor = RuleExecutor::new();
        let mut context = ExecutionContext::from_json(&json!({
            "input": 5
        }));

        let result = executor.execute(&rules, &mut context).unwrap();

        // input=5 -> a=15 -> b=30 -> c=25
        assert_eq!(result.get_output("a").unwrap().to_number(), 15.0);
        assert_eq!(result.get_output("b").unwrap().to_number(), 30.0);
        assert_eq!(result.get_output("c").unwrap().to_number(), 25.0);
    }

    #[test]
    fn test_conditional_execution() {
        let rules = vec![
            make_rule("p", &["age"], &["category"], json!({
                "if": [
                    {">": [{"var": "age"}, 60]}, "senior",
                    {">": [{"var": "age"}, 18]}, "adult",
                    "minor"
                ]
            })),
            make_rule("p", &["category", "base_price"], &["final_price"], json!({
                "if": [
                    {"==": [{"var": "category"}, "senior"]},
                    {"*": [{"var": "base_price"}, 0.7]},
                    {"==": [{"var": "category"}, "minor"]},
                    {"*": [{"var": "base_price"}, 0.8]},
                    {"var": "base_price"}
                ]
            })),
        ];

        let mut executor = RuleExecutor::new();

        // Test senior discount
        let mut ctx = ExecutionContext::from_json(&json!({
            "age": 65,
            "base_price": 100
        }));
        let result = executor.execute(&rules, &mut ctx).unwrap();
        assert_eq!(result.get_output("category").unwrap(), &Value::String("senior".into()));
        assert_eq!(result.get_output("final_price").unwrap().to_number(), 70.0);

        // Test adult (no discount)
        let mut ctx = ExecutionContext::from_json(&json!({
            "age": 30,
            "base_price": 100
        }));
        let result = executor.execute(&rules, &mut ctx).unwrap();
        assert_eq!(result.get_output("category").unwrap(), &Value::String("adult".into()));
        assert_eq!(result.get_output("final_price").unwrap().to_number(), 100.0);
    }

    #[test]
    fn test_execution_levels() {
        // Diamond pattern
        let rules = vec![
            make_rule("p", &["input"], &["a"], json!({"var": "input"})),
            make_rule("p", &["a"], &["b"], json!({"+": [{"var": "a"}, 1]})),
            make_rule("p", &["a"], &["c"], json!({"+": [{"var": "a"}, 2]})),
            make_rule("p", &["b", "c"], &["d"], json!({"+": [{"var": "b"}, {"var": "c"}]})),
        ];

        let mut executor = RuleExecutor::new();
        let mut context = ExecutionContext::from_json(&json!({"input": 10}));

        let result = executor.execute(&rules, &mut context).unwrap();

        assert_eq!(result.levels.len(), 3);
        assert_eq!(result.levels[0].len(), 1); // base
        assert_eq!(result.levels[1].len(), 2); // left, right
        assert_eq!(result.levels[2].len(), 1); // final

        // a=10, b=11, c=12, d=23
        assert_eq!(result.get_output("d").unwrap().to_number(), 23.0);
    }

    #[test]
    fn test_executor_stats() {
        let rules = vec![
            make_rule("p", &["x"], &["y"], json!({"+": [{"var": "x"}, 1]})),
            make_rule("p", &["y"], &["z"], json!({"*": [{"var": "y"}, 2]})),
        ];

        let mut executor = RuleExecutor::new();
        executor.compile_rules(&rules).unwrap();

        let stats = executor.stats();
        assert_eq!(stats.compiled_rules, 2);
    }
}
