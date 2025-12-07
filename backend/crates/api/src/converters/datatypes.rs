//! Datatype and template enumeration conversions between proto and core types

use product_farm_core::{
    DataType as CoreDataType, DataTypeConstraints as CoreDataTypeConstraints,
    PrimitiveType as CorePrimitiveType, ProductTemplateEnumeration as CoreTemplateEnumeration,
};

use crate::grpc::proto;

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
