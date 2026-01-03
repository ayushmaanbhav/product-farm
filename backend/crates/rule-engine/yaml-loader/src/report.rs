//! Inference report generation.
//!
//! Generates detailed reports showing what was detected during YAML parsing
//! and the confidence levels of various inferences.

use crate::interpreter::Confidence;
use std::fmt;
use std::path::PathBuf;

/// Complete inference report for a loaded product.
#[derive(Debug, Clone)]
pub struct InferenceReport {
    /// Product ID.
    pub product_id: String,

    /// Source files that were parsed.
    pub source_files: Vec<PathBuf>,

    /// Entity reports.
    pub entities: Vec<EntityReport>,

    /// Function reports.
    pub functions: Vec<FunctionReport>,

    /// Warnings generated during parsing.
    pub warnings: Vec<ReportWarning>,

    /// Items with low confidence that may need attention.
    pub low_confidence_items: Vec<LowConfidenceItem>,
}

impl InferenceReport {
    /// Generate a markdown summary.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# Inference Report: {}\n\n", self.product_id));

        // Source files
        md.push_str("## Source Files\n\n");
        for file in &self.source_files {
            md.push_str(&format!("- `{}`\n", file.display()));
        }
        md.push('\n');

        // Summary stats
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- **Entities**: {}\n", self.entities.len()));
        let total_attrs: usize = self.entities.iter().map(|e| e.attributes.len()).sum();
        md.push_str(&format!("- **Attributes**: {}\n", total_attrs));
        md.push_str(&format!("- **Functions**: {}\n", self.functions.len()));
        md.push_str(&format!("- **Warnings**: {}\n", self.warnings.len()));
        md.push_str(&format!(
            "- **Low Confidence Items**: {}\n",
            self.low_confidence_items.len()
        ));
        md.push('\n');

        // Entities
        md.push_str("## Entities\n\n");
        for entity in &self.entities {
            md.push_str(&format!("### {}\n\n", entity.name));
            md.push_str(&format!("Detected as: **{}**\n\n", entity.detected_as));

            if !entity.attributes.is_empty() {
                md.push_str("#### Attributes\n\n");
                md.push_str("| Name | Type | Confidence | Classification |\n");
                md.push_str("|------|------|------------|----------------|\n");
                for attr in &entity.attributes {
                    md.push_str(&format!(
                        "| {} | {} | {} | {} |\n",
                        attr.name,
                        attr.inferred_type,
                        confidence_label(attr.type_confidence),
                        attr.classification
                    ));
                }
                md.push('\n');
            }

            if !entity.relationships.is_empty() {
                md.push_str("#### Relationships\n\n");
                md.push_str("| Name | Target | Cardinality | Confidence |\n");
                md.push_str("|------|--------|-------------|------------|\n");
                for rel in &entity.relationships {
                    md.push_str(&format!(
                        "| {} | {} | {} | {} |\n",
                        rel.name,
                        rel.target,
                        rel.cardinality,
                        confidence_label(rel.confidence)
                    ));
                }
                md.push('\n');
            }
        }

        // Functions
        md.push_str("## Functions\n\n");
        if !self.functions.is_empty() {
            md.push_str("| Name | Evaluator | Inputs | Outputs | Has Expression |\n");
            md.push_str("|------|-----------|--------|---------|----------------|\n");
            for func in &self.functions {
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    func.name,
                    func.evaluator_type,
                    func.input_count,
                    func.output_count,
                    if func.has_expression { "Yes" } else { "No" }
                ));
            }
            md.push('\n');
        }

        // Warnings
        if !self.warnings.is_empty() {
            md.push_str("## Warnings\n\n");
            for warning in &self.warnings {
                md.push_str(&format!(
                    "- **{}**: {} (at `{}`)\n",
                    warning.severity, warning.message, warning.location
                ));
            }
            md.push('\n');
        }

        // Low confidence items
        if !self.low_confidence_items.is_empty() {
            md.push_str("## Low Confidence Items\n\n");
            md.push_str("These items were inferred with low confidence and may need manual review:\n\n");
            for item in &self.low_confidence_items {
                md.push_str(&format!("### `{}`\n\n", item.path));
                md.push_str(&format!("- **Type**: {}\n", item.item_type));
                md.push_str(&format!("- **Confidence**: {:.0}%\n", item.confidence * 100.0));
                md.push_str(&format!("- **Reason**: {}\n", item.reason));
                md.push_str(&format!("- **Suggestion**: {}\n\n", item.suggestion));
            }
        }

        md
    }

    /// Convert to JSON.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "product_id": self.product_id,
            "source_files": self.source_files.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "entities": self.entities.iter().map(|e| e.to_json()).collect::<Vec<_>>(),
            "functions": self.functions.iter().map(|f| f.to_json()).collect::<Vec<_>>(),
            "warnings": self.warnings.iter().map(|w| w.to_json()).collect::<Vec<_>>(),
            "low_confidence_items": self.low_confidence_items.iter().map(|i| i.to_json()).collect::<Vec<_>>(),
        })
    }

    /// Print a summary to stdout.
    pub fn print_summary(&self) {
        println!("=== Inference Report: {} ===", self.product_id);
        println!();
        println!("Source files: {}", self.source_files.len());
        println!("Entities: {}", self.entities.len());

        let total_attrs: usize = self.entities.iter().map(|e| e.attributes.len()).sum();
        println!("Attributes: {}", total_attrs);
        println!("Functions: {}", self.functions.len());

        if !self.warnings.is_empty() {
            println!();
            println!("Warnings: {}", self.warnings.len());
            for w in &self.warnings {
                println!("  - {}: {}", w.severity, w.message);
            }
        }

        if !self.low_confidence_items.is_empty() {
            println!();
            println!("Low confidence items: {}", self.low_confidence_items.len());
            for item in &self.low_confidence_items {
                println!(
                    "  - {} ({:.0}%): {}",
                    item.path,
                    item.confidence * 100.0,
                    item.reason
                );
            }
        }
    }
}

/// Report for a single entity.
#[derive(Debug, Clone)]
pub struct EntityReport {
    /// Entity name.
    pub name: String,

    /// How it was detected (entity, enum, config).
    pub detected_as: String,

    /// Attribute reports.
    pub attributes: Vec<AttributeReport>,

    /// Relationship reports.
    pub relationships: Vec<RelationshipReport>,
}

impl EntityReport {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "detected_as": self.detected_as,
            "attributes": self.attributes.iter().map(|a| a.to_json()).collect::<Vec<_>>(),
            "relationships": self.relationships.iter().map(|r| r.to_json()).collect::<Vec<_>>(),
        })
    }
}

/// Report for a single attribute.
#[derive(Debug, Clone)]
pub struct AttributeReport {
    /// Attribute name.
    pub name: String,

    /// Inferred type.
    pub inferred_type: String,

    /// Type inference confidence.
    pub type_confidence: Confidence,

    /// Classification (static/instance).
    pub classification: String,

    /// Classification confidence.
    pub classification_confidence: f32,

    /// Signals that led to the inference.
    pub signals_used: Vec<String>,
}

impl AttributeReport {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "inferred_type": self.inferred_type,
            "type_confidence": confidence_label(self.type_confidence),
            "classification": self.classification,
            "classification_confidence": self.classification_confidence,
            "signals_used": self.signals_used,
        })
    }
}

/// Report for a relationship.
#[derive(Debug, Clone)]
pub struct RelationshipReport {
    /// Relationship name.
    pub name: String,

    /// Target entity.
    pub target: String,

    /// Cardinality.
    pub cardinality: String,

    /// Detection confidence.
    pub confidence: Confidence,
}

impl RelationshipReport {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "target": self.target,
            "cardinality": self.cardinality,
            "confidence": confidence_label(self.confidence),
        })
    }
}

/// Report for a function/rule.
#[derive(Debug, Clone)]
pub struct FunctionReport {
    /// Function name.
    pub name: String,

    /// Evaluator type (json-logic, llm, custom).
    pub evaluator_type: String,

    /// Number of inputs.
    pub input_count: usize,

    /// Number of outputs.
    pub output_count: usize,

    /// Whether it has an expression.
    pub has_expression: bool,
}

impl FunctionReport {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "evaluator_type": self.evaluator_type,
            "input_count": self.input_count,
            "output_count": self.output_count,
            "has_expression": self.has_expression,
        })
    }
}

/// A warning generated during parsing.
#[derive(Debug, Clone)]
pub struct ReportWarning {
    /// Warning severity.
    pub severity: WarningSeverity,

    /// Warning message.
    pub message: String,

    /// Location in source.
    pub location: String,
}

impl ReportWarning {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "severity": self.severity.as_str(),
            "message": self.message,
            "location": self.location,
        })
    }
}

/// Warning severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

impl WarningSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl fmt::Display for WarningSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// An item with low confidence that needs attention.
#[derive(Debug, Clone)]
pub struct LowConfidenceItem {
    /// Path to the item (entity.attribute).
    pub path: String,

    /// Type of item (attribute, type, relationship).
    pub item_type: String,

    /// Confidence level (0.0 - 1.0).
    pub confidence: f32,

    /// Reason for low confidence.
    pub reason: String,

    /// Suggestion for improvement.
    pub suggestion: String,
}

impl LowConfidenceItem {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.path,
            "item_type": self.item_type,
            "confidence": self.confidence,
            "reason": self.reason,
            "suggestion": self.suggestion,
        })
    }
}

/// Get human-readable label for confidence.
fn confidence_label(confidence: Confidence) -> &'static str {
    match confidence {
        Confidence::Certain => "certain",
        Confidence::High => "high",
        Confidence::Medium => "medium",
        Confidence::Low => "low",
        Confidence::Default => "default",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_to_markdown() {
        let report = InferenceReport {
            product_id: "test-product".to_string(),
            source_files: vec![PathBuf::from("product.yaml")],
            entities: vec![EntityReport {
                name: "User".to_string(),
                detected_as: "entity".to_string(),
                attributes: vec![AttributeReport {
                    name: "name".to_string(),
                    inferred_type: "string".to_string(),
                    type_confidence: Confidence::High,
                    classification: "instance".to_string(),
                    classification_confidence: 0.8,
                    signals_used: vec!["name pattern".to_string()],
                }],
                relationships: vec![],
            }],
            functions: vec![],
            warnings: vec![],
            low_confidence_items: vec![],
        };

        let md = report.to_markdown();
        assert!(md.contains("test-product"));
        assert!(md.contains("User"));
        assert!(md.contains("name"));
    }

    #[test]
    fn test_report_to_json() {
        let report = InferenceReport {
            product_id: "test".to_string(),
            source_files: vec![],
            entities: vec![],
            functions: vec![],
            warnings: vec![],
            low_confidence_items: vec![],
        };

        let json = report.to_json();
        assert_eq!(json["product_id"], "test");
    }
}
