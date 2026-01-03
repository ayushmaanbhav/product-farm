//! JSON Logic operations registry and custom operation support
//!
//! This module provides infrastructure for registering and invoking
//! JSON Logic operations, including support for custom operations.

use crate::error::{JsonLogicError, JsonLogicResult};
use product_farm_core::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// A custom operation function type
pub type OperationFn = Arc<dyn Fn(&[Value]) -> JsonLogicResult<Value> + Send + Sync>;

/// Registry for custom JSON Logic operations
#[derive(Clone, Default)]
pub struct OperationRegistry {
    operations: HashMap<String, OperationFn>,
}

impl OperationRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
        }
    }

    /// Register a custom operation
    pub fn register<F>(&mut self, name: impl Into<String>, op: F)
    where
        F: Fn(&[Value]) -> JsonLogicResult<Value> + Send + Sync + 'static,
    {
        self.operations.insert(name.into(), Arc::new(op));
    }

    /// Get an operation by name
    pub fn get(&self, name: &str) -> Option<&OperationFn> {
        self.operations.get(name)
    }

    /// Check if an operation exists
    pub fn contains(&self, name: &str) -> bool {
        self.operations.contains_key(name)
    }

    /// List all registered operation names
    pub fn operation_names(&self) -> Vec<&str> {
        self.operations.keys().map(|s| s.as_str()).collect()
    }

    /// Invoke a custom operation
    pub fn invoke(&self, name: &str, args: &[Value]) -> JsonLogicResult<Value> {
        self.operations
            .get(name)
            .ok_or_else(|| JsonLogicError::UnknownOperation(name.to_string()))
            .and_then(|op| op(args))
    }
}

impl std::fmt::Debug for OperationRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationRegistry")
            .field("operations", &self.operations.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Built-in operation names
pub mod ops {
    // Comparison
    pub const EQ: &str = "==";
    pub const STRICT_EQ: &str = "===";
    pub const NE: &str = "!=";
    pub const STRICT_NE: &str = "!==";
    pub const LT: &str = "<";
    pub const LE: &str = "<=";
    pub const GT: &str = ">";
    pub const GE: &str = ">=";

    // Logical
    pub const NOT: &str = "!";
    pub const TO_BOOL: &str = "!!";
    pub const AND: &str = "and";
    pub const OR: &str = "or";

    // Conditional
    pub const IF: &str = "if";
    pub const TERNARY: &str = "?:";

    // Arithmetic
    pub const ADD: &str = "+";
    pub const SUB: &str = "-";
    pub const MUL: &str = "*";
    pub const DIV: &str = "/";
    pub const MOD: &str = "%";
    pub const MIN: &str = "min";
    pub const MAX: &str = "max";

    // String
    pub const CAT: &str = "cat";
    pub const SUBSTR: &str = "substr";

    // Array
    pub const MAP: &str = "map";
    pub const FILTER: &str = "filter";
    pub const REDUCE: &str = "reduce";
    pub const ALL: &str = "all";
    pub const SOME: &str = "some";
    pub const NONE: &str = "none";
    pub const MERGE: &str = "merge";
    pub const IN: &str = "in";

    // Data
    pub const VAR: &str = "var";
    pub const MISSING: &str = "missing";
    pub const MISSING_SOME: &str = "missing_some";

    // Debug
    pub const LOG: &str = "log";

    /// All built-in operation names
    pub const ALL_OPS: &[&str] = &[
        EQ, STRICT_EQ, NE, STRICT_NE, LT, LE, GT, GE,
        NOT, TO_BOOL, AND, OR,
        IF, TERNARY,
        ADD, SUB, MUL, DIV, MOD, MIN, MAX,
        CAT, SUBSTR,
        MAP, FILTER, REDUCE, ALL, SOME, NONE, MERGE, IN,
        VAR, MISSING, MISSING_SOME,
        LOG,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_operation() {
        let mut registry = OperationRegistry::new();

        // Register a custom "abs" operation
        registry.register("abs", |args: &[Value]| {
            if args.is_empty() {
                return Err(JsonLogicError::InvalidArgumentCount {
                    op: "abs".into(),
                    expected: "1".into(),
                    actual: 0,
                });
            }
            let num = args[0].to_number();
            Ok(Value::Float(num.abs()))
        });

        assert!(registry.contains("abs"));

        let result = registry.invoke("abs", &[Value::Float(-42.0)]).unwrap();
        assert_eq!(result.to_number(), 42.0);
    }

    #[test]
    fn test_unknown_operation() {
        let registry = OperationRegistry::new();
        let result = registry.invoke("unknown_op", &[]);
        assert!(result.is_err());
    }
}
