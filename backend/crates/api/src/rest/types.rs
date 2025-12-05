//! REST API types for JSON serialization
//!
//! These types match the frontend TypeScript interfaces with camelCase field names

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// INPUT VALIDATION LIMITS (DoS Prevention)
// =============================================================================

/// Maximum length for ID fields (product_id, rule_id, etc.)
pub const MAX_ID_LENGTH: usize = 128;

/// Maximum length for name fields
pub const MAX_NAME_LENGTH: usize = 256;

/// Maximum length for description fields
pub const MAX_DESCRIPTION_LENGTH: usize = 4096;

/// Maximum length for JSON expression fields
pub const MAX_EXPRESSION_LENGTH: usize = 65536; // 64KB

/// Maximum number of items in array fields (attributes, values, etc.)
pub const MAX_ARRAY_ITEMS: usize = 1000;

/// Maximum length for path fields
pub const MAX_PATH_LENGTH: usize = 512;

/// Maximum length for tag names
pub const MAX_TAG_LENGTH: usize = 64;

/// Maximum length for enum values
pub const MAX_ENUM_VALUE_LENGTH: usize = 256;

/// Validation error for input limits
#[derive(Debug)]
pub struct InputValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for InputValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for InputValidationError {}

/// Trait for validating request inputs
pub trait ValidateInput {
    fn validate_input(&self) -> Result<(), InputValidationError>;
}

/// Helper to validate string length
fn validate_string_length(
    value: &str,
    field: &str,
    max_length: usize,
) -> Result<(), InputValidationError> {
    if value.len() > max_length {
        return Err(InputValidationError {
            field: field.to_string(),
            message: format!(
                "exceeds maximum length of {} characters (got {})",
                max_length,
                value.len()
            ),
        });
    }
    Ok(())
}

/// Helper to validate optional string length
fn validate_optional_string_length(
    value: &Option<String>,
    field: &str,
    max_length: usize,
) -> Result<(), InputValidationError> {
    if let Some(s) = value {
        validate_string_length(s, field, max_length)?;
    }
    Ok(())
}

/// Helper to validate array length
fn validate_array_length<T>(
    value: &[T],
    field: &str,
    max_length: usize,
) -> Result<(), InputValidationError> {
    if value.len() > max_length {
        return Err(InputValidationError {
            field: field.to_string(),
            message: format!(
                "exceeds maximum count of {} items (got {})",
                max_length,
                value.len()
            ),
        });
    }
    Ok(())
}

// =============================================================================
// PAGINATION
// =============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub next_page_token: String,
    pub total_count: i32,
}

// =============================================================================
// PRODUCT TYPES
// =============================================================================

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductRequest {
    pub id: String,
    pub name: String,
    pub template_type: String,
    pub effective_from: i64,
    #[serde(default)]
    pub expiry_at: Option<i64>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub effective_from: Option<i64>,
    #[serde(default)]
    pub expiry_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloneProductRequest {
    pub new_product_id: String,
    pub new_product_name: String,
    #[serde(default)]
    pub new_product_description: Option<String>,
    #[serde(default)]
    pub selected_components: Vec<String>,
    #[serde(default)]
    pub selected_datatypes: Vec<String>,
    #[serde(default)]
    pub selected_enumerations: Vec<String>,
    #[serde(default)]
    pub selected_functionalities: Vec<String>,
    #[serde(default)]
    pub selected_abstract_attributes: Vec<String>,
    #[serde(default = "default_true")]
    pub clone_concrete_attributes: bool,
}

fn default_true() -> bool {
    true
}

// List response types for pagination
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProductsResponse {
    pub products: Vec<ProductResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAbstractAttributesResponse {
    pub attributes: Vec<AbstractAttributeResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAttributesResponse {
    pub attributes: Vec<AttributeResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRulesResponse {
    pub rules: Vec<RuleResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDatatypesResponse {
    pub datatypes: Vec<DatatypeResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFunctionalitiesResponse {
    pub functionalities: Vec<FunctionalityResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEnumerationsResponse {
    pub enumerations: Vec<EnumerationResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloneProductResponse {
    pub product: ProductResponse,
    pub abstract_attributes_cloned: i32,
    pub attributes_cloned: i32,
    pub rules_cloned: i32,
    pub functionalities_cloned: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRequest {
    #[serde(default)]
    pub comments: Option<String>,
}

// =============================================================================
// ABSTRACT ATTRIBUTE TYPES
// =============================================================================

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayNameResponse {
    pub name: String,
    pub format: String,
    pub order_index: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeTagResponse {
    pub name: String,
    pub order_index: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedAttributeResponse {
    pub relationship_type: String,
    pub related_path: String,
    pub order_index: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAbstractAttributeRequest {
    pub component_type: String,
    #[serde(default)]
    pub component_id: Option<String>,
    pub attribute_name: String,
    pub datatype_id: String,
    #[serde(default)]
    pub enum_name: Option<String>,
    #[serde(default)]
    pub constraint_expression: Option<String>,
    #[serde(default)]
    pub immutable: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub display_names: Vec<DisplayNameInput>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayNameInput {
    pub name: String,
    pub format: String,
    pub order_index: i32,
}

// =============================================================================
// CONCRETE ATTRIBUTE TYPES
// =============================================================================

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAttributeRequest {
    pub component_type: String,
    pub component_id: String,
    pub attribute_name: String,
    pub abstract_path: String,
    pub value_type: String,
    #[serde(default)]
    pub value: Option<AttributeValueJson>,
    #[serde(default)]
    pub rule_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAttributeRequest {
    #[serde(default)]
    pub value: Option<AttributeValueJson>,
    #[serde(default)]
    pub rule_id: Option<String>,
}

// =============================================================================
// RULE TYPES
// =============================================================================

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleAttributeResponse {
    pub rule_id: String,
    pub attribute_path: String,
    pub order_index: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuleRequest {
    pub rule_type: String,
    pub input_attributes: Vec<String>,
    pub output_attributes: Vec<String>,
    pub display_expression: String,
    pub expression_json: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub order_index: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRuleRequest {
    #[serde(default)]
    pub rule_type: Option<String>,
    #[serde(default)]
    pub input_attributes: Option<Vec<String>>,
    #[serde(default)]
    pub output_attributes: Option<Vec<String>>,
    #[serde(default)]
    pub display_expression: Option<String>,
    #[serde(default)]
    pub expression_json: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub order_index: Option<i32>,
}

// =============================================================================
// DATATYPE TYPES
// =============================================================================

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDatatypeRequest {
    pub id: String,
    pub primitive_type: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub constraints: Option<DatatypeConstraintsJson>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDatatypeRequest {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub constraints: Option<DatatypeConstraintsJson>,
}

// =============================================================================
// FUNCTIONALITY TYPES
// =============================================================================

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequiredAttributeResponse {
    pub functionality_id: String,
    pub abstract_path: String,
    pub description: String,
    pub order_index: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFunctionalityRequest {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub immutable: bool,
    #[serde(default)]
    pub required_attributes: Vec<RequiredAttributeInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequiredAttributeInput {
    pub abstract_path: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFunctionalityRequest {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required_attributes: Option<Vec<RequiredAttributeInput>>,
}

// =============================================================================
// TEMPLATE ENUMERATION TYPES
// =============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumerationResponse {
    pub id: String,
    pub name: String,
    pub template_type: String,
    pub values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEnumerationRequest {
    pub name: String,
    pub template_type: String,
    pub values: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEnumerationRequest {
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddEnumerationValueRequest {
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveEnumerationValueRequest {
    #[serde(default)]
    pub cascade: bool,
}

// =============================================================================
// EVALUATION TYPES
// =============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateRequest {
    pub product_id: String,
    pub input_data: HashMap<String, AttributeValueJson>,
    #[serde(default)]
    pub rule_ids: Vec<String>,
    #[serde(default)]
    pub options: Option<EvaluationOptionsJson>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationOptionsJson {
    #[serde(default)]
    pub include_intermediate_results: bool,
    #[serde(default)]
    pub max_execution_time_ms: i64,
    #[serde(default)]
    pub debug_mode: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateResponse {
    pub success: bool,
    pub outputs: HashMap<String, AttributeValueJson>,
    pub rule_results: Vec<RuleResultJson>,
    pub metrics: ExecutionMetricsJson,
    pub errors: Vec<EvaluationErrorJson>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputValueJson {
    pub path: String,
    pub value: AttributeValueJson,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionMetricsJson {
    pub total_time_ns: i64,
    pub rules_executed: i32,
    pub rules_skipped: i32,
    pub cache_hits: i32,
    pub levels: Vec<LevelMetricsJson>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelMetricsJson {
    pub level: i32,
    pub time_ns: i64,
    pub rules_count: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationErrorJson {
    pub attribute: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeResult {
    pub path: String,
    pub value: AttributeValueJson,
    pub computed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchEvaluateRequest {
    pub product_id: String,
    pub requests: Vec<BatchRequestItem>,
    #[serde(default)]
    pub options: Option<EvaluationOptionsJson>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchRequestItem {
    pub request_id: String,
    pub input_data: HashMap<String, AttributeValueJson>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchEvaluateResponse {
    pub results: Vec<BatchResultItem>,
    pub total_time_ns: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResultItem {
    pub request_id: String,
    pub results: Vec<AttributeResult>,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionStep {
    pub order: i32,
    pub rule_id: String,
    pub rule_type: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionPlanResponse {
    pub levels: Vec<ExecutionLevelJson>,
    pub dependencies: Vec<RuleDependencyJson>,
    pub missing_inputs: Vec<MissingInputJson>,
    pub dot_graph: String,
    pub mermaid_graph: String,
    pub ascii_graph: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLevelJson {
    pub level: i32,
    pub rule_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleDependencyJson {
    pub rule_id: String,
    pub depends_on: Vec<String>,
    pub produces: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingInputJson {
    pub rule_id: String,
    pub input_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResultJson {
    pub valid: bool,
    pub errors: Vec<ValidationErrorJson>,
    pub warnings: Vec<ValidationWarningJson>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationErrorJson {
    pub rule_id: String,
    pub error_type: String,
    pub message: String,
    pub field: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationWarningJson {
    pub rule_id: String,
    pub warning_type: String,
    pub message: String,
}

// Validation types for product validation endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResponse {
    pub product_id: String,
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub severity: String,
}

#[derive(Debug, Serialize)]
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
// PRODUCT TEMPLATES (for frontend wizard)
// =============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductTemplateResponse {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub template_type: String,
    pub description: String,
    pub components: Vec<TemplateComponentJson>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateComponentJson {
    pub id: String,
    pub name: String,
    pub description: String,
}

// =============================================================================
// DELETE RESPONSE
// =============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// =============================================================================
// INPUT VALIDATION IMPLEMENTATIONS
// =============================================================================

impl ValidateInput for CreateProductRequest {
    fn validate_input(&self) -> Result<(), InputValidationError> {
        validate_string_length(&self.id, "id", MAX_ID_LENGTH)?;
        validate_string_length(&self.name, "name", MAX_NAME_LENGTH)?;
        validate_string_length(&self.template_type, "template_type", MAX_ID_LENGTH)?;
        validate_optional_string_length(&self.description, "description", MAX_DESCRIPTION_LENGTH)?;
        Ok(())
    }
}

impl ValidateInput for UpdateProductRequest {
    fn validate_input(&self) -> Result<(), InputValidationError> {
        validate_optional_string_length(&self.name, "name", MAX_NAME_LENGTH)?;
        validate_optional_string_length(&self.description, "description", MAX_DESCRIPTION_LENGTH)?;
        Ok(())
    }
}

impl ValidateInput for CloneProductRequest {
    fn validate_input(&self) -> Result<(), InputValidationError> {
        validate_string_length(&self.new_product_id, "new_product_id", MAX_ID_LENGTH)?;
        validate_string_length(&self.new_product_name, "new_product_name", MAX_NAME_LENGTH)?;
        validate_optional_string_length(
            &self.new_product_description,
            "new_product_description",
            MAX_DESCRIPTION_LENGTH,
        )?;
        validate_array_length(
            &self.selected_components,
            "selected_components",
            MAX_ARRAY_ITEMS,
        )?;
        validate_array_length(
            &self.selected_datatypes,
            "selected_datatypes",
            MAX_ARRAY_ITEMS,
        )?;
        validate_array_length(
            &self.selected_enumerations,
            "selected_enumerations",
            MAX_ARRAY_ITEMS,
        )?;
        validate_array_length(
            &self.selected_functionalities,
            "selected_functionalities",
            MAX_ARRAY_ITEMS,
        )?;
        validate_array_length(
            &self.selected_abstract_attributes,
            "selected_abstract_attributes",
            MAX_ARRAY_ITEMS,
        )?;
        Ok(())
    }
}

impl ValidateInput for CreateAbstractAttributeRequest {
    fn validate_input(&self) -> Result<(), InputValidationError> {
        validate_string_length(&self.component_type, "component_type", MAX_ID_LENGTH)?;
        validate_optional_string_length(&self.component_id, "component_id", MAX_ID_LENGTH)?;
        validate_string_length(&self.attribute_name, "attribute_name", MAX_NAME_LENGTH)?;
        validate_string_length(&self.datatype_id, "datatype_id", MAX_ID_LENGTH)?;
        validate_optional_string_length(&self.enum_name, "enum_name", MAX_NAME_LENGTH)?;
        validate_optional_string_length(
            &self.constraint_expression,
            "constraint_expression",
            MAX_EXPRESSION_LENGTH,
        )?;
        validate_optional_string_length(&self.description, "description", MAX_DESCRIPTION_LENGTH)?;
        validate_array_length(&self.display_names, "display_names", MAX_ARRAY_ITEMS)?;
        validate_array_length(&self.tags, "tags", MAX_ARRAY_ITEMS)?;
        for tag in &self.tags {
            validate_string_length(tag, "tags[]", MAX_TAG_LENGTH)?;
        }
        Ok(())
    }
}

impl ValidateInput for CreateAttributeRequest {
    fn validate_input(&self) -> Result<(), InputValidationError> {
        validate_string_length(&self.component_type, "component_type", MAX_ID_LENGTH)?;
        validate_string_length(&self.component_id, "component_id", MAX_ID_LENGTH)?;
        validate_string_length(&self.attribute_name, "attribute_name", MAX_NAME_LENGTH)?;
        validate_string_length(&self.abstract_path, "abstract_path", MAX_PATH_LENGTH)?;
        validate_string_length(&self.value_type, "value_type", MAX_ID_LENGTH)?;
        validate_optional_string_length(&self.rule_id, "rule_id", MAX_ID_LENGTH)?;
        Ok(())
    }
}

impl ValidateInput for CreateRuleRequest {
    fn validate_input(&self) -> Result<(), InputValidationError> {
        validate_string_length(&self.rule_type, "rule_type", MAX_ID_LENGTH)?;
        validate_array_length(&self.input_attributes, "input_attributes", MAX_ARRAY_ITEMS)?;
        validate_array_length(&self.output_attributes, "output_attributes", MAX_ARRAY_ITEMS)?;
        validate_string_length(
            &self.display_expression,
            "display_expression",
            MAX_EXPRESSION_LENGTH,
        )?;
        validate_string_length(&self.expression_json, "expression_json", MAX_EXPRESSION_LENGTH)?;
        validate_optional_string_length(&self.description, "description", MAX_DESCRIPTION_LENGTH)?;
        for path in &self.input_attributes {
            validate_string_length(path, "input_attributes[]", MAX_PATH_LENGTH)?;
        }
        for path in &self.output_attributes {
            validate_string_length(path, "output_attributes[]", MAX_PATH_LENGTH)?;
        }
        Ok(())
    }
}

impl ValidateInput for CreateDatatypeRequest {
    fn validate_input(&self) -> Result<(), InputValidationError> {
        validate_string_length(&self.id, "id", MAX_ID_LENGTH)?;
        validate_string_length(&self.primitive_type, "primitive_type", MAX_ID_LENGTH)?;
        validate_optional_string_length(&self.description, "description", MAX_DESCRIPTION_LENGTH)?;
        Ok(())
    }
}

impl ValidateInput for CreateFunctionalityRequest {
    fn validate_input(&self) -> Result<(), InputValidationError> {
        validate_string_length(&self.name, "name", MAX_NAME_LENGTH)?;
        validate_string_length(&self.description, "description", MAX_DESCRIPTION_LENGTH)?;
        validate_array_length(
            &self.required_attributes,
            "required_attributes",
            MAX_ARRAY_ITEMS,
        )?;
        for ra in &self.required_attributes {
            validate_string_length(&ra.abstract_path, "required_attributes[].abstract_path", MAX_PATH_LENGTH)?;
            validate_string_length(&ra.description, "required_attributes[].description", MAX_DESCRIPTION_LENGTH)?;
        }
        Ok(())
    }
}

impl ValidateInput for CreateEnumerationRequest {
    fn validate_input(&self) -> Result<(), InputValidationError> {
        validate_string_length(&self.name, "name", MAX_NAME_LENGTH)?;
        validate_string_length(&self.template_type, "template_type", MAX_ID_LENGTH)?;
        validate_array_length(&self.values, "values", MAX_ARRAY_ITEMS)?;
        validate_optional_string_length(&self.description, "description", MAX_DESCRIPTION_LENGTH)?;
        for v in &self.values {
            validate_string_length(v, "values[]", MAX_ENUM_VALUE_LENGTH)?;
        }
        Ok(())
    }
}

impl ValidateInput for EvaluateRequest {
    fn validate_input(&self) -> Result<(), InputValidationError> {
        validate_string_length(&self.product_id, "product_id", MAX_ID_LENGTH)?;
        validate_array_length(&self.rule_ids, "rule_ids", MAX_ARRAY_ITEMS)?;
        for rule_id in &self.rule_ids {
            validate_string_length(rule_id, "rule_ids[]", MAX_ID_LENGTH)?;
        }
        // Note: input_data HashMap could also be bounded, but serde limits help there
        Ok(())
    }
}

impl ValidateInput for BatchEvaluateRequest {
    fn validate_input(&self) -> Result<(), InputValidationError> {
        validate_string_length(&self.product_id, "product_id", MAX_ID_LENGTH)?;
        validate_array_length(&self.requests, "requests", MAX_ARRAY_ITEMS)?;
        for req in &self.requests {
            validate_string_length(&req.request_id, "requests[].request_id", MAX_ID_LENGTH)?;
        }
        Ok(())
    }
}
