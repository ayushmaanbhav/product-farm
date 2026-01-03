//! Rule executor with DAG-based dependency resolution
//!
//! Provides:
//! - Sequential execution following topological order
//! - Parallel execution within dependency levels (using rayon)
//! - Caching of compiled expressions

use hashbrown::HashMap;
use parking_lot::RwLock;
use product_farm_core::{Rule, RuleId, Value};
use product_farm_json_logic::{CachedExpression, Evaluator, compile};
use rayon::prelude::*;
use crate::context::ExecutionContext;
use crate::dag::RuleDag;
use crate::error::{RuleEngineError, RuleEngineResult};
use std::sync::Arc;
use tracing::{debug, info, instrument};

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
///
/// Thread-safe: compiled rules are wrapped in Arc for cheap cloning and sharing.
/// After calling `compile_rules()`, the executor can be cloned and shared across
/// threads. Each thread should have its own `ExecutionContext`.
#[derive(Debug, Clone)]
pub struct RuleExecutor {
    /// Compiled rules cache (Arc-wrapped for thread-safe sharing)
    compiled_rules: Arc<HashMap<RuleId, CompiledRule>>,
}

impl Default for RuleExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleExecutor {
    /// Create a new executor
    pub fn new() -> Self {
        Self {
            compiled_rules: Arc::new(HashMap::new()),
        }
    }

    /// Pre-compile a set of rules
    ///
    /// Note: This mutates the executor. After compilation is complete,
    /// the executor can be cloned and shared across threads for execution.
    pub fn compile_rules(&mut self, rules: &[Rule]) -> RuleEngineResult<()> {
        // Get mutable access to inner map (Arc::make_mut handles CoW)
        let map = Arc::make_mut(&mut self.compiled_rules);
        for rule in rules {
            if !map.contains_key(&rule.id) {
                let compiled = CompiledRule::compile(rule.clone())?;
                map.insert(rule.id.clone(), compiled);
            }
        }
        Ok(())
    }

    /// Compile rules locally (for use with &self methods)
    ///
    /// Returns a local map containing only the rules that weren't in the shared cache.
    /// This allows execute() to work with &self while still compiling on-demand.
    fn compile_rules_local(&self, rules: &[Rule]) -> RuleEngineResult<HashMap<RuleId, CompiledRule>> {
        let mut local = HashMap::new();
        for rule in rules {
            if !self.compiled_rules.contains_key(&rule.id) && !local.contains_key(&rule.id) {
                let compiled = CompiledRule::compile(rule.clone())?;
                local.insert(rule.id.clone(), compiled);
            }
        }
        Ok(local)
    }

    /// Get a compiled rule, checking both shared cache and local map
    fn get_compiled_rule<'a>(
        &'a self,
        rule_id: &RuleId,
        local: &'a HashMap<RuleId, CompiledRule>,
    ) -> Option<&'a CompiledRule> {
        self.compiled_rules.get(rule_id).or_else(|| local.get(rule_id))
    }

    /// Execute rules in topological order with parallel execution within levels
    ///
    /// Rules within the same level have no dependencies on each other and
    /// are executed in parallel using rayon.
    ///
    /// Note: This method compiles rules internally if needed. For best performance
    /// with repeated executions, pre-compile rules using `compile_rules()`.
    #[instrument(skip(self, rules, context))]
    pub fn execute(
        &self,
        rules: &[Rule],
        context: &mut ExecutionContext,
    ) -> RuleEngineResult<ExecutionResult> {
        self.execute_with_parallelism(rules, context, true)
    }

    /// Execute rules sequentially (for comparison/debugging)
    #[instrument(skip(self, rules, context))]
    pub fn execute_sequential(
        &self,
        rules: &[Rule],
        context: &mut ExecutionContext,
    ) -> RuleEngineResult<ExecutionResult> {
        self.execute_with_parallelism(rules, context, false)
    }

    /// Internal execution with configurable parallelism
    fn execute_with_parallelism(
        &self,
        rules: &[Rule],
        context: &mut ExecutionContext,
        parallel: bool,
    ) -> RuleEngineResult<ExecutionResult> {
        let start = std::time::Instant::now();

        // Build DAG
        let dag = RuleDag::from_rules(rules)?;

        // Validate that all rule inputs are satisfied (either by other rules or context)
        let available_inputs = context.available_inputs();
        let missing = dag.find_missing_inputs(&available_inputs);
        if !missing.is_empty() {
            let missing_deps: Vec<(String, String)> = missing
                .into_iter()
                .map(|(rule_id, dep)| (format!("{:?}", rule_id), dep))
                .collect();
            return Err(RuleEngineError::MissingDependencies(missing_deps));
        }

        let levels = dag.execution_levels()?;

        // Compile any rules not already in the cache into a local map
        let local_compiled = self.compile_rules_local(rules)?;

        let mut rule_results = Vec::with_capacity(rules.len());

        // Execute level by level
        for level in &levels {
            if parallel && level.len() > 1 {
                // Parallel execution for this level
                let level_results = self.execute_level_parallel(level, context, &local_compiled)?;

                // Update context with all outputs from this level
                for result in &level_results {
                    for (key, value) in &result.outputs {
                        context.set(key.clone(), value.clone());
                    }
                }
                rule_results.extend(level_results);
            } else {
                // Sequential execution (single rule or parallel disabled)
                for rule_id in level {
                    let result = self.execute_rule(rule_id, context, &local_compiled)?;
                    rule_results.push(result);
                }
            }
        }

        let total_time_ns = start.elapsed().as_nanos() as u64;
        let parallel_levels = levels.iter().filter(|l| l.len() > 1).count();

        info!(
            total_rules = rules.len(),
            total_levels = levels.len(),
            parallel_levels = parallel_levels,
            total_time_ms = total_time_ns / 1_000_000,
            "Execution complete"
        );

        Ok(ExecutionResult {
            rule_results,
            context: context.clone(),
            total_time_ns,
            levels,
        })
    }

    /// Execute a level of rules in parallel using rayon
    fn execute_level_parallel(
        &self,
        level: &[RuleId],
        context: &ExecutionContext,
        local_compiled: &HashMap<RuleId, CompiledRule>,
    ) -> RuleEngineResult<Vec<RuleResult>> {
        // Snapshot the context data once for all rules in this level (avoids JSON conversion)
        let context_data = context.to_value();

        // Thread-safe collection for results and errors with rule IDs
        let results: RwLock<Vec<RuleResult>> = RwLock::new(Vec::with_capacity(level.len()));
        let errors: RwLock<Vec<(RuleId, RuleEngineError)>> = RwLock::new(Vec::new());

        // Execute rules in parallel
        level.par_iter().for_each(|rule_id| {
            match self.execute_rule_with_data(rule_id, &context_data, local_compiled) {
                Ok(result) => {
                    results.write().push(result);
                }
                Err(e) => {
                    errors.write().push((rule_id.clone(), e));
                }
            }
        });

        // Check for errors - aggregate all errors with rule names
        let errors = errors.into_inner();
        if !errors.is_empty() {
            return Err(RuleEngineError::MultipleRuleFailures(errors));
        }

        Ok(results.into_inner())
    }

    /// Execute a single rule with pre-computed context data (for parallel execution)
    fn execute_rule_with_data(
        &self,
        rule_id: &RuleId,
        context_data: &Value,
        local_compiled: &HashMap<RuleId, CompiledRule>,
    ) -> RuleEngineResult<RuleResult> {
        let compiled = self.get_compiled_rule(rule_id, local_compiled)
            .ok_or_else(|| RuleEngineError::RuleNotFound(format!("{:?}", rule_id)))?;

        let start = std::time::Instant::now();

        // Create a thread-local evaluator for parallel execution
        let mut evaluator = Evaluator::new();

        // Evaluate the expression using Value directly (more efficient for AST tier)
        let value = evaluator
            .evaluate_cached_value(&compiled.expression, context_data)
            .map_err(|e| RuleEngineError::EvaluationError(format!(
                "Rule '{:?}' evaluation failed: {}",
                rule_id, e
            )))?;

        let execution_time_ns = start.elapsed().as_nanos() as u64;

        // Collect outputs
        let outputs: Vec<(String, Value)> = compiled.rule.output_attributes
            .iter()
            .map(|output_path| {
                let output_str = output_path.path.as_str().to_string();
                (output_str, value.clone())
            })
            .collect();

        debug!(
            rule_id = ?rule_id,
            outputs = ?outputs.iter().map(|(k, _)| k).collect::<Vec<_>>(),
            execution_time_ns = execution_time_ns,
            parallel = true,
            "Rule executed (parallel)"
        );

        Ok(RuleResult {
            rule_id: rule_id.clone(),
            outputs,
            execution_time_ns,
        })
    }

    /// Execute a single rule
    fn execute_rule(
        &self,
        rule_id: &RuleId,
        context: &mut ExecutionContext,
        local_compiled: &HashMap<RuleId, CompiledRule>,
    ) -> RuleEngineResult<RuleResult> {
        let compiled = self.get_compiled_rule(rule_id, local_compiled)
            .ok_or_else(|| RuleEngineError::RuleNotFound(format!("{:?}", rule_id)))?;

        let start = std::time::Instant::now();

        // Build data Value from context (avoids JSON conversion for AST evaluation)
        let data = context.to_value();

        // Create a local evaluator for this execution
        let mut evaluator = Evaluator::new();

        // Evaluate the expression using Value directly (more efficient for AST tier)
        let value = evaluator
            .evaluate_cached_value(&compiled.expression, &data)
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
        Arc::make_mut(&mut self.compiled_rules).clear();
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

        let executor = RuleExecutor::new();
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

        let executor = RuleExecutor::new();
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

        let executor = RuleExecutor::new();

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

        let executor = RuleExecutor::new();
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

    #[test]
    fn test_parallel_execution_diamond() {
        // Diamond pattern: r1 -> (r2, r3) -> r4
        // r2 and r3 can execute in parallel
        let rules = vec![
            make_rule("p", &["input"], &["a"], json!({"var": "input"})),
            make_rule("p", &["a"], &["b"], json!({"+": [{"var": "a"}, 1]})),
            make_rule("p", &["a"], &["c"], json!({"+": [{"var": "a"}, 2]})),
            make_rule("p", &["b", "c"], &["d"], json!({"+": [{"var": "b"}, {"var": "c"}]})),
        ];

        let executor = RuleExecutor::new();
        let mut context = ExecutionContext::from_json(&json!({"input": 10}));

        // Test parallel execution
        let result = executor.execute(&rules, &mut context).unwrap();

        assert_eq!(result.levels.len(), 3);
        assert_eq!(result.levels[0].len(), 1); // base
        assert_eq!(result.levels[1].len(), 2); // left, right (parallel!)
        assert_eq!(result.levels[2].len(), 1); // final

        // a=10, b=11, c=12, d=23
        assert_eq!(result.get_output("a").unwrap().to_number(), 10.0);
        assert_eq!(result.get_output("b").unwrap().to_number(), 11.0);
        assert_eq!(result.get_output("c").unwrap().to_number(), 12.0);
        assert_eq!(result.get_output("d").unwrap().to_number(), 23.0);
    }

    #[test]
    fn test_parallel_vs_sequential_same_result() {
        // Wide parallel level: many independent rules
        let rules = vec![
            make_rule("p", &["input"], &["a"], json!({"+": [{"var": "input"}, 1]})),
            make_rule("p", &["input"], &["b"], json!({"+": [{"var": "input"}, 2]})),
            make_rule("p", &["input"], &["c"], json!({"+": [{"var": "input"}, 3]})),
            make_rule("p", &["input"], &["d"], json!({"+": [{"var": "input"}, 4]})),
            make_rule("p", &["a", "b", "c", "d"], &["result"], json!({
                "+": [{"var": "a"}, {"var": "b"}, {"var": "c"}, {"var": "d"}]
            })),
        ];

        let input_data = json!({"input": 10});

        // Parallel execution
        let executor_par = RuleExecutor::new();
        let mut ctx_par = ExecutionContext::from_json(&input_data);
        let result_par = executor_par.execute(&rules, &mut ctx_par).unwrap();

        // Sequential execution
        let executor_seq = RuleExecutor::new();
        let mut ctx_seq = ExecutionContext::from_json(&input_data);
        let result_seq = executor_seq.execute_sequential(&rules, &mut ctx_seq).unwrap();

        // Both should produce the same result: (10+1)+(10+2)+(10+3)+(10+4) = 50
        assert_eq!(
            result_par.get_output("result").unwrap().to_number(),
            result_seq.get_output("result").unwrap().to_number()
        );
        assert_eq!(result_par.get_output("result").unwrap().to_number(), 50.0);

        // Parallel should have identified level with 4 rules
        assert_eq!(result_par.levels[0].len(), 4); // a, b, c, d in parallel
        assert_eq!(result_par.levels[1].len(), 1); // result
    }

    #[test]
    fn test_many_parallel_rules() {
        // Create 100 independent rules to stress test parallelism
        let mut rules: Vec<Rule> = (0..100)
            .map(|i| {
                make_rule(
                    "p",
                    &["input"],
                    &[&format!("out_{}", i)],
                    json!({"+": [{"var": "input"}, i]}),
                )
            })
            .collect();

        // Add a final rule that depends on all outputs
        let all_outputs: Vec<String> = (0..100).map(|i| format!("out_{}", i)).collect();
        let final_rule = Rule::from_json_logic("p", "final", json!({"var": "out_0"}))
            .with_inputs(all_outputs.iter().map(|s| s.to_string()))
            .with_outputs(["final_result".to_string()]);
        rules.push(final_rule);

        let executor = RuleExecutor::new();
        let mut context = ExecutionContext::from_json(&json!({"input": 1}));

        let result = executor.execute(&rules, &mut context).unwrap();

        // Should have 2 levels: 100 parallel rules, then 1 final
        assert_eq!(result.levels.len(), 2);
        assert_eq!(result.levels[0].len(), 100);
        assert_eq!(result.levels[1].len(), 1);

        // Verify a few outputs
        assert_eq!(result.get_output("out_0").unwrap().to_number(), 1.0);
        assert_eq!(result.get_output("out_50").unwrap().to_number(), 51.0);
        assert_eq!(result.get_output("out_99").unwrap().to_number(), 100.0);
    }

    #[test]
    fn test_missing_dependency_error() {
        // Create a rule that requires an input that doesn't exist
        let rule = Rule::from_json_logic("p", "r1", json!({"+": [{"var": "missing_input"}, 10]}))
            .with_inputs(["missing_input".to_string()])
            .with_outputs(["result".to_string()]);

        let executor = RuleExecutor::new();
        // Context without the required input
        let mut context = ExecutionContext::from_json(&json!({"other_input": 5}));

        let result = executor.execute(&[rule], &mut context);

        // Should fail with MissingDependencies error
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            RuleEngineError::MissingDependencies(deps) => {
                assert_eq!(deps.len(), 1);
                assert!(deps[0].1.contains("missing_input"));
            }
            other => panic!("Expected MissingDependencies error, got: {:?}", other),
        }
    }

    #[test]
    fn test_dependency_satisfied_by_other_rule() {
        // Rule A outputs "intermediate", Rule B requires "intermediate"
        let rule_a = Rule::from_json_logic("p", "rule_a", json!({"+": [{"var": "input"}, 10]}))
            .with_inputs(["input".to_string()])
            .with_outputs(["intermediate".to_string()]);

        let rule_b = Rule::from_json_logic("p", "rule_b", json!({"*": [{"var": "intermediate"}, 2]}))
            .with_inputs(["intermediate".to_string()])
            .with_outputs(["result".to_string()]);

        let executor = RuleExecutor::new();
        let mut context = ExecutionContext::from_json(&json!({"input": 5}));

        // Should succeed because "intermediate" is produced by rule_a
        let result = executor.execute(&[rule_a, rule_b], &mut context).unwrap();

        // input=5 -> intermediate=15 -> result=30
        assert_eq!(result.get_output("intermediate").unwrap().to_number(), 15.0);
        assert_eq!(result.get_output("result").unwrap().to_number(), 30.0);
    }
}
