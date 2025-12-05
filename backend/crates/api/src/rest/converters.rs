//! Type converters between Core types and REST JSON types

use product_farm_core::{
    AbstractAttribute, AbstractAttributeTag, Attribute, AttributeDisplayName,
    DataType, Product, ProductFunctionality, ProductTemplateEnumeration, Rule,
    Value, FunctionalityRequiredAttribute,
};

use super::types::*;

// =============================================================================
// PRODUCT CONVERTERS
// =============================================================================

impl From<&Product> for ProductResponse {
    fn from(p: &Product) -> Self {
        Self {
            id: p.id.as_str().to_string(),
            name: p.name.clone(),
            description: p.description.clone().unwrap_or_default(),
            template_type: p.template_type.as_str().to_string(),
            status: format!("{:?}", p.status).to_uppercase(),
            parent_product_id: p.parent_product_id.as_ref().map(|id| id.as_str().to_string()),
            effective_from: p.effective_from.timestamp(),
            expiry_at: p.expiry_at.map(|dt| dt.timestamp()),
            created_at: p.created_at.timestamp(),
            updated_at: p.updated_at.timestamp(),
            version: p.version as i64,
        }
    }
}

// =============================================================================
// ABSTRACT ATTRIBUTE CONVERTERS
// =============================================================================

impl From<&AbstractAttribute> for AbstractAttributeResponse {
    fn from(a: &AbstractAttribute) -> Self {
        // Parse path to extract attribute name
        let parsed = a.abstract_path.parse();
        let attribute_name = parsed
            .as_ref()
            .map(|p| p.attribute_name.clone())
            .unwrap_or_else(|| a.abstract_path.as_str().split(':').next_back().unwrap_or("").to_string());

        Self {
            abstract_path: a.abstract_path.as_str().to_string(),
            product_id: a.product_id.as_str().to_string(),
            component_type: a.component_type.clone(),
            component_id: a.component_id.clone(),
            attribute_name,
            datatype_id: a.datatype_id.as_str().to_string(),
            enum_name: a.enum_name.clone(),
            description: a.description.clone().unwrap_or_default(),
            constraint_expression: a.constraint_expression.as_ref().map(|v| v.to_string()),
            display_expression: String::new(), // TODO: Generate from display names
            display_names: a.display_names.iter().map(|dn| dn.into()).collect(),
            tags: a.tags.iter().map(|t| t.into()).collect(),
            related_attributes: a.related_attributes.iter().map(|r| {
                RelatedAttributeResponse {
                    relationship_type: format!("{:?}", r.relationship).to_uppercase(),
                    related_path: r.reference_abstract_path.as_str().to_string(),
                    order_index: r.order,
                }
            }).collect(),
            immutable: a.immutable,
        }
    }
}

impl From<&AttributeDisplayName> for DisplayNameResponse {
    fn from(dn: &AttributeDisplayName) -> Self {
        Self {
            name: dn.display_name.clone(),
            format: format!("{:?}", dn.display_name_format).to_uppercase(),
            order_index: dn.order,
        }
    }
}

impl From<&AbstractAttributeTag> for AttributeTagResponse {
    fn from(t: &AbstractAttributeTag) -> Self {
        Self {
            name: t.tag.to_string(),
            order_index: t.order,
        }
    }
}

// =============================================================================
// CONCRETE ATTRIBUTE CONVERTERS
// =============================================================================

impl From<&Attribute> for AttributeResponse {
    fn from(a: &Attribute) -> Self {
        // Parse path to extract components
        let parsed = a.path.parse();
        let (component_type, component_id, attribute_name) = parsed
            .map(|p| (p.component_type, p.component_id, p.attribute_name))
            .unwrap_or_else(|| ("".to_string(), "".to_string(), "".to_string()));

        Self {
            path: a.path.as_str().to_string(),
            abstract_path: a.abstract_path.as_str().to_string(),
            product_id: a.product_id.as_str().to_string(),
            component_type,
            component_id,
            attribute_name,
            value_type: format!("{:?}", a.value_type).to_uppercase(),
            value: a.value.as_ref().map(|v| v.into()),
            rule_id: a.rule_id.as_ref().map(|id| id.to_string()),
            created_at: a.created_at.timestamp(),
            updated_at: a.updated_at.timestamp(),
        }
    }
}

impl From<&Value> for AttributeValueJson {
    fn from(v: &Value) -> Self {
        match v {
            Value::Null => AttributeValueJson::Null,
            Value::Bool(b) => AttributeValueJson::Bool { value: *b },
            Value::Int(i) => AttributeValueJson::Int { value: *i },
            Value::Float(f) => AttributeValueJson::Float { value: *f },
            Value::String(s) => AttributeValueJson::String { value: s.clone() },
            Value::Decimal(d) => AttributeValueJson::Decimal { value: d.to_string() },
            Value::Array(arr) => AttributeValueJson::Array {
                value: arr.iter().map(|v| v.into()).collect(),
            },
            Value::Object(obj) => AttributeValueJson::Object {
                value: obj.iter().map(|(k, v)| (k.clone(), v.into())).collect(),
            },
        }
    }
}

impl From<&AttributeValueJson> for Value {
    fn from(v: &AttributeValueJson) -> Self {
        match v {
            AttributeValueJson::Null => Value::Null,
            AttributeValueJson::Bool { value } => Value::Bool(*value),
            AttributeValueJson::Int { value } => Value::Int(*value),
            AttributeValueJson::Float { value } => Value::Float(*value),
            AttributeValueJson::String { value } => Value::String(value.clone()),
            AttributeValueJson::Decimal { value } => {
                // Parse decimal, fall back to string if invalid to preserve data
                value.parse()
                    .map(Value::Decimal)
                    .unwrap_or_else(|_| Value::String(value.clone()))
            }
            AttributeValueJson::Array { value } => {
                Value::Array(value.iter().map(|v| v.into()).collect())
            }
            AttributeValueJson::Object { value } => {
                Value::Object(value.iter().map(|(k, v)| (k.clone(), v.into())).collect())
            }
        }
    }
}

// =============================================================================
// RULE CONVERTERS
// =============================================================================

impl From<&Rule> for RuleResponse {
    fn from(r: &Rule) -> Self {
        Self {
            id: r.id.to_string(),
            product_id: r.product_id.as_str().to_string(),
            rule_type: r.rule_type.clone(),
            display_expression: r.display_expression.clone(),
            compiled_expression: r.compiled_expression.to_string(),
            description: r.description.clone(),
            enabled: r.enabled,
            order_index: r.order_index,
            input_attributes: r.input_attributes.iter().map(|a| {
                RuleAttributeResponse {
                    rule_id: a.rule_id.to_string(),
                    attribute_path: a.path.as_str().to_string(),
                    order_index: a.order,
                }
            }).collect(),
            output_attributes: r.output_attributes.iter().map(|a| {
                RuleAttributeResponse {
                    rule_id: a.rule_id.to_string(),
                    attribute_path: a.path.as_str().to_string(),
                    order_index: a.order,
                }
            }).collect(),
            created_at: r.created_at.timestamp(),
            updated_at: r.updated_at.timestamp(),
        }
    }
}

// =============================================================================
// DATATYPE CONVERTERS
// =============================================================================

impl From<&DataType> for DatatypeResponse {
    fn from(d: &DataType) -> Self {
        Self {
            id: d.id.to_string(),
            name: d.id.to_string(), // Use ID as name for now
            primitive_type: format!("{:?}", d.primitive_type).to_uppercase(),
            constraints: DatatypeConstraintsJson {
                min: d.constraints.as_ref().and_then(|c| c.min),
                max: d.constraints.as_ref().and_then(|c| c.max),
                min_length: d.constraints.as_ref().and_then(|c| c.min_length.map(|v| v as i32)),
                max_length: d.constraints.as_ref().and_then(|c| c.max_length.map(|v| v as i32)),
                pattern: d.constraints.as_ref().and_then(|c| c.pattern.clone()),
                precision: d.constraints.as_ref().and_then(|c| c.precision.map(|v| v as i32)),
                scale: d.constraints.as_ref().and_then(|c| c.scale.map(|v| v as i32)),
                constraint_rule_expression: d.constraints.as_ref().and_then(|c| c.constraint_rule_expression.clone()),
                constraint_error_message: d.constraints.as_ref().and_then(|c| c.constraint_error_message.clone()),
            },
            description: d.description.clone(),
        }
    }
}

// =============================================================================
// FUNCTIONALITY CONVERTERS
// =============================================================================

impl From<&ProductFunctionality> for FunctionalityResponse {
    fn from(f: &ProductFunctionality) -> Self {
        Self {
            id: f.id.as_str().to_string(),
            product_id: f.product_id.as_str().to_string(),
            name: f.name.clone(),
            display_name: f.name.clone(), // Use name as display name
            description: f.description.clone(),
            status: format!("{:?}", f.status).to_uppercase(),
            immutable: f.immutable,
            required_attributes: f.required_attributes.iter().map(|r| r.into()).collect(),
            created_at: f.created_at.timestamp(),
            updated_at: f.updated_at.timestamp(),
        }
    }
}

impl From<&FunctionalityRequiredAttribute> for RequiredAttributeResponse {
    fn from(r: &FunctionalityRequiredAttribute) -> Self {
        Self {
            functionality_id: r.functionality_id.as_str().to_string(),
            abstract_path: r.abstract_path.as_str().to_string(),
            description: r.description.clone(),
            order_index: r.order,
        }
    }
}

// =============================================================================
// TEMPLATE ENUMERATION CONVERTERS
// =============================================================================

impl From<&ProductTemplateEnumeration> for EnumerationResponse {
    fn from(e: &ProductTemplateEnumeration) -> Self {
        Self {
            id: e.id.as_str().to_string(),
            name: e.name.clone(),
            template_type: e.template_type.as_str().to_string(),
            values: e.values.iter().cloned().collect(),
            description: e.description.clone(),
        }
    }
}
