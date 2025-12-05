//! Natural Language to JSON Logic Translator
//!
//! This module provides utilities to help translate natural language
//! descriptions into JSON Logic expressions. The actual translation
//! is performed by an LLM, but this module provides:
//! - Context about available attributes and their types
//! - Examples of common patterns
//! - Validation of generated expressions

use crate::error::{AgentError, AgentResult};
use crate::tools::{CreateRuleOutput, GeneratedRule};
use crate::validator::RuleValidator;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Context for translating natural language to JSON Logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationContext {
    /// Available attributes with their types
    pub attributes: HashMap<String, AttributeContext>,
    /// Available enums with their values
    pub enums: HashMap<String, Vec<String>>,
    /// Example patterns for common rule types
    pub examples: Vec<RuleExample>,
}

/// Context about an attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeContext {
    pub path: String,
    pub datatype: String,
    pub description: Option<String>,
    pub is_input: bool,
    pub is_computed: bool,
    pub enum_values: Option<Vec<String>>,
}

/// An example rule pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleExample {
    pub description: String,
    pub natural_language: String,
    pub json_logic: Value,
    pub rule_type: String,
}

impl TranslationContext {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
            enums: HashMap::new(),
            examples: default_examples(),
        }
    }

    pub fn add_attribute(
        mut self,
        path: impl Into<String>,
        datatype: impl Into<String>,
        is_input: bool,
    ) -> Self {
        let path = path.into();
        self.attributes.insert(
            path.clone(),
            AttributeContext {
                path: path.clone(),
                datatype: datatype.into(),
                description: None,
                is_input,
                is_computed: !is_input,
                enum_values: None,
            },
        );
        self
    }

    pub fn add_enum(mut self, name: impl Into<String>, values: Vec<String>) -> Self {
        self.enums.insert(name.into(), values);
        self
    }

    /// Generate a system prompt for the LLM with context
    pub fn to_system_prompt(&self) -> String {
        let mut prompt = String::new();

        prompt.push_str("You are a JSON Logic expert. Convert natural language rule descriptions into valid JSON Logic expressions.\n\n");

        // Add attribute context
        if !self.attributes.is_empty() {
            prompt.push_str("## Available Attributes\n\n");
            for (path, attr) in &self.attributes {
                prompt.push_str(&format!(
                    "- `{}` ({}){}\n",
                    path,
                    attr.datatype,
                    if attr.is_computed { " [computed]" } else { "" }
                ));
            }
            prompt.push('\n');
        }

        // Add enum context
        if !self.enums.is_empty() {
            prompt.push_str("## Available Enums\n\n");
            for (name, values) in &self.enums {
                prompt.push_str(&format!("- `{}`: {}\n", name, values.join(", ")));
            }
            prompt.push('\n');
        }

        // Add examples
        prompt.push_str("## JSON Logic Examples\n\n");
        for example in &self.examples {
            prompt.push_str(&format!("### {}\n", example.description));
            prompt.push_str(&format!("Natural language: \"{}\"\n", example.natural_language));
            prompt.push_str(&format!(
                "JSON Logic:\n```json\n{}\n```\n\n",
                serde_json::to_string_pretty(&example.json_logic).unwrap_or_default()
            ));
        }

        prompt.push_str("## Important Rules\n\n");
        prompt.push_str("1. Use `{\"var\": \"path\"}` to access variables\n");
        prompt.push_str("2. Comparisons: ==, !=, >, >=, <, <=\n");
        prompt.push_str("3. Logic: and, or, !, if\n");
        prompt.push_str("4. Math: +, -, *, /, %\n");
        prompt.push_str("5. Arrays: in, map, filter, reduce, all, some, none\n");
        prompt.push_str("6. Always return valid JSON\n");

        prompt
    }
}

impl Default for TranslationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Default examples for common rule patterns
fn default_examples() -> Vec<RuleExample> {
    vec![
        RuleExample {
            description: "Simple comparison".to_string(),
            natural_language: "If age is greater than 60".to_string(),
            json_logic: json!({">": [{"var": "age"}, 60]}),
            rule_type: "VALIDATION".to_string(),
        },
        RuleExample {
            description: "Conditional calculation".to_string(),
            natural_language: "If age > 60, multiply base_rate by 1.2, otherwise use base_rate".to_string(),
            json_logic: json!({
                "if": [
                    {">": [{"var": "age"}, 60]},
                    {"*": [{"var": "base_rate"}, 1.2]},
                    {"var": "base_rate"}
                ]
            }),
            rule_type: "CALCULATION".to_string(),
        },
        RuleExample {
            description: "Multiple conditions (AND)".to_string(),
            natural_language: "Check if age is between 18 and 65".to_string(),
            json_logic: json!({
                "and": [
                    {">=": [{"var": "age"}, 18]},
                    {"<=": [{"var": "age"}, 65]}
                ]
            }),
            rule_type: "VALIDATION".to_string(),
        },
        RuleExample {
            description: "Multiple conditions (OR)".to_string(),
            natural_language: "Check if status is 'ACTIVE' or 'PENDING'".to_string(),
            json_logic: json!({
                "or": [
                    {"==": [{"var": "status"}, "ACTIVE"]},
                    {"==": [{"var": "status"}, "PENDING"]}
                ]
            }),
            rule_type: "VALIDATION".to_string(),
        },
        RuleExample {
            description: "Check membership".to_string(),
            natural_language: "Check if category is in the list of allowed categories".to_string(),
            json_logic: json!({
                "in": [{"var": "category"}, ["A", "B", "C"]]
            }),
            rule_type: "VALIDATION".to_string(),
        },
        RuleExample {
            description: "Chained if-else".to_string(),
            natural_language: "If RSI < 30, return 'BUY'; if RSI > 70, return 'SELL'; otherwise 'HOLD'".to_string(),
            json_logic: json!({
                "if": [
                    {"<": [{"var": "rsi"}, 30]},
                    "BUY",
                    {">": [{"var": "rsi"}, 70]},
                    "SELL",
                    "HOLD"
                ]
            }),
            rule_type: "SIGNAL".to_string(),
        },
        RuleExample {
            description: "Percentage calculation".to_string(),
            natural_language: "Calculate 5% stop loss from entry price".to_string(),
            json_logic: json!({
                "*": [{"var": "entry_price"}, 0.95]
            }),
            rule_type: "CALCULATION".to_string(),
        },
        RuleExample {
            description: "Complex trading rule".to_string(),
            natural_language: "If price drops 5% below entry, trigger stop loss".to_string(),
            json_logic: json!({
                "if": [
                    {"<": [
                        {"var": "current_price"},
                        {"*": [{"var": "entry_price"}, 0.95]}
                    ]},
                    "SELL",
                    "HOLD"
                ]
            }),
            rule_type: "EXIT".to_string(),
        },
    ]
}

/// Translator that helps convert NL to JSON Logic
pub struct RuleTranslator {
    context: TranslationContext,
    validator: RuleValidator,
}

impl RuleTranslator {
    pub fn new(context: TranslationContext) -> Self {
        Self {
            context,
            validator: RuleValidator::new(),
        }
    }

    /// Parse and validate a JSON Logic expression from string
    pub fn parse_and_validate(
        &self,
        json_str: &str,
        input_attributes: &[String],
        output_attributes: &[String],
    ) -> AgentResult<CreateRuleOutput> {
        // Parse the JSON
        let expression: Value = serde_json::from_str(json_str)
            .map_err(|e| AgentError::JsonLogicParseError(e.to_string()))?;

        self.validate_expression(&expression, input_attributes, output_attributes)
    }

    /// Validate a JSON Logic expression
    pub fn validate_expression(
        &self,
        expression: &Value,
        input_attributes: &[String],
        output_attributes: &[String],
    ) -> AgentResult<CreateRuleOutput> {
        // Validate the expression
        let validation = self.validator.validate(expression, input_attributes, output_attributes)?;

        let mut warnings = Vec::new();
        for w in &validation.warnings {
            warnings.push(w.message.clone());
        }

        if !validation.is_valid {
            return Err(AgentError::ValidationError(
                validation.errors.iter()
                    .map(|e| e.message.clone())
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }

        // Generate display expression
        let display_expression = self.generate_display_expression(expression);

        Ok(CreateRuleOutput {
            rule: GeneratedRule {
                rule_type: "CALCULATION".to_string(), // Default, should be overridden
                expression: expression.clone(),
                display_expression,
                description: String::new(), // Should be filled by caller
                input_attributes: input_attributes.to_vec(),
                output_attributes: output_attributes.to_vec(),
            },
            explanation: "Rule validated successfully".to_string(),
            warnings,
        })
    }

    /// Generate a human-readable display expression from JSON Logic
    pub fn generate_display_expression(&self, expr: &Value) -> String {
        match expr {
            Value::Object(map) if map.len() == 1 => {
                let (op, args) = map.iter().next().unwrap();
                self.format_operation(op, args)
            }
            Value::Number(n) => n.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter()
                    .map(|v| self.generate_display_expression(v))
                    .collect();
                format!("[{}]", items.join(", "))
            }
            _ => expr.to_string(),
        }
    }

    fn format_operation(&self, op: &str, args: &Value) -> String {
        match op {
            "var" => {
                if let Value::String(path) = args {
                    path.clone()
                } else if let Value::Array(arr) = args {
                    arr.first()
                        .and_then(|v| v.as_str())
                        .unwrap_or("?")
                        .to_string()
                } else {
                    "?".to_string()
                }
            }
            "if" => {
                if let Value::Array(arr) = args {
                    if arr.len() >= 3 {
                        format!(
                            "IF {} THEN {} ELSE {}",
                            self.generate_display_expression(&arr[0]),
                            self.generate_display_expression(&arr[1]),
                            self.generate_display_expression(&arr[2])
                        )
                    } else if arr.len() == 2 {
                        format!(
                            "IF {} THEN {}",
                            self.generate_display_expression(&arr[0]),
                            self.generate_display_expression(&arr[1])
                        )
                    } else {
                        "IF ?".to_string()
                    }
                } else {
                    "IF ?".to_string()
                }
            }
            "and" | "or" => {
                if let Value::Array(arr) = args {
                    let parts: Vec<String> = arr.iter()
                        .map(|v| self.generate_display_expression(v))
                        .collect();
                    format!("({})", parts.join(&format!(" {} ", op.to_uppercase())))
                } else {
                    format!("{} ?", op.to_uppercase())
                }
            }
            "!" | "not" => {
                format!("NOT {}", self.generate_display_expression(args))
            }
            ">" | ">=" | "<" | "<=" | "==" | "!=" => {
                if let Value::Array(arr) = args {
                    if arr.len() >= 2 {
                        format!(
                            "{} {} {}",
                            self.generate_display_expression(&arr[0]),
                            op,
                            self.generate_display_expression(&arr[1])
                        )
                    } else {
                        format!("? {} ?", op)
                    }
                } else {
                    format!("? {} ?", op)
                }
            }
            "+" | "-" | "*" | "/" | "%" => {
                if let Value::Array(arr) = args {
                    let parts: Vec<String> = arr.iter()
                        .map(|v| self.generate_display_expression(v))
                        .collect();
                    format!("({})", parts.join(&format!(" {} ", op)))
                } else {
                    format!("{} ?", op)
                }
            }
            "in" => {
                if let Value::Array(arr) = args {
                    if arr.len() >= 2 {
                        format!(
                            "{} IN {}",
                            self.generate_display_expression(&arr[0]),
                            self.generate_display_expression(&arr[1])
                        )
                    } else {
                        "? IN ?".to_string()
                    }
                } else {
                    "? IN ?".to_string()
                }
            }
            _ => format!("{}({})", op, self.generate_display_expression(args)),
        }
    }

    /// Get the translation context for building prompts
    pub fn get_context(&self) -> &TranslationContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_context_prompt() {
        let ctx = TranslationContext::new()
            .add_attribute("age", "number", true)
            .add_attribute("premium", "number", false)
            .add_enum("status", vec!["ACTIVE".into(), "INACTIVE".into()]);

        let prompt = ctx.to_system_prompt();
        assert!(prompt.contains("age"));
        assert!(prompt.contains("premium"));
        assert!(prompt.contains("status"));
        assert!(prompt.contains("ACTIVE"));
    }

    #[test]
    fn test_parse_and_validate() {
        let ctx = TranslationContext::new();
        let translator = RuleTranslator::new(ctx);

        let json_str = r#"{"if": [{">": [{"var": "age"}, 60]}, 1.2, 1.0]}"#;
        let result = translator.parse_and_validate(
            json_str,
            &["age".into()],
            &["loading".into()],
        );

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.rule.display_expression.contains("IF"));
    }

    #[test]
    fn test_display_expression_generation() {
        let ctx = TranslationContext::new();
        let translator = RuleTranslator::new(ctx);

        let expr = json!({
            "if": [
                {">": [{"var": "age"}, 60]},
                {"*": [{"var": "base"}, 1.2]},
                {"var": "base"}
            ]
        });

        let display = translator.generate_display_expression(&expr);
        assert!(display.contains("IF"));
        assert!(display.contains("age"));
        assert!(display.contains("base"));
    }
}
