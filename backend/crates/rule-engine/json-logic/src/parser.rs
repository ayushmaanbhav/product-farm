//! JSON Logic parser - converts JSON to optimized AST

use product_farm_core::Value;
use serde_json::Value as JsonValue;

use crate::{Expr, JsonLogicError, JsonLogicResult, VarExpr};

/// Parse a JSON Logic expression into an AST
pub fn parse(json: &JsonValue) -> JsonLogicResult<Expr> {
    Parser::new().parse(json)
}

/// JSON Logic parser
pub struct Parser {
    // Future: could add configuration options here
}

impl Parser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {}
    }

    /// Parse a JSON value into an expression
    pub fn parse(&self, json: &JsonValue) -> JsonLogicResult<Expr> {
        match json {
            // Primitives become literals
            JsonValue::Null => Ok(Expr::Literal(Value::Null)),
            JsonValue::Bool(b) => Ok(Expr::Literal(Value::Bool(*b))),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Expr::Literal(Value::Int(i)))
                } else if let Some(f) = n.as_f64() {
                    Ok(Expr::Literal(Value::Float(f)))
                } else {
                    Err(JsonLogicError::InvalidStructure("Invalid number".to_string()))
                }
            }
            JsonValue::String(s) => Ok(Expr::Literal(Value::String(s.clone()))),

            // Arrays are literal arrays (unless inside an operation)
            JsonValue::Array(arr) => {
                // Top-level arrays are treated as literal arrays
                // In operation context, they're arguments (parsed separately)
                let values: Vec<Value> = arr.iter().map(Value::from_json).collect();
                Ok(Expr::Literal(Value::Array(values)))
            }

            // Objects are operations
            JsonValue::Object(obj) => {
                if obj.is_empty() {
                    return Ok(Expr::Literal(Value::Object(Default::default())));
                }

                // JSON Logic operations have exactly one key
                if obj.len() != 1 {
                    // Multiple keys = literal object
                    let map: std::collections::HashMap<String, Value> = obj
                        .iter()
                        .map(|(k, v)| (k.clone(), Value::from_json(v)))
                        .collect();
                    return Ok(Expr::Literal(Value::Object(map)));
                }

                let (op, args) = obj.iter().next().unwrap();
                self.parse_operation(op, args)
            }
        }
    }

    /// Parse an operation with its arguments
    fn parse_operation(&self, op: &str, args: &JsonValue) -> JsonLogicResult<Expr> {
        // Get arguments as array (single arg becomes array of 1)
        let args_vec = self.normalize_args(args);

        match op {
            // Variable access
            "var" => self.parse_var(args),

            // Comparison
            "==" => self.parse_binary_comparison(args_vec, Expr::Eq),
            "===" => self.parse_binary_comparison(args_vec, Expr::StrictEq),
            "!=" => self.parse_binary_comparison(args_vec, Expr::Ne),
            "!==" => self.parse_binary_comparison(args_vec, Expr::StrictNe),
            "<" => self.parse_chain_comparison(args_vec, Expr::Lt),
            "<=" => self.parse_chain_comparison(args_vec, Expr::Le),
            ">" => self.parse_chain_comparison(args_vec, Expr::Gt),
            ">=" => self.parse_chain_comparison(args_vec, Expr::Ge),

            // Logical
            "!" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() != 1 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "!".to_string(),
                        expected: "1".to_string(),
                        actual: parsed.len(),
                    });
                }
                Ok(Expr::Not(Box::new(parsed.into_iter().next().unwrap())))
            }
            "!!" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() != 1 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "!!".to_string(),
                        expected: "1".to_string(),
                        actual: parsed.len(),
                    });
                }
                Ok(Expr::ToBool(Box::new(parsed.into_iter().next().unwrap())))
            }
            "and" => Ok(Expr::And(self.parse_args(&args_vec)?)),
            "or" => Ok(Expr::Or(self.parse_args(&args_vec)?)),

            // Conditional
            "if" => Ok(Expr::If(self.parse_args(&args_vec)?)),
            "?:" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() != 3 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "?:".to_string(),
                        expected: "3".to_string(),
                        actual: parsed.len(),
                    });
                }
                let mut iter = parsed.into_iter();
                Ok(Expr::Ternary(
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                ))
            }

            // Arithmetic
            "+" => Ok(Expr::Add(self.parse_args(&args_vec)?)),
            "-" => Ok(Expr::Sub(self.parse_args(&args_vec)?)),
            "*" => Ok(Expr::Mul(self.parse_args(&args_vec)?)),
            "/" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() != 2 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "/".to_string(),
                        expected: "2".to_string(),
                        actual: parsed.len(),
                    });
                }
                let mut iter = parsed.into_iter();
                Ok(Expr::Div(
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                ))
            }
            "%" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() != 2 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "%".to_string(),
                        expected: "2".to_string(),
                        actual: parsed.len(),
                    });
                }
                let mut iter = parsed.into_iter();
                Ok(Expr::Mod(
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                ))
            }
            "min" => Ok(Expr::Min(self.parse_args(&args_vec)?)),
            "max" => Ok(Expr::Max(self.parse_args(&args_vec)?)),

            // String
            "cat" => Ok(Expr::Cat(self.parse_args(&args_vec)?)),
            "substr" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() < 2 || parsed.len() > 3 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "substr".to_string(),
                        expected: "2-3".to_string(),
                        actual: parsed.len(),
                    });
                }
                let mut iter = parsed.into_iter();
                let s = Box::new(iter.next().unwrap());
                let start = Box::new(iter.next().unwrap());
                let len = iter.next().map(Box::new);
                Ok(Expr::Substr(s, start, len))
            }

            // Array
            "map" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() != 2 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "map".to_string(),
                        expected: "2".to_string(),
                        actual: parsed.len(),
                    });
                }
                let mut iter = parsed.into_iter();
                Ok(Expr::Map(
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                ))
            }
            "filter" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() != 2 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "filter".to_string(),
                        expected: "2".to_string(),
                        actual: parsed.len(),
                    });
                }
                let mut iter = parsed.into_iter();
                Ok(Expr::Filter(
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                ))
            }
            "reduce" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() != 3 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "reduce".to_string(),
                        expected: "3".to_string(),
                        actual: parsed.len(),
                    });
                }
                let mut iter = parsed.into_iter();
                Ok(Expr::Reduce(
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                ))
            }
            "all" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() != 2 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "all".to_string(),
                        expected: "2".to_string(),
                        actual: parsed.len(),
                    });
                }
                let mut iter = parsed.into_iter();
                Ok(Expr::All(
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                ))
            }
            "some" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() != 2 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "some".to_string(),
                        expected: "2".to_string(),
                        actual: parsed.len(),
                    });
                }
                let mut iter = parsed.into_iter();
                Ok(Expr::Some(
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                ))
            }
            "none" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() != 2 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "none".to_string(),
                        expected: "2".to_string(),
                        actual: parsed.len(),
                    });
                }
                let mut iter = parsed.into_iter();
                Ok(Expr::None(
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                ))
            }
            "merge" => Ok(Expr::Merge(self.parse_args(&args_vec)?)),
            "in" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() != 2 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "in".to_string(),
                        expected: "2".to_string(),
                        actual: parsed.len(),
                    });
                }
                let mut iter = parsed.into_iter();
                Ok(Expr::In(
                    Box::new(iter.next().unwrap()),
                    Box::new(iter.next().unwrap()),
                ))
            }

            // Data access
            "missing" => Ok(Expr::Missing(self.parse_args(&args_vec)?)),
            "missing_some" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() != 2 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "missing_some".to_string(),
                        expected: "2".to_string(),
                        actual: parsed.len(),
                    });
                }
                let mut iter = parsed.into_iter();
                let min = Box::new(iter.next().unwrap());
                // Second arg should be array of keys
                let keys_expr = iter.next().unwrap();
                let keys = match keys_expr {
                    Expr::Literal(Value::Array(arr)) => arr
                        .into_iter()
                        .map(Expr::Literal)
                        .collect(),
                    other => vec![other],
                };
                Ok(Expr::MissingSome(min, keys))
            }

            // Logging
            "log" => {
                let parsed = self.parse_args(&args_vec)?;
                if parsed.len() != 1 {
                    return Err(JsonLogicError::InvalidArgumentCount {
                        op: "log".to_string(),
                        expected: "1".to_string(),
                        actual: parsed.len(),
                    });
                }
                Ok(Expr::Log(Box::new(parsed.into_iter().next().unwrap())))
            }

            // Unknown operation
            _ => Err(JsonLogicError::UnknownOperation(op.to_string())),
        }
    }

    /// Parse variable access
    fn parse_var(&self, args: &JsonValue) -> JsonLogicResult<Expr> {
        match args {
            // {"var": "path"} or {"var": ""} for entire data object
            JsonValue::String(path) => Ok(Expr::Var(VarExpr::new(path))),

            // {"var": 0} or {"var": 1} - array index access
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Expr::Var(VarExpr::new(i.to_string())))
                } else {
                    Err(JsonLogicError::InvalidVariablePath(format!("{}", n)))
                }
            }

            // {"var": ["path", default]}
            JsonValue::Array(arr) => {
                if arr.is_empty() {
                    return Ok(Expr::Var(VarExpr::new("")));
                }

                let path = match &arr[0] {
                    JsonValue::String(s) => s.clone(),
                    JsonValue::Number(n) => n.to_string(),
                    _ => {
                        return Err(JsonLogicError::InvalidVariablePath(
                            "Path must be string or number".to_string(),
                        ))
                    }
                };

                let default = if arr.len() > 1 {
                    Some(Value::from_json(&arr[1]))
                } else {
                    None
                };

                Ok(Expr::Var(VarExpr { path, default }))
            }

            _ => Err(JsonLogicError::InvalidVariablePath(format!("{:?}", args))),
        }
    }

    /// Normalize arguments to a vector
    fn normalize_args(&self, args: &JsonValue) -> Vec<JsonValue> {
        match args {
            JsonValue::Array(arr) => arr.clone(),
            other => vec![other.clone()],
        }
    }

    /// Parse a vector of JSON values into expressions
    fn parse_args(&self, args: &[JsonValue]) -> JsonLogicResult<Vec<Expr>> {
        args.iter().map(|a| self.parse(a)).collect()
    }

    /// Parse binary comparison (==, !=, etc.)
    fn parse_binary_comparison<F>(
        &self,
        args: Vec<JsonValue>,
        make_expr: F,
    ) -> JsonLogicResult<Expr>
    where
        F: FnOnce(Box<Expr>, Box<Expr>) -> Expr,
    {
        let parsed = self.parse_args(&args)?;
        if parsed.len() != 2 {
            return Err(JsonLogicError::InvalidArgumentCount {
                op: "comparison".to_string(),
                expected: "2".to_string(),
                actual: parsed.len(),
            });
        }
        let mut iter = parsed.into_iter();
        Ok(make_expr(
            Box::new(iter.next().unwrap()),
            Box::new(iter.next().unwrap()),
        ))
    }

    /// Parse chained comparison (<, <=, >, >=)
    fn parse_chain_comparison<F>(&self, args: Vec<JsonValue>, make_expr: F) -> JsonLogicResult<Expr>
    where
        F: FnOnce(Vec<Expr>) -> Expr,
    {
        let parsed = self.parse_args(&args)?;
        if parsed.len() < 2 {
            return Err(JsonLogicError::InvalidArgumentCount {
                op: "comparison".to_string(),
                expected: "at least 2".to_string(),
                actual: parsed.len(),
            });
        }
        Ok(make_expr(parsed))
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_literal() {
        let expr = parse(&json!(42)).unwrap();
        assert_eq!(expr, Expr::Literal(Value::Int(42)));

        let expr = parse(&json!("hello")).unwrap();
        assert_eq!(expr, Expr::Literal(Value::String("hello".to_string())));

        let expr = parse(&json!(true)).unwrap();
        assert_eq!(expr, Expr::Literal(Value::Bool(true)));
    }

    #[test]
    fn test_parse_var() {
        let expr = parse(&json!({"var": "age"})).unwrap();
        assert!(matches!(expr, Expr::Var(v) if v.path == "age"));

        let expr = parse(&json!({"var": ["age", 0]})).unwrap();
        assert!(matches!(expr, Expr::Var(v) if v.path == "age" && v.default == Some(Value::Int(0))));
    }

    #[test]
    fn test_parse_comparison() {
        let expr = parse(&json!({">": [{"var": "age"}, 60]})).unwrap();
        assert!(matches!(expr, Expr::Gt(_)));

        let expr = parse(&json!({"<": [1, {"var": "x"}, 10]})).unwrap();
        assert!(matches!(expr, Expr::Lt(v) if v.len() == 3));
    }

    #[test]
    fn test_parse_arithmetic() {
        let expr = parse(&json!({"+": [1, 2, 3]})).unwrap();
        assert!(matches!(expr, Expr::Add(v) if v.len() == 3));

        let expr = parse(&json!({"*": [{"var": "base"}, 1.2]})).unwrap();
        assert!(matches!(expr, Expr::Mul(_)));
    }

    #[test]
    fn test_parse_if() {
        let expr = parse(&json!({
            "if": [
                {">": [{"var": "age"}, 60]},
                "senior",
                "regular"
            ]
        }))
        .unwrap();
        assert!(matches!(expr, Expr::If(v) if v.len() == 3));
    }

    #[test]
    fn test_parse_complex() {
        // {"if": [{">": [{"var": "age"}, 60]}, {"*": [{"var": "base"}, 1.2]}, {"var": "base"}]}
        let expr = parse(&json!({
            "if": [
                {">": [{"var": "age"}, 60]},
                {"*": [{"var": "base"}, 1.2]},
                {"var": "base"}
            ]
        }))
        .unwrap();

        let vars = expr.collect_variables();
        assert!(vars.contains(&"age"));
        assert!(vars.contains(&"base"));
    }
}
