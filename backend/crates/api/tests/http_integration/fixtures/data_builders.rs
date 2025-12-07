//! Data Builders for Test Entities
//!
//! Builder pattern for creating test request/response structures.

use std::collections::HashMap;
use chrono::Utc;

use product_farm_api::rest::types::{
    AttributeValueJson, CreateAbstractAttributeRequest, CreateAttributeRequest,
    CreateDatatypeRequest, CreateEnumerationRequest, CreateFunctionalityRequest,
    CreateProductRequest, CreateRuleRequest, DatatypeConstraintsJson, DisplayNameInput,
    EvaluateRequest, EvaluationOptionsJson, RequiredAttributeInput, UpdateProductRequest,
    UpdateRuleRequest, CloneProductRequest, BatchEvaluateRequest, BatchRequestItem,
};

// =============================================================================
// Product Builders
// =============================================================================

/// Builder for CreateProductRequest
pub struct ProductBuilder {
    id: String,
    name: String,
    template_type: String,
    effective_from: i64,
    expiry_at: Option<i64>,
    description: Option<String>,
}

impl ProductBuilder {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: format!("Test Product {}", id),
            template_type: "insurance".to_string(),
            effective_from: Utc::now().timestamp(),
            expiry_at: None,
            description: None,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_template(mut self, template_type: &str) -> Self {
        self.template_type = template_type.to_string();
        self
    }

    pub fn with_effective_from(mut self, timestamp: i64) -> Self {
        self.effective_from = timestamp;
        self
    }

    pub fn with_expiry(mut self, timestamp: i64) -> Self {
        self.expiry_at = Some(timestamp);
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn build(self) -> CreateProductRequest {
        CreateProductRequest {
            id: self.id,
            name: self.name,
            template_type: self.template_type,
            effective_from: self.effective_from,
            expiry_at: self.expiry_at,
            description: self.description,
        }
    }
}

/// Builder for CloneProductRequest
pub struct CloneProductBuilder {
    new_product_id: String,
    new_product_name: String,
    new_product_description: Option<String>,
    selected_components: Vec<String>,
    selected_datatypes: Vec<String>,
    selected_enumerations: Vec<String>,
    selected_functionalities: Vec<String>,
    selected_abstract_attributes: Vec<String>,
    clone_concrete_attributes: bool,
}

impl CloneProductBuilder {
    pub fn new(new_id: &str, new_name: &str) -> Self {
        Self {
            new_product_id: new_id.to_string(),
            new_product_name: new_name.to_string(),
            new_product_description: None,
            selected_components: Vec::new(),
            selected_datatypes: Vec::new(),
            selected_enumerations: Vec::new(),
            selected_functionalities: Vec::new(),
            selected_abstract_attributes: Vec::new(),
            clone_concrete_attributes: true,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.new_product_description = Some(desc.to_string());
        self
    }

    pub fn with_components(mut self, components: Vec<&str>) -> Self {
        self.selected_components = components.into_iter().map(String::from).collect();
        self
    }

    pub fn with_concrete_attributes(mut self, clone: bool) -> Self {
        self.clone_concrete_attributes = clone;
        self
    }

    pub fn build(self) -> CloneProductRequest {
        CloneProductRequest {
            new_product_id: self.new_product_id,
            new_product_name: self.new_product_name,
            new_product_description: self.new_product_description,
            selected_components: self.selected_components,
            selected_datatypes: self.selected_datatypes,
            selected_enumerations: self.selected_enumerations,
            selected_functionalities: self.selected_functionalities,
            selected_abstract_attributes: self.selected_abstract_attributes,
            clone_concrete_attributes: self.clone_concrete_attributes,
        }
    }
}

// =============================================================================
// Abstract Attribute Builders
// =============================================================================

/// Builder for CreateAbstractAttributeRequest
pub struct AbstractAttributeBuilder {
    component_type: String,
    component_id: Option<String>,
    attribute_name: String,
    datatype_id: String,
    enum_name: Option<String>,
    constraint_expression: Option<String>,
    immutable: bool,
    description: Option<String>,
    display_names: Vec<DisplayNameInput>,
    tags: Vec<String>,
}

impl AbstractAttributeBuilder {
    pub fn new(component_type: &str, attribute_name: &str) -> Self {
        Self {
            component_type: component_type.to_string(),
            component_id: None,
            attribute_name: attribute_name.to_string(),
            datatype_id: "decimal".to_string(),
            enum_name: None,
            constraint_expression: None,
            immutable: false,
            description: None,
            display_names: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn with_component_id(mut self, id: &str) -> Self {
        self.component_id = Some(id.to_string());
        self
    }

    pub fn with_datatype(mut self, datatype_id: &str) -> Self {
        self.datatype_id = datatype_id.to_string();
        self
    }

    pub fn with_enum(mut self, enum_name: &str) -> Self {
        self.enum_name = Some(enum_name.to_string());
        self
    }

    pub fn with_constraint(mut self, constraint: &str) -> Self {
        self.constraint_expression = Some(constraint.to_string());
        self
    }

    pub fn immutable(mut self) -> Self {
        self.immutable = true;
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_display_name(mut self, name: &str, format: &str, order: i32) -> Self {
        self.display_names.push(DisplayNameInput {
            name: name.to_string(),
            format: format.to_string(),
            order_index: order,
        });
        self
    }

    pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(String::from).collect();
        self
    }

    pub fn build(self) -> CreateAbstractAttributeRequest {
        CreateAbstractAttributeRequest {
            component_type: self.component_type,
            component_id: self.component_id,
            attribute_name: self.attribute_name,
            datatype_id: self.datatype_id,
            enum_name: self.enum_name,
            constraint_expression: self.constraint_expression,
            immutable: self.immutable,
            description: self.description,
            display_names: self.display_names,
            tags: self.tags,
        }
    }
}

// =============================================================================
// Concrete Attribute Builders
// =============================================================================

/// Builder for CreateAttributeRequest
pub struct ConcreteAttributeBuilder {
    component_type: String,
    component_id: String,
    attribute_name: String,
    abstract_path: String,
    value_type: String,
    value: Option<AttributeValueJson>,
    rule_id: Option<String>,
}

impl ConcreteAttributeBuilder {
    pub fn new(component_type: &str, component_id: &str, attribute_name: &str) -> Self {
        Self {
            component_type: component_type.to_string(),
            component_id: component_id.to_string(),
            attribute_name: attribute_name.to_string(),
            abstract_path: String::new(),
            value_type: "FIXED_VALUE".to_string(),
            value: None,
            rule_id: None,
        }
    }

    pub fn with_abstract_path(mut self, path: &str) -> Self {
        self.abstract_path = path.to_string();
        self
    }

    pub fn fixed_value(mut self, value: AttributeValueJson) -> Self {
        self.value_type = "FIXED_VALUE".to_string();
        self.value = Some(value);
        self.rule_id = None;
        self
    }

    pub fn rule_driven(mut self, rule_id: &str) -> Self {
        self.value_type = "RULE_DRIVEN".to_string();
        self.value = None;
        self.rule_id = Some(rule_id.to_string());
        self
    }

    pub fn just_definition(mut self) -> Self {
        self.value_type = "JUST_DEFINITION".to_string();
        self.value = None;
        self.rule_id = None;
        self
    }

    pub fn build(self) -> CreateAttributeRequest {
        CreateAttributeRequest {
            component_type: self.component_type,
            component_id: self.component_id,
            attribute_name: self.attribute_name,
            abstract_path: self.abstract_path,
            value_type: self.value_type,
            value: self.value,
            rule_id: self.rule_id,
        }
    }
}

// =============================================================================
// Rule Builders
// =============================================================================

/// Builder for CreateRuleRequest
pub struct RuleBuilder {
    rule_type: String,
    input_attributes: Vec<String>,
    output_attributes: Vec<String>,
    display_expression: String,
    expression_json: String,
    description: Option<String>,
    order_index: i32,
}

impl RuleBuilder {
    pub fn new(rule_type: &str) -> Self {
        Self {
            rule_type: rule_type.to_string(),
            input_attributes: Vec::new(),
            output_attributes: Vec::new(),
            display_expression: String::new(),
            expression_json: String::new(),
            description: None,
            order_index: 0,
        }
    }

    /// Set up a simple multiplication rule (builder method)
    /// Takes (input, factor, output) to match test usage
    pub fn multiply(mut self, input: &str, factor: f64, output: &str) -> Self {
        self.input_attributes = vec![input.to_string()];
        self.output_attributes = vec![output.to_string()];
        self.display_expression = format!("{} = {} * {}", output, input, factor);
        self.expression_json = format!(r#"{{"*": [{{"var": "{}"}}, {}]}}"#, input, factor);
        // Only set description if not already set
        if self.description.is_none() {
            self.description = Some(format!("Multiply {} by {}", input, factor));
        }
        self
    }

    /// Set up a simple addition rule (builder method)
    pub fn add(mut self, input: &str, addend: f64, output: &str) -> Self {
        self.input_attributes = vec![input.to_string()];
        self.output_attributes = vec![output.to_string()];
        self.display_expression = format!("{} = {} + {}", output, input, addend);
        self.expression_json = format!(r#"{{"+": [{{"var": "{}"}}, {}]}}"#, input, addend);
        // Only set description if not already set
        if self.description.is_none() {
            self.description = Some(format!("Add {} to {}", addend, input));
        }
        self
    }

    /// Set up a chain rule that uses output from another rule (builder method)
    pub fn chain(mut self, input: &str, operation: &str, operand: f64, output: &str, order: i32) -> Self {
        let (display_op, json_op) = match operation {
            "multiply" | "*" => ("*", "*"),
            "add" | "+" => ("+", "+"),
            "subtract" | "-" => ("-", "-"),
            "divide" | "/" => ("/", "/"),
            _ => ("+", "+"),
        };

        self.input_attributes = vec![input.to_string()];
        self.output_attributes = vec![output.to_string()];
        self.display_expression = format!("{} = {} {} {}", output, input, display_op, operand);
        self.expression_json = format!(r#"{{"{}": [{{"var": "{}"}}, {}]}}"#, json_op, input, operand);
        // Only set description if not already set
        if self.description.is_none() {
            self.description = Some(format!("Chain: {} {} {}", input, display_op, operand));
        }
        self.order_index = order;
        self
    }

    pub fn with_inputs(mut self, inputs: Vec<&str>) -> Self {
        self.input_attributes = inputs.into_iter().map(String::from).collect();
        self
    }

    pub fn with_outputs(mut self, outputs: Vec<&str>) -> Self {
        self.output_attributes = outputs.into_iter().map(String::from).collect();
        self
    }

    pub fn with_display(mut self, display: &str) -> Self {
        self.display_expression = display.to_string();
        self
    }

    pub fn with_expression(mut self, json: &str) -> Self {
        self.expression_json = json.to_string();
        self
    }

    pub fn with_expression_json(mut self, json: serde_json::Value) -> Self {
        self.expression_json = serde_json::to_string(&json).unwrap_or_default();
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.order_index = order;
        self
    }

    pub fn build(self) -> CreateRuleRequest {
        CreateRuleRequest {
            rule_type: self.rule_type,
            input_attributes: self.input_attributes,
            output_attributes: self.output_attributes,
            display_expression: self.display_expression,
            expression_json: self.expression_json,
            description: self.description,
            order_index: self.order_index,
        }
    }
}

// =============================================================================
// Datatype Builders
// =============================================================================

/// Builder for CreateDatatypeRequest
pub struct DatatypeBuilder {
    id: String,
    primitive_type: String,
    description: Option<String>,
    constraints: Option<DatatypeConstraintsJson>,
}

impl DatatypeBuilder {
    pub fn new(id: &str, primitive_type: &str) -> Self {
        Self {
            id: id.to_string(),
            primitive_type: primitive_type.to_string(),
            description: None,
            constraints: None,
        }
    }

    pub fn string(id: &str) -> Self {
        Self::new(id, "STRING")
    }

    pub fn int(id: &str) -> Self {
        Self::new(id, "INT")
    }

    pub fn float(id: &str) -> Self {
        Self::new(id, "FLOAT")
    }

    pub fn decimal(id: &str) -> Self {
        Self::new(id, "DECIMAL")
    }

    pub fn boolean(id: &str) -> Self {
        Self::new(id, "BOOL")
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_min_max(mut self, min: f64, max: f64) -> Self {
        let mut constraints = self.constraints.unwrap_or_default();
        constraints.min = Some(min);
        constraints.max = Some(max);
        self.constraints = Some(constraints);
        self
    }

    pub fn with_length(mut self, min: i32, max: i32) -> Self {
        let mut constraints = self.constraints.unwrap_or_default();
        constraints.min_length = Some(min);
        constraints.max_length = Some(max);
        self.constraints = Some(constraints);
        self
    }

    pub fn with_pattern(mut self, pattern: &str) -> Self {
        let mut constraints = self.constraints.unwrap_or_default();
        constraints.pattern = Some(pattern.to_string());
        self.constraints = Some(constraints);
        self
    }

    pub fn with_precision(mut self, precision: i32, scale: i32) -> Self {
        let mut constraints = self.constraints.unwrap_or_default();
        constraints.precision = Some(precision);
        constraints.scale = Some(scale);
        self.constraints = Some(constraints);
        self
    }

    pub fn build(self) -> CreateDatatypeRequest {
        CreateDatatypeRequest {
            id: self.id,
            primitive_type: self.primitive_type,
            description: self.description,
            constraints: self.constraints,
        }
    }
}

// =============================================================================
// Enumeration Builders
// =============================================================================

/// Builder for CreateEnumerationRequest
pub struct EnumerationBuilder {
    name: String,
    template_type: String,
    values: Vec<String>,
    description: Option<String>,
}

impl EnumerationBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            template_type: "insurance".to_string(), // default
            values: Vec::new(),
            description: None,
        }
    }

    pub fn with_template(mut self, template_type: &str) -> Self {
        self.template_type = template_type.to_string();
        self
    }

    pub fn with_values(mut self, values: Vec<&str>) -> Self {
        self.values = values.into_iter().map(String::from).collect();
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn build(self) -> CreateEnumerationRequest {
        CreateEnumerationRequest {
            name: self.name,
            template_type: self.template_type,
            values: self.values,
            description: self.description,
        }
    }
}

// =============================================================================
// Functionality Builders
// =============================================================================

/// Builder for CreateFunctionalityRequest
pub struct FunctionalityBuilder {
    name: String,
    description: String,
    immutable: bool,
    required_attributes: Vec<RequiredAttributeInput>,
}

impl FunctionalityBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: format!("Functionality: {}", name),
            immutable: false,
            required_attributes: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn immutable(mut self) -> Self {
        self.immutable = true;
        self
    }

    pub fn with_required_attribute(mut self, abstract_path: &str, description: &str) -> Self {
        self.required_attributes.push(RequiredAttributeInput {
            abstract_path: abstract_path.to_string(),
            description: description.to_string(),
        });
        self
    }

    pub fn build(self) -> CreateFunctionalityRequest {
        CreateFunctionalityRequest {
            name: self.name,
            description: self.description,
            immutable: self.immutable,
            required_attributes: self.required_attributes,
        }
    }
}

// =============================================================================
// Evaluation Builders
// =============================================================================

/// Builder for EvaluateRequest
pub struct EvaluationBuilder {
    product_id: String,
    input_data: HashMap<String, AttributeValueJson>,
    rule_ids: Vec<String>,
    include_intermediate: bool,
    debug_mode: bool,
}

impl EvaluationBuilder {
    pub fn new(product_id: &str) -> Self {
        Self {
            product_id: product_id.to_string(),
            input_data: HashMap::new(),
            rule_ids: Vec::new(),
            include_intermediate: false,
            debug_mode: false,
        }
    }

    pub fn with_input(mut self, key: &str, value: AttributeValueJson) -> Self {
        self.input_data.insert(key.to_string(), value);
        self
    }

    pub fn with_int_input(mut self, key: &str, value: i64) -> Self {
        self.input_data.insert(key.to_string(), AttributeValueJson::Int { value });
        self
    }

    pub fn with_float_input(mut self, key: &str, value: f64) -> Self {
        self.input_data.insert(key.to_string(), AttributeValueJson::Float { value });
        self
    }

    pub fn with_string_input(mut self, key: &str, value: &str) -> Self {
        self.input_data.insert(key.to_string(), AttributeValueJson::String { value: value.to_string() });
        self
    }

    pub fn with_bool_input(mut self, key: &str, value: bool) -> Self {
        self.input_data.insert(key.to_string(), AttributeValueJson::Bool { value });
        self
    }

    pub fn with_rules(mut self, rule_ids: Vec<&str>) -> Self {
        self.rule_ids = rule_ids.into_iter().map(String::from).collect();
        self
    }

    pub fn include_intermediate(mut self) -> Self {
        self.include_intermediate = true;
        self
    }

    pub fn debug(mut self) -> Self {
        self.debug_mode = true;
        self
    }

    pub fn build(self) -> EvaluateRequest {
        let options = if self.include_intermediate || self.debug_mode {
            Some(EvaluationOptionsJson {
                include_intermediate_results: self.include_intermediate,
                max_execution_time_ms: 30000,
                debug_mode: self.debug_mode,
            })
        } else {
            None
        };

        EvaluateRequest {
            product_id: self.product_id,
            input_data: self.input_data,
            rule_ids: self.rule_ids,
            options,
        }
    }
}

/// Builder for BatchEvaluateRequest
pub struct BatchEvaluationBuilder {
    product_id: String,
    requests: Vec<BatchRequestItem>,
    include_intermediate: bool,
}

impl BatchEvaluationBuilder {
    pub fn new(product_id: &str) -> Self {
        Self {
            product_id: product_id.to_string(),
            requests: Vec::new(),
            include_intermediate: false,
        }
    }

    pub fn add_request(mut self, request_id: &str, inputs: HashMap<String, AttributeValueJson>) -> Self {
        self.requests.push(BatchRequestItem {
            request_id: request_id.to_string(),
            input_data: inputs,
        });
        self
    }

    pub fn add_simple_request(mut self, request_id: &str, key: &str, value: f64) -> Self {
        let mut inputs = HashMap::new();
        inputs.insert(key.to_string(), AttributeValueJson::Float { value });
        self.requests.push(BatchRequestItem {
            request_id: request_id.to_string(),
            input_data: inputs,
        });
        self
    }

    pub fn include_intermediate(mut self) -> Self {
        self.include_intermediate = true;
        self
    }

    pub fn build(self) -> BatchEvaluateRequest {
        let options = if self.include_intermediate {
            Some(EvaluationOptionsJson {
                include_intermediate_results: true,
                max_execution_time_ms: 30000,
                debug_mode: false,
            })
        } else {
            None
        };

        BatchEvaluateRequest {
            product_id: self.product_id,
            requests: self.requests,
            options,
        }
    }
}

// =============================================================================
// Value Helpers
// =============================================================================

/// Helper functions for creating AttributeValueJson
pub mod values {
    use super::AttributeValueJson;
    use std::collections::HashMap;

    pub fn null() -> AttributeValueJson {
        AttributeValueJson::Null
    }

    pub fn bool_val(v: bool) -> AttributeValueJson {
        AttributeValueJson::Bool { value: v }
    }

    pub fn boolean(v: bool) -> AttributeValueJson {
        bool_val(v)
    }

    pub fn int(v: i64) -> AttributeValueJson {
        AttributeValueJson::Int { value: v }
    }

    pub fn float(v: f64) -> AttributeValueJson {
        AttributeValueJson::Float { value: v }
    }

    pub fn string(v: &str) -> AttributeValueJson {
        AttributeValueJson::String { value: v.to_string() }
    }

    pub fn decimal(v: &str) -> AttributeValueJson {
        AttributeValueJson::Decimal { value: v.to_string() }
    }

    pub fn array(items: Vec<AttributeValueJson>) -> AttributeValueJson {
        AttributeValueJson::Array { value: items }
    }

    pub fn object(fields: HashMap<String, AttributeValueJson>) -> AttributeValueJson {
        AttributeValueJson::Object { value: fields }
    }
}
