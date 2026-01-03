//! Prompt Builder for LLM Rule Evaluation
//!
//! Generates context-rich prompts for LLM-based rule evaluation,
//! including rule metadata, input/output descriptions, and evaluation guidance.

use product_farm_core::Value;
use std::collections::HashMap;

/// Metadata about an attribute for prompt context
#[derive(Debug, Clone, Default)]
pub struct AttributeInfo {
    /// Attribute name/path
    pub name: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Data type (e.g., "string", "number", "boolean")
    pub data_type: Option<String>,
    /// Example values
    pub examples: Vec<String>,
    /// Constraints or validation rules
    pub constraints: Option<String>,
}

impl AttributeInfo {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_data_type(mut self, dt: impl Into<String>) -> Self {
        self.data_type = Some(dt.into());
        self
    }

    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.examples.push(example.into());
        self
    }

    pub fn with_constraints(mut self, constraints: impl Into<String>) -> Self {
        self.constraints = Some(constraints.into());
        self
    }
}

/// Full context for LLM rule evaluation
#[derive(Debug, Clone, Default)]
pub struct RuleEvaluationContext {
    /// Rule name/identifier
    pub rule_name: String,
    /// Rule description
    pub rule_description: Option<String>,
    /// Rule type/category
    pub rule_type: Option<String>,
    /// Input attributes with their metadata
    pub inputs: Vec<AttributeInfo>,
    /// Output attributes with their metadata
    pub outputs: Vec<AttributeInfo>,
    /// The actual input values to evaluate
    pub input_values: HashMap<String, Value>,
    /// Additional context or instructions
    pub additional_context: Option<String>,
    /// Product/domain name
    pub product_name: Option<String>,
}

impl RuleEvaluationContext {
    pub fn new(rule_name: impl Into<String>) -> Self {
        Self {
            rule_name: rule_name.into(),
            ..Default::default()
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.rule_description = Some(desc.into());
        self
    }

    pub fn with_rule_type(mut self, rule_type: impl Into<String>) -> Self {
        self.rule_type = Some(rule_type.into());
        self
    }

    pub fn with_product(mut self, product: impl Into<String>) -> Self {
        self.product_name = Some(product.into());
        self
    }

    pub fn add_input(mut self, info: AttributeInfo) -> Self {
        self.inputs.push(info);
        self
    }

    pub fn add_output(mut self, info: AttributeInfo) -> Self {
        self.outputs.push(info);
        self
    }

    pub fn with_input_value(mut self, name: impl Into<String>, value: Value) -> Self {
        self.input_values.insert(name.into(), value);
        self
    }

    pub fn with_input_values(mut self, values: HashMap<String, Value>) -> Self {
        self.input_values = values;
        self
    }

    pub fn with_additional_context(mut self, ctx: impl Into<String>) -> Self {
        self.additional_context = Some(ctx.into());
        self
    }
}

/// Builds prompts for LLM evaluation with rich context
#[derive(Debug, Clone, Default)]
pub struct PromptBuilder {
    /// Custom prompt template (if provided)
    custom_template: Option<String>,
    /// Whether to include examples in the prompt
    include_examples: bool,
    /// Output format instructions
    output_format: OutputFormatInstructions,
}

#[derive(Debug, Clone, Default)]
pub enum OutputFormatInstructions {
    #[default]
    Json,
    SingleValue,
    Boolean,
    Structured,
}

// =============================================================================
// Instruction Deduplication Detection
// =============================================================================

/// Detects if user's template already contains specific instructions.
/// Only returns true for 100% certain matches - conservative approach.
#[derive(Debug, Default)]
struct InstructionDetector {
    /// User's template text (lowercased for matching)
    template_lower: String,
}

impl InstructionDetector {
    fn new(template: &str) -> Self {
        Self {
            template_lower: template.to_lowercase(),
        }
    }

    /// Check if user explicitly specified JSON output format with complete instructions.
    /// Must contain BOTH "json" AND output structure indicators.
    fn has_complete_json_format_instruction(&self) -> bool {
        let has_json = self.template_lower.contains("json");
        let has_structure = self.template_lower.contains("```json")
            || (self.template_lower.contains("{") && self.template_lower.contains("}"))
            || self.template_lower.contains("json object")
            || self.template_lower.contains("json format");

        // Must have both JSON mention AND structural indication
        has_json && has_structure
    }

    /// Check if user explicitly mentioned "only" or "just" output restrictions.
    /// e.g., "return only the json", "respond with just the value"
    fn has_output_only_instruction(&self) -> bool {
        let patterns = [
            "return only",
            "respond only",
            "output only",
            "return just",
            "respond with just",
            "only return",
            "only respond",
            "no explanation",
            "no additional text",
            "nothing else",
        ];
        patterns.iter().any(|p| self.template_lower.contains(p))
    }

    /// Check if user mentioned constraints/validation - but we still include ours
    /// unless they have COMPLETE constraint definitions (enum values listed, etc.)
    fn has_complete_constraint_instructions(&self, context: &RuleEvaluationContext) -> bool {
        // Check if user mentioned constraints
        let mentions_constraints = self.template_lower.contains("constraint")
            || self.template_lower.contains("validation")
            || self.template_lower.contains("valid values");

        if !mentions_constraints {
            return false;
        }

        // Check if ALL outputs with constraints are covered in template
        // This is strict - if any constraint is missing, return false
        for output in &context.outputs {
            if let Some(constraints) = &output.constraints {
                // User must have mentioned this specific constraint
                if !self.template_lower.contains(&constraints.to_lowercase()) {
                    return false;
                }
            }
        }

        true
    }

    /// Check if user mentioned data types - but we still include ours
    /// unless they covered ALL output types completely
    fn has_complete_datatype_instructions(&self, context: &RuleEvaluationContext) -> bool {
        let mentions_types = self.template_lower.contains("type")
            || self.template_lower.contains("data type")
            || self.template_lower.contains("datatype");

        if !mentions_types {
            return false;
        }

        // Check if ALL outputs with types are covered
        for output in &context.outputs {
            if let Some(dtype) = &output.data_type {
                // User must have mentioned this specific type
                if !self.template_lower.contains(&dtype.to_lowercase()) {
                    return false;
                }
            }
        }

        true
    }
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Use a custom prompt template
    pub fn with_template(mut self, template: impl Into<String>) -> Self {
        self.custom_template = Some(template.into());
        self
    }

    /// Include examples in the prompt
    pub fn with_examples(mut self, include: bool) -> Self {
        self.include_examples = include;
        self
    }

    /// Set output format instructions
    pub fn with_output_format(mut self, format: OutputFormatInstructions) -> Self {
        self.output_format = format;
        self
    }

    /// Build the prompt from context
    pub fn build(&self, context: &RuleEvaluationContext) -> String {
        if let Some(template) = &self.custom_template {
            self.render_template(template, context)
        } else {
            self.generate_default_prompt(context)
        }
    }

    /// Render a custom template with context
    fn render_template(&self, template: &str, context: &RuleEvaluationContext) -> String {
        let mut result = template.to_string();

        // Replace rule-level placeholders
        result = result.replace("{{rule_name}}", &context.rule_name);
        result = result.replace(
            "{{rule_description}}",
            context.rule_description.as_deref().unwrap_or(""),
        );
        result = result.replace(
            "{{rule_type}}",
            context.rule_type.as_deref().unwrap_or(""),
        );
        result = result.replace(
            "{{product_name}}",
            context.product_name.as_deref().unwrap_or(""),
        );

        // Replace inputs JSON - only include defined inputs, not all values
        if result.contains("{{inputs_json}}") {
            let filtered_inputs: HashMap<&String, &Value> = context
                .inputs
                .iter()
                .filter_map(|input| {
                    context.input_values.get(&input.name).map(|v| (&input.name, v))
                })
                .collect();
            let json = serde_json::to_string_pretty(&filtered_inputs).unwrap_or_default();
            result = result.replace("{{inputs_json}}", &json);
        }

        // Replace inputs description
        if result.contains("{{inputs_description}}") {
            let desc = self.format_inputs_description(context);
            result = result.replace("{{inputs_description}}", &desc);
        }

        // Replace outputs description
        if result.contains("{{outputs_description}}") {
            let desc = self.format_outputs_description(context);
            result = result.replace("{{outputs_description}}", &desc);
        }

        // Replace output names
        if result.contains("{{output_names}}") {
            let names: Vec<_> = context.outputs.iter().map(|o| o.name.as_str()).collect();
            result = result.replace("{{output_names}}", &names.join(", "));
        }

        // Replace individual input values
        for (key, value) in &context.input_values {
            let placeholder = format!("{{{{{}}}}}", key);
            let value_str = match value {
                Value::String(s) => s.clone(),
                other => serde_json::to_string(other).unwrap_or_default(),
            };
            result = result.replace(&placeholder, &value_str);
        }

        // Replace additional context
        result = result.replace(
            "{{additional_context}}",
            context.additional_context.as_deref().unwrap_or(""),
        );

        result
    }

    /// Generate a comprehensive default prompt
    fn generate_default_prompt(&self, context: &RuleEvaluationContext) -> String {
        let mut sections = Vec::new();

        // Header
        sections.push(self.build_header(context));

        // Rule description
        if let Some(desc) = &context.rule_description {
            sections.push(format!("## Rule Description\n{}", desc));
        }

        // Inputs section
        sections.push(self.build_inputs_section(context));

        // Expected outputs section
        sections.push(self.build_outputs_section(context));

        // Evaluation instructions
        sections.push(self.build_instructions(context));

        // Output format
        sections.push(self.build_output_format_instructions(context));

        sections.join("\n\n")
    }

    fn build_header(&self, context: &RuleEvaluationContext) -> String {
        let mut header = format!("# Rule Evaluation: {}", context.rule_name);

        if let Some(product) = &context.product_name {
            header.push_str(&format!("\nProduct: {}", product));
        }

        if let Some(rule_type) = &context.rule_type {
            header.push_str(&format!("\nRule Type: {}", rule_type));
        }

        header
    }

    fn build_inputs_section(&self, context: &RuleEvaluationContext) -> String {
        let mut section = String::from("## Input Values\n\n");

        if context.inputs.is_empty() {
            // No inputs defined - skip showing any values
            section.push_str("(No inputs defined)\n");
        } else {
            // Show only the defined inputs with their values
            for input in &context.inputs {
                section.push_str(&format!("### {}\n", input.name));

                if let Some(desc) = &input.description {
                    section.push_str(&format!("- **Description**: {}\n", desc));
                }

                if let Some(dt) = &input.data_type {
                    section.push_str(&format!("- **Type**: {}\n", dt));
                }

                if let Some(constraints) = &input.constraints {
                    section.push_str(&format!("- **Constraints**: {}\n", constraints));
                }

                // Show current value only if it exists for this defined input
                if let Some(value) = context.input_values.get(&input.name) {
                    let value_str = serde_json::to_string(value).unwrap_or_default();
                    section.push_str(&format!("- **Current Value**: `{}`\n", value_str));
                }

                if self.include_examples && !input.examples.is_empty() {
                    section.push_str(&format!(
                        "- **Examples**: {}\n",
                        input.examples.join(", ")
                    ));
                }

                section.push('\n');
            }
        }

        section
    }

    fn build_outputs_section(&self, context: &RuleEvaluationContext) -> String {
        let mut section = String::from("## Expected Outputs\n\n");
        section.push_str("You must provide values for the following outputs:\n\n");

        if context.outputs.is_empty() {
            section.push_str("(No specific output attributes defined)\n");
        } else {
            for output in &context.outputs {
                section.push_str(&format!("### {}\n", output.name));

                if let Some(desc) = &output.description {
                    section.push_str(&format!("- **Description**: {}\n", desc));
                }

                if let Some(dt) = &output.data_type {
                    section.push_str(&format!("- **Expected Type**: {}\n", dt));
                }

                if let Some(constraints) = &output.constraints {
                    section.push_str(&format!("- **Constraints**: {}\n", constraints));
                }

                if self.include_examples && !output.examples.is_empty() {
                    section.push_str(&format!(
                        "- **Example Values**: {}\n",
                        output.examples.join(", ")
                    ));
                }

                section.push('\n');
            }
        }

        section
    }

    fn build_instructions(&self, context: &RuleEvaluationContext) -> String {
        let mut instructions = String::from("## Task\n\n");

        instructions.push_str(
            "Evaluate the inputs above and determine the output values.\n",
        );

        // Add additional context if provided
        if let Some(additional) = &context.additional_context {
            instructions.push_str(&format!("\n**Context**: {}\n", additional));
        }

        instructions
    }

    fn build_output_format_instructions(&self, context: &RuleEvaluationContext) -> String {
        // Minimal format hints - system prompt already says "respond with ONLY the requested output format"
        match self.output_format {
            OutputFormatInstructions::Json => {
                let output_names: Vec<_> =
                    context.outputs.iter().map(|o| o.name.as_str()).collect();
                let keys = if output_names.is_empty() {
                    "result".to_string()
                } else {
                    output_names.join(", ")
                };
                format!("## Output\n\nJSON with keys: {}", keys)
            }

            OutputFormatInstructions::SingleValue => {
                "## Output\n\nSingle value".to_string()
            }

            OutputFormatInstructions::Boolean => {
                "## Output\n\n`true` or `false`".to_string()
            }

            OutputFormatInstructions::Structured => {
                let output_names: Vec<_> =
                    context.outputs.iter().map(|o| o.name.as_str()).collect();
                format!("## Output\n\n{}", output_names.join(", "))
            }
        }
    }

    fn format_inputs_description(&self, context: &RuleEvaluationContext) -> String {
        let mut desc = String::new();
        for input in &context.inputs {
            desc.push_str(&format!("- **{}**", input.name));
            if let Some(d) = &input.description {
                desc.push_str(&format!(": {}", d));
            }
            if let Some(dt) = &input.data_type {
                desc.push_str(&format!(" ({})", dt));
            }
            desc.push('\n');
        }
        desc
    }

    fn format_outputs_description(&self, context: &RuleEvaluationContext) -> String {
        let mut desc = String::new();
        for output in &context.outputs {
            desc.push_str(&format!("- **{}**", output.name));
            if let Some(d) = &output.description {
                desc.push_str(&format!(": {}", d));
            }
            if let Some(dt) = &output.data_type {
                desc.push_str(&format!(" ({})", dt));
            }
            desc.push('\n');
        }
        desc
    }
}

/// Generate a system prompt for rule evaluation
pub fn default_system_prompt() -> String {
    r#"You are a rule evaluation engine. Your task is to evaluate business rules based on provided inputs and return the computed outputs.

Guidelines:
1. Analyze all input values carefully
2. Apply the rule logic as described
3. Return outputs in the exact format requested
4. Be precise with data types (numbers, strings, booleans)
5. Handle edge cases appropriately
6. If a value cannot be determined, return null

You must respond with ONLY the requested output format, no explanations or additional text."#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder_default() {
        let context = RuleEvaluationContext::new("calculate-premium")
            .with_description("Calculate insurance premium based on risk factors")
            .with_rule_type("premium-calculation")
            .with_product("auto-insurance")
            .add_input(
                AttributeInfo::new("age")
                    .with_description("Driver's age in years")
                    .with_data_type("integer"),
            )
            .add_input(
                AttributeInfo::new("driving_record")
                    .with_description("Number of accidents in past 5 years")
                    .with_data_type("integer"),
            )
            .add_output(
                AttributeInfo::new("premium")
                    .with_description("Monthly premium amount in dollars")
                    .with_data_type("number"),
            )
            .with_input_value("age".to_string(), Value::Int(25))
            .with_input_value("driving_record".to_string(), Value::Int(0));

        let builder = PromptBuilder::new();
        let prompt = builder.build(&context);

        assert!(prompt.contains("calculate-premium"));
        assert!(prompt.contains("auto-insurance"));
        assert!(prompt.contains("Driver's age"));
        assert!(prompt.contains("premium"));
    }

    #[test]
    fn test_prompt_builder_custom_template() {
        let context = RuleEvaluationContext::new("test-rule")
            // Define inputs first (required for {{inputs_json}} to include them)
            .add_input(AttributeInfo::new("x"))
            .add_input(AttributeInfo::new("y"))
            // Then set their values
            .with_input_value("x".to_string(), Value::Int(10))
            .with_input_value("y".to_string(), Value::Int(20));

        let builder = PromptBuilder::new()
            .with_template("Calculate: x={{x}}, y={{y}}. Inputs: {{inputs_json}}");

        let prompt = builder.build(&context);

        assert!(prompt.contains("x=10"));
        assert!(prompt.contains("y=20"));
        // Only defined inputs are included in inputs_json
        assert!(prompt.contains("\"x\": 10"));
    }

    #[test]
    fn test_attribute_info_builder() {
        let info = AttributeInfo::new("score")
            .with_description("Overall score")
            .with_data_type("number")
            .with_example("85.5")
            .with_example("92.0")
            .with_constraints("Must be between 0 and 100");

        assert_eq!(info.name, "score");
        assert_eq!(info.description, Some("Overall score".to_string()));
        assert_eq!(info.examples.len(), 2);
    }

    #[test]
    fn test_prompt_token_optimization() {
        // This test shows the optimized prompt structure
        let context = RuleEvaluationContext::new("calculate-premium")
            .with_description("Calculate insurance premium")
            .add_input(
                AttributeInfo::new("age")
                    .with_description("Driver's age")
                    .with_data_type("integer"),
            )
            .add_output(
                AttributeInfo::new("premium")
                    .with_description("Monthly premium")
                    .with_data_type("number"),
            )
            .with_input_value("age".to_string(), Value::Int(25));

        let prompt = PromptBuilder::new().build(&context);

        // Verify optimized structure:
        // - Short "## Task" section instead of verbose instructions
        // - Minimal "## Output" instead of full format instructions
        assert!(prompt.contains("## Task"));
        assert!(prompt.contains("Evaluate the inputs"));
        assert!(prompt.contains("## Output"));
        assert!(prompt.contains("JSON with keys: premium"));

        // Should NOT contain verbose redundant instructions
        assert!(!prompt.contains("edge cases and boundary conditions"));
        assert!(!prompt.contains("Return ONLY the JSON object, no additional text"));
        assert!(!prompt.contains("Consider:"));

        // Verify prompt is concise (under 700 chars for this simple case)
        assert!(prompt.len() < 700, "Prompt too long: {} chars", prompt.len());
    }
}
