//! High-level evaluator interface for JSON Logic
//!
//! This module provides a unified interface for parsing, compiling, and
//! evaluating JSON Logic expressions with automatic tiering.

use crate::{
    ast::Expr,
    compiler::{CompiledBytecode, Compiler},
    error::{JsonLogicError, JsonLogicResult},
    parser::parse,
    vm::{EvalContext, VM},
};
use product_farm_core::Value;
use serde_json::Value as JsonValue;
use std::sync::Arc;

/// Threshold for bytecode compilation (node count)
const BYTECODE_COMPILE_THRESHOLD: usize = 5;

/// A cached, compiled JSON Logic expression ready for evaluation
#[derive(Debug, Clone)]
pub struct CachedExpression {
    /// Parsed AST
    pub ast: Arc<Expr>,
    /// Compiled bytecode (if expression is complex enough)
    pub bytecode: Option<Arc<CompiledBytecode>>,
    /// Variables referenced in the expression
    pub variables: Vec<String>,
    /// Node count for complexity estimation
    pub node_count: usize,
}

impl CachedExpression {
    /// Create a new cached expression from JSON
    pub fn from_json(json: &JsonValue) -> JsonLogicResult<Self> {
        let ast = parse(json)?;
        let node_count = ast.node_count();
        let variables = ast.collect_variables().iter().map(|s| s.to_string()).collect();

        // Compile to bytecode if expression is complex enough
        let bytecode = if node_count >= BYTECODE_COMPILE_THRESHOLD {
            let mut compiler = Compiler::new();
            match compiler.compile(&ast) {
                Ok(bc) => Some(Arc::new(bc)),
                Err(_) => None, // Fall back to AST interpreter if compilation fails
            }
        } else {
            None
        };

        Ok(Self {
            ast: Arc::new(ast),
            bytecode,
            variables,
            node_count,
        })
    }

    /// Force bytecode compilation regardless of complexity
    pub fn compile_bytecode(&mut self) -> JsonLogicResult<()> {
        if self.bytecode.is_none() {
            let mut compiler = Compiler::new();
            self.bytecode = Some(Arc::new(compiler.compile(&self.ast)?));
        }
        Ok(())
    }

    /// Check if this expression has compiled bytecode
    pub fn has_bytecode(&self) -> bool {
        self.bytecode.is_some()
    }
}

/// The main JSON Logic evaluator
///
/// Provides a high-level interface for evaluating JSON Logic expressions
/// with automatic caching and tiered execution.
#[derive(Debug, Default)]
pub struct Evaluator {
    /// Reusable VM instance
    vm: VM,
}

impl Evaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        Self { vm: VM::new() }
    }

    /// Parse and evaluate a JSON Logic expression in one step
    pub fn evaluate(&mut self, rule: &JsonValue, data: &JsonValue) -> JsonLogicResult<Value> {
        let ast = parse(rule)?;

        // For simple expressions, use AST interpretation
        if ast.node_count() < BYTECODE_COMPILE_THRESHOLD {
            self.evaluate_ast(&ast, data)
        } else {
            // Try to compile and execute bytecode
            let mut compiler = Compiler::new();
            match compiler.compile(&ast) {
                Ok(bytecode) => {
                    let context = EvalContext::from_json(data, &bytecode);
                    self.vm.execute(&bytecode, &context)
                }
                Err(_) => {
                    // Fall back to AST interpreter
                    self.evaluate_ast(&ast, data)
                }
            }
        }
    }

    /// Evaluate a cached expression
    pub fn evaluate_cached(&mut self, expr: &CachedExpression, data: &JsonValue) -> JsonLogicResult<Value> {
        if let Some(ref bytecode) = expr.bytecode {
            let context = EvalContext::from_json(data, bytecode);
            self.vm.execute(bytecode, &context)
        } else {
            self.evaluate_ast(&expr.ast, data)
        }
    }

    /// Evaluate with a pre-built context (for bytecode execution)
    pub fn evaluate_with_bytecode(
        &mut self,
        bytecode: &CompiledBytecode,
        context: &EvalContext,
    ) -> JsonLogicResult<Value> {
        self.vm.execute(bytecode, context)
    }

    /// Evaluate an AST expression directly (Tier 0 - interpreted)
    pub fn evaluate_ast(&self, expr: &Expr, data: &JsonValue) -> JsonLogicResult<Value> {
        let data_value = Value::from_json(data);
        self.eval_expr(expr, &data_value)
    }

    /// Internal AST evaluation
    fn eval_expr(&self, expr: &Expr, data: &Value) -> JsonLogicResult<Value> {
        match expr {
            Expr::Literal(v) => Ok(v.clone()),

            Expr::Var(var) => {
                self.get_variable(&var.path, data)
                    .or_else(|| var.default.clone())
                    .ok_or_else(|| JsonLogicError::VariableNotFound(var.path.clone()))
            }

            Expr::Eq(a, b) => {
                let a_val = self.eval_expr(a, data)?;
                let b_val = self.eval_expr(b, data)?;
                Ok(Value::Bool(a_val.loose_equals(&b_val)))
            }

            Expr::StrictEq(a, b) => {
                let a_val = self.eval_expr(a, data)?;
                let b_val = self.eval_expr(b, data)?;
                Ok(Value::Bool(a_val == b_val))
            }

            Expr::Ne(a, b) => {
                let a_val = self.eval_expr(a, data)?;
                let b_val = self.eval_expr(b, data)?;
                Ok(Value::Bool(!a_val.loose_equals(&b_val)))
            }

            Expr::StrictNe(a, b) => {
                let a_val = self.eval_expr(a, data)?;
                let b_val = self.eval_expr(b, data)?;
                Ok(Value::Bool(a_val != b_val))
            }

            Expr::Lt(exprs) => self.eval_chain_comparison(exprs, data, |a, b| a < b),
            Expr::Le(exprs) => self.eval_chain_comparison(exprs, data, |a, b| a <= b),
            Expr::Gt(exprs) => self.eval_chain_comparison(exprs, data, |a, b| a > b),
            Expr::Ge(exprs) => self.eval_chain_comparison(exprs, data, |a, b| a >= b),

            Expr::Not(a) => {
                let val = self.eval_expr(a, data)?;
                Ok(Value::Bool(!val.is_truthy()))
            }

            Expr::ToBool(a) => {
                let val = self.eval_expr(a, data)?;
                Ok(Value::Bool(val.is_truthy()))
            }

            Expr::And(exprs) => {
                let mut result = Value::Bool(true);
                for expr in exprs {
                    result = self.eval_expr(expr, data)?;
                    if !result.is_truthy() {
                        return Ok(result);
                    }
                }
                Ok(result)
            }

            Expr::Or(exprs) => {
                let mut result = Value::Bool(false);
                for expr in exprs {
                    result = self.eval_expr(expr, data)?;
                    if result.is_truthy() {
                        return Ok(result);
                    }
                }
                Ok(result)
            }

            Expr::If(exprs) => {
                let mut i = 0;
                while i < exprs.len() {
                    if i + 1 >= exprs.len() {
                        // Last element is the else clause
                        return self.eval_expr(&exprs[i], data);
                    }

                    let cond = self.eval_expr(&exprs[i], data)?;
                    if cond.is_truthy() {
                        return self.eval_expr(&exprs[i + 1], data);
                    }
                    i += 2;
                }
                Ok(Value::Null)
            }

            Expr::Ternary(cond, then, else_) => {
                let cond_val = self.eval_expr(cond, data)?;
                if cond_val.is_truthy() {
                    self.eval_expr(then, data)
                } else {
                    self.eval_expr(else_, data)
                }
            }

            Expr::Add(exprs) => {
                let mut sum = 0.0_f64;
                for expr in exprs {
                    let val = self.eval_expr(expr, data)?;
                    sum += val.to_number();
                }
                Ok(Value::Float(sum))
            }

            Expr::Sub(exprs) => {
                if exprs.is_empty() {
                    return Ok(Value::Float(0.0));
                }
                if exprs.len() == 1 {
                    let val = self.eval_expr(&exprs[0], data)?;
                    return Ok(Value::Float(-val.to_number()));
                }
                let first = self.eval_expr(&exprs[0], data)?.to_number();
                let second = self.eval_expr(&exprs[1], data)?.to_number();
                Ok(Value::Float(first - second))
            }

            Expr::Mul(exprs) => {
                let mut product = 1.0_f64;
                for expr in exprs {
                    let val = self.eval_expr(expr, data)?;
                    product *= val.to_number();
                }
                Ok(Value::Float(product))
            }

            Expr::Div(a, b) => {
                let a_val = self.eval_expr(a, data)?.to_number();
                let b_val = self.eval_expr(b, data)?.to_number();
                if b_val == 0.0 {
                    return Err(JsonLogicError::DivisionByZero);
                }
                Ok(Value::Float(a_val / b_val))
            }

            Expr::Mod(a, b) => {
                let a_val = self.eval_expr(a, data)?.to_number();
                let b_val = self.eval_expr(b, data)?.to_number();
                if b_val == 0.0 {
                    return Err(JsonLogicError::DivisionByZero);
                }
                Ok(Value::Float(a_val % b_val))
            }

            Expr::Min(exprs) => {
                let mut min = f64::INFINITY;
                for expr in exprs {
                    let val = self.eval_expr(expr, data)?.to_number();
                    if val < min {
                        min = val;
                    }
                }
                Ok(Value::Float(min))
            }

            Expr::Max(exprs) => {
                let mut max = f64::NEG_INFINITY;
                for expr in exprs {
                    let val = self.eval_expr(expr, data)?.to_number();
                    if val > max {
                        max = val;
                    }
                }
                Ok(Value::Float(max))
            }

            Expr::Cat(exprs) => {
                let mut result = String::new();
                for expr in exprs {
                    let val = self.eval_expr(expr, data)?;
                    result.push_str(&val.to_display_string());
                }
                Ok(Value::String(result))
            }

            Expr::Substr(s, start, len) => {
                let s_val = self.eval_expr(s, data)?;
                let s_str = s_val.to_display_string();
                let start_val = self.eval_expr(start, data)?.to_number() as i64;

                let chars: Vec<char> = s_str.chars().collect();
                let len_val = chars.len() as i64;

                // Handle negative start
                let actual_start = if start_val < 0 {
                    (len_val + start_val).max(0) as usize
                } else {
                    start_val.min(len_val) as usize
                };

                let result = if let Some(length_expr) = len {
                    let length = self.eval_expr(length_expr, data)?.to_number() as i64;
                    if length < 0 {
                        // Negative length means "up to N characters from the end"
                        let end = (len_val + length).max(actual_start as i64) as usize;
                        chars[actual_start..end].iter().collect()
                    } else {
                        let end = (actual_start + length as usize).min(chars.len());
                        chars[actual_start..end].iter().collect()
                    }
                } else {
                    chars[actual_start..].iter().collect()
                };

                Ok(Value::String(result))
            }

            Expr::Map(arr, mapper) => {
                let arr_val = self.eval_expr(arr, data)?;
                let items = match arr_val {
                    Value::Array(items) => items,
                    _ => return Ok(Value::Array(vec![])),
                };

                let mut results = Vec::with_capacity(items.len());
                for item in items {
                    results.push(self.eval_expr(mapper, &item)?);
                }
                Ok(Value::Array(results))
            }

            Expr::Filter(arr, predicate) => {
                let arr_val = self.eval_expr(arr, data)?;
                let items = match arr_val {
                    Value::Array(items) => items,
                    _ => return Ok(Value::Array(vec![])),
                };

                let mut results = Vec::new();
                for item in items {
                    let keep = self.eval_expr(predicate, &item)?;
                    if keep.is_truthy() {
                        results.push(item);
                    }
                }
                Ok(Value::Array(results))
            }

            Expr::Reduce(arr, reducer, initial) => {
                let arr_val = self.eval_expr(arr, data)?;
                let items = match arr_val {
                    Value::Array(items) => items,
                    _ => return self.eval_expr(initial, data),
                };

                let mut accumulator = self.eval_expr(initial, data)?;
                for item in items {
                    // Build reduce context with "current" and "accumulator"
                    let reduce_data = Value::Object(std::collections::HashMap::from([
                        ("current".to_string(), item),
                        ("accumulator".to_string(), accumulator.clone()),
                    ]));
                    accumulator = self.eval_expr(reducer, &reduce_data)?;
                }
                Ok(accumulator)
            }

            Expr::All(arr, predicate) => {
                let arr_val = self.eval_expr(arr, data)?;
                let items = match arr_val {
                    Value::Array(items) => items,
                    _ => return Ok(Value::Bool(false)),
                };

                if items.is_empty() {
                    return Ok(Value::Bool(false));
                }

                for item in items {
                    let result = self.eval_expr(predicate, &item)?;
                    if !result.is_truthy() {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }

            Expr::Some(arr, predicate) => {
                let arr_val = self.eval_expr(arr, data)?;
                let items = match arr_val {
                    Value::Array(items) => items,
                    _ => return Ok(Value::Bool(false)),
                };

                for item in items {
                    let result = self.eval_expr(predicate, &item)?;
                    if result.is_truthy() {
                        return Ok(Value::Bool(true));
                    }
                }
                Ok(Value::Bool(false))
            }

            Expr::None(arr, predicate) => {
                let arr_val = self.eval_expr(arr, data)?;
                let items = match arr_val {
                    Value::Array(items) => items,
                    _ => return Ok(Value::Bool(true)),
                };

                for item in items {
                    let result = self.eval_expr(predicate, &item)?;
                    if result.is_truthy() {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }

            Expr::Merge(exprs) => {
                let mut result = Vec::new();
                for expr in exprs {
                    let val = self.eval_expr(expr, data)?;
                    match val {
                        Value::Array(items) => result.extend(items),
                        other => result.push(other),
                    }
                }
                Ok(Value::Array(result))
            }

            Expr::In(needle, haystack) => {
                let needle_val = self.eval_expr(needle, data)?;
                let haystack_val = self.eval_expr(haystack, data)?;

                let found = match haystack_val {
                    Value::Array(items) => items.iter().any(|item| needle_val.loose_equals(item)),
                    Value::String(s) => {
                        let needle_str = needle_val.to_display_string();
                        s.contains(&needle_str)
                    }
                    _ => false,
                };
                Ok(Value::Bool(found))
            }

            Expr::Missing(keys) => {
                let mut missing = Vec::new();
                for key_expr in keys {
                    let key = self.eval_expr(key_expr, data)?;
                    let key_str = key.to_display_string();
                    if self.get_variable(&key_str, data).is_none() {
                        missing.push(Value::String(key_str));
                    }
                }
                Ok(Value::Array(missing))
            }

            Expr::MissingSome(min_required, keys) => {
                let min = self.eval_expr(min_required, data)?.to_number() as usize;
                let mut missing = Vec::new();
                let mut found = 0;

                for key_expr in keys {
                    let key = self.eval_expr(key_expr, data)?;
                    let key_str = key.to_display_string();
                    if self.get_variable(&key_str, data).is_some() {
                        found += 1;
                    } else {
                        missing.push(Value::String(key_str));
                    }
                }

                if found >= min {
                    Ok(Value::Array(vec![]))
                } else {
                    Ok(Value::Array(missing))
                }
            }

            Expr::Log(expr) => {
                let val = self.eval_expr(expr, data)?;
                #[cfg(debug_assertions)]
                eprintln!("[JSON Logic log] {:?}", val);
                Ok(val)
            }
        }
    }

    /// Get a variable from data by path
    fn get_variable(&self, path: &str, data: &Value) -> Option<Value> {
        if path.is_empty() {
            return Some(data.clone());
        }

        let segments: Vec<&str> = path.split('.').collect();
        let mut current = data.clone();

        for segment in segments {
            current = match current {
                Value::Object(map) => map.get(segment)?.clone(),
                Value::Array(arr) => {
                    let idx: usize = segment.parse().ok()?;
                    arr.get(idx)?.clone()
                }
                _ => return None,
            };
        }
        Some(current)
    }

    /// Evaluate chained comparison (e.g., a < b < c)
    fn eval_chain_comparison<F>(&self, exprs: &[Expr], data: &Value, cmp: F) -> JsonLogicResult<Value>
    where
        F: Fn(f64, f64) -> bool,
    {
        if exprs.len() < 2 {
            return Ok(Value::Bool(false));
        }

        let mut prev = self.eval_expr(&exprs[0], data)?.to_number();
        for expr in &exprs[1..] {
            let curr = self.eval_expr(expr, data)?.to_number();
            if !cmp(prev, curr) {
                return Ok(Value::Bool(false));
            }
            prev = curr;
        }
        Ok(Value::Bool(true))
    }
}

/// Convenience function for one-shot evaluation
pub fn evaluate(rule: &JsonValue, data: &JsonValue) -> JsonLogicResult<Value> {
    Evaluator::new().evaluate(rule, data)
}

/// Convenience function for parsing and caching an expression
pub fn compile(rule: &JsonValue) -> JsonLogicResult<CachedExpression> {
    CachedExpression::from_json(rule)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_evaluation() {
        let rule = json!({"==": [1, 1]});
        let data = json!({});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_variable_access() {
        let rule = json!({"var": "x"});
        let data = json!({"x": 42});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_nested_variable() {
        let rule = json!({"var": "user.age"});
        let data = json!({"user": {"age": 25}});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result, Value::Int(25));
    }

    #[test]
    fn test_arithmetic() {
        let rule = json!({"+": [{"var": "a"}, {"var": "b"}]});
        let data = json!({"a": 10, "b": 20});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result.to_number(), 30.0);
    }

    #[test]
    fn test_comparison() {
        let rule = json!({">": [{"var": "age"}, 18]});
        let data = json!({"age": 25});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_if_then_else() {
        let rule = json!({
            "if": [
                {">": [{"var": "age"}, 60]},
                "senior",
                {">": [{"var": "age"}, 18]},
                "adult",
                "minor"
            ]
        });

        let senior = evaluate(&rule, &json!({"age": 70})).unwrap();
        assert_eq!(senior, Value::String("senior".into()));

        let adult = evaluate(&rule, &json!({"age": 30})).unwrap();
        assert_eq!(adult, Value::String("adult".into()));

        let minor = evaluate(&rule, &json!({"age": 15})).unwrap();
        assert_eq!(minor, Value::String("minor".into()));
    }

    #[test]
    fn test_and_or() {
        let rule = json!({"and": [{"var": "a"}, {"var": "b"}]});

        let both_true = evaluate(&rule, &json!({"a": true, "b": true})).unwrap();
        assert!(both_true.is_truthy());

        let one_false = evaluate(&rule, &json!({"a": true, "b": false})).unwrap();
        assert!(!one_false.is_truthy());
    }

    #[test]
    fn test_cached_expression() {
        let rule = json!({"+": [{"var": "x"}, {"var": "y"}, {"var": "z"}]});
        let cached = compile(&rule).unwrap();

        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate_cached(&cached, &json!({"x": 1, "y": 2, "z": 3})).unwrap();
        assert_eq!(result.to_number(), 6.0);
    }

    #[test]
    fn test_map_filter() {
        let rule = json!({
            "map": [
                {"var": "numbers"},
                {"*": [{"var": ""}, 2]}
            ]
        });

        let result = evaluate(&rule, &json!({"numbers": [1, 2, 3]})).unwrap();
        match result {
            Value::Array(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].to_number(), 2.0);
                assert_eq!(items[1].to_number(), 4.0);
                assert_eq!(items[2].to_number(), 6.0);
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_in_array() {
        let rule = json!({"in": [3, [1, 2, 3, 4, 5]]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::Bool(true));

        let rule2 = json!({"in": [6, [1, 2, 3, 4, 5]]});
        let result2 = evaluate(&rule2, &json!({})).unwrap();
        assert_eq!(result2, Value::Bool(false));
    }

    #[test]
    fn test_reduce() {
        let rule = json!({
            "reduce": [
                {"var": "numbers"},
                {"+": [{"var": "accumulator"}, {"var": "current"}]},
                0
            ]
        });

        let result = evaluate(&rule, &json!({"numbers": [1, 2, 3, 4, 5]})).unwrap();
        assert_eq!(result.to_number(), 15.0);
    }
}

/// Consistency tests between AST interpreter and bytecode VM
#[cfg(test)]
mod consistency_tests {
    use super::*;
    use crate::compiler::Compiler;
    use crate::vm::{EvalContext, VM};
    use serde_json::json;

    /// Helper to evaluate using AST interpreter
    fn eval_ast(rule: &serde_json::Value, data: &serde_json::Value) -> Value {
        let expr = crate::parse(rule).unwrap();
        let evaluator = Evaluator::new();
        evaluator.evaluate_ast(&expr, data).unwrap()
    }

    /// Helper to evaluate using bytecode VM
    fn eval_bytecode(rule: &serde_json::Value, data: &serde_json::Value) -> Value {
        let expr = crate::parse(rule).unwrap();
        let mut compiler = Compiler::new();
        let bytecode = compiler.compile(&expr).unwrap();
        let context = EvalContext::from_json(data, &bytecode);
        let mut vm = VM::new();
        vm.execute(&bytecode, &context).unwrap()
    }

    /// Compare AST and bytecode results (handling floating point)
    fn results_match(ast: &Value, bc: &Value) -> bool {
        match (ast, bc) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < 1e-10,
            (Value::Int(a), Value::Float(b)) | (Value::Float(b), Value::Int(a)) => {
                ((*a as f64) - b).abs() < 1e-10
            }
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| results_match(x, y))
            }
            _ => false,
        }
    }

    #[test]
    fn test_arithmetic_consistency() {
        let cases = vec![
            (json!({"+": [1, 2]}), json!({})),
            (json!({"-": [10, 3]}), json!({})),
            (json!({"*": [4, 5]}), json!({})),
            (json!({"/": [20, 4]}), json!({})),
            (json!({"%": [17, 5]}), json!({})),
            (json!({"+": [{"var": "a"}, {"var": "b"}]}), json!({"a": 10, "b": 20})),
            (json!({"*": [{"var": "x"}, 2.5]}), json!({"x": 4})),
        ];

        for (rule, data) in cases {
            let ast_result = eval_ast(&rule, &data);
            let bc_result = eval_bytecode(&rule, &data);
            assert!(
                results_match(&ast_result, &bc_result),
                "Mismatch for {:?}: AST={:?}, BC={:?}",
                rule, ast_result, bc_result
            );
        }
    }

    #[test]
    fn test_comparison_consistency() {
        let cases = vec![
            (json!({"==": [1, 1]}), json!({})),
            (json!({"!=": [1, 2]}), json!({})),
            (json!({">": [5, 3]}), json!({})),
            (json!({">=": [5, 5]}), json!({})),
            (json!({"<": [3, 5]}), json!({})),
            (json!({"<=": [5, 5]}), json!({})),
            (json!({">": [{"var": "age"}, 18]}), json!({"age": 25})),
        ];

        for (rule, data) in cases {
            let ast_result = eval_ast(&rule, &data);
            let bc_result = eval_bytecode(&rule, &data);
            assert!(
                results_match(&ast_result, &bc_result),
                "Mismatch for {:?}: AST={:?}, BC={:?}",
                rule, ast_result, bc_result
            );
        }
    }

    #[test]
    fn test_logical_consistency() {
        let cases = vec![
            (json!({"!": [true]}), json!({})),
            (json!({"!!": [0]}), json!({})),
            (json!({"and": [true, true]}), json!({})),
            (json!({"and": [true, false]}), json!({})),
            (json!({"or": [false, true]}), json!({})),
            (json!({"or": [false, false]}), json!({})),
        ];

        for (rule, data) in cases {
            let ast_result = eval_ast(&rule, &data);
            let bc_result = eval_bytecode(&rule, &data);
            assert!(
                results_match(&ast_result, &bc_result),
                "Mismatch for {:?}: AST={:?}, BC={:?}",
                rule, ast_result, bc_result
            );
        }
    }

    #[test]
    fn test_conditional_consistency() {
        let cases = vec![
            (json!({"if": [true, "yes", "no"]}), json!({})),
            (json!({"if": [false, "yes", "no"]}), json!({})),
            (json!({
                "if": [
                    {">": [{"var": "x"}, 10]}, "big",
                    {">": [{"var": "x"}, 5]}, "medium",
                    "small"
                ]
            }), json!({"x": 15})),
            (json!({
                "if": [
                    {">": [{"var": "x"}, 10]}, "big",
                    {">": [{"var": "x"}, 5]}, "medium",
                    "small"
                ]
            }), json!({"x": 7})),
            (json!({
                "if": [
                    {">": [{"var": "x"}, 10]}, "big",
                    {">": [{"var": "x"}, 5]}, "medium",
                    "small"
                ]
            }), json!({"x": 3})),
        ];

        for (rule, data) in cases {
            let ast_result = eval_ast(&rule, &data);
            let bc_result = eval_bytecode(&rule, &data);
            assert!(
                results_match(&ast_result, &bc_result),
                "Mismatch for {:?}: AST={:?}, BC={:?}",
                rule, ast_result, bc_result
            );
        }
    }

    #[test]
    fn test_complex_expression_consistency() {
        // Insurance premium calculation
        let rule = json!({
            "*": [
                {"var": "base"},
                {"if": [
                    {">": [{"var": "age"}, 60]}, 1.5,
                    {">": [{"var": "age"}, 40]}, 1.2,
                    1.0
                ]}
            ]
        });

        let test_data = vec![
            json!({"base": 100, "age": 25}),
            json!({"base": 100, "age": 45}),
            json!({"base": 100, "age": 65}),
        ];

        for data in test_data {
            let ast_result = eval_ast(&rule, &data);
            let bc_result = eval_bytecode(&rule, &data);
            assert!(
                results_match(&ast_result, &bc_result),
                "Mismatch for data {:?}: AST={:?}, BC={:?}",
                data, ast_result, bc_result
            );
        }
    }
}

/// Property-based tests using proptest
#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;

    proptest! {
        /// Arithmetic addition is commutative: a + b == b + a
        #[test]
        fn addition_is_commutative(a in -1000i64..1000, b in -1000i64..1000) {
            let rule1 = json!({"+": [{"var": "a"}, {"var": "b"}]});
            let rule2 = json!({"+": [{"var": "b"}, {"var": "a"}]});
            let data = json!({"a": a, "b": b});

            let result1 = evaluate(&rule1, &data).unwrap().to_number();
            let result2 = evaluate(&rule2, &data).unwrap().to_number();

            prop_assert!((result1 - result2).abs() < 1e-10);
        }

        /// Multiplication is commutative: a * b == b * a
        #[test]
        fn multiplication_is_commutative(a in -100i64..100, b in -100i64..100) {
            let rule1 = json!({"*": [{"var": "a"}, {"var": "b"}]});
            let rule2 = json!({"*": [{"var": "b"}, {"var": "a"}]});
            let data = json!({"a": a, "b": b});

            let result1 = evaluate(&rule1, &data).unwrap().to_number();
            let result2 = evaluate(&rule2, &data).unwrap().to_number();

            prop_assert!((result1 - result2).abs() < 1e-10);
        }

        /// Addition identity: a + 0 == a
        #[test]
        fn addition_identity(a in -1000i64..1000) {
            let rule = json!({"+": [{"var": "a"}, 0]});
            let data = json!({"a": a});

            let result = evaluate(&rule, &data).unwrap().to_number();
            prop_assert!((result - (a as f64)).abs() < 1e-10);
        }

        /// Multiplication identity: a * 1 == a
        #[test]
        fn multiplication_identity(a in -1000i64..1000) {
            let rule = json!({"*": [{"var": "a"}, 1]});
            let data = json!({"a": a});

            let result = evaluate(&rule, &data).unwrap().to_number();
            prop_assert!((result - (a as f64)).abs() < 1e-10);
        }

        /// Comparison reflexivity: a == a is always true
        #[test]
        fn equality_reflexive(a in -1000i64..1000) {
            let rule = json!({"==": [{"var": "a"}, {"var": "a"}]});
            let data = json!({"a": a});

            let result = evaluate(&rule, &data).unwrap();
            prop_assert_eq!(result, Value::Bool(true));
        }

        /// Comparison: a > b implies !(a <= b)
        #[test]
        fn comparison_consistency(a in -100i64..100, b in -100i64..100) {
            let gt_rule = json!({">": [{"var": "a"}, {"var": "b"}]});
            let le_rule = json!({"<=": [{"var": "a"}, {"var": "b"}]});
            let data = json!({"a": a, "b": b});

            let gt_result = evaluate(&gt_rule, &data).unwrap();
            let le_result = evaluate(&le_rule, &data).unwrap();

            // a > b and a <= b should be mutually exclusive
            match (gt_result, le_result) {
                (Value::Bool(gt), Value::Bool(le)) => {
                    prop_assert_ne!(gt, le, "a={}, b={}: > and <= should be exclusive", a, b);
                }
                _ => prop_assert!(false, "Expected boolean results"),
            }
        }

        /// Boolean double negation: !!a == truthy(a)
        #[test]
        fn double_negation(a in prop::bool::ANY) {
            let rule = json!({"!!": [{"var": "a"}]});
            let data = json!({"a": a});

            let result = evaluate(&rule, &data).unwrap();
            prop_assert_eq!(result, Value::Bool(a));
        }

        /// AND with false is always false
        #[test]
        fn and_with_false(a in prop::bool::ANY) {
            let rule = json!({"and": [{"var": "a"}, false]});
            let data = json!({"a": a});

            let result = evaluate(&rule, &data).unwrap();
            prop_assert!(!result.is_truthy());
        }

        /// OR with true is always truthy
        #[test]
        fn or_with_true(a in prop::bool::ANY) {
            let rule = json!({"or": [{"var": "a"}, true]});
            let data = json!({"a": a});

            let result = evaluate(&rule, &data).unwrap();
            prop_assert!(result.is_truthy());
        }

        /// If condition selects correct branch
        #[test]
        fn if_selects_correct_branch(cond in prop::bool::ANY) {
            let rule = json!({
                "if": [{"var": "cond"}, "then", "else"]
            });
            let data = json!({"cond": cond});

            let result = evaluate(&rule, &data).unwrap();
            let expected = if cond { "then" } else { "else" };
            prop_assert_eq!(result, Value::String(expected.into()));
        }

        /// Variable with default uses default when null
        #[test]
        fn var_default_when_null(default in -100i64..100) {
            let rule = json!({"var": ["missing", default]});
            let data = json!({});

            let result = evaluate(&rule, &data).unwrap().to_number() as i64;
            prop_assert_eq!(result, default);
        }
    }
}

/// Edge case and error handling tests
#[cfg(test)]
mod edge_case_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_division_by_zero() {
        let rule = json!({"/": [10, 0]});
        let result = evaluate(&rule, &json!({}));
        // Division by zero should return an error
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::JsonLogicError::DivisionByZero));
    }

    #[test]
    fn test_modulo_by_zero() {
        let rule = json!({"%": [10, 0]});
        let result = evaluate(&rule, &json!({}));
        // Modulo by zero should return an error
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::JsonLogicError::DivisionByZero));
    }

    #[test]
    fn test_null_arithmetic() {
        // Null in arithmetic should be treated as 0
        let rule = json!({"+": [null, 5]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result.to_number(), 5.0);
    }

    #[test]
    fn test_empty_array_operations() {
        // Empty array sum
        let rule = json!({"+": []});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result.to_number(), 0.0);

        // Empty array product
        let rule = json!({"*": []});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result.to_number(), 1.0);
    }

    #[test]
    fn test_empty_string_truthiness() {
        let rule = json!({"!!": [""]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_zero_truthiness() {
        let rule = json!({"!!": [0]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_empty_array_truthiness() {
        let rule = json!({"!!": [[]]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_nested_var_access() {
        let rule = json!({"var": "user.profile.name"});
        let data = json!({
            "user": {
                "profile": {
                    "name": "Alice"
                }
            }
        });
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result, Value::String("Alice".into()));
    }

    #[test]
    fn test_missing_nested_var_with_default() {
        let rule = json!({"var": ["user.profile.missing", "default_value"]});
        let data = json!({"user": {"profile": {}}});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result, Value::String("default_value".into()));
    }

    #[test]
    fn test_array_index_access() {
        let rule = json!({"var": "items.1"});
        let data = json!({"items": ["a", "b", "c"]});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result, Value::String("b".into()));
    }

    #[test]
    fn test_array_index_out_of_bounds() {
        let rule = json!({"var": ["items.10", "not_found"]});
        let data = json!({"items": ["a", "b", "c"]});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result, Value::String("not_found".into()));
    }

    #[test]
    fn test_string_concatenation_with_numbers() {
        let rule = json!({"cat": ["Value: ", 42, " units"]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::String("Value: 42 units".into()));
    }

    #[test]
    fn test_substr_negative_start() {
        let rule = json!({"substr": ["Hello World", -5]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::String("World".into()));
    }

    #[test]
    fn test_substr_with_length() {
        let rule = json!({"substr": ["Hello World", 0, 5]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::String("Hello".into()));
    }

    #[test]
    fn test_in_operator_string() {
        // Substring check
        let rule = json!({"in": ["ell", "Hello"]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_in_operator_array() {
        // Array membership check
        let rule = json!({"in": [2, [1, 2, 3]]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_min_max_with_variables() {
        let rule = json!({"min": [{"var": "a"}, {"var": "b"}, {"var": "c"}]});
        let data = json!({"a": 5, "b": 2, "c": 8});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result.to_number(), 2.0);

        let rule = json!({"max": [{"var": "a"}, {"var": "b"}, {"var": "c"}]});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result.to_number(), 8.0);
    }

    #[test]
    fn test_chained_comparison() {
        // 1 < 5 < 10 should be true
        let rule = json!({"<": [1, 5, 10]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::Bool(true));

        // 1 < 5 < 3 should be false
        let rule = json!({"<": [1, 5, 3]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_map_operation() {
        let rule = json!({
            "map": [
                {"var": "numbers"},
                {"*": [{"var": ""}, 2]}
            ]
        });
        let data = json!({"numbers": [1, 2, 3]});
        let result = evaluate(&rule, &data).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0].to_number(), 2.0);
                assert_eq!(arr[1].to_number(), 4.0);
                assert_eq!(arr[2].to_number(), 6.0);
            }
            _ => panic!("Expected array result"),
        }
    }

    #[test]
    fn test_filter_operation() {
        let rule = json!({
            "filter": [
                {"var": "numbers"},
                {">": [{"var": ""}, 2]}
            ]
        });
        let data = json!({"numbers": [1, 2, 3, 4, 5]});
        let result = evaluate(&rule, &data).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0].to_number(), 3.0);
                assert_eq!(arr[1].to_number(), 4.0);
                assert_eq!(arr[2].to_number(), 5.0);
            }
            _ => panic!("Expected array result"),
        }
    }

    #[test]
    fn test_all_operation() {
        // All numbers > 0
        let rule = json!({
            "all": [
                {"var": "numbers"},
                {">": [{"var": ""}, 0]}
            ]
        });
        let data = json!({"numbers": [1, 2, 3]});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result, Value::Bool(true));

        let data = json!({"numbers": [1, 0, 3]});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_some_operation() {
        // Some numbers > 5
        let rule = json!({
            "some": [
                {"var": "numbers"},
                {">": [{"var": ""}, 5]}
            ]
        });
        let data = json!({"numbers": [1, 2, 10]});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result, Value::Bool(true));

        let data = json!({"numbers": [1, 2, 3]});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_none_operation() {
        // None of the numbers are negative
        let rule = json!({
            "none": [
                {"var": "numbers"},
                {"<": [{"var": ""}, 0]}
            ]
        });
        let data = json!({"numbers": [1, 2, 3]});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result, Value::Bool(true));

        let data = json!({"numbers": [1, -2, 3]});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_merge_arrays() {
        let rule = json!({"merge": [[1, 2], [3, 4], [5]]});
        let result = evaluate(&rule, &json!({})).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 5);
                for (i, v) in arr.iter().enumerate() {
                    assert_eq!(v.to_number(), (i + 1) as f64);
                }
            }
            _ => panic!("Expected array result"),
        }
    }

    #[test]
    fn test_missing_operation() {
        let rule = json!({"missing": ["a", "b", "c"]});
        let data = json!({"a": 1, "c": 3});
        let result = evaluate(&rule, &data).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 1);
                assert_eq!(arr[0], Value::String("b".into()));
            }
            _ => panic!("Expected array result"),
        }
    }

    #[test]
    fn test_missing_some_operation() {
        // Need at least 2 of ["a", "b", "c"], only have "a"
        let rule = json!({"missing_some": [2, ["a", "b", "c"]]});
        let data = json!({"a": 1});
        let result = evaluate(&rule, &data).unwrap();
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2); // b and c are missing
            }
            _ => panic!("Expected array result"),
        }

        // Need at least 2, have a and b - should return empty
        let data = json!({"a": 1, "b": 2});
        let result = evaluate(&rule, &data).unwrap();
        match result {
            Value::Array(arr) => {
                assert!(arr.is_empty());
            }
            _ => panic!("Expected array result"),
        }
    }

    #[test]
    fn test_strict_equality() {
        // Strict equality should not do type coercion
        let rule = json!({"===": [1, "1"]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::Bool(false));

        let rule = json!({"===": [1, 1]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_ternary_operator() {
        let rule = json!({"?:": [true, "yes", "no"]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::String("yes".into()));

        let rule = json!({"?:": [false, "yes", "no"]});
        let result = evaluate(&rule, &json!({})).unwrap();
        assert_eq!(result, Value::String("no".into()));
    }

    #[test]
    fn test_deeply_nested_expression() {
        // ((((x + 1) * 2) - 3) / 2) where x = 5
        // ((((5 + 1) * 2) - 3) / 2) = (((6 * 2) - 3) / 2) = ((12 - 3) / 2) = 4.5
        let rule = json!({
            "/": [
                {"-": [
                    {"*": [
                        {"+": [{"var": "x"}, 1]},
                        2
                    ]},
                    3
                ]},
                2
            ]
        });
        let data = json!({"x": 5});
        let result = evaluate(&rule, &data).unwrap();
        assert_eq!(result.to_number(), 4.5);
    }
}
