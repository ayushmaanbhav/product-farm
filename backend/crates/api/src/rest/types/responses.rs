//! REST API response types
//!
//! All response types for JSON serialization with camelCase field names.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// PAGINATION
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub next_page_token: String,
    pub total_count: i32,
}

// =============================================================================
// PRODUCT RESPONSES
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_product_id: Option<String>,
    pub effective_from: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub version: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloneProductResponse {
    pub product: ProductResponse,
    pub abstract_attributes_cloned: i32,
    pub attributes_cloned: i32,
    pub rules_cloned: i32,
    pub functionalities_cloned: i32,
}

// =============================================================================
// ABSTRACT ATTRIBUTE RESPONSES
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractAttributeResponse {
    pub abstract_path: String,
    pub product_id: String,
    pub component_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,
    pub attribute_name: String,
    pub datatype_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_name: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint_expression: Option<String>,
    pub display_expression: String,
    pub display_names: Vec<DisplayNameResponse>,
    pub tags: Vec<AttributeTagResponse>,
    pub related_attributes: Vec<RelatedAttributeResponse>,
    pub immutable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayNameResponse {
    pub name: String,
    pub format: String,
    pub order_index: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeTagResponse {
    pub name: String,
    pub order_index: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedAttributeResponse {
    pub relationship_type: String,
    pub related_path: String,
    pub order_index: i32,
}

// =============================================================================
// CONCRETE ATTRIBUTE RESPONSES
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeResponse {
    pub path: String,
    pub abstract_path: String,
    pub product_id: String,
    pub component_type: String,
    pub component_id: String,
    pub attribute_name: String,
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<AttributeValueJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AttributeValueJson {
    #[serde(rename = "null")]
    Null,
    #[serde(rename = "bool")]
    Bool { value: bool },
    #[serde(rename = "int")]
    Int { value: i64 },
    #[serde(rename = "float")]
    Float { value: f64 },
    #[serde(rename = "string")]
    String { value: String },
    #[serde(rename = "decimal")]
    Decimal { value: String },
    #[serde(rename = "array")]
    Array { value: Vec<AttributeValueJson> },
    #[serde(rename = "object")]
    Object { value: HashMap<String, AttributeValueJson> },
}

// =============================================================================
// RULE RESPONSES
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleResponse {
    pub id: String,
    pub product_id: String,
    pub rule_type: String,
    pub display_expression: String,
    pub compiled_expression: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub enabled: bool,
    pub order_index: i32,
    pub input_attributes: Vec<RuleAttributeResponse>,
    pub output_attributes: Vec<RuleAttributeResponse>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleAttributeResponse {
    pub rule_id: String,
    pub attribute_path: String,
    pub order_index: i32,
}

// =============================================================================
// DATATYPE RESPONSES
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatatypeResponse {
    pub id: String,
    pub name: String,
    pub primitive_type: String,
    pub constraints: DatatypeConstraintsJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DatatypeConstraintsJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint_rule_expression: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint_error_message: Option<String>,
}

// =============================================================================
// FUNCTIONALITY RESPONSES
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionalityResponse {
    pub id: String,
    pub product_id: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub status: String,
    pub immutable: bool,
    pub required_attributes: Vec<RequiredAttributeResponse>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequiredAttributeResponse {
    pub functionality_id: String,
    pub abstract_path: String,
    pub description: String,
    pub order_index: i32,
}

// =============================================================================
// ENUMERATION RESPONSES
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumerationResponse {
    pub id: String,
    pub name: String,
    pub template_type: String,
    pub values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// =============================================================================
// EVALUATION RESPONSES
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateResponse {
    pub success: bool,
    pub outputs: HashMap<String, AttributeValueJson>,
    pub rule_results: Vec<RuleResultJson>,
    pub metrics: ExecutionMetricsJson,
    pub errors: Vec<EvaluationErrorJson>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleResultJson {
    pub rule_id: String,
    pub outputs: Vec<OutputValueJson>,
    pub execution_time_ns: i64,
    pub skipped: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputValueJson {
    pub path: String,
    pub value: AttributeValueJson,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionMetricsJson {
    pub total_time_ns: i64,
    pub rules_executed: i32,
    pub rules_skipped: i32,
    pub cache_hits: i32,
    pub levels: Vec<LevelMetricsJson>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelMetricsJson {
    pub level: i32,
    pub time_ns: i64,
    pub rules_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationErrorJson {
    pub attribute: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeResult {
    pub path: String,
    pub value: AttributeValueJson,
    pub computed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchEvaluateResponse {
    pub results: Vec<BatchResultItem>,
    pub total_time_ns: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResultItem {
    pub request_id: String,
    pub results: Vec<AttributeResult>,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionStep {
    pub order: i32,
    pub rule_id: String,
    pub rule_type: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionPlanResponse {
    pub levels: Vec<ExecutionLevelJson>,
    pub dependencies: Vec<RuleDependencyJson>,
    pub missing_inputs: Vec<MissingInputJson>,
    pub dot_graph: String,
    pub mermaid_graph: String,
    pub ascii_graph: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLevelJson {
    pub level: i32,
    pub rule_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleDependencyJson {
    pub rule_id: String,
    pub depends_on: Vec<String>,
    pub produces: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingInputJson {
    pub rule_id: String,
    pub input_path: String,
}

// =============================================================================
// VALIDATION RESPONSES
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResultJson {
    pub valid: bool,
    pub errors: Vec<ValidationErrorJson>,
    pub warnings: Vec<ValidationWarningJson>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationErrorJson {
    pub rule_id: String,
    pub error_type: String,
    pub message: String,
    pub field: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationWarningJson {
    pub rule_id: String,
    pub warning_type: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResponse {
    pub product_id: String,
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub severity: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

// =============================================================================
// IMPACT ANALYSIS RESPONSES
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactAnalysisResponse {
    pub target_path: String,
    pub direct_dependencies: Vec<DependencyInfoJson>,
    pub transitive_dependencies: Vec<DependencyInfoJson>,
    pub affected_rules: Vec<String>,
    pub affected_functionalities: Vec<String>,
    pub has_immutable_dependents: bool,
    pub immutable_paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyInfoJson {
    pub path: String,
    pub attribute_name: String,
    pub direction: String, // "upstream" or "downstream"
    pub distance: i32,
    pub is_immutable: bool,
}

// =============================================================================
// PRODUCT TEMPLATE RESPONSES
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductTemplateResponse {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub template_type: String,
    pub description: String,
    pub components: Vec<TemplateComponentJson>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateComponentJson {
    pub id: String,
    pub name: String,
    pub description: String,
}

// =============================================================================
// COMMON RESPONSES
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResponse {
    #[serde(alias = "deleted")]
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// =============================================================================
// LIST RESPONSE TYPE ALIASES
// =============================================================================

pub type ListProductsResponse = PaginatedResponse<ProductResponse>;
pub type ListAbstractAttributesResponse = PaginatedResponse<AbstractAttributeResponse>;
pub type ListAttributesResponse = PaginatedResponse<AttributeResponse>;
pub type ListRulesResponse = PaginatedResponse<RuleResponse>;
pub type ListDatatypesResponse = PaginatedResponse<DatatypeResponse>;
pub type ListFunctionalitiesResponse = PaginatedResponse<FunctionalityResponse>;
pub type ListEnumerationsResponse = PaginatedResponse<EnumerationResponse>;
