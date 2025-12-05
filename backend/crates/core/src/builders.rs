//! Rule builder helpers for common patterns
//!
//! Provides convenient builders for common rule types:
//! - Arithmetic rules (calculations)
//! - Conditional rules (if/then/else)
//! - Comparison rules (thresholds, ranges)
//! - Aggregation rules (sum, average, etc.)

use crate::Rule;
use serde_json::{json, Value};

/// Builder for creating arithmetic calculation rules
pub struct CalcRuleBuilder {
    product_id: String,
    inputs: Vec<String>,
    output: String,
    description: Option<String>,
}

impl CalcRuleBuilder {
    pub fn new(product_id: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            product_id: product_id.into(),
            inputs: Vec::new(),
            output: output.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Create a multiplication rule: output = a * b
    pub fn multiply(mut self, a: &str, b: &str) -> Rule {
        self.inputs = vec![a.to_string(), b.to_string()];
        self.build(json!({"*": [{"var": a}, {"var": b}]}))
    }

    /// Create a division rule: output = a / b
    pub fn divide(mut self, a: &str, b: &str) -> Rule {
        self.inputs = vec![a.to_string(), b.to_string()];
        self.build(json!({"/": [{"var": a}, {"var": b}]}))
    }

    /// Create an addition rule: output = a + b
    pub fn add(mut self, a: &str, b: &str) -> Rule {
        self.inputs = vec![a.to_string(), b.to_string()];
        self.build(json!({"+": [{"var": a}, {"var": b}]}))
    }

    /// Create a subtraction rule: output = a - b
    pub fn subtract(mut self, a: &str, b: &str) -> Rule {
        self.inputs = vec![a.to_string(), b.to_string()];
        self.build(json!({"-": [{"var": a}, {"var": b}]}))
    }

    /// Create a percentage rule: output = value * (percentage / 100)
    pub fn percentage(mut self, value: &str, percentage: &str) -> Rule {
        self.inputs = vec![value.to_string(), percentage.to_string()];
        self.build(json!({"*": [{"var": value}, {"/": [{"var": percentage}, 100]}]}))
    }

    /// Create a sum rule: output = sum of all inputs
    pub fn sum(mut self, inputs: &[&str]) -> Rule {
        self.inputs = inputs.iter().map(|s| s.to_string()).collect();
        let vars: Vec<Value> = inputs.iter().map(|i| json!({"var": i})).collect();
        self.build(json!({"+": vars}))
    }

    /// Create a custom formula rule
    pub fn formula(mut self, inputs: &[&str], expression: Value) -> Rule {
        self.inputs = inputs.iter().map(|s| s.to_string()).collect();
        self.build(expression)
    }

    fn build(self, expression: Value) -> Rule {
        let mut rule = Rule::from_json_logic(self.product_id, "calc", expression)
            .with_inputs(self.inputs)
            .with_outputs([self.output]);

        if let Some(desc) = self.description {
            rule = rule.with_description(desc);
        }

        rule
    }
}

/// Builder for creating conditional rules
pub struct ConditionalRuleBuilder {
    product_id: String,
    output: String,
    description: Option<String>,
}

impl ConditionalRuleBuilder {
    pub fn new(product_id: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            product_id: product_id.into(),
            output: output.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Create a threshold rule: output = above_value if input > threshold else below_value
    pub fn threshold<V: Into<Value>>(
        self,
        input: &str,
        threshold: f64,
        above_value: V,
        below_value: V,
    ) -> Rule {
        let expression = json!({
            "if": [
                {">": [{"var": input}, threshold]},
                above_value.into(),
                below_value.into()
            ]
        });
        self.build(vec![input.to_string()], expression)
    }

    /// Create a range rule: output = in_range_value if min <= input <= max else out_of_range_value
    pub fn in_range<V: Into<Value>>(
        self,
        input: &str,
        min: f64,
        max: f64,
        in_range_value: V,
        out_of_range_value: V,
    ) -> Rule {
        let expression = json!({
            "if": [
                {"and": [
                    {">=": [{"var": input}, min]},
                    {"<=": [{"var": input}, max]}
                ]},
                in_range_value.into(),
                out_of_range_value.into()
            ]
        });
        self.build(vec![input.to_string()], expression)
    }

    /// Create a multi-tier rule with multiple thresholds
    pub fn tiers<V: Into<Value> + Clone>(
        self,
        input: &str,
        tiers: &[(f64, V)],  // (threshold, value) pairs
        default: V,
    ) -> Rule {
        let mut conditions: Vec<Value> = Vec::new();

        for (threshold, value) in tiers {
            conditions.push(json!({">": [{"var": input}, threshold]}));
            conditions.push(value.clone().into());
        }
        conditions.push(default.into());

        let expression = json!({"if": conditions});
        self.build(vec![input.to_string()], expression)
    }

    /// Create a boolean condition rule
    pub fn when<V: Into<Value>>(
        self,
        condition: Value,
        inputs: &[&str],
        then_value: V,
        else_value: V,
    ) -> Rule {
        let expression = json!({
            "if": [condition, then_value.into(), else_value.into()]
        });
        self.build(inputs.iter().map(|s| s.to_string()).collect(), expression)
    }

    fn build(self, inputs: Vec<String>, expression: Value) -> Rule {
        let mut rule = Rule::from_json_logic(self.product_id, "conditional", expression)
            .with_inputs(inputs)
            .with_outputs([self.output]);

        if let Some(desc) = self.description {
            rule = rule.with_description(desc);
        }

        rule
    }
}

/// Builder for creating signal/classification rules (common in trading)
pub struct SignalRuleBuilder {
    product_id: String,
    output: String,
    description: Option<String>,
}

impl SignalRuleBuilder {
    pub fn new(product_id: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            product_id: product_id.into(),
            output: output.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Create an RSI signal: BUY if rsi < oversold, SELL if rsi > overbought, else HOLD
    pub fn rsi_signal(self, rsi_input: &str, oversold: f64, overbought: f64) -> Rule {
        let expression = json!({
            "if": [
                {"<": [{"var": rsi_input}, oversold]}, "BUY",
                {">": [{"var": rsi_input}, overbought]}, "SELL",
                "HOLD"
            ]
        });
        self.build(vec![rsi_input.to_string()], expression)
    }

    /// Create a crossover signal: BUY if fast > slow, SELL if fast < slow
    pub fn crossover_signal(self, fast: &str, slow: &str) -> Rule {
        let expression = json!({
            "if": [
                {">": [{"var": fast}, {"var": slow}]}, "BUY",
                {"<": [{"var": fast}, {"var": slow}]}, "SELL",
                "HOLD"
            ]
        });
        self.build(vec![fast.to_string(), slow.to_string()], expression)
    }

    /// Create a price vs moving average signal
    pub fn price_vs_ma(self, price: &str, ma: &str, buffer_pct: f64) -> Rule {
        let expression = json!({
            "if": [
                {">": [{"var": price}, {"*": [{"var": ma}, 1.0 + buffer_pct]}]}, "BUY",
                {"<": [{"var": price}, {"*": [{"var": ma}, 1.0 - buffer_pct]}]}, "SELL",
                "HOLD"
            ]
        });
        self.build(vec![price.to_string(), ma.to_string()], expression)
    }

    fn build(self, inputs: Vec<String>, expression: Value) -> Rule {
        let mut rule = Rule::from_json_logic(self.product_id, "signal", expression)
            .with_inputs(inputs)
            .with_outputs([self.output]);

        if let Some(desc) = self.description {
            rule = rule.with_description(desc);
        }

        rule
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input_paths(rule: &Rule) -> Vec<String> {
        rule.input_attributes.iter().map(|a| a.path.as_str().to_string()).collect()
    }

    fn output_paths(rule: &Rule) -> Vec<String> {
        rule.output_attributes.iter().map(|a| a.path.as_str().to_string()).collect()
    }

    #[test]
    fn test_calc_multiply() {
        let rule = CalcRuleBuilder::new("test", "result")
            .with_description("Multiply x by y")
            .multiply("x", "y");

        assert_eq!(input_paths(&rule), vec!["x", "y"]);
        assert_eq!(output_paths(&rule), vec!["result"]);
        assert_eq!(rule.rule_type, "calc");
        assert_eq!(rule.description.as_deref(), Some("Multiply x by y"));
        // Expression should be {"*": [{"var": "x"}, {"var": "y"}]}
        let expr = rule.get_expression().unwrap();
        assert!(expr.get("*").is_some());
    }

    #[test]
    fn test_calc_percentage() {
        let rule = CalcRuleBuilder::new("test", "tax")
            .percentage("amount", "tax_rate");

        assert_eq!(input_paths(&rule), vec!["amount", "tax_rate"]);
        assert_eq!(output_paths(&rule), vec!["tax"]);
        // Expression should calculate percentage
        let expr = rule.get_expression().unwrap();
        assert!(expr.get("*").is_some());
    }

    #[test]
    fn test_calc_sum() {
        let rule = CalcRuleBuilder::new("test", "total")
            .sum(&["a", "b", "c"]);

        assert_eq!(input_paths(&rule), vec!["a", "b", "c"]);
        assert_eq!(output_paths(&rule), vec!["total"]);
        // Expression should be {"+": [...]}
        let expr = rule.get_expression().unwrap();
        assert!(expr.get("+").is_some());
    }

    #[test]
    fn test_conditional_threshold() {
        let rule = ConditionalRuleBuilder::new("test", "category")
            .threshold("score", 50.0, "PASS", "FAIL");

        assert_eq!(input_paths(&rule), vec!["score"]);
        assert_eq!(output_paths(&rule), vec!["category"]);
        assert_eq!(rule.rule_type, "conditional");
        // Expression should be {"if": [...]}
        let expr = rule.get_expression().unwrap();
        assert!(expr.get("if").is_some());
    }

    #[test]
    fn test_conditional_tiers() {
        let rule = ConditionalRuleBuilder::new("test", "grade")
            .tiers("score", &[(90.0, "A"), (80.0, "B"), (70.0, "C"), (60.0, "D")], "F");

        assert_eq!(input_paths(&rule), vec!["score"]);
        assert_eq!(output_paths(&rule), vec!["grade"]);
        // Should have if expression with multiple conditions
        let expr = rule.get_expression().unwrap();
        let if_expr = expr.get("if").unwrap().as_array().unwrap();
        assert!(if_expr.len() > 5); // Multiple tiers + default
    }

    #[test]
    fn test_conditional_range() {
        let rule = ConditionalRuleBuilder::new("test", "status")
            .in_range("temp", 36.0, 37.5, "NORMAL", "ABNORMAL");

        assert_eq!(input_paths(&rule), vec!["temp"]);
        assert_eq!(output_paths(&rule), vec!["status"]);
        // Should check both >= and <=
        let expr = rule.get_expression().unwrap();
        let if_expr = expr.get("if").unwrap();
        assert!(if_expr.to_string().contains("and"));
    }

    #[test]
    fn test_rsi_signal() {
        let rule = SignalRuleBuilder::new("test", "signal")
            .rsi_signal("rsi", 30.0, 70.0);

        assert_eq!(input_paths(&rule), vec!["rsi"]);
        assert_eq!(output_paths(&rule), vec!["signal"]);
        assert_eq!(rule.rule_type, "signal");
        // Should have if expression
        let expr = rule.get_expression().unwrap();
        assert!(expr.get("if").is_some());
    }

    #[test]
    fn test_crossover_signal() {
        let rule = SignalRuleBuilder::new("test", "signal")
            .crossover_signal("sma_fast", "sma_slow");

        assert_eq!(input_paths(&rule), vec!["sma_fast", "sma_slow"]);
        assert_eq!(output_paths(&rule), vec!["signal"]);
        // Should have if expression comparing the two
        let expr = rule.get_expression().unwrap();
        assert!(expr.get("if").is_some());
    }

    #[test]
    fn test_price_vs_ma_signal() {
        let rule = SignalRuleBuilder::new("test", "signal")
            .price_vs_ma("price", "sma_50", 0.02);

        assert_eq!(input_paths(&rule), vec!["price", "sma_50"]);
        assert_eq!(output_paths(&rule), vec!["signal"]);
    }
}
