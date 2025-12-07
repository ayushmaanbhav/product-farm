//! Attribute conversions between proto and core types
//!
//! Includes both abstract and concrete attribute converters.

use product_farm_core::{
    AbstractAttribute as CoreAbstractAttribute, AbstractAttributeRelatedAttribute,
    AbstractAttributeTag, Attribute as CoreAttribute, AttributeDisplayName,
    AttributeRelationshipType, AttributeValueType as CoreAttributeValueType,
    DisplayNameFormat as CoreDisplayNameFormat,
};

use crate::grpc::proto;

use super::values::core_to_proto_value;

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
// CONCRETE ATTRIBUTE CONVERSIONS
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
