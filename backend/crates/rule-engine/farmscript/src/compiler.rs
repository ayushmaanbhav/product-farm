//! Compiler from FarmScript AST to JSON Logic
//!
//! Translates FarmScript expressions into JSON Logic format
//! that can be executed by the rule engine.

use crate::ast::*;
use serde_json::{json, Value as JsonValue};
use thiserror::Error;

/// Compilation error
#[derive(Debug, Error, Clone)]
pub enum CompileError {
    #[error("Unsupported expression: {0}")]
    Unsupported(String),

    #[error("Invalid lambda: {0}")]
    InvalidLambda(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

impl From<crate::parser::ParseError> for CompileError {
    fn from(e: crate::parser::ParseError) -> Self {
        CompileError::ParseError(e.to_string())
    }
}

/// Compilation options
#[derive(Debug, Clone, Default)]
pub struct CompileOptions {
    /// Use strict equality (===) by default for == operator
    pub strict_equality_default: bool,
    /// Emit safe division as custom operations
    pub emit_safe_division: bool,
}

/// The FarmScript to JSON Logic compiler
#[allow(dead_code)]
pub struct Compiler {
    options: CompileOptions,
}

impl Compiler {
    /// Create a new compiler with options
    pub fn new(options: CompileOptions) -> Self {
        Self { options }
    }

    /// Create a compiler with default options
    pub fn default_options() -> Self {
        Self::new(CompileOptions::default())
    }

    /// Compile an expression to JSON Logic
    pub fn compile(&self, expr: &Expr) -> Result<JsonValue, CompileError> {
        match expr {
            Expr::Literal(lit) => self.compile_literal(lit),
            Expr::Var(var) => self.compile_var(var),
            Expr::Binary(binary) => self.compile_binary(binary),
            Expr::Unary(unary) => self.compile_unary(unary),
            Expr::Postfix(postfix) => self.compile_postfix(postfix),
            Expr::If(if_expr) => self.compile_if(if_expr),
            Expr::Let(let_expr) => self.compile_let(let_expr),
            Expr::Call(call) => self.compile_call(call),
            Expr::MethodCall(method_call) => self.compile_method_call(method_call),
            Expr::Property(prop) => self.compile_property(prop),
            Expr::Index(idx) => self.compile_index(idx),
            Expr::Array(items) => self.compile_array(items),
            Expr::Lambda(lambda) => self.compile_lambda(lambda),
            Expr::Query(query) => self.compile_query(query),
            Expr::Template(parts) => self.compile_template(parts),
            Expr::NullCoalesce(a, b) => self.compile_null_coalesce(a, b),
        }
    }

    fn compile_literal(&self, lit: &Literal) -> Result<JsonValue, CompileError> {
        Ok(match lit {
            Literal::Null => JsonValue::Null,
            Literal::Bool(b) => JsonValue::Bool(*b),
            Literal::Integer(n) => json!(*n),
            Literal::Float(n) => json!(*n),
            Literal::String(s) => JsonValue::String(s.clone()),
        })
    }

    fn compile_var(&self, var: &VarExpr) -> Result<JsonValue, CompileError> {
        // Handle path-style variables by removing leading /
        let path = if var.name.starts_with('/') {
            &var.name[1..] // Remove leading slash for JSON Logic
        } else {
            &var.name
        };

        // Convert /path/to/var to path.to.var for JSON Logic dot notation
        let normalized_path = path.replace('/', ".");

        Ok(json!({ "var": normalized_path }))
    }

    fn compile_binary(&self, binary: &BinaryExpr) -> Result<JsonValue, CompileError> {
        // Handle and/or specially to flatten nested expressions
        match binary.op {
            BinaryOp::And => return self.compile_logical_chain(&binary.left, &binary.right, "and"),
            BinaryOp::Or => return self.compile_logical_chain(&binary.left, &binary.right, "or"),
            _ => {}
        }

        let left = self.compile(&binary.left)?;
        let right = self.compile(&binary.right)?;

        let op = match binary.op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Pow => {
                // JSON Logic doesn't have power, emit as custom operation
                return Ok(json!({ "pow": [left, right] }));
            }
            BinaryOp::SafeDivZero => {
                // Safe division returning 0: if b == 0 then 0 else a / b
                return Ok(json!({
                    "if": [
                        { "==": [right.clone(), 0] },
                        0,
                        { "/": [left, right] }
                    ]
                }));
            }
            BinaryOp::SafeDivNull => {
                // Safe division returning null: if b == 0 then null else a / b
                return Ok(json!({
                    "if": [
                        { "==": [right.clone(), 0] },
                        null,
                        { "/": [left, right] }
                    ]
                }));
            }
            BinaryOp::Eq => "==",
            BinaryOp::StrictEq => "===",
            BinaryOp::NotEq => "!=",
            BinaryOp::StrictNotEq => "!==",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::LtEq => "<=",
            BinaryOp::GtEq => ">=",
            BinaryOp::And | BinaryOp::Or => unreachable!(), // Handled above
            BinaryOp::In => "in",
        };

        Ok(json!({ op: [left, right] }))
    }

    /// Compile logical and/or chains into flat arrays
    /// e.g., `a and b and c` becomes `{"and": [a, b, c]}` instead of nested
    fn compile_logical_chain(&self, left: &Expr, right: &Expr, op: &str) -> Result<JsonValue, CompileError> {
        let target_op = if op == "and" { BinaryOp::And } else { BinaryOp::Or };

        let mut operands = Vec::new();
        self.collect_logical_operands(left, target_op, &mut operands)?;
        self.collect_logical_operands(right, target_op, &mut operands)?;

        Ok(json!({ op: operands }))
    }

    /// Recursively collect operands for a logical chain
    fn collect_logical_operands(&self, expr: &Expr, target_op: BinaryOp, operands: &mut Vec<JsonValue>) -> Result<(), CompileError> {
        if let Expr::Binary(binary) = expr {
            if binary.op == target_op {
                // Same operator - flatten
                self.collect_logical_operands(&binary.left, target_op, operands)?;
                self.collect_logical_operands(&binary.right, target_op, operands)?;
                return Ok(());
            }
        }
        // Different expression - compile and add to list
        operands.push(self.compile(expr)?);
        Ok(())
    }

    fn compile_unary(&self, unary: &UnaryExpr) -> Result<JsonValue, CompileError> {
        let expr = self.compile(&unary.expr)?;

        match unary.op {
            UnaryOp::Not => Ok(json!({ "!": expr })),
            UnaryOp::Neg => Ok(json!({ "-": [expr] })),
            UnaryOp::Plus => Ok(expr), // No-op
        }
    }

    fn compile_postfix(&self, postfix: &PostfixExpr) -> Result<JsonValue, CompileError> {
        let expr = self.compile(&postfix.expr)?;

        match postfix.op {
            PostfixOp::Truthy => Ok(json!({ "!!": expr })),
        }
    }

    fn compile_if(&self, if_expr: &IfExpr) -> Result<JsonValue, CompileError> {
        let mut args = Vec::new();

        // Main condition and then branch
        args.push(self.compile(&if_expr.condition)?);
        args.push(self.compile(&if_expr.then_branch)?);

        // Else-if chains
        for (cond, then) in &if_expr.else_ifs {
            args.push(self.compile(cond)?);
            args.push(self.compile(then)?);
        }

        // Final else branch
        if let Some(ref else_branch) = if_expr.else_branch {
            args.push(self.compile(else_branch)?);
        }

        Ok(json!({ "if": args }))
    }

    fn compile_let(&self, let_expr: &LetExpr) -> Result<JsonValue, CompileError> {
        // JSON Logic doesn't have let bindings, so we inline the value
        // by substituting variable references in the body
        let compiled_value = self.compile(&let_expr.value)?;
        self.compile_with_substitution(&let_expr.body, &let_expr.name, &compiled_value)
    }

    /// Compile an expression with variable substitution
    fn compile_with_substitution(&self, expr: &Expr, var_name: &str, replacement: &JsonValue) -> Result<JsonValue, CompileError> {
        match expr {
            Expr::Var(v) if v.name == var_name => Ok(replacement.clone()),
            Expr::Var(v) => self.compile_var(v),
            Expr::Literal(lit) => self.compile_literal(lit),
            Expr::Binary(b) => {
                // Handle and/or specially to flatten
                match b.op {
                    BinaryOp::And => {
                        let mut operands = Vec::new();
                        self.collect_logical_operands_with_sub(&b.left, BinaryOp::And, &mut operands, var_name, replacement)?;
                        self.collect_logical_operands_with_sub(&b.right, BinaryOp::And, &mut operands, var_name, replacement)?;
                        return Ok(json!({ "and": operands }));
                    }
                    BinaryOp::Or => {
                        let mut operands = Vec::new();
                        self.collect_logical_operands_with_sub(&b.left, BinaryOp::Or, &mut operands, var_name, replacement)?;
                        self.collect_logical_operands_with_sub(&b.right, BinaryOp::Or, &mut operands, var_name, replacement)?;
                        return Ok(json!({ "or": operands }));
                    }
                    _ => {}
                }

                let left = self.compile_with_substitution(&b.left, var_name, replacement)?;
                let right = self.compile_with_substitution(&b.right, var_name, replacement)?;
                let op = match b.op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Mod => "%",
                    BinaryOp::Eq => "==",
                    BinaryOp::StrictEq => "===",
                    BinaryOp::NotEq => "!=",
                    BinaryOp::StrictNotEq => "!==",
                    BinaryOp::Lt => "<",
                    BinaryOp::Gt => ">",
                    BinaryOp::LtEq => "<=",
                    BinaryOp::GtEq => ">=",
                    BinaryOp::In => "in",
                    BinaryOp::Pow => return Ok(json!({ "pow": [left, right] })),
                    BinaryOp::SafeDivZero => {
                        return Ok(json!({
                            "if": [{ "==": [right.clone(), 0] }, 0, { "/": [left, right] }]
                        }));
                    }
                    BinaryOp::SafeDivNull => {
                        return Ok(json!({
                            "if": [{ "==": [right.clone(), 0] }, null, { "/": [left, right] }]
                        }));
                    }
                    BinaryOp::And | BinaryOp::Or => unreachable!(),
                };
                Ok(json!({ op: [left, right] }))
            }
            Expr::Unary(u) => {
                let inner = self.compile_with_substitution(&u.expr, var_name, replacement)?;
                match u.op {
                    UnaryOp::Not => Ok(json!({ "!": inner })),
                    UnaryOp::Neg => Ok(json!({ "-": [inner] })),
                    UnaryOp::Plus => Ok(inner),
                }
            }
            Expr::Call(c) => {
                let args: Vec<JsonValue> = c.args.iter()
                    .map(|a| self.compile_with_substitution(a, var_name, replacement))
                    .collect::<Result<_, _>>()?;
                // Delegate to regular call compilation with pre-compiled args
                self.compile_call_with_args(&c.function, args)
            }
            Expr::If(if_expr) => {
                let mut args = Vec::new();
                args.push(self.compile_with_substitution(&if_expr.condition, var_name, replacement)?);
                args.push(self.compile_with_substitution(&if_expr.then_branch, var_name, replacement)?);
                for (cond, then) in &if_expr.else_ifs {
                    args.push(self.compile_with_substitution(cond, var_name, replacement)?);
                    args.push(self.compile_with_substitution(then, var_name, replacement)?);
                }
                if let Some(ref else_branch) = if_expr.else_branch {
                    args.push(self.compile_with_substitution(else_branch, var_name, replacement)?);
                }
                Ok(json!({ "if": args }))
            }
            Expr::Array(items) => {
                let compiled: Vec<JsonValue> = items.iter()
                    .map(|i| self.compile_with_substitution(i, var_name, replacement))
                    .collect::<Result<_, _>>()?;
                Ok(JsonValue::Array(compiled))
            }
            // For other expressions, fall back to regular compilation
            _ => self.compile(expr),
        }
    }

    /// Collect logical operands with substitution
    fn collect_logical_operands_with_sub(
        &self,
        expr: &Expr,
        target_op: BinaryOp,
        operands: &mut Vec<JsonValue>,
        var_name: &str,
        replacement: &JsonValue,
    ) -> Result<(), CompileError> {
        if let Expr::Binary(binary) = expr {
            if binary.op == target_op {
                self.collect_logical_operands_with_sub(&binary.left, target_op, operands, var_name, replacement)?;
                self.collect_logical_operands_with_sub(&binary.right, target_op, operands, var_name, replacement)?;
                return Ok(());
            }
        }
        operands.push(self.compile_with_substitution(expr, var_name, replacement)?);
        Ok(())
    }

    /// Compile a function call with pre-compiled arguments
    fn compile_call_with_args(&self, function: &str, args: Vec<JsonValue>) -> Result<JsonValue, CompileError> {
        match function {
            "min" => Ok(json!({ "min": args })),
            "max" => Ok(json!({ "max": args })),
            "sum" => Ok(json!({ "+": args })),
            "abs" => {
                if args.len() != 1 {
                    return Err(CompileError::Unsupported("abs requires 1 argument".into()));
                }
                let x = &args[0];
                Ok(json!({ "if": [{ "<": [x, 0] }, { "-": [0, x] }, x] }))
            }
            "clamp" => {
                if args.len() != 3 {
                    return Err(CompileError::Unsupported("clamp requires 3 arguments (min, max, value)".into()));
                }
                Ok(json!({ "max": [args[0].clone(), { "min": [args[1].clone(), args[2].clone()] }] }))
            }
            "floor" => Ok(json!({ "floor": args })),
            "ceil" => Ok(json!({ "ceil": args })),
            "round" => Ok(json!({ "round": args })),
            "sqrt" => Ok(json!({ "sqrt": args })),
            "pow" => Ok(json!({ "pow": args })),
            "len" | "length" | "count" => Ok(json!({ "count": args })),
            "substr" | "substring" => Ok(json!({ "substr": args })),
            "cat" | "concat" => Ok(json!({ "cat": args })),
            "upper" => Ok(json!({ "upper": args })),
            "lower" => Ok(json!({ "lower": args })),
            "merge" => Ok(json!({ "merge": args })),
            other => Ok(json!({ other: args })),
        }
    }

    fn compile_call(&self, call: &CallExpr) -> Result<JsonValue, CompileError> {
        let args: Vec<JsonValue> = call.args.iter()
            .map(|a| self.compile(a))
            .collect::<Result<_, _>>()?;

        // Map function names to JSON Logic operations
        match call.function.as_str() {
            // Aggregation functions
            "min" => Ok(json!({ "min": args })),
            "max" => Ok(json!({ "max": args })),
            "sum" => {
                // JSON Logic + takes variadic args
                Ok(json!({ "+": args }))
            }

            // Math functions
            "abs" => {
                if args.len() != 1 {
                    return Err(CompileError::Unsupported("abs requires 1 argument".into()));
                }
                // abs(x) = if x < 0 then -x else x
                let x = &args[0];
                Ok(json!({
                    "if": [
                        { "<": [x, 0] },
                        { "-": [0, x] },
                        x
                    ]
                }))
            }
            "round" => {
                // JSON Logic doesn't have round, emit custom operation
                Ok(json!({ "round": args }))
            }
            "floor" => Ok(json!({ "floor": args })),
            "ceil" => Ok(json!({ "ceil": args })),
            "pow" => {
                if args.len() != 2 {
                    return Err(CompileError::Unsupported("pow requires 2 arguments".into()));
                }
                Ok(json!({ "pow": args }))
            }
            "sqrt" => {
                if args.len() != 1 {
                    return Err(CompileError::Unsupported("sqrt requires 1 argument".into()));
                }
                Ok(json!({ "sqrt": args }))
            }

            // Clamp: clamp(min, max, value)
            "clamp" => {
                if args.len() != 3 {
                    return Err(CompileError::Unsupported("clamp requires 3 arguments (min, max, value)".into()));
                }
                let min_val = &args[0];
                let max_val = &args[1];
                let value = &args[2];
                // max(min_val, min(max_val, value))
                Ok(json!({
                    "max": [
                        min_val,
                        { "min": [max_val, value] }
                    ]
                }))
            }

            // Safe division
            "safe_div" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(CompileError::Unsupported("safe_div requires 2-3 arguments".into()));
                }
                let a = &args[0];
                let b = &args[1];
                let default_val = if args.len() > 2 {
                    args[2].clone()
                } else {
                    json!(0)
                };
                Ok(json!({
                    "if": [
                        { "==": [b, 0] },
                        default_val,
                        { "/": [a, b] }
                    ]
                }))
            }

            // String functions
            "cat" => Ok(json!({ "cat": args })),
            "substr" => Ok(json!({ "substr": args })),
            "len" | "length" => {
                if args.len() != 1 {
                    return Err(CompileError::Unsupported("len requires 1 argument".into()));
                }
                // JSON Logic doesn't have len, use custom operation
                Ok(json!({ "len": args }))
            }

            // Array functions
            "merge" => Ok(json!({ "merge": args })),
            "count" => {
                if args.len() != 1 {
                    return Err(CompileError::Unsupported("count requires 1 argument".into()));
                }
                Ok(json!({ "count": args }))
            }

            // Data operations
            "missing" => Ok(json!({ "missing": args })),
            "missing_some" => {
                if args.len() != 2 {
                    return Err(CompileError::Unsupported("missing_some requires 2 arguments".into()));
                }
                Ok(json!({ "missing_some": args }))
            }

            // Debug
            "log" => {
                if args.len() != 1 {
                    return Err(CompileError::Unsupported("log requires 1 argument".into()));
                }
                Ok(json!({ "log": args[0] }))
            }

            // Contains
            "contains" => {
                if args.len() != 2 {
                    return Err(CompileError::Unsupported("contains requires 2 arguments".into()));
                }
                // contains(haystack, needle) = in(needle, haystack)
                Ok(json!({ "in": [&args[1], &args[0]] }))
            }

            // Unknown function - emit as custom operation
            other => Ok(json!({ other: args })),
        }
    }

    fn compile_method_call(&self, method: &MethodCallExpr) -> Result<JsonValue, CompileError> {
        let object = self.compile(&method.object)?;
        let args: Vec<JsonValue> = method.args.iter()
            .map(|a| self.compile(a))
            .collect::<Result<_, _>>()?;

        match method.method.as_str() {
            // Array methods
            "map" => {
                if args.len() != 1 {
                    return Err(CompileError::Unsupported("map requires 1 argument".into()));
                }
                Ok(json!({ "map": [object, args[0]] }))
            }
            "filter" => {
                if args.len() != 1 {
                    return Err(CompileError::Unsupported("filter requires 1 argument".into()));
                }
                Ok(json!({ "filter": [object, args[0]] }))
            }
            "reduce" => {
                if args.len() != 2 {
                    return Err(CompileError::Unsupported("reduce requires 2 arguments (initial, reducer)".into()));
                }
                Ok(json!({ "reduce": [object, args[1], args[0]] }))
            }
            "all" => {
                if args.len() != 1 {
                    return Err(CompileError::Unsupported("all requires 1 argument".into()));
                }
                Ok(json!({ "all": [object, args[0]] }))
            }
            "some" => {
                if args.len() != 1 {
                    return Err(CompileError::Unsupported("some requires 1 argument".into()));
                }
                Ok(json!({ "some": [object, args[0]] }))
            }
            "none" => {
                if args.len() != 1 {
                    return Err(CompileError::Unsupported("none requires 1 argument".into()));
                }
                Ok(json!({ "none": [object, args[0]] }))
            }
            "contains" => {
                if args.len() != 1 {
                    return Err(CompileError::Unsupported("contains requires 1 argument".into()));
                }
                // obj.contains(x) = in(x, obj)
                Ok(json!({ "in": [args[0], object] }))
            }

            // String methods
            "substr" => {
                let mut substr_args = vec![object];
                substr_args.extend(args);
                Ok(json!({ "substr": substr_args }))
            }
            "length" | "len" => {
                Ok(json!({ "len": [object] }))
            }

            // Unknown method
            other => {
                let mut all_args = vec![object];
                all_args.extend(args);
                Ok(json!({ other: all_args }))
            }
        }
    }

    fn compile_property(&self, prop: &PropertyExpr) -> Result<JsonValue, CompileError> {
        // For nested property access, we need to build the path
        // obj.prop becomes {"var": "obj.prop"}
        let path = self.build_property_path(&prop.object, &prop.property)?;
        Ok(json!({ "var": path }))
    }

    fn build_property_path(&self, object: &Expr, property: &str) -> Result<String, CompileError> {
        match object {
            Expr::Var(v) => {
                let base = if v.name.starts_with('/') {
                    v.name[1..].replace('/', ".")
                } else {
                    v.name.clone()
                };
                Ok(format!("{}.{}", base, property))
            }
            Expr::Property(p) => {
                let base = self.build_property_path(&p.object, &p.property)?;
                Ok(format!("{}.{}", base, property))
            }
            _ => {
                // For complex expressions, we can't use simple var access
                // Fall back to a computed approach (not standard JSON Logic)
                Err(CompileError::Unsupported(
                    "Complex property access not yet supported".into()
                ))
            }
        }
    }

    fn compile_index(&self, idx: &IndexExpr) -> Result<JsonValue, CompileError> {
        // arr[idx] in JSON Logic is tricky
        // We need to use var with computed index
        // For simple cases: arr[0] -> {"var": "arr.0"}
        match (&idx.object, &idx.index) {
            (Expr::Var(v), Expr::Literal(Literal::Integer(n))) => {
                let path = if v.name.starts_with('/') {
                    v.name[1..].replace('/', ".")
                } else {
                    v.name.clone()
                };
                Ok(json!({ "var": format!("{}.{}", path, n) }))
            }
            _ => {
                // For dynamic indexing, emit a custom operation
                let arr = self.compile(&idx.object)?;
                let index = self.compile(&idx.index)?;
                Ok(json!({ "index": [arr, index] }))
            }
        }
    }

    fn compile_array(&self, items: &[Expr]) -> Result<JsonValue, CompileError> {
        let compiled: Vec<JsonValue> = items.iter()
            .map(|i| self.compile(i))
            .collect::<Result<_, _>>()?;
        Ok(JsonValue::Array(compiled))
    }

    fn compile_lambda(&self, lambda: &LambdaExpr) -> Result<JsonValue, CompileError> {
        // Lambdas in FarmScript are used with array methods
        // In JSON Logic, the lambda body references "current" element via {"var": ""}
        // We need to transform variable references accordingly

        // For single-param lambdas, the param becomes {"var": ""}
        // For multi-param (reduce), we have "current" and "accumulator"

        if lambda.params.len() == 1 {
            // Replace references to the single param with {"var": ""}
            let body = self.transform_lambda_body(&lambda.body, &lambda.params[0], "")?;
            Ok(body)
        } else if lambda.params.len() == 2 {
            // Reduce lambda: (acc, cur) => ...
            // acc -> {"var": "accumulator"}, cur -> {"var": "current"}
            let body = self.transform_reduce_lambda(&lambda.body, &lambda.params[0], &lambda.params[1])?;
            Ok(body)
        } else {
            Err(CompileError::InvalidLambda(
                "Lambdas must have 1 or 2 parameters".into()
            ))
        }
    }

    fn transform_lambda_body(&self, expr: &Expr, param: &str, replacement: &str) -> Result<JsonValue, CompileError> {
        match expr {
            Expr::Var(v) if v.name == param => {
                Ok(json!({ "var": replacement }))
            }
            Expr::Var(v) => {
                // Keep other variables as-is
                self.compile_var(v)
            }
            Expr::Literal(lit) => self.compile_literal(lit),
            Expr::Binary(b) => {
                let left = self.transform_lambda_body(&b.left, param, replacement)?;
                let right = self.transform_lambda_body(&b.right, param, replacement)?;
                let op = match b.op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Mod => "%",
                    BinaryOp::Eq => "==",
                    BinaryOp::StrictEq => "===",
                    BinaryOp::NotEq => "!=",
                    BinaryOp::StrictNotEq => "!==",
                    BinaryOp::Lt => "<",
                    BinaryOp::Gt => ">",
                    BinaryOp::LtEq => "<=",
                    BinaryOp::GtEq => ">=",
                    BinaryOp::And => "and",
                    BinaryOp::Or => "or",
                    BinaryOp::In => "in",
                    BinaryOp::Pow => "pow",
                    BinaryOp::SafeDivZero | BinaryOp::SafeDivNull => {
                        return self.compile_binary(&BinaryExpr {
                            left: b.left.clone(),
                            op: b.op,
                            right: b.right.clone(),
                            span: b.span,
                        });
                    }
                };
                Ok(json!({ op: [left, right] }))
            }
            Expr::Unary(u) => {
                let inner = self.transform_lambda_body(&u.expr, param, replacement)?;
                match u.op {
                    UnaryOp::Not => Ok(json!({ "!": inner })),
                    UnaryOp::Neg => Ok(json!({ "-": [inner] })),
                    UnaryOp::Plus => Ok(inner),
                }
            }
            Expr::Call(c) => {
                let args: Vec<JsonValue> = c.args.iter()
                    .map(|a| self.transform_lambda_body(a, param, replacement))
                    .collect::<Result<_, _>>()?;
                match c.function.as_str() {
                    "min" => Ok(json!({ "min": args })),
                    "max" => Ok(json!({ "max": args })),
                    other => Ok(json!({ other: args })),
                }
            }
            _ => {
                // For other expressions, fall back to regular compilation
                // This might not correctly transform all nested param references
                self.compile(expr)
            }
        }
    }

    fn transform_reduce_lambda(&self, expr: &Expr, acc_param: &str, cur_param: &str) -> Result<JsonValue, CompileError> {
        match expr {
            Expr::Var(v) if v.name == acc_param => {
                Ok(json!({ "var": "accumulator" }))
            }
            Expr::Var(v) if v.name == cur_param => {
                Ok(json!({ "var": "current" }))
            }
            Expr::Var(v) => {
                self.compile_var(v)
            }
            Expr::Literal(lit) => self.compile_literal(lit),
            Expr::Binary(b) => {
                let left = self.transform_reduce_lambda(&b.left, acc_param, cur_param)?;
                let right = self.transform_reduce_lambda(&b.right, acc_param, cur_param)?;
                let op = match b.op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Mod => "%",
                    BinaryOp::Eq => "==",
                    BinaryOp::StrictEq => "===",
                    BinaryOp::NotEq => "!=",
                    BinaryOp::StrictNotEq => "!==",
                    BinaryOp::Lt => "<",
                    BinaryOp::Gt => ">",
                    BinaryOp::LtEq => "<=",
                    BinaryOp::GtEq => ">=",
                    BinaryOp::And => "and",
                    BinaryOp::Or => "or",
                    BinaryOp::In => "in",
                    BinaryOp::Pow => "pow",
                    _ => return self.compile(expr),
                };
                Ok(json!({ op: [left, right] }))
            }
            _ => self.compile(expr),
        }
    }

    fn compile_query(&self, query: &QueryExpr) -> Result<JsonValue, CompileError> {
        // SQL-like: from items where cond select expr
        // Translates to: map(filter(items, cond), expr)

        let source = self.compile(&query.source)?;
        let projection = self.compile(&query.projection)?;

        if let Some(ref filter) = query.filter {
            let filter_compiled = self.compile(filter)?;
            // filter then map
            Ok(json!({
                "map": [
                    { "filter": [source, filter_compiled] },
                    projection
                ]
            }))
        } else {
            // just map
            Ok(json!({ "map": [source, projection] }))
        }
    }

    fn compile_template(&self, parts: &[TemplateExpr]) -> Result<JsonValue, CompileError> {
        // Template string: `Hello {name}!`
        // Translates to: {"cat": ["Hello ", {"var": "name"}, "!"]}

        let compiled_parts: Vec<JsonValue> = parts.iter()
            .map(|part| match part {
                TemplateExpr::Literal(s) => Ok(JsonValue::String(s.clone())),
                TemplateExpr::Expr(e) => self.compile(e),
            })
            .collect::<Result<_, _>>()?;

        Ok(json!({ "cat": compiled_parts }))
    }

    fn compile_null_coalesce(&self, a: &Expr, b: &Expr) -> Result<JsonValue, CompileError> {
        // a ?? b = if a != null then a else b
        let a_compiled = self.compile(a)?;
        let b_compiled = self.compile(b)?;

        Ok(json!({
            "if": [
                { "!=": [a_compiled.clone(), null] },
                a_compiled,
                b_compiled
            ]
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Lexer, Parser};

    fn compile(source: &str) -> Result<JsonValue, CompileError> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let ast = parser.parse()?;
        let compiler = Compiler::default_options();
        compiler.compile(&ast)
    }

    #[test]
    fn test_simple_comparison() {
        let result = compile("x < 10").unwrap();
        assert_eq!(result, json!({"<": [{"var": "x"}, 10]}));
    }

    #[test]
    fn test_and_expression() {
        let result = compile("a and b").unwrap();
        assert_eq!(result, json!({"and": [{"var": "a"}, {"var": "b"}]}));
    }

    #[test]
    fn test_and_chain_flattening() {
        // BODMAS: Chained and/or should flatten to a single array
        let result = compile("a and b and c and d").unwrap();
        assert_eq!(
            result,
            json!({"and": [{"var": "a"}, {"var": "b"}, {"var": "c"}, {"var": "d"}]})
        );
    }

    #[test]
    fn test_or_chain_flattening() {
        let result = compile("a or b or c").unwrap();
        assert_eq!(
            result,
            json!({"or": [{"var": "a"}, {"var": "b"}, {"var": "c"}]})
        );
    }

    #[test]
    fn test_mixed_and_or_no_flatten() {
        // Different operators should NOT flatten together
        let result = compile("a and b or c").unwrap();
        // 'or' has lower precedence than 'and', so: (a and b) or c
        assert_eq!(
            result,
            json!({"or": [{"and": [{"var": "a"}, {"var": "b"}]}, {"var": "c"}]})
        );
    }

    #[test]
    fn test_brackets_precedence() {
        // Brackets should override precedence (BODMAS)
        let result = compile("a and (b or c)").unwrap();
        assert_eq!(
            result,
            json!({"and": [{"var": "a"}, {"or": [{"var": "b"}, {"var": "c"}]}]})
        );
    }

    #[test]
    fn test_let_expression() {
        // let x = 5 in x * 2 should substitute x with 5
        let result = compile("let x = 5 in x * 2").unwrap();
        assert_eq!(result, json!({"*": [5, 2]}));
    }

    #[test]
    fn test_let_expression_with_var() {
        // let factor = /multiplier in factor * 10
        let result = compile("let factor = /multiplier in factor * 10").unwrap();
        assert_eq!(result, json!({"*": [{"var": "multiplier"}, 10]}));
    }

    #[test]
    fn test_complex_expression() {
        let result = compile("alert_acknowledged and time_since_alert_secs < 120").unwrap();
        assert_eq!(
            result,
            json!({
                "and": [
                    {"var": "alert_acknowledged"},
                    {"<": [{"var": "time_since_alert_secs"}, 120]}
                ]
            })
        );
    }

    #[test]
    fn test_clamp() {
        let result = compile("clamp(0, 100, raw_score)").unwrap();
        assert_eq!(
            result,
            json!({
                "max": [
                    0,
                    {"min": [100, {"var": "raw_score"}]}
                ]
            })
        );
    }

    #[test]
    fn test_if_then_else() {
        let result = compile("if x > 0 then 1 else 0").unwrap();
        assert_eq!(
            result,
            json!({
                "if": [
                    {">": [{"var": "x"}, 0]},
                    1,
                    0
                ]
            })
        );
    }

    #[test]
    fn test_path_variable() {
        let result = compile("/users/count").unwrap();
        assert_eq!(result, json!({"var": "users.count"}));
    }

    #[test]
    fn test_safe_division() {
        let result = compile("a /? b").unwrap();
        assert_eq!(
            result,
            json!({
                "if": [
                    {"==": [{"var": "b"}, 0]},
                    0,
                    {"/": [{"var": "a"}, {"var": "b"}]}
                ]
            })
        );
    }

    #[test]
    fn test_safe_div_function() {
        let result = compile("safe_div(a, b, 0)").unwrap();
        assert_eq!(
            result,
            json!({
                "if": [
                    {"==": [{"var": "b"}, 0]},
                    0,
                    {"/": [{"var": "a"}, {"var": "b"}]}
                ]
            })
        );
    }

    #[test]
    fn test_in_operator() {
        let result = compile("x in [1, 2, 3]").unwrap();
        assert_eq!(
            result,
            json!({"in": [{"var": "x"}, [1, 2, 3]]})
        );
    }

    #[test]
    fn test_method_call() {
        let result = compile("items.filter(x => x > 0)").unwrap();
        assert_eq!(
            result,
            json!({
                "filter": [
                    {"var": "items"},
                    {">": [{"var": ""}, 0]}
                ]
            })
        );
    }

    #[test]
    fn test_contains_method() {
        let result = compile("[1, 2, 3].contains(x)").unwrap();
        assert_eq!(
            result,
            json!({"in": [{"var": "x"}, [1, 2, 3]]})
        );
    }

    #[test]
    fn test_truthy_postfix() {
        let result = compile("x?").unwrap();
        assert_eq!(result, json!({"!!": {"var": "x"}}));
    }

    #[test]
    fn test_equality_synonyms() {
        // All should compile to ===
        let sources = ["a === b", "a is b", "a eq b", "a equals b"];
        for source in sources {
            let result = compile(source).unwrap();
            assert_eq!(
                result,
                json!({"===": [{"var": "a"}, {"var": "b"}]}),
                "Failed for: {}", source
            );
        }
    }

    #[test]
    fn test_nested_property() {
        let result = compile("user.address.city").unwrap();
        assert_eq!(result, json!({"var": "user.address.city"}));
    }

    #[test]
    fn test_null_coalesce() {
        let result = compile("a ?? b").unwrap();
        assert_eq!(
            result,
            json!({
                "if": [
                    {"!=": [{"var": "a"}, null]},
                    {"var": "a"},
                    {"var": "b"}
                ]
            })
        );
    }
}
