//! Abstract Syntax Tree for JSON Logic expressions
//!
//! This AST is optimized for fast interpretation and compilation to bytecode.

use product_farm_core::Value;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A parsed JSON Logic expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Literal value
    Literal(Value),

    /// Variable access: {"var": "path.to.value"} or {"var": ["path", default]}
    Var(VarExpr),

    // Comparison operations
    /// Equality: {"==": [a, b]}
    Eq(Box<Expr>, Box<Expr>),
    /// Strict equality: {"===": [a, b]}
    StrictEq(Box<Expr>, Box<Expr>),
    /// Not equal: {"!=": [a, b]}
    Ne(Box<Expr>, Box<Expr>),
    /// Strict not equal: {"!==": [a, b]}
    StrictNe(Box<Expr>, Box<Expr>),
    /// Less than: {"<": [a, b]} or {"<": [a, b, c]} for a < b < c
    Lt(Vec<Expr>),
    /// Less than or equal: {"<=": [a, b]}
    Le(Vec<Expr>),
    /// Greater than: {">": [a, b]}
    Gt(Vec<Expr>),
    /// Greater than or equal: {">=": [a, b]}
    Ge(Vec<Expr>),

    // Logical operations
    /// Logical NOT: {"!": [a]}
    Not(Box<Expr>),
    /// Double negation (to boolean): {"!!": [a]}
    ToBool(Box<Expr>),
    /// Logical AND: {"and": [a, b, ...]}
    And(Vec<Expr>),
    /// Logical OR: {"or": [a, b, ...]}
    Or(Vec<Expr>),

    // Conditional
    /// If-then-else: {"if": [cond, then, else]} or {"if": [cond1, then1, cond2, then2, ..., else]}
    If(Vec<Expr>),
    /// Ternary: {"?:": [cond, then, else]}
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),

    // Arithmetic operations
    /// Addition: {"+": [a, b, ...]}
    Add(Vec<Expr>),
    /// Subtraction: {"-": [a, b]} or {"-": [a]} for negation
    Sub(Vec<Expr>),
    /// Multiplication: {"*": [a, b, ...]}
    Mul(Vec<Expr>),
    /// Division: {"/": [a, b]}
    Div(Box<Expr>, Box<Expr>),
    /// Modulo: {"%": [a, b]}
    Mod(Box<Expr>, Box<Expr>),
    /// Minimum: {"min": [a, b, ...]}
    Min(Vec<Expr>),
    /// Maximum: {"max": [a, b, ...]}
    Max(Vec<Expr>),

    // String operations
    /// String concatenation: {"cat": [a, b, ...]}
    Cat(Vec<Expr>),
    /// Substring: {"substr": [str, start, length?]}
    Substr(Box<Expr>, Box<Expr>, Option<Box<Expr>>),

    // Array operations
    /// Map: {"map": [[array], {"var": ""}, expr]}
    Map(Box<Expr>, Box<Expr>),
    /// Filter: {"filter": [[array], {"var": ""}, expr]}
    Filter(Box<Expr>, Box<Expr>),
    /// Reduce: {"reduce": [[array], expr, initial]}
    Reduce(Box<Expr>, Box<Expr>, Box<Expr>),
    /// All: {"all": [[array], expr]}
    All(Box<Expr>, Box<Expr>),
    /// Some: {"some": [[array], expr]}
    Some(Box<Expr>, Box<Expr>),
    /// None: {"none": [[array], expr]}
    None(Box<Expr>, Box<Expr>),
    /// Merge arrays: {"merge": [[a], [b], ...]}
    Merge(Vec<Expr>),
    /// In array: {"in": [value, array]}
    In(Box<Expr>, Box<Expr>),

    // Data access
    /// Missing: {"missing": [keys...]}
    Missing(Vec<Expr>),
    /// Missing some: {"missing_some": [min_required, [keys...]]}
    MissingSome(Box<Expr>, Vec<Expr>),

    // Logging (for debugging)
    /// Log: {"log": [value]}
    Log(Box<Expr>),
}

/// Variable access expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VarExpr {
    /// Variable path (e.g., "user.age" or "" for entire data)
    pub path: String,
    /// Default value if variable is not found
    pub default: Option<Value>,
}

impl VarExpr {
    /// Create a new variable expression
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            default: None,
        }
    }

    /// Create with default value
    pub fn with_default(path: impl Into<String>, default: Value) -> Self {
        Self {
            path: path.into(),
            default: Some(default),
        }
    }

    /// Parse the path into segments
    pub fn path_segments(&self) -> Vec<&str> {
        if self.path.is_empty() {
            Vec::new()
        } else {
            self.path.split('.').collect()
        }
    }
}

impl Expr {
    /// Check if this is a literal expression
    pub fn is_literal(&self) -> bool {
        matches!(self, Expr::Literal(_))
    }

    /// Check if this is a variable expression
    pub fn is_var(&self) -> bool {
        matches!(self, Expr::Var(_))
    }

    /// Get the literal value if this is a literal
    pub fn as_literal(&self) -> Option<&Value> {
        match self {
            Expr::Literal(v) => Some(v),
            _ => None,
        }
    }

    /// Create a literal expression
    pub fn literal(value: impl Into<Value>) -> Self {
        Expr::Literal(value.into())
    }

    /// Create a variable expression
    pub fn var(path: impl Into<String>) -> Self {
        Expr::Var(VarExpr::new(path))
    }

    /// Collect all variable paths referenced in this expression
    pub fn collect_variables(&self) -> Vec<&str> {
        let mut vars = Vec::new();
        self.collect_variables_into(&mut vars);
        vars
    }

    fn collect_variables_into<'a>(&'a self, vars: &mut Vec<&'a str>) {
        match self {
            Expr::Var(v) => {
                if !v.path.is_empty() {
                    vars.push(&v.path);
                }
            }
            Expr::Literal(_) => {}
            Expr::Eq(a, b)
            | Expr::StrictEq(a, b)
            | Expr::Ne(a, b)
            | Expr::StrictNe(a, b)
            | Expr::Div(a, b)
            | Expr::Mod(a, b)
            | Expr::In(a, b) => {
                a.collect_variables_into(vars);
                b.collect_variables_into(vars);
            }
            Expr::Lt(exprs)
            | Expr::Le(exprs)
            | Expr::Gt(exprs)
            | Expr::Ge(exprs)
            | Expr::And(exprs)
            | Expr::Or(exprs)
            | Expr::If(exprs)
            | Expr::Add(exprs)
            | Expr::Sub(exprs)
            | Expr::Mul(exprs)
            | Expr::Min(exprs)
            | Expr::Max(exprs)
            | Expr::Cat(exprs)
            | Expr::Merge(exprs)
            | Expr::Missing(exprs) => {
                for expr in exprs {
                    expr.collect_variables_into(vars);
                }
            }
            Expr::Not(a) | Expr::ToBool(a) | Expr::Log(a) => {
                a.collect_variables_into(vars);
            }
            Expr::Ternary(cond, then, else_) => {
                cond.collect_variables_into(vars);
                then.collect_variables_into(vars);
                else_.collect_variables_into(vars);
            }
            Expr::Substr(s, start, len) => {
                s.collect_variables_into(vars);
                start.collect_variables_into(vars);
                if let Some(l) = len {
                    l.collect_variables_into(vars);
                }
            }
            Expr::Map(arr, expr)
            | Expr::Filter(arr, expr)
            | Expr::All(arr, expr)
            | Expr::Some(arr, expr)
            | Expr::None(arr, expr) => {
                arr.collect_variables_into(vars);
                expr.collect_variables_into(vars);
            }
            Expr::Reduce(arr, expr, init) => {
                arr.collect_variables_into(vars);
                expr.collect_variables_into(vars);
                init.collect_variables_into(vars);
            }
            Expr::MissingSome(min, keys) => {
                min.collect_variables_into(vars);
                for key in keys {
                    key.collect_variables_into(vars);
                }
            }
        }
    }

    /// Count the total number of nodes in this expression tree
    pub fn node_count(&self) -> usize {
        match self {
            Expr::Literal(_) | Expr::Var(_) => 1,
            Expr::Not(a) | Expr::ToBool(a) | Expr::Log(a) => 1 + a.node_count(),
            Expr::Eq(a, b)
            | Expr::StrictEq(a, b)
            | Expr::Ne(a, b)
            | Expr::StrictNe(a, b)
            | Expr::Div(a, b)
            | Expr::Mod(a, b)
            | Expr::In(a, b) => 1 + a.node_count() + b.node_count(),
            Expr::Lt(v)
            | Expr::Le(v)
            | Expr::Gt(v)
            | Expr::Ge(v)
            | Expr::And(v)
            | Expr::Or(v)
            | Expr::If(v)
            | Expr::Add(v)
            | Expr::Sub(v)
            | Expr::Mul(v)
            | Expr::Min(v)
            | Expr::Max(v)
            | Expr::Cat(v)
            | Expr::Merge(v)
            | Expr::Missing(v) => 1 + v.iter().map(|e| e.node_count()).sum::<usize>(),
            Expr::Ternary(a, b, c) => 1 + a.node_count() + b.node_count() + c.node_count(),
            Expr::Substr(a, b, c) => {
                1 + a.node_count()
                    + b.node_count()
                    + c.as_ref().map(|e| e.node_count()).unwrap_or(0)
            }
            Expr::Map(a, b)
            | Expr::Filter(a, b)
            | Expr::All(a, b)
            | Expr::Some(a, b)
            | Expr::None(a, b) => 1 + a.node_count() + b.node_count(),
            Expr::Reduce(a, b, c) => 1 + a.node_count() + b.node_count() + c.node_count(),
            Expr::MissingSome(a, v) => {
                1 + a.node_count() + v.iter().map(|e| e.node_count()).sum::<usize>()
            }
        }
    }
}

/// A compiled expression with cached metadata
#[derive(Debug, Clone)]
pub struct CompiledExpr {
    /// The parsed expression tree
    pub expr: Arc<Expr>,
    /// Variables referenced in this expression (for fast context building)
    pub variables: Vec<String>,
    /// Total node count (for complexity estimation)
    pub node_count: usize,
}

impl CompiledExpr {
    /// Create a new compiled expression
    pub fn new(expr: Expr) -> Self {
        let variables = expr.collect_variables().iter().map(|s| s.to_string()).collect();
        let node_count = expr.node_count();
        Self {
            expr: Arc::new(expr),
            variables,
            node_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_path_segments() {
        let var = VarExpr::new("user.profile.age");
        assert_eq!(var.path_segments(), vec!["user", "profile", "age"]);

        let empty = VarExpr::new("");
        assert!(empty.path_segments().is_empty());
    }

    #[test]
    fn test_collect_variables() {
        // {"if": [{">": [{"var": "age"}, 60]}, {"*": [{"var": "base"}, 1.2]}, {"var": "base"}]}
        let expr = Expr::If(vec![
            Expr::Gt(vec![Expr::var("age"), Expr::literal(60i64)]),
            Expr::Mul(vec![Expr::var("base"), Expr::literal(1.2)]),
            Expr::var("base"),
        ]);

        let vars = expr.collect_variables();
        assert!(vars.contains(&"age"));
        assert!(vars.contains(&"base"));
    }

    #[test]
    fn test_node_count() {
        let simple = Expr::literal(42i64);
        assert_eq!(simple.node_count(), 1);

        let complex = Expr::Add(vec![
            Expr::var("a"),
            Expr::Mul(vec![Expr::var("b"), Expr::literal(2i64)]),
        ]);
        assert_eq!(complex.node_count(), 5); // Add + var(a) + Mul + var(b) + literal(2)
    }
}
