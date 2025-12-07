//! DatatypeGrpcService - Datatype management
//!
//! Provides create, read, update, delete, list, usage tracking,
//! and value validation endpoints for datatypes.

use tonic::{Request, Response, Status};

use product_farm_core::DataType;

use crate::converters::{core_to_proto_datatype, proto_to_core_constraints, proto_to_core_primitive_type};
use crate::store::SharedStore;

use super::helpers::parse_page_token;
use super::proto;

/// gRPC service for datatype management
pub struct DatatypeGrpcService {
    store: SharedStore,
}

impl DatatypeGrpcService {
    pub fn new(store: SharedStore) -> Self {
        Self { store }
    }
}

#[tonic::async_trait]
impl proto::datatype_service_server::DatatypeService for DatatypeGrpcService {
    async fn create_datatype(
        &self,
        request: Request<proto::CreateDatatypeRequest>,
    ) -> Result<Response<proto::Datatype>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        let primitive_type = proto_to_core_primitive_type(req.primitive_type);
        let mut datatype = DataType::new(req.id.as_str(), primitive_type);

        if let Some(ref desc) = req.description {
            datatype = datatype.with_description(desc.clone());
        }

        if let Some(ref c) = req.constraints {
            datatype = datatype.with_constraints(proto_to_core_constraints(c));
        }

        let proto_dt = core_to_proto_datatype(&datatype);

        // Use entry API to check and insert atomically
        use hashbrown::hash_map::Entry;
        match store.datatypes.entry(req.id.clone()) {
            Entry::Occupied(_) => {
                return Err(Status::already_exists(format!(
                    "Datatype '{}' already exists",
                    req.id
                )));
            }
            Entry::Vacant(entry) => {
                entry.insert(datatype);
            }
        }

        Ok(Response::new(proto_dt))
    }

    async fn get_datatype(
        &self,
        request: Request<proto::GetDatatypeRequest>,
    ) -> Result<Response<proto::Datatype>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        store
            .datatypes
            .get(&req.id)
            .map(|d| Response::new(core_to_proto_datatype(d)))
            .ok_or_else(|| Status::not_found(format!("Datatype '{}' not found", req.id)))
    }

    async fn update_datatype(
        &self,
        request: Request<proto::UpdateDatatypeRequest>,
    ) -> Result<Response<proto::Datatype>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        let datatype = store
            .datatypes
            .get_mut(&req.id)
            .ok_or_else(|| Status::not_found(format!("Datatype '{}' not found", req.id)))?;

        if let Some(desc) = req.description {
            datatype.description = Some(desc);
        }

        if let Some(ref c) = req.constraints {
            datatype.constraints = Some(proto_to_core_constraints(c));
        }

        Ok(Response::new(core_to_proto_datatype(datatype)))
    }

    async fn delete_datatype(
        &self,
        request: Request<proto::DeleteDatatypeRequest>,
    ) -> Result<Response<proto::DeleteResponse>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        let existed = store.datatypes.remove(&req.id).is_some();

        Ok(Response::new(proto::DeleteResponse {
            success: existed,
            message: None,
        }))
    }

    async fn list_datatypes(
        &self,
        request: Request<proto::ListDatatypesRequest>,
    ) -> Result<Response<proto::ListDatatypesResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let mut datatypes: Vec<_> = store.datatypes.values().collect();

        // Apply filter
        if let Some(pt) = req.primitive_type_filter {
            let filter_type = proto_to_core_primitive_type(pt);
            datatypes.retain(|d| d.primitive_type == filter_type);
        }

        let total_count = datatypes.len() as i32;

        // Pagination
        let page_size = if req.page_size > 0 {
            req.page_size as usize
        } else {
            50
        };
        let offset = parse_page_token(&req.page_token)?;

        let paginated: Vec<proto::Datatype> = datatypes
            .into_iter()
            .skip(offset)
            .take(page_size)
            .map(core_to_proto_datatype)
            .collect();

        let next_token = if paginated.len() == page_size {
            (offset + page_size).to_string()
        } else {
            String::new()
        };

        Ok(Response::new(proto::ListDatatypesResponse {
            datatypes: paginated,
            next_page_token: next_token,
            total_count,
        }))
    }

    async fn get_datatype_usage(
        &self,
        request: Request<proto::GetDatatypeUsageRequest>,
    ) -> Result<Response<proto::DatatypeUsageResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        // Find all abstract attributes that use this datatype
        let mut used_by: Vec<proto::AttributeReference> = Vec::new();

        for (_, abstract_attr) in &store.abstract_attributes {
            if abstract_attr.datatype_id.as_str() == req.datatype_id {
                used_by.push(proto::AttributeReference {
                    product_id: abstract_attr.product_id.as_str().to_string(),
                    attribute_path: abstract_attr.abstract_path.as_str().to_string(),
                    value_type: "ABSTRACT".to_string(),
                });
            }
        }

        // Also check concrete attributes by looking up their abstract attribute
        for (_, attr) in &store.attributes {
            if let Some(abstract_attr) = store.abstract_attributes.get(attr.abstract_path.as_str())
            {
                if abstract_attr.datatype_id.as_str() == req.datatype_id {
                    let value_type = match attr.value_type {
                        product_farm_core::AttributeValueType::FixedValue => "FIXED_VALUE",
                        product_farm_core::AttributeValueType::RuleDriven => "RULE_DRIVEN",
                        product_farm_core::AttributeValueType::JustDefinition => "JUST_DEFINITION",
                    };
                    used_by.push(proto::AttributeReference {
                        product_id: attr.product_id.as_str().to_string(),
                        attribute_path: attr.path.as_str().to_string(),
                        value_type: value_type.to_string(),
                    });
                }
            }
        }

        let total_count = used_by.len() as i32;

        Ok(Response::new(proto::DatatypeUsageResponse {
            used_by_attributes: used_by,
            total_count,
        }))
    }

    async fn validate_datatype_value(
        &self,
        request: Request<proto::ValidateDatatypeValueRequest>,
    ) -> Result<Response<proto::ValidateDatatypeValueResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        // Get the datatype
        let datatype = store.datatypes.get(&req.datatype_id).ok_or_else(|| {
            Status::not_found(format!("Datatype not found: {}", req.datatype_id))
        })?;

        let mut errors: Vec<String> = Vec::new();

        // Parse the value
        let value: serde_json::Value = match serde_json::from_str(&req.value_json) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("Invalid JSON value: {}", e));
                return Ok(Response::new(proto::ValidateDatatypeValueResponse {
                    is_valid: false,
                    errors,
                }));
            }
        };

        // Validate against constraints if present
        if let Some(constraints) = &datatype.constraints {
            // Check min constraint
            if let Some(min) = constraints.min {
                if let Some(num) = value.as_f64() {
                    if num < min {
                        errors.push(format!("Value {} is less than minimum {}", num, min));
                    }
                }
            }

            // Check max constraint
            if let Some(max) = constraints.max {
                if let Some(num) = value.as_f64() {
                    if num > max {
                        errors.push(format!("Value {} is greater than maximum {}", num, max));
                    }
                }
            }

            // Check min_length constraint
            if let Some(min_len) = constraints.min_length {
                if let Some(s) = value.as_str() {
                    if s.len() < min_len {
                        errors.push(format!(
                            "String length {} is less than minimum {}",
                            s.len(),
                            min_len
                        ));
                    }
                }
            }

            // Check max_length constraint
            if let Some(max_len) = constraints.max_length {
                if let Some(s) = value.as_str() {
                    if s.len() > max_len {
                        errors.push(format!(
                            "String length {} is greater than maximum {}",
                            s.len(),
                            max_len
                        ));
                    }
                }
            }

            // Check pattern constraint
            if let Some(pattern) = &constraints.pattern {
                if let Some(s) = value.as_str() {
                    match regex::Regex::new(pattern) {
                        Ok(re) => {
                            if !re.is_match(s) {
                                errors.push(format!(
                                    "Value '{}' does not match pattern '{}'",
                                    s, pattern
                                ));
                            }
                        }
                        Err(e) => {
                            errors.push(format!("Invalid regex pattern '{}': {}", pattern, e));
                        }
                    }
                }
            }

            // Check constraint rule expression if present
            if let Some(rule_expr) = &constraints.constraint_rule_expression {
                // Build context with $value
                let mut context = serde_json::Map::new();
                context.insert("$value".to_string(), value.clone());

                // Add any additional context from the request
                for (k, v) in &req.context {
                    if let Ok(parsed) = serde_json::from_str(v) {
                        context.insert(k.clone(), parsed);
                    } else {
                        context.insert(k.clone(), serde_json::Value::String(v.clone()));
                    }
                }

                // Parse and evaluate the JSON Logic expression
                match serde_json::from_str::<serde_json::Value>(rule_expr) {
                    Ok(_expr) => {
                        // TODO: Integrate with JSON Logic engine for evaluation
                        // For now, we'll skip complex rule evaluation
                        // In a full implementation, this would call the rule engine
                    }
                    Err(e) => {
                        errors.push(format!("Invalid constraint rule expression: {}", e));
                    }
                }
            }
        }

        Ok(Response::new(proto::ValidateDatatypeValueResponse {
            is_valid: errors.is_empty(),
            errors,
        }))
    }
}
