//! REST API request types
//!
//! All request types for JSON deserialization with camelCase field names.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::limits::{
    MAX_ARRAY_ITEMS, MAX_DESCRIPTION_LENGTH, MAX_ENUM_VALUE_LENGTH, MAX_EXPRESSION_LENGTH,
    MAX_ID_LENGTH, MAX_NAME_LENGTH, MAX_PATH_LENGTH, MAX_TAG_LENGTH,
};

use super::responses::AttributeValueJson;
use super::validation::{
    validate_array_length, validate_array_not_empty, validate_optional_string_length,
    validate_required_string, validate_string_length, InputValidationError, ValidateInput,
};

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn default_true() -> bool {
    true
}

fn default_approved() -> bool {
    true
}

// =============================================================================
// PRODUCT REQUESTS
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRequest {
    /// Whether to approve (true) or reject (false)
    #[serde(default = "default_approved")]
    pub approved: bool,
    #[serde(default)]
    pub comments: Option<String>,
}

// =============================================================================
// ABSTRACT ATTRIBUTE REQUESTS
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayNameInput {
    pub name: String,
    pub format: String,
    pub order_index: i32,
}

// =============================================================================
// CONCRETE ATTRIBUTE REQUESTS
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAttributeRequest {
    #[serde(default)]
    pub value: Option<AttributeValueJson>,
    #[serde(default)]
    pub rule_id: Option<String>,
}

// =============================================================================
// RULE REQUESTS
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
// DATATYPE REQUESTS
// =============================================================================

use super::responses::DatatypeConstraintsJson;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDatatypeRequest {
    pub id: String,
    pub primitive_type: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub constraints: Option<DatatypeConstraintsJson>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDatatypeRequest {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub constraints: Option<DatatypeConstraintsJson>,
}

// =============================================================================
// FUNCTIONALITY REQUESTS
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFunctionalityRequest {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub immutable: bool,
    #[serde(default)]
    pub required_attributes: Vec<RequiredAttributeInput>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequiredAttributeInput {
    pub abstract_path: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFunctionalityRequest {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required_attributes: Option<Vec<RequiredAttributeInput>>,
}

// =============================================================================
// ENUMERATION REQUESTS
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEnumerationRequest {
    pub name: String,
    pub template_type: String,
    pub values: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEnumerationRequest {
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddEnumerationValueRequest {
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveEnumerationValueRequest {
    #[serde(default)]
    pub cascade: bool,
}

// =============================================================================
// EVALUATION REQUESTS
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateRequest {
    pub product_id: String,
    pub input_data: HashMap<String, AttributeValueJson>,
    #[serde(default)]
    pub rule_ids: Vec<String>,
    #[serde(default)]
    pub options: Option<EvaluationOptionsJson>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationOptionsJson {
    #[serde(default)]
    pub include_intermediate_results: bool,
    #[serde(default)]
    pub max_execution_time_ms: i64,
    #[serde(default)]
    pub debug_mode: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchEvaluateRequest {
    pub product_id: String,
    pub requests: Vec<BatchRequestItem>,
    #[serde(default)]
    pub options: Option<EvaluationOptionsJson>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchRequestItem {
    pub request_id: String,
    pub input_data: HashMap<String, AttributeValueJson>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactAnalysisRequest {
    pub target_path: String,
}

// =============================================================================
// VALIDATE INPUT IMPLEMENTATIONS
// =============================================================================

impl ValidateInput for CreateProductRequest {
    fn validate_input(&self) -> Result<(), InputValidationError> {
        validate_required_string(&self.id, "id")?;
        validate_string_length(&self.id, "id", MAX_ID_LENGTH)?;
        validate_required_string(&self.name, "name")?;
        validate_string_length(&self.name, "name", MAX_NAME_LENGTH)?;
        validate_required_string(&self.template_type, "template_type")?;
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
        validate_required_string(&self.component_type, "component_type")?;
        validate_string_length(&self.component_type, "component_type", MAX_ID_LENGTH)?;
        validate_optional_string_length(&self.component_id, "component_id", MAX_ID_LENGTH)?;
        validate_required_string(&self.attribute_name, "attribute_name")?;
        validate_string_length(&self.attribute_name, "attribute_name", MAX_NAME_LENGTH)?;
        validate_required_string(&self.datatype_id, "datatype_id")?;
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
        validate_required_string(&self.rule_type, "rule_type")?;
        validate_string_length(&self.rule_type, "rule_type", MAX_ID_LENGTH)?;
        validate_array_length(&self.input_attributes, "input_attributes", MAX_ARRAY_ITEMS)?;
        validate_array_not_empty(&self.output_attributes, "output_attributes")?;
        validate_array_length(&self.output_attributes, "output_attributes", MAX_ARRAY_ITEMS)?;
        validate_required_string(&self.display_expression, "display_expression")?;
        validate_string_length(
            &self.display_expression,
            "display_expression",
            MAX_EXPRESSION_LENGTH,
        )?;
        validate_required_string(&self.expression_json, "expression_json")?;
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
        validate_required_string(&self.name, "name")?;
        validate_string_length(&self.name, "name", MAX_NAME_LENGTH)?;
        validate_required_string(&self.template_type, "template_type")?;
        validate_string_length(&self.template_type, "template_type", MAX_ID_LENGTH)?;
        validate_array_not_empty(&self.values, "values")?;
        validate_array_length(&self.values, "values", MAX_ARRAY_ITEMS)?;
        validate_optional_string_length(&self.description, "description", MAX_DESCRIPTION_LENGTH)?;
        for v in &self.values {
            validate_required_string(v, "values[]")?;
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
