//! FarmScript - Human-friendly expression language for Product-FARM
//!
//! FarmScript compiles to JSON Logic for execution, providing a natural
//! syntax for writing business rules and expressions.
//!
//! # Example
//! ```text
//! // FarmScript
//! alert_acknowledged and time_since_alert_secs < 120
//!
//! // Compiles to JSON Logic
//! {"and": [{"var": "alert_acknowledged"}, {"<": [{"var": "time_since_alert_secs"}, 120]}]}
//! ```
//!
//! # Features
//!
//! - Natural language operators: `is`, `isnt`, `equals`, `same_as`
//! - Path-style variables: `/users/count`
//! - Safe division: `a /? b` (returns 0), `a /! b` (returns null)
//! - Template strings: `` `Hello {name}!` ``
//! - SQL-like queries: `from items where x > 0 select x * 2`
//! - Method chaining: `items.filter(x => x > 0).map(x => x * 2)`
//! - Truthy check: `x?`
//! - Null coalescing: `a ?? b`

mod token;
mod lexer;
mod ast;
mod parser;
mod compiler;
mod builtins;

pub use token::{Token, TokenKind, Span};
pub use lexer::Lexer;
pub use ast::{Expr, BinaryOp, UnaryOp, Literal};
pub use parser::{Parser, ParseError};
pub use compiler::{Compiler, CompileError, CompileOptions};
pub use builtins::{BuiltinFn, BUILTINS, FnCategory, get_builtin};

use serde_json::Value as JsonValue;

/// Compile FarmScript source to JSON Logic
pub fn compile(source: &str) -> Result<JsonValue, CompileError> {
    compile_with_options(source, &CompileOptions::default())
}

/// Compile FarmScript source to JSON Logic with options
pub fn compile_with_options(source: &str, options: &CompileOptions) -> Result<JsonValue, CompileError> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse()?;
    let compiler = Compiler::new(options.clone());
    compiler.compile(&ast)
}

/// Parse FarmScript source to AST (for inspection/transformation)
pub fn parse(source: &str) -> Result<Expr, ParseError> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    parser.parse()
}

/// Tokenize FarmScript source (for debugging)
pub fn tokenize(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    lexer.collect_tokens()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_comparison() {
        let result = compile("x < 10").unwrap();
        assert_eq!(result, serde_json::json!({"<": [{"var": "x"}, 10]}));
    }

    #[test]
    fn test_and_expression() {
        let result = compile("a and b").unwrap();
        assert_eq!(result, serde_json::json!({"and": [{"var": "a"}, {"var": "b"}]}));
    }

    #[test]
    fn test_complex_expression() {
        let result = compile("alert_acknowledged and time_since_alert_secs < 120").unwrap();
        assert_eq!(
            result,
            serde_json::json!({
                "and": [
                    {"var": "alert_acknowledged"},
                    {"<": [{"var": "time_since_alert_secs"}, 120]}
                ]
            })
        );
    }

    #[test]
    fn test_clamp_expression() {
        let result = compile("clamp(0, 100, max_possible_score * (positive_signals - negative_signals * 0.5))").unwrap();
        // Should produce: max(0, min(100, ...))
        let json_str = serde_json::to_string(&result).unwrap();
        assert!(json_str.contains("max"));
        assert!(json_str.contains("min"));
    }

    #[test]
    fn test_if_chain() {
        let source = r#"
            if critical_failures > 0 then "strong_no_hire"
            else if overall_score >= 85 then "strong_hire"
            else if overall_score >= 65 then "hire"
            else if overall_score >= 45 then "no_hire"
            else "strong_no_hire"
        "#;
        let result = compile(source).unwrap();
        let json_str = serde_json::to_string(&result).unwrap();
        assert!(json_str.contains("if"));
        assert!(json_str.contains("strong_hire"));
    }

    #[test]
    fn test_path_variable() {
        let result = compile("/users/active/count").unwrap();
        assert_eq!(result, serde_json::json!({"var": "users.active.count"}));
    }

    #[test]
    fn test_safe_division() {
        let result = compile("revenue /? expenses").unwrap();
        // Should produce: if expenses == 0 then 0 else revenue / expenses
        let json_str = serde_json::to_string(&result).unwrap();
        assert!(json_str.contains("if"));
        assert!(json_str.contains("/"));
    }

    #[test]
    fn test_method_chain() {
        let result = compile("items.filter(x => x > 0).map(x => x * 2)").unwrap();
        let json_str = serde_json::to_string(&result).unwrap();
        assert!(json_str.contains("filter"));
        assert!(json_str.contains("map"));
    }

    #[test]
    fn test_equality_synonyms() {
        // All should compile to strict equality
        let sources = ["a === b", "a is b", "a eq b", "a equals b", "a same_as b"];
        for source in sources {
            let result = compile(source).unwrap();
            assert_eq!(
                result,
                serde_json::json!({"===": [{"var": "a"}, {"var": "b"}]}),
                "Failed for: {}", source
            );
        }
    }

    #[test]
    fn test_in_operator() {
        let result = compile("x in [1, 2, 3]").unwrap();
        assert_eq!(result, serde_json::json!({"in": [{"var": "x"}, [1, 2, 3]]}));
    }

    #[test]
    fn test_contains_method() {
        let result = compile("[1, 2, 3].contains(x)").unwrap();
        assert_eq!(result, serde_json::json!({"in": [{"var": "x"}, [1, 2, 3]]}));
    }

    #[test]
    fn test_truthy_operator() {
        let result = compile("x?").unwrap();
        assert_eq!(result, serde_json::json!({"!!": {"var": "x"}}));
    }

    #[test]
    fn test_null_coalesce() {
        let result = compile("a ?? b").unwrap();
        // if a != null then a else b
        let json_str = serde_json::to_string(&result).unwrap();
        assert!(json_str.contains("if"));
        assert!(json_str.contains("!="));
    }

    #[test]
    fn test_db_outage_detect_quick_response() {
        // Real expression from fixtures
        let source = "alert_acknowledged and time_since_alert_secs < 120";
        let result = compile(source).unwrap();
        assert_eq!(
            result,
            serde_json::json!({
                "and": [
                    {"var": "alert_acknowledged"},
                    {"<": [{"var": "time_since_alert_secs"}, 120]}
                ]
            })
        );
    }

    #[test]
    fn test_db_outage_compute_signal_score() {
        // Real expression from fixtures (simplified)
        let source = "clamp(0, 100, max_possible_score * (positive_signals - negative_signals * 0.5))";
        let result = compile(source).unwrap();
        // Verify structure
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("max")); // clamp uses max(..., min(...))
    }

    #[test]
    fn test_db_outage_compute_recommendation() {
        // Real expression from fixtures
        let source = r#"
            if critical_failures > 0 then "strong_no_hire"
            else if overall_score >= 85 then "strong_hire"
            else if overall_score >= 65 then "hire"
            else if overall_score >= 45 then "no_hire"
            else "strong_no_hire"
        "#;
        let result = compile(source).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("if"));
    }
}
