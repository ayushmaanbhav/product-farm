//! ProductTemplateGrpcService - Template enumeration management
//!
//! Provides create, read, update, delete, list, value management,
//! and usage tracking endpoints for template enumerations.

use tonic::{Request, Response, Status};

use product_farm_core::{ProductTemplateEnumeration, TemplateEnumerationId};

use crate::converters::core_to_proto_enumeration;
use crate::store::SharedStore;

use super::helpers::parse_page_token;
use super::proto;

/// gRPC service for template enumeration management
pub struct ProductTemplateGrpcService {
    store: SharedStore,
}

impl ProductTemplateGrpcService {
    pub fn new(store: SharedStore) -> Self {
        Self { store }
    }
}

#[tonic::async_trait]
impl proto::product_template_service_server::ProductTemplateService for ProductTemplateGrpcService {
    async fn create_enumeration(
        &self,
        request: Request<proto::CreateEnumerationRequest>,
    ) -> Result<Response<proto::TemplateEnumeration>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // Generate ID
        let id = TemplateEnumerationId::new(format!("{}:{}", req.template_type, req.name));

        // Create enumeration
        let mut enumeration = ProductTemplateEnumeration::new(
            id.clone(),
            req.name.as_str(),
            req.template_type.as_str(),
        )
        .with_values(req.values.iter().cloned());

        if let Some(desc) = req.description {
            enumeration = enumeration.with_description(desc);
        }

        // Validate
        enumeration
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let proto_enum = core_to_proto_enumeration(&enumeration);
        let id_str = id.as_str().to_string();

        // Use entry API to check and insert atomically
        use hashbrown::hash_map::Entry;
        match store.enumerations.entry(id_str.clone()) {
            Entry::Occupied(_) => {
                return Err(Status::already_exists(format!(
                    "Enumeration '{}' already exists for template '{}'",
                    req.name, req.template_type
                )));
            }
            Entry::Vacant(entry) => {
                entry.insert(enumeration);
            }
        }
        store
            .enums_by_template
            .entry(req.template_type)
            .or_default()
            .push(id_str);

        Ok(Response::new(proto_enum))
    }

    async fn get_enumeration(
        &self,
        request: Request<proto::GetEnumerationRequest>,
    ) -> Result<Response<proto::TemplateEnumeration>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        store
            .enumerations
            .get(&req.id)
            .map(|e| Response::new(core_to_proto_enumeration(e)))
            .ok_or_else(|| Status::not_found(format!("Enumeration '{}' not found", req.id)))
    }

    async fn update_enumeration(
        &self,
        request: Request<proto::UpdateEnumerationRequest>,
    ) -> Result<Response<proto::TemplateEnumeration>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        let enumeration = store
            .enumerations
            .get_mut(&req.id)
            .ok_or_else(|| Status::not_found(format!("Enumeration '{}' not found", req.id)))?;

        if let Some(desc) = req.description {
            enumeration.description = Some(desc);
        }

        enumeration
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        Ok(Response::new(core_to_proto_enumeration(enumeration)))
    }

    async fn delete_enumeration(
        &self,
        request: Request<proto::DeleteEnumerationRequest>,
    ) -> Result<Response<proto::DeleteResponse>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        let existed = if let Some(enumeration) = store.enumerations.remove(&req.id) {
            let template_type = enumeration.template_type.as_str().to_string();
            if let Some(ids) = store.enums_by_template.get_mut(&template_type) {
                ids.retain(|id| id != &req.id);
            }
            true
        } else {
            false
        };

        Ok(Response::new(proto::DeleteResponse {
            success: existed,
            message: None,
        }))
    }

    async fn list_enumerations(
        &self,
        request: Request<proto::ListEnumerationsRequest>,
    ) -> Result<Response<proto::ListEnumerationsResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let enums: Vec<_> = store.get_enums_for_template(&req.template_type);
        let total_count = enums.len() as i32;

        // Pagination
        let page_size = if req.page_size > 0 {
            req.page_size as usize
        } else {
            50
        };
        let offset = parse_page_token(&req.page_token)?;

        let paginated: Vec<proto::TemplateEnumeration> = enums
            .into_iter()
            .skip(offset)
            .take(page_size)
            .map(core_to_proto_enumeration)
            .collect();

        let next_token = if paginated.len() == page_size {
            (offset + page_size).to_string()
        } else {
            String::new()
        };

        Ok(Response::new(proto::ListEnumerationsResponse {
            enumerations: paginated,
            next_page_token: next_token,
            total_count,
        }))
    }

    async fn add_enumeration_value(
        &self,
        request: Request<proto::AddEnumerationValueRequest>,
    ) -> Result<Response<proto::TemplateEnumeration>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        let enumeration = store
            .enumerations
            .get_mut(&req.id)
            .ok_or_else(|| Status::not_found(format!("Enumeration '{}' not found", req.id)))?;

        // Check for duplicate value
        if enumeration.values.contains(&req.value) {
            return Err(Status::already_exists(format!(
                "Value '{}' already exists in enumeration",
                req.value
            )));
        }

        enumeration.values.insert(req.value);

        Ok(Response::new(core_to_proto_enumeration(enumeration)))
    }

    async fn remove_enumeration_value(
        &self,
        request: Request<proto::RemoveEnumerationValueRequest>,
    ) -> Result<Response<proto::RemoveEnumerationValueResponse>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // First check if enumeration exists
        let enumeration = store
            .enumerations
            .get(&req.id)
            .ok_or_else(|| Status::not_found(format!("Enumeration '{}' not found", req.id)))?;

        // Check if value exists
        if !enumeration.values.contains(&req.value) {
            return Err(Status::not_found(format!(
                "Value '{}' not found in enumeration",
                req.value
            )));
        }

        // Find all attributes that use this specific value
        let mut affected_attributes = Vec::new();
        for (_, attr) in &store.attributes {
            // Look up the abstract attribute to check enum_name
            if let Some(abstract_attr) = store.abstract_attributes.get(attr.abstract_path.as_str())
            {
                // Check if abstract attribute uses this enumeration
                if let Some(ref enum_name) = abstract_attr.enum_name {
                    if enum_name == &req.id {
                        // Check if the concrete attribute has this value
                        if let Some(product_farm_core::Value::String(v)) = &attr.value {
                            if v == &req.value {
                                let value_type = match attr.value_type {
                                    product_farm_core::AttributeValueType::FixedValue => {
                                        "FIXED_VALUE"
                                    }
                                    product_farm_core::AttributeValueType::RuleDriven => {
                                        "RULE_DRIVEN"
                                    }
                                    product_farm_core::AttributeValueType::JustDefinition => {
                                        "JUST_DEFINITION"
                                    }
                                };
                                affected_attributes.push(proto::AttributeReference {
                                    product_id: attr.product_id.as_str().to_string(),
                                    attribute_path: attr.path.as_str().to_string(),
                                    value_type: value_type.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // If there are affected attributes and cascade is false, return error
        if !affected_attributes.is_empty() && !req.cascade {
            return Err(Status::failed_precondition(format!(
                "Cannot remove value '{}': {} attributes use this value. Set cascade=true to update them.",
                req.value,
                affected_attributes.len()
            )));
        }

        // If cascade is true, set affected attribute values to null
        if req.cascade && !affected_attributes.is_empty() {
            let mut failed_updates = Vec::new();
            for attr_ref in &affected_attributes {
                if let Some(attr) = store.attributes.get_mut(&attr_ref.attribute_path) {
                    attr.value = None;
                } else {
                    // This shouldn't happen since we hold the write lock, but track it
                    failed_updates.push(attr_ref.attribute_path.clone());
                }
            }
            // If any updates failed, this indicates an internal inconsistency
            if !failed_updates.is_empty() {
                tracing::error!(
                    "Cascade failed for {} attributes: {:?}",
                    failed_updates.len(),
                    failed_updates
                );
                return Err(Status::internal(format!(
                    "Cascade partially failed: {} attributes could not be updated",
                    failed_updates.len()
                )));
            }
        }

        // Now remove the value from the enumeration
        let enumeration = store
            .enumerations
            .get_mut(&req.id)
            .ok_or_else(|| Status::not_found(format!("Enumeration '{}' not found", req.id)))?;

        enumeration.values.remove(&req.value);

        Ok(Response::new(proto::RemoveEnumerationValueResponse {
            enumeration: Some(core_to_proto_enumeration(enumeration)),
            affected_attributes,
        }))
    }

    async fn get_enumeration_usage(
        &self,
        request: Request<proto::GetEnumerationUsageRequest>,
    ) -> Result<Response<proto::EnumerationUsageResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        // Verify enumeration exists
        if !store.enumerations.contains_key(&req.enumeration_id) {
            return Err(Status::not_found(format!(
                "Enumeration '{}' not found",
                req.enumeration_id
            )));
        }

        // Find all abstract attributes that reference this enumeration
        let mut used_by_attributes = Vec::new();
        for (_, abstract_attr) in &store.abstract_attributes {
            if let Some(ref enum_name) = abstract_attr.enum_name {
                if enum_name == &req.enumeration_id {
                    used_by_attributes.push(proto::AttributeReference {
                        product_id: abstract_attr.product_id.as_str().to_string(),
                        attribute_path: abstract_attr.abstract_path.as_str().to_string(),
                        value_type: "ABSTRACT".to_string(),
                    });
                }
            }
        }

        // Also find concrete attributes that reference this enumeration (via abstract attribute)
        for (_, attr) in &store.attributes {
            if let Some(abstract_attr) = store.abstract_attributes.get(attr.abstract_path.as_str())
            {
                if let Some(ref enum_name) = abstract_attr.enum_name {
                    if enum_name == &req.enumeration_id {
                        let value_type = match attr.value_type {
                            product_farm_core::AttributeValueType::FixedValue => "FIXED_VALUE",
                            product_farm_core::AttributeValueType::RuleDriven => "RULE_DRIVEN",
                            product_farm_core::AttributeValueType::JustDefinition => {
                                "JUST_DEFINITION"
                            }
                        };
                        used_by_attributes.push(proto::AttributeReference {
                            product_id: attr.product_id.as_str().to_string(),
                            attribute_path: attr.path.as_str().to_string(),
                            value_type: value_type.to_string(),
                        });
                    }
                }
            }
        }

        let total_count = used_by_attributes.len() as i32;

        Ok(Response::new(proto::EnumerationUsageResponse {
            used_by_attributes,
            total_count,
        }))
    }

    async fn get_enumeration_value_usage(
        &self,
        request: Request<proto::GetEnumerationValueUsageRequest>,
    ) -> Result<Response<proto::EnumerationValueUsageResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        // Verify enumeration exists
        let enumeration = store.enumerations.get(&req.enumeration_id).ok_or_else(|| {
            Status::not_found(format!("Enumeration '{}' not found", req.enumeration_id))
        })?;

        // Verify value exists in enumeration
        if !enumeration.values.contains(&req.value) {
            return Err(Status::not_found(format!(
                "Value '{}' not found in enumeration '{}'",
                req.value, req.enumeration_id
            )));
        }

        // Find all concrete attributes that have this specific value
        let mut attributes_with_value = Vec::new();
        for (_, attr) in &store.attributes {
            // Look up the abstract attribute to check enum_name
            if let Some(abstract_attr) = store.abstract_attributes.get(attr.abstract_path.as_str())
            {
                if let Some(ref enum_name) = abstract_attr.enum_name {
                    if enum_name == &req.enumeration_id {
                        // Check if the concrete attribute has this specific value
                        if let Some(product_farm_core::Value::String(v)) = &attr.value {
                            if v == &req.value {
                                let value_type = match attr.value_type {
                                    product_farm_core::AttributeValueType::FixedValue => {
                                        "FIXED_VALUE"
                                    }
                                    product_farm_core::AttributeValueType::RuleDriven => {
                                        "RULE_DRIVEN"
                                    }
                                    product_farm_core::AttributeValueType::JustDefinition => {
                                        "JUST_DEFINITION"
                                    }
                                };
                                attributes_with_value.push(proto::AttributeReference {
                                    product_id: attr.product_id.as_str().to_string(),
                                    attribute_path: attr.path.as_str().to_string(),
                                    value_type: value_type.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        let total_count = attributes_with_value.len() as i32;

        Ok(Response::new(proto::EnumerationValueUsageResponse {
            attributes_with_value,
            total_count,
        }))
    }
}
