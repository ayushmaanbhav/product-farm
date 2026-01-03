//! Built-in functions for FarmScript
//!
//! Defines the standard library of functions available in FarmScript expressions.

use std::collections::HashMap;

/// Description of a built-in function
#[derive(Debug, Clone)]
pub struct BuiltinFn {
    /// Function name
    pub name: &'static str,
    /// Description
    pub description: &'static str,
    /// Number of arguments (min, max). None for max means variadic.
    pub arity: (usize, Option<usize>),
    /// Argument names (for documentation)
    pub args: &'static [&'static str],
    /// JSON Logic operator (if direct mapping)
    pub json_logic_op: Option<&'static str>,
    /// Category
    pub category: FnCategory,
}

/// Function category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FnCategory {
    Math,
    Aggregation,
    String,
    Array,
    Logic,
    Data,
    Debug,
}

/// All built-in functions
pub static BUILTINS: &[BuiltinFn] = &[
    // ========================
    // Math functions
    // ========================
    BuiltinFn {
        name: "abs",
        description: "Absolute value",
        arity: (1, Some(1)),
        args: &["x"],
        json_logic_op: None, // Emulated with if
        category: FnCategory::Math,
    },
    BuiltinFn {
        name: "round",
        description: "Round to nearest integer or specified decimals",
        arity: (1, Some(2)),
        args: &["x", "decimals?"],
        json_logic_op: Some("round"),
        category: FnCategory::Math,
    },
    BuiltinFn {
        name: "floor",
        description: "Round down to integer",
        arity: (1, Some(1)),
        args: &["x"],
        json_logic_op: Some("floor"),
        category: FnCategory::Math,
    },
    BuiltinFn {
        name: "ceil",
        description: "Round up to integer",
        arity: (1, Some(1)),
        args: &["x"],
        json_logic_op: Some("ceil"),
        category: FnCategory::Math,
    },
    BuiltinFn {
        name: "pow",
        description: "Raise to power (also: x ^ y)",
        arity: (2, Some(2)),
        args: &["base", "exponent"],
        json_logic_op: Some("pow"),
        category: FnCategory::Math,
    },
    BuiltinFn {
        name: "sqrt",
        description: "Square root",
        arity: (1, Some(1)),
        args: &["x"],
        json_logic_op: Some("sqrt"),
        category: FnCategory::Math,
    },
    BuiltinFn {
        name: "clamp",
        description: "Clamp value between min and max",
        arity: (3, Some(3)),
        args: &["min", "max", "value"],
        json_logic_op: None, // Emulated with max(min, min(max, value))
        category: FnCategory::Math,
    },
    BuiltinFn {
        name: "safe_div",
        description: "Safe division, returns default if divisor is 0",
        arity: (2, Some(3)),
        args: &["a", "b", "default?"],
        json_logic_op: None, // Emulated with if
        category: FnCategory::Math,
    },

    // ========================
    // Aggregation functions
    // ========================
    BuiltinFn {
        name: "min",
        description: "Minimum of values",
        arity: (1, None),
        args: &["values..."],
        json_logic_op: Some("min"),
        category: FnCategory::Aggregation,
    },
    BuiltinFn {
        name: "max",
        description: "Maximum of values",
        arity: (1, None),
        args: &["values..."],
        json_logic_op: Some("max"),
        category: FnCategory::Aggregation,
    },
    BuiltinFn {
        name: "sum",
        description: "Sum of values",
        arity: (1, None),
        args: &["values..."],
        json_logic_op: Some("+"),
        category: FnCategory::Aggregation,
    },
    BuiltinFn {
        name: "count",
        description: "Count elements in array",
        arity: (1, Some(1)),
        args: &["array"],
        json_logic_op: Some("count"),
        category: FnCategory::Aggregation,
    },

    // ========================
    // String functions
    // ========================
    BuiltinFn {
        name: "cat",
        description: "Concatenate strings",
        arity: (1, None),
        args: &["values..."],
        json_logic_op: Some("cat"),
        category: FnCategory::String,
    },
    BuiltinFn {
        name: "substr",
        description: "Extract substring",
        arity: (2, Some(3)),
        args: &["string", "start", "length?"],
        json_logic_op: Some("substr"),
        category: FnCategory::String,
    },
    BuiltinFn {
        name: "len",
        description: "Length of string or array",
        arity: (1, Some(1)),
        args: &["value"],
        json_logic_op: Some("len"),
        category: FnCategory::String,
    },
    BuiltinFn {
        name: "length",
        description: "Alias for len",
        arity: (1, Some(1)),
        args: &["value"],
        json_logic_op: Some("len"),
        category: FnCategory::String,
    },

    // ========================
    // Array functions
    // ========================
    BuiltinFn {
        name: "map",
        description: "Transform each element",
        arity: (2, Some(2)),
        args: &["array", "mapper"],
        json_logic_op: Some("map"),
        category: FnCategory::Array,
    },
    BuiltinFn {
        name: "filter",
        description: "Filter elements by predicate",
        arity: (2, Some(2)),
        args: &["array", "predicate"],
        json_logic_op: Some("filter"),
        category: FnCategory::Array,
    },
    BuiltinFn {
        name: "reduce",
        description: "Reduce array to single value",
        arity: (3, Some(3)),
        args: &["array", "reducer", "initial"],
        json_logic_op: Some("reduce"),
        category: FnCategory::Array,
    },
    BuiltinFn {
        name: "all",
        description: "Check if all elements satisfy predicate",
        arity: (2, Some(2)),
        args: &["array", "predicate"],
        json_logic_op: Some("all"),
        category: FnCategory::Array,
    },
    BuiltinFn {
        name: "some",
        description: "Check if any element satisfies predicate",
        arity: (2, Some(2)),
        args: &["array", "predicate"],
        json_logic_op: Some("some"),
        category: FnCategory::Array,
    },
    BuiltinFn {
        name: "none",
        description: "Check if no element satisfies predicate",
        arity: (2, Some(2)),
        args: &["array", "predicate"],
        json_logic_op: Some("none"),
        category: FnCategory::Array,
    },
    BuiltinFn {
        name: "merge",
        description: "Merge arrays",
        arity: (1, None),
        args: &["arrays..."],
        json_logic_op: Some("merge"),
        category: FnCategory::Array,
    },
    BuiltinFn {
        name: "contains",
        description: "Check if array/string contains value",
        arity: (2, Some(2)),
        args: &["haystack", "needle"],
        json_logic_op: None, // Reversed to "in"
        category: FnCategory::Array,
    },

    // ========================
    // Data functions
    // ========================
    BuiltinFn {
        name: "missing",
        description: "List of missing variables",
        arity: (1, None),
        args: &["vars..."],
        json_logic_op: Some("missing"),
        category: FnCategory::Data,
    },
    BuiltinFn {
        name: "missing_some",
        description: "Missing variables if fewer than required",
        arity: (2, Some(2)),
        args: &["required_count", "vars"],
        json_logic_op: Some("missing_some"),
        category: FnCategory::Data,
    },

    // ========================
    // Debug functions
    // ========================
    BuiltinFn {
        name: "log",
        description: "Log value for debugging",
        arity: (1, Some(1)),
        args: &["value"],
        json_logic_op: Some("log"),
        category: FnCategory::Debug,
    },
];

/// Get builtin by name
pub fn get_builtin(name: &str) -> Option<&'static BuiltinFn> {
    BUILTINS.iter().find(|b| b.name == name)
}

/// Get all builtins in a category
#[allow(dead_code)]
pub fn builtins_by_category(category: FnCategory) -> Vec<&'static BuiltinFn> {
    BUILTINS.iter().filter(|b| b.category == category).collect()
}

/// Get a map of all builtins by name
#[allow(dead_code)]
pub fn builtins_map() -> HashMap<&'static str, &'static BuiltinFn> {
    BUILTINS.iter().map(|b| (b.name, b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_builtin() {
        let clamp = get_builtin("clamp");
        assert!(clamp.is_some());
        assert_eq!(clamp.unwrap().arity, (3, Some(3)));
    }

    #[test]
    fn test_builtins_by_category() {
        let math_fns = builtins_by_category(FnCategory::Math);
        assert!(math_fns.len() >= 5);
        assert!(math_fns.iter().any(|f| f.name == "abs"));
    }

    #[test]
    fn test_all_builtins_have_descriptions() {
        for builtin in BUILTINS {
            assert!(!builtin.description.is_empty(), "Missing description for {}", builtin.name);
        }
    }
}
