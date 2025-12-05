//! Rule Explanation Generator
//!
//! Converts JSON Logic expressions into plain English explanations.
//! This is used by the `explain_rule` tool.

use crate::error::{AgentError, AgentResult};
use crate::tools::{ExplainRuleOutput, ExplanationStep, VariableInfo};
use serde_json::Value;
use std::collections::HashSet;

/// Explains a JSON Logic expression in plain English
pub struct RuleExplainer {
    verbosity: Verbosity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Verbosity {
    Brief,
    Detailed,
    Technical,
}

impl From<&str> for Verbosity {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "brief" => Verbosity::Brief,
            "technical" => Verbosity::Technical,
            _ => Verbosity::Detailed,
        }
    }
}

impl RuleExplainer {
    pub fn new(verbosity: Verbosity) -> Self {
        Self { verbosity }
    }

    /// Explain a JSON Logic expression
    pub fn explain(&self, expression: &Value) -> AgentResult<ExplainRuleOutput> {
        let mut variables = HashSet::new();
        let mut steps = Vec::new();
        let mut step_num = 1;

        // Extract the main explanation
        let explanation = self.explain_expression(expression, &mut variables, &mut steps, &mut step_num)?;

        // Convert variables to VariableInfo
        let variable_infos: Vec<VariableInfo> = variables
            .into_iter()
            .map(|name| VariableInfo {
                name,
                role: "input".to_string(),
                inferred_type: None,
            })
            .collect();

        Ok(ExplainRuleOutput {
            explanation,
            steps,
            variables: variable_infos,
        })
    }

    fn explain_expression(
        &self,
        expr: &Value,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        match expr {
            Value::Object(map) if map.len() == 1 => {
                let (op, args) = map.iter().next().unwrap();
                self.explain_operation(op, args, variables, steps, step_num)
            }
            Value::Array(_) => {
                // Literal array
                Ok(format!("the list {}", expr))
            }
            Value::Number(n) => Ok(format!("{}", n)),
            Value::String(s) => Ok(format!("\"{}\"", s)),
            Value::Bool(b) => Ok(if *b { "true" } else { "false" }.to_string()),
            Value::Null => Ok("null".to_string()),
            _ => Err(AgentError::JsonLogicParseError(
                "Unexpected expression structure".to_string(),
            )),
        }
    }

    fn explain_operation(
        &self,
        op: &str,
        args: &Value,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        let explanation = match op {
            // Variable access
            "var" => self.explain_var(args, variables)?,

            // Comparison operators
            "==" => self.explain_binary_op(args, "equals", variables, steps, step_num)?,
            "!=" => self.explain_binary_op(args, "is not equal to", variables, steps, step_num)?,
            ">" => self.explain_binary_op(args, "is greater than", variables, steps, step_num)?,
            ">=" => self.explain_binary_op(args, "is greater than or equal to", variables, steps, step_num)?,
            "<" => self.explain_binary_op(args, "is less than", variables, steps, step_num)?,
            "<=" => self.explain_binary_op(args, "is less than or equal to", variables, steps, step_num)?,

            // Arithmetic operators
            "+" => self.explain_arithmetic(args, "add", "plus", variables, steps, step_num)?,
            "-" => self.explain_arithmetic(args, "subtract", "minus", variables, steps, step_num)?,
            "*" => self.explain_arithmetic(args, "multiply", "times", variables, steps, step_num)?,
            "/" => self.explain_arithmetic(args, "divide", "divided by", variables, steps, step_num)?,
            "%" => self.explain_arithmetic(args, "modulo", "mod", variables, steps, step_num)?,

            // Logical operators
            "and" => self.explain_logical(args, "AND", "all of the following are true", variables, steps, step_num)?,
            "or" => self.explain_logical(args, "OR", "any of the following is true", variables, steps, step_num)?,
            "!" | "not" => self.explain_not(args, variables, steps, step_num)?,

            // Conditional
            "if" => self.explain_if(args, variables, steps, step_num)?,

            // Array operations
            "in" => self.explain_in(args, variables, steps, step_num)?,
            "merge" => self.explain_merge(args, variables, steps, step_num)?,
            "map" => self.explain_map(args, variables, steps, step_num)?,
            "filter" => self.explain_filter(args, variables, steps, step_num)?,
            "reduce" => self.explain_reduce(args, variables, steps, step_num)?,
            "all" => self.explain_quantifier(args, "all", variables, steps, step_num)?,
            "some" => self.explain_quantifier(args, "some", variables, steps, step_num)?,
            "none" => self.explain_quantifier(args, "none of", variables, steps, step_num)?,

            // String operations
            "cat" => self.explain_cat(args, variables, steps, step_num)?,
            "substr" => self.explain_substr(args, variables, steps, step_num)?,

            // Misc
            "missing" => self.explain_missing(args, variables)?,
            "missing_some" => self.explain_missing_some(args, variables)?,
            "log" => self.explain_log(args, variables, steps, step_num)?,

            // Default for unknown operations
            _ => format!("apply {} operation to {}", op, args),
        };

        // Add step if detailed
        if self.verbosity != Verbosity::Brief {
            steps.push(ExplanationStep {
                step_number: *step_num,
                description: explanation.clone(),
                expression_part: if self.verbosity == Verbosity::Technical {
                    Some(format!("{{\"{}\": {}}}", op, args))
                } else {
                    None
                },
            });
            *step_num += 1;
        }

        Ok(explanation)
    }

    fn explain_var(&self, args: &Value, variables: &mut HashSet<String>) -> AgentResult<String> {
        match args {
            Value::String(path) => {
                variables.insert(path.clone());
                Ok(format!("the value of '{}'", path))
            }
            Value::Array(arr) if !arr.is_empty() => {
                if let Some(Value::String(path)) = arr.first() {
                    variables.insert(path.clone());
                    if arr.len() > 1 {
                        Ok(format!("the value of '{}' (or {} if missing)", path, arr.get(1).unwrap_or(&Value::Null)))
                    } else {
                        Ok(format!("the value of '{}'", path))
                    }
                } else {
                    Ok("a variable".to_string())
                }
            }
            _ => Ok("a variable".to_string()),
        }
    }

    fn explain_binary_op(
        &self,
        args: &Value,
        op_phrase: &str,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            if arr.len() >= 2 {
                let left = self.explain_expression(&arr[0], variables, steps, step_num)?;
                let right = self.explain_expression(&arr[1], variables, steps, step_num)?;
                return Ok(format!("{} {} {}", left, op_phrase, right));
            }
        }
        Ok(format!("comparison: {} {}", op_phrase, args))
    }

    fn explain_arithmetic(
        &self,
        args: &Value,
        verb: &str,
        operator: &str,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            if arr.len() >= 2 {
                let parts: Result<Vec<String>, AgentError> = arr
                    .iter()
                    .map(|a| self.explain_expression(a, variables, steps, step_num))
                    .collect();
                let parts = parts?;
                return Ok(parts.join(&format!(" {} ", operator)));
            } else if arr.len() == 1 {
                // Unary minus
                let val = self.explain_expression(&arr[0], variables, steps, step_num)?;
                return Ok(format!("negative {}", val));
            }
        }
        Ok(format!("{} {}", verb, args))
    }

    fn explain_logical(
        &self,
        args: &Value,
        op_name: &str,
        phrase: &str,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            let conditions: Result<Vec<String>, AgentError> = arr
                .iter()
                .map(|a| self.explain_expression(a, variables, steps, step_num))
                .collect();
            let conditions = conditions?;

            if self.verbosity == Verbosity::Brief {
                Ok(format!("({})", conditions.join(&format!(" {} ", op_name))))
            } else {
                Ok(format!("{}: {}", phrase, conditions.join("; ")))
            }
        } else {
            Ok(format!("{} condition", op_name))
        }
    }

    fn explain_not(
        &self,
        args: &Value,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            if let Some(inner) = arr.first() {
                let inner_exp = self.explain_expression(inner, variables, steps, step_num)?;
                return Ok(format!("NOT ({})", inner_exp));
            }
        }
        let inner_exp = self.explain_expression(args, variables, steps, step_num)?;
        Ok(format!("NOT ({})", inner_exp))
    }

    fn explain_if(
        &self,
        args: &Value,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            match arr.len() {
                3 => {
                    let condition = self.explain_expression(&arr[0], variables, steps, step_num)?;
                    let then_val = self.explain_expression(&arr[1], variables, steps, step_num)?;
                    let else_val = self.explain_expression(&arr[2], variables, steps, step_num)?;
                    Ok(format!("IF {} THEN {} ELSE {}", condition, then_val, else_val))
                }
                n if n > 3 => {
                    let mut result = String::new();
                    let mut i = 0;
                    while i < arr.len() {
                        if i + 1 < arr.len() && (i + 2 == arr.len() || i + 2 < arr.len()) {
                            let condition = self.explain_expression(&arr[i], variables, steps, step_num)?;
                            let then_val = self.explain_expression(&arr[i + 1], variables, steps, step_num)?;
                            if i == 0 {
                                result.push_str(&format!("IF {} THEN {}", condition, then_val));
                            } else if i + 2 == arr.len() {
                                let else_val = self.explain_expression(&arr[i + 1], variables, steps, step_num)?;
                                result.push_str(&format!(" ELSE {}", else_val));
                                break;
                            } else {
                                result.push_str(&format!(" ELSE IF {} THEN {}", condition, then_val));
                            }
                            i += 2;
                        } else {
                            let else_val = self.explain_expression(&arr[i], variables, steps, step_num)?;
                            result.push_str(&format!(" ELSE {}", else_val));
                            break;
                        }
                    }
                    Ok(result)
                }
                2 => {
                    let condition = self.explain_expression(&arr[0], variables, steps, step_num)?;
                    let then_val = self.explain_expression(&arr[1], variables, steps, step_num)?;
                    Ok(format!("IF {} THEN {}", condition, then_val))
                }
                _ => Ok("conditional expression".to_string()),
            }
        } else {
            Ok("conditional expression".to_string())
        }
    }

    fn explain_in(
        &self,
        args: &Value,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            if arr.len() >= 2 {
                let needle = self.explain_expression(&arr[0], variables, steps, step_num)?;
                let haystack = self.explain_expression(&arr[1], variables, steps, step_num)?;
                return Ok(format!("{} is in {}", needle, haystack));
            }
        }
        Ok("membership check".to_string())
    }

    fn explain_merge(
        &self,
        args: &Value,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            let arrays: Result<Vec<String>, AgentError> = arr
                .iter()
                .map(|a| self.explain_expression(a, variables, steps, step_num))
                .collect();
            return Ok(format!("merge arrays: {}", arrays?.join(", ")));
        }
        Ok("merge arrays".to_string())
    }

    fn explain_map(
        &self,
        args: &Value,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            if arr.len() >= 2 {
                let source = self.explain_expression(&arr[0], variables, steps, step_num)?;
                let transform = self.explain_expression(&arr[1], variables, steps, step_num)?;
                return Ok(format!("for each item in {}, compute {}", source, transform));
            }
        }
        Ok("map over array".to_string())
    }

    fn explain_filter(
        &self,
        args: &Value,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            if arr.len() >= 2 {
                let source = self.explain_expression(&arr[0], variables, steps, step_num)?;
                let condition = self.explain_expression(&arr[1], variables, steps, step_num)?;
                return Ok(format!("filter {} where {}", source, condition));
            }
        }
        Ok("filter array".to_string())
    }

    fn explain_reduce(
        &self,
        args: &Value,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            if arr.len() >= 3 {
                let source = self.explain_expression(&arr[0], variables, steps, step_num)?;
                let reducer = self.explain_expression(&arr[1], variables, steps, step_num)?;
                let initial = self.explain_expression(&arr[2], variables, steps, step_num)?;
                return Ok(format!(
                    "reduce {} starting from {} by applying {}",
                    source, initial, reducer
                ));
            }
        }
        Ok("reduce array".to_string())
    }

    fn explain_quantifier(
        &self,
        args: &Value,
        quantifier: &str,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            if arr.len() >= 2 {
                let source = self.explain_expression(&arr[0], variables, steps, step_num)?;
                let condition = self.explain_expression(&arr[1], variables, steps, step_num)?;
                return Ok(format!("{} items in {} satisfy: {}", quantifier, source, condition));
            }
        }
        Ok(format!("{} check", quantifier))
    }

    fn explain_cat(
        &self,
        args: &Value,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            let parts: Result<Vec<String>, AgentError> = arr
                .iter()
                .map(|a| self.explain_expression(a, variables, steps, step_num))
                .collect();
            return Ok(format!("concatenate: {}", parts?.join(" + ")));
        }
        Ok("concatenate strings".to_string())
    }

    fn explain_substr(
        &self,
        args: &Value,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            if arr.len() >= 2 {
                let source = self.explain_expression(&arr[0], variables, steps, step_num)?;
                let start = self.explain_expression(&arr[1], variables, steps, step_num)?;
                if arr.len() >= 3 {
                    let len = self.explain_expression(&arr[2], variables, steps, step_num)?;
                    return Ok(format!(
                        "substring of {} from position {} with length {}",
                        source, start, len
                    ));
                }
                return Ok(format!("substring of {} from position {}", source, start));
            }
        }
        Ok("substring operation".to_string())
    }

    fn explain_missing(&self, args: &Value, variables: &mut HashSet<String>) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            let paths: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| {
                    variables.insert(s.to_string());
                    format!("'{}'", s)
                }))
                .collect();
            return Ok(format!("check if any of {} are missing", paths.join(", ")));
        }
        Ok("check for missing values".to_string())
    }

    fn explain_missing_some(
        &self,
        args: &Value,
        variables: &mut HashSet<String>,
    ) -> AgentResult<String> {
        if let Value::Array(arr) = args {
            if arr.len() >= 2 {
                let min = arr[0].as_u64().unwrap_or(1);
                if let Value::Array(paths) = &arr[1] {
                    let path_strs: Vec<String> = paths
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| {
                            variables.insert(s.to_string());
                            format!("'{}'", s)
                        }))
                        .collect();
                    return Ok(format!(
                        "check if at least {} of {} are present",
                        min,
                        path_strs.join(", ")
                    ));
                }
            }
        }
        Ok("check for partially missing values".to_string())
    }

    fn explain_log(
        &self,
        args: &Value,
        variables: &mut HashSet<String>,
        steps: &mut Vec<ExplanationStep>,
        step_num: &mut usize,
    ) -> AgentResult<String> {
        let inner = self.explain_expression(args, variables, steps, step_num)?;
        Ok(format!("log {} for debugging", inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_explain_simple_comparison() {
        let explainer = RuleExplainer::new(Verbosity::Brief);
        let expr = json!({">": [{"var": "age"}, 60]});
        let result = explainer.explain(&expr).unwrap();
        assert!(result.explanation.contains("age"));
        assert!(result.explanation.contains("greater than"));
    }

    #[test]
    fn test_explain_if_then_else() {
        let explainer = RuleExplainer::new(Verbosity::Detailed);
        let expr = json!({
            "if": [
                {">": [{"var": "age"}, 60]},
                {"*": [{"var": "base"}, 1.2]},
                {"var": "base"}
            ]
        });
        let result = explainer.explain(&expr).unwrap();
        assert!(result.explanation.contains("IF"));
        assert!(result.explanation.contains("THEN"));
        assert!(result.explanation.contains("ELSE"));
    }

    #[test]
    fn test_extract_variables() {
        let explainer = RuleExplainer::new(Verbosity::Brief);
        let expr = json!({
            "and": [
                {">": [{"var": "age"}, 18]},
                {"<": [{"var": "income"}, 50000]}
            ]
        });
        let result = explainer.explain(&expr).unwrap();
        assert!(result.variables.iter().any(|v| v.name == "age"));
        assert!(result.variables.iter().any(|v| v.name == "income"));
    }
}
