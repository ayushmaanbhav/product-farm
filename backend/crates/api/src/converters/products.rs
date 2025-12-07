//! Product and functionality conversions between proto and core types

use product_farm_core::{
    FunctionalityRequiredAttribute, Product as CoreProduct,
    ProductFunctionality as CoreProductFunctionality,
    ProductFunctionalityStatus as CoreFunctionalityStatus, ProductStatus as CoreProductStatus,
};

use crate::grpc::proto;

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
