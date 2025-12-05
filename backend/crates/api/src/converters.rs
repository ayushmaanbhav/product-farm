//! Type converters between proto and core types
//!
//! Provides bidirectional conversion between gRPC proto types
//! and internal core domain types.

use product_farm_core::{
    AbstractAttribute as CoreAbstractAttribute,
    AbstractAttributeRelatedAttribute, AbstractAttributeTag,
    Attribute as CoreAttribute, AttributeDisplayName, AttributeRelationshipType,
    AttributeValueType as CoreAttributeValueType, DataType as CoreDataType,
    DataTypeConstraints as CoreDataTypeConstraints,
    DisplayNameFormat as CoreDisplayNameFormat,
    FunctionalityRequiredAttribute, PrimitiveType as CorePrimitiveType,
    Product as CoreProduct, ProductFunctionality as CoreProductFunctionality,
    ProductFunctionalityStatus as CoreFunctionalityStatus,
    ProductStatus as CoreProductStatus, ProductTemplateEnumeration as CoreTemplateEnumeration,
    Rule as CoreRule, RuleId, RuleInputAttribute, RuleOutputAttribute, Value as CoreValue,
};

use crate::grpc::proto;

// ============================================================================
// VALUE CONVERSIONS
// ============================================================================

/// Convert proto Value to core Value (fallible version)
/// Returns an error if decimal parsing fails or other validation fails
pub fn try_proto_to_core_value(v: &proto::Value) -> Result<CoreValue, tonic::Status> {
    match &v.value {
        Some(proto::value::Value::NullValue(_)) => Ok(CoreValue::Null),
        Some(proto::value::Value::BoolValue(b)) => Ok(CoreValue::Bool(*b)),
        Some(proto::value::Value::IntValue(i)) => Ok(CoreValue::Int(*i)),
        Some(proto::value::Value::FloatValue(f)) => Ok(CoreValue::Float(*f)),
        Some(proto::value::Value::StringValue(s)) => Ok(CoreValue::String(s.clone())),
        Some(proto::value::Value::DecimalValue(s)) => {
            s.parse()
                .map(CoreValue::Decimal)
                .map_err(|_| tonic::Status::invalid_argument(format!("Invalid decimal value: '{}'", s)))
        }
        Some(proto::value::Value::ArrayValue(arr)) => {
            let values: Result<Vec<_>, _> = arr.values.iter().map(try_proto_to_core_value).collect();
            Ok(CoreValue::Array(values?))
        }
        Some(proto::value::Value::ObjectValue(obj)) => {
            let fields: Result<Vec<_>, _> = obj.fields
                .iter()
                .map(|(k, v)| try_proto_to_core_value(v).map(|cv| (k.clone(), cv)))
                .collect();
            Ok(CoreValue::Object(fields?.into_iter().collect()))
        }
        None => Ok(CoreValue::Null),
    }
}

/// Convert proto Value to core Value
/// This is a convenience wrapper for contexts where errors cannot be returned.
/// For gRPC handlers, prefer try_proto_to_core_value for proper error handling.
pub fn proto_to_core_value(v: &proto::Value) -> CoreValue {
    // Use unwrap_or_default for backwards compatibility, but log warnings
    try_proto_to_core_value(v).unwrap_or_else(|e| {
        tracing::warn!("Failed to convert proto value: {}", e);
        CoreValue::Null
    })
}

/// Convert core Value to proto Value
pub fn core_to_proto_value(v: &CoreValue) -> proto::Value {
    let value = match v {
        CoreValue::Null => proto::value::Value::NullValue(true),
        CoreValue::Bool(b) => proto::value::Value::BoolValue(*b),
        CoreValue::Int(i) => proto::value::Value::IntValue(*i),
        CoreValue::Float(f) => proto::value::Value::FloatValue(*f),
        CoreValue::String(s) => proto::value::Value::StringValue(s.clone()),
        CoreValue::Decimal(d) => proto::value::Value::DecimalValue(d.to_string()),
        CoreValue::Array(arr) => proto::value::Value::ArrayValue(proto::ArrayValue {
            values: arr.iter().map(core_to_proto_value).collect(),
        }),
        CoreValue::Object(obj) => proto::value::Value::ObjectValue(proto::ObjectValue {
            fields: obj
                .iter()
                .map(|(k, v)| (k.clone(), core_to_proto_value(v)))
                .collect(),
        }),
    };
    proto::Value { value: Some(value) }
}

// ============================================================================
// PRODUCT CONVERSIONS
// ============================================================================

pub fn core_to_proto_product(p: &CoreProduct) -> proto::Product {
    proto::Product {
        id: p.id.as_str().to_string(),
        name: p.name.clone(),
        description: p.description.clone().unwrap_or_default(),
        template_type: p.template_type.as_str().to_string(),
        status: core_to_proto_product_status(&p.status) as i32,
        parent_product_id: p.parent_product_id.as_ref().map(|id| id.as_str().to_string()),
        effective_from: p.effective_from.timestamp(),
        expiry_at: p.expiry_at.map(|dt| dt.timestamp()),
        created_at: p.created_at.timestamp(),
        updated_at: p.updated_at.timestamp(),
        version: p.version as i64,
    }
}

pub fn core_to_proto_product_status(s: &CoreProductStatus) -> proto::ProductStatus {
    match s {
        CoreProductStatus::Draft => proto::ProductStatus::Draft,
        CoreProductStatus::PendingApproval => proto::ProductStatus::PendingApproval,
        CoreProductStatus::Active => proto::ProductStatus::Active,
        CoreProductStatus::Discontinued => proto::ProductStatus::Discontinued,
    }
}

pub fn proto_to_core_product_status(s: i32) -> CoreProductStatus {
    match proto::ProductStatus::try_from(s) {
        Ok(proto::ProductStatus::Draft) => CoreProductStatus::Draft,
        Ok(proto::ProductStatus::PendingApproval) => CoreProductStatus::PendingApproval,
        Ok(proto::ProductStatus::Active) => CoreProductStatus::Active,
        Ok(proto::ProductStatus::Discontinued) => CoreProductStatus::Discontinued,
        _ => CoreProductStatus::Draft,
    }
}

// ============================================================================
// ABSTRACT ATTRIBUTE CONVERSIONS
// ============================================================================

pub fn core_to_proto_abstract_attribute(a: &CoreAbstractAttribute) -> proto::AbstractAttribute {
    proto::AbstractAttribute {
        abstract_path: a.abstract_path.as_str().to_string(),
        product_id: a.product_id.as_str().to_string(),
        component_type: a.component_type.clone(),
        component_id: a.component_id.clone(),
        datatype_id: a.datatype_id.as_str().to_string(),
        enum_name: a.enum_name.clone(),
        constraint_expression: a
            .constraint_expression
            .as_ref()
            .map(|c| serde_json::to_string(c).unwrap_or_default()),
        immutable: a.immutable,
        description: a.description.clone(),
        display_names: a.display_names.iter().map(core_to_proto_display_name).collect(),
        tags: a.tags.iter().map(core_to_proto_tag).collect(),
        related_attributes: a
            .related_attributes
            .iter()
            .map(core_to_proto_related_attr)
            .collect(),
    }
}

pub fn core_to_proto_display_name(d: &AttributeDisplayName) -> proto::DisplayName {
    proto::DisplayName {
        display_name: d.display_name.clone(),
        format: core_to_proto_display_format(&d.display_name_format) as i32,
        order: d.order,
    }
}

pub fn core_to_proto_display_format(f: &CoreDisplayNameFormat) -> proto::DisplayNameFormat {
    match f {
        CoreDisplayNameFormat::System => proto::DisplayNameFormat::System,
        CoreDisplayNameFormat::Human => proto::DisplayNameFormat::Human,
        CoreDisplayNameFormat::Original => proto::DisplayNameFormat::Original,
    }
}

pub fn proto_to_core_display_format(f: i32) -> CoreDisplayNameFormat {
    match proto::DisplayNameFormat::try_from(f) {
        Ok(proto::DisplayNameFormat::System) => CoreDisplayNameFormat::System,
        Ok(proto::DisplayNameFormat::Human) => CoreDisplayNameFormat::Human,
        Ok(proto::DisplayNameFormat::Original) => CoreDisplayNameFormat::Original,
        _ => CoreDisplayNameFormat::System,
    }
}

pub fn core_to_proto_tag(t: &AbstractAttributeTag) -> proto::Tag {
    proto::Tag {
        name: t.tag.as_str().to_string(),
        order: t.order,
    }
}

pub fn core_to_proto_related_attr(r: &AbstractAttributeRelatedAttribute) -> proto::RelatedAttribute {
    proto::RelatedAttribute {
        reference_abstract_path: r.reference_abstract_path.as_str().to_string(),
        relationship: core_to_proto_relationship_type(&r.relationship) as i32,
        order: r.order,
    }
}

pub fn core_to_proto_relationship_type(
    r: &AttributeRelationshipType,
) -> proto::AttributeRelationshipType {
    match r {
        AttributeRelationshipType::Enumeration => proto::AttributeRelationshipType::Enumeration,
        AttributeRelationshipType::KeyEnumeration => proto::AttributeRelationshipType::KeyEnumeration,
        AttributeRelationshipType::ValueEnumeration => {
            proto::AttributeRelationshipType::ValueEnumeration
        }
    }
}

// ============================================================================
// ATTRIBUTE CONVERSIONS
// ============================================================================

pub fn core_to_proto_attribute(a: &CoreAttribute) -> proto::Attribute {
    proto::Attribute {
        path: a.path.as_str().to_string(),
        abstract_path: a.abstract_path.as_str().to_string(),
        product_id: a.product_id.as_str().to_string(),
        value_type: core_to_proto_value_type(&a.value_type) as i32,
        value: a.value.as_ref().map(core_to_proto_value),
        rule_id: a.rule_id.as_ref().map(|id| id.to_string()),
        display_names: a.display_names.iter().map(core_to_proto_display_name).collect(),
    }
}

pub fn core_to_proto_value_type(t: &CoreAttributeValueType) -> proto::AttributeValueType {
    match t {
        CoreAttributeValueType::FixedValue => proto::AttributeValueType::FixedValue,
        CoreAttributeValueType::RuleDriven => proto::AttributeValueType::RuleDriven,
        CoreAttributeValueType::JustDefinition => proto::AttributeValueType::JustDefinition,
    }
}

pub fn proto_to_core_value_type(t: i32) -> CoreAttributeValueType {
    match proto::AttributeValueType::try_from(t) {
        Ok(proto::AttributeValueType::FixedValue) => CoreAttributeValueType::FixedValue,
        Ok(proto::AttributeValueType::RuleDriven) => CoreAttributeValueType::RuleDriven,
        Ok(proto::AttributeValueType::JustDefinition) => CoreAttributeValueType::JustDefinition,
        _ => CoreAttributeValueType::FixedValue,
    }
}

// ============================================================================
// RULE CONVERSIONS
// ============================================================================

pub fn core_to_proto_rule(r: &CoreRule) -> proto::Rule {
    proto::Rule {
        id: r.id.to_string(),
        product_id: r.product_id.as_str().to_string(),
        rule_type: r.rule_type.clone(),
        input_attributes: r
            .input_attributes
            .iter()
            .map(|a| proto::RuleAttribute {
                path: a.path.as_str().to_string(),
                order: a.order,
            })
            .collect(),
        output_attributes: r
            .output_attributes
            .iter()
            .map(|a| proto::RuleAttribute {
                path: a.path.as_str().to_string(),
                order: a.order,
            })
            .collect(),
        display_expression: r.display_expression.clone(),
        display_expression_version: r.display_expression_version.clone(),
        expression_json: r.compiled_expression.clone(),
        description: r.description.clone(),
        enabled: r.enabled,
        order_index: r.order_index,
    }
}

pub fn proto_to_core_rule(r: &proto::Rule) -> Result<CoreRule, String> {
    // Validate JSON
    let _: serde_json::Value = serde_json::from_str(&r.expression_json)
        .map_err(|e| format!("Invalid expression JSON: {}", e))?;

    let rule_id = if r.id.is_empty() {
        RuleId::new()
    } else {
        RuleId::from_string(&r.id)
    };

    let mut rule = CoreRule::new(
        r.product_id.as_str(),
        r.rule_type.as_str(),
        r.expression_json.as_str(),
    )
    .with_id(rule_id.clone())
    .with_display(r.display_expression.as_str())
    .with_display_version(r.display_expression_version.as_str())
    .with_order(r.order_index)
    .with_enabled(r.enabled);

    if let Some(desc) = &r.description {
        rule = rule.with_description(desc.as_str());
    }

    // Set inputs
    rule.input_attributes = r
        .input_attributes
        .iter()
        .map(|a| RuleInputAttribute::new(rule_id.clone(), a.path.as_str(), a.order))
        .collect();

    // Set outputs
    rule.output_attributes = r
        .output_attributes
        .iter()
        .map(|a| RuleOutputAttribute::new(rule_id.clone(), a.path.as_str(), a.order))
        .collect();

    Ok(rule)
}

// ============================================================================
// DATATYPE CONVERSIONS
// ============================================================================

pub fn core_to_proto_datatype(d: &CoreDataType) -> proto::Datatype {
    proto::Datatype {
        id: d.id.as_str().to_string(),
        primitive_type: core_to_proto_primitive_type(&d.primitive_type) as i32,
        description: d.description.clone(),
        constraints: d.constraints.as_ref().map(core_to_proto_constraints),
    }
}

pub fn core_to_proto_primitive_type(t: &CorePrimitiveType) -> proto::PrimitiveType {
    match t {
        CorePrimitiveType::String => proto::PrimitiveType::String,
        CorePrimitiveType::Int => proto::PrimitiveType::Int,
        CorePrimitiveType::Float => proto::PrimitiveType::Float,
        CorePrimitiveType::Decimal => proto::PrimitiveType::Decimal,
        CorePrimitiveType::Bool => proto::PrimitiveType::Bool,
        CorePrimitiveType::Datetime => proto::PrimitiveType::Datetime,
        CorePrimitiveType::Enum => proto::PrimitiveType::Enum,
        CorePrimitiveType::Array => proto::PrimitiveType::Array,
        CorePrimitiveType::Object => proto::PrimitiveType::Object,
        _ => proto::PrimitiveType::Unspecified,
    }
}

pub fn proto_to_core_primitive_type(t: i32) -> CorePrimitiveType {
    match proto::PrimitiveType::try_from(t) {
        Ok(proto::PrimitiveType::String) => CorePrimitiveType::String,
        Ok(proto::PrimitiveType::Int) => CorePrimitiveType::Int,
        Ok(proto::PrimitiveType::Float) => CorePrimitiveType::Float,
        Ok(proto::PrimitiveType::Decimal) => CorePrimitiveType::Decimal,
        Ok(proto::PrimitiveType::Bool) => CorePrimitiveType::Bool,
        Ok(proto::PrimitiveType::Datetime) => CorePrimitiveType::Datetime,
        Ok(proto::PrimitiveType::Enum) => CorePrimitiveType::Enum,
        Ok(proto::PrimitiveType::Array) => CorePrimitiveType::Array,
        Ok(proto::PrimitiveType::Object) => CorePrimitiveType::Object,
        _ => CorePrimitiveType::String,
    }
}

pub fn core_to_proto_constraints(c: &CoreDataTypeConstraints) -> proto::DatatypeConstraints {
    proto::DatatypeConstraints {
        min: c.min,
        max: c.max,
        min_length: c.min_length.map(|v| v as i32),
        max_length: c.max_length.map(|v| v as i32),
        pattern: c.pattern.clone(),
        precision: c.precision.map(|v| v as i32),
        scale: c.scale.map(|v| v as i32),
        constraint_rule_expression: c.constraint_rule_expression.clone(),
        constraint_error_message: c.constraint_error_message.clone(),
    }
}

pub fn proto_to_core_constraints(c: &proto::DatatypeConstraints) -> CoreDataTypeConstraints {
    CoreDataTypeConstraints {
        min: c.min,
        max: c.max,
        min_length: c.min_length.map(|v| v as usize),
        max_length: c.max_length.map(|v| v as usize),
        pattern: c.pattern.clone(),
        precision: c.precision.map(|v| v as u8),
        scale: c.scale.map(|v| v as u8),
        constraint_rule_expression: c.constraint_rule_expression.clone(),
        constraint_error_message: c.constraint_error_message.clone(),
    }
}

// ============================================================================
// FUNCTIONALITY CONVERSIONS
// ============================================================================

pub fn core_to_proto_functionality(f: &CoreProductFunctionality) -> proto::ProductFunctionality {
    proto::ProductFunctionality {
        id: f.id.as_str().to_string(),
        name: f.name.clone(),
        product_id: f.product_id.as_str().to_string(),
        immutable: f.immutable,
        description: f.description.clone(),
        required_attributes: f
            .required_attributes
            .iter()
            .map(core_to_proto_required_attr)
            .collect(),
        status: core_to_proto_functionality_status(&f.status) as i32,
    }
}

pub fn core_to_proto_required_attr(r: &FunctionalityRequiredAttribute) -> proto::RequiredAttribute {
    proto::RequiredAttribute {
        abstract_path: r.abstract_path.as_str().to_string(),
        description: r.description.clone(),
        order: r.order,
    }
}

pub fn core_to_proto_functionality_status(
    s: &CoreFunctionalityStatus,
) -> proto::FunctionalityStatus {
    match s {
        CoreFunctionalityStatus::Draft => proto::FunctionalityStatus::Draft,
        CoreFunctionalityStatus::PendingApproval => proto::FunctionalityStatus::PendingApproval,
        CoreFunctionalityStatus::Active => proto::FunctionalityStatus::Active,
    }
}

pub fn proto_to_core_functionality_status(s: i32) -> CoreFunctionalityStatus {
    match proto::FunctionalityStatus::try_from(s) {
        Ok(proto::FunctionalityStatus::Draft) => CoreFunctionalityStatus::Draft,
        Ok(proto::FunctionalityStatus::PendingApproval) => CoreFunctionalityStatus::PendingApproval,
        Ok(proto::FunctionalityStatus::Active) => CoreFunctionalityStatus::Active,
        _ => CoreFunctionalityStatus::Draft,
    }
}

// ============================================================================
// TEMPLATE ENUMERATION CONVERSIONS
// ============================================================================

pub fn core_to_proto_enumeration(e: &CoreTemplateEnumeration) -> proto::TemplateEnumeration {
    proto::TemplateEnumeration {
        id: e.id.as_str().to_string(),
        name: e.name.clone(),
        template_type: e.template_type.as_str().to_string(),
        values: e.values.iter().cloned().collect(),
        description: e.description.clone(),
    }
}
