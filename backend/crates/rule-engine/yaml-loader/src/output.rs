//! Structured YAML output generation.
//!
//! Generates conventionally-structured YAML files from inferred schemas,
//! providing a canonical representation that can be used as reference.

use crate::error::LoaderResult;
use crate::report::InferenceReport;
use crate::schema::MasterSchema;
use std::fs;
use std::path::Path;

/// Structured output generator.
pub struct StructuredOutput<'a> {
    /// The master schema to output.
    pub schema: &'a MasterSchema,

    /// Optional inference report.
    pub report: Option<&'a InferenceReport>,
}

impl<'a> StructuredOutput<'a> {
    /// Create a new structured output generator.
    pub fn new(schema: &'a MasterSchema) -> Self {
        Self {
            schema,
            report: None,
        }
    }

    /// Include inference report in output.
    pub fn with_report(mut self, report: &'a InferenceReport) -> Self {
        self.report = Some(report);
        self
    }

    /// Write structured output to a folder.
    ///
    /// Creates the following structure:
    /// ```text
    /// output/
    ///   product.yaml         # Product metadata
    ///   types.yaml           # DataTypes and Enums
    ///   entities/
    ///     <entity>.yaml      # One per entity
    ///   functions.yaml       # All rules
    ///   inference-report.md  # Human-readable report (if report provided)
    /// ```
    pub fn write_to_folder(&self, output_path: &Path) -> LoaderResult<()> {
        // Create output directory
        fs::create_dir_all(output_path)?;

        // Write product.yaml
        self.write_product_yaml(output_path)?;

        // Write types.yaml
        self.write_types_yaml(output_path)?;

        // Write entities folder
        self.write_entities_yaml(output_path)?;

        // Write functions.yaml
        self.write_functions_yaml(output_path)?;

        // Write inference report if provided
        if let Some(report) = self.report {
            self.write_inference_report(output_path, report)?;
        }

        Ok(())
    }

    /// Write product metadata.
    fn write_product_yaml(&self, output_path: &Path) -> LoaderResult<()> {
        let product = &self.schema.product;

        let content = format!(
            r#"# Product Definition
# AUTO-GENERATED - Canonical structure

product:
  id: {}
  name: {}
  version: {}
  status: {}
{}{}
"#,
            product.id,
            product.name,
            product.version,
            format!("{:?}", product.status).to_lowercase(),
            product
                .description
                .as_ref()
                .map(|d| format!("  description: {}\n", d))
                .unwrap_or_default(),
            String::new()  // Tags not available in Product
        );

        fs::write(output_path.join("product.yaml"), content)?;
        Ok(())
    }

    /// Write types/enums.
    fn write_types_yaml(&self, output_path: &Path) -> LoaderResult<()> {
        if self.schema.data_types.is_empty() {
            return Ok(());
        }

        let mut content = String::from("# Data Types and Enums\n# AUTO-GENERATED - Canonical structure\n\ntypes:\n");

        for dt in &self.schema.data_types {
            content.push_str(&format!("  {}:\n", dt.id.as_str()));
            content.push_str(&format!("    base_type: {}\n", dt.primitive_type.as_str()));

            if let Some(desc) = &dt.description {
                content.push_str(&format!("    description: {}\n", desc));
            }

            if let Some(constraints) = &dt.constraints {
                content.push_str("    constraints:\n");
                if let Some(min) = constraints.min {
                    content.push_str(&format!("      min: {}\n", min));
                }
                if let Some(max) = constraints.max {
                    content.push_str(&format!("      max: {}\n", max));
                }
                if let Some(pattern) = &constraints.pattern {
                    content.push_str(&format!("      pattern: {}\n", pattern));
                }
            }
            content.push('\n');
        }

        fs::write(output_path.join("types.yaml"), content)?;
        Ok(())
    }

    /// Write entities as individual files.
    fn write_entities_yaml(&self, output_path: &Path) -> LoaderResult<()> {
        if self.schema.attributes.is_empty() {
            return Ok(());
        }

        // Create entities folder
        let entities_path = output_path.join("entities");
        fs::create_dir_all(&entities_path)?;

        // Group attributes by component type
        let mut entities: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
        for attr in &self.schema.attributes {
            entities
                .entry(attr.component_type.clone())
                .or_default()
                .push(attr);
        }

        // Write each entity
        for (entity_name, attrs) in entities {
            let mut content = format!(
                "# Entity: {}\n# AUTO-GENERATED - Canonical structure\n\nentity: {}\ncomponent_type: {}\n\nattributes:\n",
                to_pascal_case(&entity_name),
                to_pascal_case(&entity_name),
                entity_name
            );

            for attr in attrs {
                // Extract attribute name from abstract path
                let attr_name = attr.abstract_path.as_str()
                    .split(':')
                    .last()
                    .unwrap_or("unknown");

                content.push_str(&format!("  - name: {}\n", attr_name));
                content.push_str(&format!("    abstract_path: \"{}\"\n", attr.abstract_path));
                content.push_str(&format!("    datatype_id: {}\n", attr.datatype_id));
                content.push_str(&format!("    component_type: {}\n", attr.component_type));

                if let Some(component_id) = &attr.component_id {
                    content.push_str(&format!("    component_id: {}\n", component_id));
                }

                if let Some(desc) = &attr.description {
                    content.push_str(&format!("    description: {}\n", desc));
                }

                if let Some(enum_name) = &attr.enum_name {
                    content.push_str(&format!("    enum_name: {}\n", enum_name));
                }

                content.push_str(&format!("    immutable: {}\n", attr.immutable));
                content.push('\n');
            }

            let file_name = format!("{}.yaml", entity_name);
            fs::write(entities_path.join(file_name), content)?;
        }

        Ok(())
    }

    /// Write functions/rules.
    fn write_functions_yaml(&self, output_path: &Path) -> LoaderResult<()> {
        if self.schema.rules.is_empty() {
            return Ok(());
        }

        let mut content = String::from("# Functions/Rules\n# AUTO-GENERATED - Canonical structure\n\nfunctions:\n");

        for rule in &self.schema.rules {
            content.push_str(&format!("  {}:\n", rule.rule_type));

            if let Some(desc) = &rule.description {
                content.push_str(&format!("    description: {}\n", desc));
            }

            content.push_str(&format!("    evaluator: {}\n", format_evaluator(&rule.evaluator)));

            if !rule.input_attributes.is_empty() {
                content.push_str("    inputs:\n");
                for input in &rule.input_attributes {
                    content.push_str(&format!("      - \"{}\"\n", input.path));
                }
            }

            if !rule.output_attributes.is_empty() {
                content.push_str("    outputs:\n");
                for output in &rule.output_attributes {
                    content.push_str(&format!("      - \"{}\"\n", output.path));
                }
            }

            if !rule.display_expression.is_empty() {
                content.push_str(&format!("    expression: {}\n", rule.display_expression));
            }

            content.push_str(&format!("    order: {}\n", rule.order_index));
            content.push_str(&format!("    enabled: {}\n", rule.enabled));
            content.push('\n');
        }

        fs::write(output_path.join("functions.yaml"), content)?;
        Ok(())
    }

    /// Write inference report.
    fn write_inference_report(
        &self,
        output_path: &Path,
        report: &InferenceReport,
    ) -> LoaderResult<()> {
        let markdown = report.to_markdown();
        fs::write(output_path.join("inference-report.md"), markdown)?;

        // Also write JSON version
        let json = serde_json::to_string_pretty(&report.to_json())
            .unwrap_or_else(|_| "{}".to_string());
        fs::write(output_path.join("inference-report.json"), json)?;

        Ok(())
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn format_value(value: &product_farm_core::Value) -> String {
    match value {
        product_farm_core::Value::Null => "null".to_string(),
        product_farm_core::Value::Bool(b) => b.to_string(),
        product_farm_core::Value::Int(i) => i.to_string(),
        product_farm_core::Value::Float(f) => f.to_string(),
        product_farm_core::Value::Decimal(d) => d.to_string(),
        product_farm_core::Value::String(s) => format!("\"{}\"", s),
        product_farm_core::Value::Array(arr) => {
            let items: Vec<_> = arr.iter().map(format_value).collect();
            format!("[{}]", items.join(", "))
        }
        product_farm_core::Value::Object(obj) => {
            let items: Vec<_> = obj
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}

fn format_evaluator(evaluator: &product_farm_core::RuleEvaluator) -> String {
    match &evaluator.name {
        product_farm_core::EvaluatorType::JsonLogic => "json-logic".to_string(),
        product_farm_core::EvaluatorType::LargeLanguageModel => {
            if let Some(config) = &evaluator.config {
                if let Some(model) = config.get("model") {
                    return format!("llm (model: {})", format_value(model));
                }
            }
            "llm".to_string()
        }
        product_farm_core::EvaluatorType::Custom(name) => format!("custom:{}", name),
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-')
        .filter(|p| !p.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Write structured output from a schema.
pub fn write_structured_output(
    schema: &MasterSchema,
    output_path: &Path,
) -> LoaderResult<()> {
    StructuredOutput::new(schema).write_to_folder(output_path)
}

/// Write structured output with inference report.
pub fn write_structured_output_with_report(
    schema: &MasterSchema,
    report: &InferenceReport,
    output_path: &Path,
) -> LoaderResult<()> {
    StructuredOutput::new(schema)
        .with_report(report)
        .write_to_folder(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use product_farm_core::{Product, ProductId, ProductStatus};
    use tempfile::TempDir;

    fn create_test_schema() -> MasterSchema {
        use product_farm_core::TemplateType;
        let product = Product {
            id: ProductId::new("test-product"),
            name: "Test Product".to_string(),
            description: Some("A test product".to_string()),
            version: 1,
            status: ProductStatus::Draft,
            template_type: TemplateType::new("test"),
            parent_product_id: None,
            effective_from: chrono::Utc::now(),
            expiry_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        MasterSchema::new(product)
    }

    #[test]
    fn test_write_structured_output() {
        let schema = create_test_schema();
        let temp = TempDir::new().unwrap();

        write_structured_output(&schema, temp.path()).unwrap();

        // Check product.yaml was created
        assert!(temp.path().join("product.yaml").exists());

        // Read and verify content
        let content = fs::read_to_string(temp.path().join("product.yaml")).unwrap();
        assert!(content.contains("test-product"));
        assert!(content.contains("Test Product"));
    }

    #[test]
    fn test_format_value() {
        assert_eq!(format_value(&product_farm_core::Value::Int(42)), "42");
        assert_eq!(
            format_value(&product_farm_core::Value::String("hello".into())),
            "\"hello\""
        );
        assert_eq!(format_value(&product_farm_core::Value::Bool(true)), "true");
    }
}
