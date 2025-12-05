//! AI Tool Definitions for Product-FARM
//!
//! These tools can be invoked by an AI agent to manage rules.
//! Each tool has structured input/output for reliable agent interaction.

// Note: AgentError and core types will be used when implementing tool execution
#[allow(unused_imports)]
use crate::error::{AgentError, AgentResult};
#[allow(unused_imports)]
use product_farm_core::{ProductId, Rule, RuleId, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Tool Definitions (JSON Schema-compatible for LLM tool use)
// ============================================================================

/// All available tools for the AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tool", content = "input")]
pub enum AgentTool {
    /// List all rules for a product
    ListRules(ListRulesInput),

    /// Create a new rule from natural language description
    CreateRule(CreateRuleInput),

    /// Validate a rule for correctness
    ValidateRule(ValidateRuleInput),

    /// Explain a rule in natural language
    ExplainRule(ExplainRuleInput),

    /// Test a rule with sample inputs
    TestRule(TestRuleInput),

    /// Visualize the dependency graph
    VisualizeGraph(VisualizeGraphInput),

    /// Clone a rule with modifications
    CloneRule(CloneRuleInput),

    /// Suggest optimizations for a product's rules
    SuggestOptimizations(SuggestOptimizationsInput),

    /// Get impact analysis for changing a rule
    AnalyzeImpact(AnalyzeImpactInput),
}

// ============================================================================
// Tool Inputs
// ============================================================================

/// Input for listing rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRulesInput {
    /// The product ID to list rules for
    pub product_id: String,
    /// Optional filter by rule type
    pub rule_type: Option<String>,
    /// Whether to include disabled rules
    #[serde(default)]
    pub include_disabled: bool,
}

/// Input for creating a rule from natural language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRuleInput {
    /// The product ID to add the rule to
    pub product_id: String,
    /// Natural language description of the rule
    pub description: String,
    /// The type of rule (e.g., "CALCULATION", "VALIDATION", "ENTRY", "EXIT")
    pub rule_type: String,
    /// Optional: Explicit input attributes (if not specified, inferred from description)
    pub input_attributes: Option<Vec<String>>,
    /// Optional: Explicit output attributes (if not specified, inferred from description)
    pub output_attributes: Option<Vec<String>>,
}

/// Input for validating a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRuleInput {
    /// The rule to validate (as JSON Logic)
    pub expression: serde_json::Value,
    /// The product ID (for dependency checking)
    pub product_id: String,
    /// Input attribute paths
    pub input_attributes: Vec<String>,
    /// Output attribute paths
    pub output_attributes: Vec<String>,
}

/// Input for explaining a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainRuleInput {
    /// The rule ID to explain, OR
    pub rule_id: Option<String>,
    /// The JSON Logic expression to explain directly
    pub expression: Option<serde_json::Value>,
    /// Desired verbosity level: "brief", "detailed", "technical"
    #[serde(default = "default_verbosity")]
    pub verbosity: String,
}

fn default_verbosity() -> String {
    "detailed".to_string()
}

/// Input for testing a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRuleInput {
    /// The rule ID to test, OR
    pub rule_id: Option<String>,
    /// The JSON Logic expression to test directly
    pub expression: Option<serde_json::Value>,
    /// Test input values
    pub inputs: HashMap<String, serde_json::Value>,
}

/// Input for visualizing the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizeGraphInput {
    /// The product ID to visualize
    pub product_id: String,
    /// Output format: "mermaid", "dot", "ascii"
    #[serde(default = "default_format")]
    pub format: String,
    /// Optional: Focus on a specific rule or attribute
    pub focus: Option<String>,
    /// Maximum depth for traversal
    #[serde(default = "default_depth")]
    pub max_depth: usize,
}

fn default_format() -> String {
    "mermaid".to_string()
}

fn default_depth() -> usize {
    5
}

/// Input for cloning a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneRuleInput {
    /// The source rule ID to clone
    pub source_rule_id: String,
    /// The target product ID (can be same or different)
    pub target_product_id: String,
    /// Modifications to apply
    pub modifications: Option<RuleModifications>,
}

/// Modifications to apply when cloning a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleModifications {
    /// New description
    pub description: Option<String>,
    /// New rule type
    pub rule_type: Option<String>,
    /// Replace specific variables
    pub variable_replacements: Option<HashMap<String, String>>,
    /// Additional conditions to AND with existing
    pub additional_conditions: Option<serde_json::Value>,
}

/// Input for suggesting optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestOptimizationsInput {
    /// The product ID to analyze
    pub product_id: String,
    /// Focus areas: "performance", "readability", "all"
    #[serde(default = "default_focus_areas")]
    pub focus: String,
}

fn default_focus_areas() -> String {
    "all".to_string()
}

/// Input for impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeImpactInput {
    /// The rule ID to analyze impact for
    pub rule_id: String,
    /// Type of change: "modify", "delete", "disable"
    pub change_type: String,
}

// ============================================================================
// Tool Outputs
// ============================================================================

/// Output from listing rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRulesOutput {
    pub rules: Vec<RuleSummary>,
    pub total_count: usize,
}

/// Summary of a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSummary {
    pub id: String,
    pub rule_type: String,
    pub description: Option<String>,
    pub display_expression: Option<String>,
    pub enabled: bool,
    pub input_count: usize,
    pub output_count: usize,
}

/// Output from creating a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRuleOutput {
    /// The generated rule
    pub rule: GeneratedRule,
    /// Explanation of what was created
    pub explanation: String,
    /// Any warnings
    pub warnings: Vec<String>,
}

/// A generated rule ready for saving
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedRule {
    pub rule_type: String,
    pub expression: serde_json::Value,
    pub display_expression: String,
    pub description: String,
    pub input_attributes: Vec<String>,
    pub output_attributes: Vec<String>,
}

/// Output from validating a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRuleOutput {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub inferred_types: HashMap<String, String>,
}

/// A validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
}

/// A validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Output from explaining a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainRuleOutput {
    /// Plain English explanation
    pub explanation: String,
    /// Step-by-step breakdown
    pub steps: Vec<ExplanationStep>,
    /// Variables used
    pub variables: Vec<VariableInfo>,
}

/// A step in the rule explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationStep {
    pub step_number: usize,
    pub description: String,
    pub expression_part: Option<String>,
}

/// Information about a variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    pub name: String,
    pub role: String, // "input" or "output"
    pub inferred_type: Option<String>,
}

/// Output from testing a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRuleOutput {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ns: u64,
    pub intermediate_values: HashMap<String, serde_json::Value>,
}

/// Output from visualizing the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizeGraphOutput {
    pub format: String,
    pub content: String,
    pub node_count: usize,
    pub edge_count: usize,
}

/// Output from cloning a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneRuleOutput {
    pub new_rule_id: String,
    pub changes_applied: Vec<String>,
}

/// Output from suggesting optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestOptimizationsOutput {
    pub suggestions: Vec<OptimizationSuggestion>,
    pub overall_score: f64,
}

/// An optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub category: String,
    pub priority: String, // "high", "medium", "low"
    pub rule_id: Option<String>,
    pub description: String,
    pub expected_improvement: Option<String>,
}

/// Output from impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeImpactOutput {
    /// Directly affected rules
    pub affected_rules: Vec<String>,
    /// Attributes that would be impacted
    pub affected_attributes: Vec<String>,
    /// Risk level: "low", "medium", "high", "critical"
    pub risk_level: String,
    /// Detailed impact description
    pub description: String,
}

// ============================================================================
// Tool Metadata (for LLM tool registration)
// ============================================================================

/// Metadata about a tool for LLM registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Get metadata for all available tools
pub fn get_tool_definitions() -> Vec<ToolMetadata> {
    vec![
        ToolMetadata {
            name: "list_rules".to_string(),
            description: "List all rules for a product. Returns rule summaries including type, description, and dependency counts.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "product_id": {"type": "string", "description": "The product ID to list rules for"},
                    "rule_type": {"type": "string", "description": "Optional filter by rule type"},
                    "include_disabled": {"type": "boolean", "description": "Whether to include disabled rules", "default": false}
                },
                "required": ["product_id"]
            }),
        },
        ToolMetadata {
            name: "create_rule".to_string(),
            description: "Create a new rule from a natural language description. The AI will translate the description into JSON Logic and validate it.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "product_id": {"type": "string", "description": "The product ID to add the rule to"},
                    "description": {"type": "string", "description": "Natural language description of what the rule should do"},
                    "rule_type": {"type": "string", "description": "The type of rule (CALCULATION, VALIDATION, ENTRY, EXIT, etc.)"},
                    "input_attributes": {"type": "array", "items": {"type": "string"}, "description": "Explicit input attribute paths"},
                    "output_attributes": {"type": "array", "items": {"type": "string"}, "description": "Explicit output attribute paths"}
                },
                "required": ["product_id", "description", "rule_type"]
            }),
        },
        ToolMetadata {
            name: "validate_rule".to_string(),
            description: "Validate a JSON Logic expression for correctness. Checks for cycles, type mismatches, and missing dependencies.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {"type": "object", "description": "The JSON Logic expression to validate"},
                    "product_id": {"type": "string", "description": "The product ID for dependency checking"},
                    "input_attributes": {"type": "array", "items": {"type": "string"}, "description": "Input attribute paths"},
                    "output_attributes": {"type": "array", "items": {"type": "string"}, "description": "Output attribute paths"}
                },
                "required": ["expression", "product_id", "input_attributes", "output_attributes"]
            }),
        },
        ToolMetadata {
            name: "explain_rule".to_string(),
            description: "Explain a rule in plain English. Breaks down the logic step by step.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rule_id": {"type": "string", "description": "The ID of the rule to explain"},
                    "expression": {"type": "object", "description": "Or provide a JSON Logic expression directly"},
                    "verbosity": {"type": "string", "enum": ["brief", "detailed", "technical"], "default": "detailed"}
                }
            }),
        },
        ToolMetadata {
            name: "test_rule".to_string(),
            description: "Test a rule with sample input values and see the result.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rule_id": {"type": "string", "description": "The ID of the rule to test"},
                    "expression": {"type": "object", "description": "Or provide a JSON Logic expression directly"},
                    "inputs": {"type": "object", "description": "Map of variable names to test values"}
                },
                "required": ["inputs"]
            }),
        },
        ToolMetadata {
            name: "visualize_graph".to_string(),
            description: "Generate a visual representation of the rule dependency graph.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "product_id": {"type": "string", "description": "The product ID to visualize"},
                    "format": {"type": "string", "enum": ["mermaid", "dot", "ascii"], "default": "mermaid"},
                    "focus": {"type": "string", "description": "Optional rule or attribute to focus on"},
                    "max_depth": {"type": "integer", "description": "Maximum traversal depth", "default": 5}
                },
                "required": ["product_id"]
            }),
        },
        ToolMetadata {
            name: "analyze_impact".to_string(),
            description: "Analyze the impact of changing or deleting a rule. Shows affected downstream rules and attributes.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rule_id": {"type": "string", "description": "The rule ID to analyze"},
                    "change_type": {"type": "string", "enum": ["modify", "delete", "disable"], "description": "Type of change"}
                },
                "required": ["rule_id", "change_type"]
            }),
        },
        ToolMetadata {
            name: "suggest_optimizations".to_string(),
            description: "Suggest optimizations for a product's rules. Identifies redundant rules, potential simplifications, and performance improvements.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "product_id": {"type": "string", "description": "The product ID to analyze"},
                    "focus": {"type": "string", "enum": ["performance", "readability", "all"], "default": "all"}
                },
                "required": ["product_id"]
            }),
        },
    ]
}
