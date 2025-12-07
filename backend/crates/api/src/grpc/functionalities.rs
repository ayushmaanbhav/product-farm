//! ProductFunctionalityGrpcService - Functionality management
//!
//! Provides create, read, update, delete, list, evaluation, and
//! workflow endpoints for product functionalities.

use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use product_farm_core::{FunctionalityId, FunctionalityRequiredAttribute, ProductFunctionality, Rule};
use product_farm_rule_engine::{ExecutionContext, RuleExecutor};

use crate::converters::{
    core_to_proto_abstract_attribute, core_to_proto_functionality, core_to_proto_value,
    proto_to_core_functionality_status, try_proto_to_core_value,
};
use crate::store::{EntityStore, SharedStore};

use super::helpers::parse_page_token;
use super::proto;

/// gRPC service for product functionality management
pub struct ProductFunctionalityGrpcService {
    store: SharedStore,
    executor: Arc<RwLock<RuleExecutor>>,
}

impl ProductFunctionalityGrpcService {
    pub fn new(store: SharedStore) -> Self {
        Self {
            store,
            executor: Arc::new(RwLock::new(RuleExecutor::new())),
        }
    }
}

#[tonic::async_trait]
impl proto::product_functionality_service_server::ProductFunctionalityService
    for ProductFunctionalityGrpcService
{
    async fn create_functionality(
        &self,
        request: Request<proto::CreateFunctionalityRequest>,
    ) -> Result<Response<proto::ProductFunctionality>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // Check product exists and is editable
        let product = store
            .products
            .get(&req.product_id)
            .ok_or_else(|| Status::not_found(format!("Product '{}' not found", req.product_id)))?;

        if !product.is_editable() {
            return Err(Status::failed_precondition(
                "Product must be in DRAFT status to add functionalities",
            ));
        }

        let key = EntityStore::functionality_key(&req.product_id, &req.name);

        // Generate functionality ID
        let func_id = FunctionalityId::new(format!("{}:{}", req.product_id, req.name));

        // Build required attributes
        let required_attrs: Vec<FunctionalityRequiredAttribute> = req
            .required_attributes
            .iter()
            .enumerate()
            .map(|(i, ra)| {
                FunctionalityRequiredAttribute::new(
                    func_id.clone(),
                    ra.abstract_path.as_str(),
                    ra.description.as_str(),
                    i as i32,
                )
            })
            .collect();

        // Create functionality
        let mut func = ProductFunctionality::new(
            func_id.clone(),
            req.name.as_str(),
            req.product_id.as_str(),
            req.description.as_str(),
        );
        if req.immutable {
            func = func.with_immutable(true);
        }
        func.required_attributes = required_attrs;

        // Validate
        func.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let proto_func = core_to_proto_functionality(&func);

        // Use entry API to check and insert atomically
        use hashbrown::hash_map::Entry;
        match store.functionalities.entry(key.clone()) {
            Entry::Occupied(_) => {
                return Err(Status::already_exists(format!(
                    "Functionality '{}' already exists for product '{}'",
                    req.name, req.product_id
                )));
            }
            Entry::Vacant(entry) => {
                entry.insert(func);
            }
        }
        store
            .funcs_by_product
            .entry(req.product_id)
            .or_default()
            .push(key);

        Ok(Response::new(proto_func))
    }

    async fn get_functionality(
        &self,
        request: Request<proto::GetFunctionalityRequest>,
    ) -> Result<Response<proto::ProductFunctionality>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let key = EntityStore::functionality_key(&req.product_id, &req.name);
        store
            .functionalities
            .get(&key)
            .map(|f| Response::new(core_to_proto_functionality(f)))
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Functionality '{}' not found for product '{}'",
                    req.name, req.product_id
                ))
            })
    }

    async fn update_functionality(
        &self,
        request: Request<proto::UpdateFunctionalityRequest>,
    ) -> Result<Response<proto::ProductFunctionality>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // Check product is editable
        let product = store
            .products
            .get(&req.product_id)
            .ok_or_else(|| Status::not_found(format!("Product '{}' not found", req.product_id)))?;

        if !product.is_editable() {
            return Err(Status::failed_precondition(
                "Product must be in DRAFT status to update functionalities",
            ));
        }

        let key = EntityStore::functionality_key(&req.product_id, &req.name);
        let func = store.functionalities.get_mut(&key).ok_or_else(|| {
            Status::not_found(format!(
                "Functionality '{}' not found for product '{}'",
                req.name, req.product_id
            ))
        })?;

        // Check modifiable
        func.check_modifiable()
            .map_err(|e| Status::failed_precondition(e.to_string()))?;

        if let Some(desc) = req.description {
            func.description = desc;
        }

        // Update required attributes if provided
        if !req.required_attributes.is_empty() {
            let func_id = func.id.clone();
            func.required_attributes = req
                .required_attributes
                .iter()
                .enumerate()
                .map(|(i, ra)| {
                    FunctionalityRequiredAttribute::new(
                        func_id.clone(),
                        ra.abstract_path.as_str(),
                        ra.description.as_str(),
                        i as i32,
                    )
                })
                .collect();
        }

        func.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        Ok(Response::new(core_to_proto_functionality(func)))
    }

    async fn delete_functionality(
        &self,
        request: Request<proto::DeleteFunctionalityRequest>,
    ) -> Result<Response<proto::DeleteResponse>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // Check product is editable
        if let Some(product) = store.products.get(&req.product_id) {
            if !product.is_editable() {
                return Err(Status::failed_precondition(
                    "Product must be in DRAFT status to delete functionalities",
                ));
            }
        }

        let key = EntityStore::functionality_key(&req.product_id, &req.name);

        // Check not immutable
        if let Some(func) = store.functionalities.get(&key) {
            if func.immutable {
                return Err(Status::failed_precondition(
                    "Cannot delete immutable functionality",
                ));
            }
        }

        let existed = store.functionalities.remove(&key).is_some();

        if existed {
            if let Some(keys) = store.funcs_by_product.get_mut(&req.product_id) {
                keys.retain(|k| k != &key);
            }
        }

        Ok(Response::new(proto::DeleteResponse {
            success: existed,
            message: None,
        }))
    }

    async fn list_functionalities(
        &self,
        request: Request<proto::ListFunctionalitiesRequest>,
    ) -> Result<Response<proto::ListFunctionalitiesResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let mut funcs: Vec<_> = store.get_funcs_for_product(&req.product_id);

        // Apply filter
        if let Some(status) = req.status_filter {
            let filter_status = proto_to_core_functionality_status(status);
            funcs.retain(|f| f.status == filter_status);
        }

        let total_count = funcs.len() as i32;

        // Pagination
        let page_size = if req.page_size > 0 {
            req.page_size as usize
        } else {
            50
        };
        let offset = parse_page_token(&req.page_token)?;

        let paginated: Vec<proto::ProductFunctionality> = funcs
            .into_iter()
            .skip(offset)
            .take(page_size)
            .map(core_to_proto_functionality)
            .collect();

        let next_token = if paginated.len() == page_size {
            (offset + page_size).to_string()
        } else {
            String::new()
        };

        Ok(Response::new(proto::ListFunctionalitiesResponse {
            functionalities: paginated,
            next_page_token: next_token,
            total_count,
        }))
    }

    async fn get_functionality_abstract_attributes(
        &self,
        request: Request<proto::GetFunctionalityAbstractAttributesRequest>,
    ) -> Result<Response<proto::ListAbstractAttributesResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let attrs =
            store.get_abstract_attrs_by_functionality(&req.product_id, &req.functionality_name);
        let proto_attrs: Vec<_> = attrs
            .into_iter()
            .map(core_to_proto_abstract_attribute)
            .collect();
        let total_count = proto_attrs.len() as i32;

        Ok(Response::new(proto::ListAbstractAttributesResponse {
            attributes: proto_attrs,
            next_page_token: String::new(),
            total_count,
        }))
    }

    async fn evaluate_functionality(
        &self,
        request: Request<proto::EvaluateFunctionalityRequest>,
    ) -> Result<Response<proto::EvaluateFunctionalityResponse>, Status> {
        let req = request.into_inner();
        let start_time = std::time::Instant::now();

        let store = self.store.read().await;

        // Get functionality
        let key = EntityStore::functionality_key(&req.product_id, &req.functionality_name);
        let functionality = store.functionalities.get(&key).ok_or_else(|| {
            Status::not_found(format!(
                "Functionality '{}' not found for product '{}'",
                req.functionality_name, req.product_id
            ))
        })?;

        // Validate inputs against required attributes
        let mut validation_results = Vec::new();
        let mut all_valid = true;

        for required in &functionality.required_attributes {
            let path = required.abstract_path.as_str();
            let has_input = req.input_values.contains_key(path);

            if !has_input {
                validation_results.push(proto::InputValidationResult {
                    path: path.to_string(),
                    valid: false,
                    error_message: Some("Required input missing".into()),
                    expected_type: None,
                    provided_value: None,
                });
                all_valid = false;
            } else {
                // Get abstract attribute to check datatype
                if let Some(abstract_attr) = store.abstract_attributes.get(path) {
                    let provided_value = req.input_values.get(path);
                    let expected_type = abstract_attr.datatype_id.as_str().to_string();

                    // Basic type validation
                    let type_valid = if let Some(pv) = provided_value {
                        // Simple validation - check value exists
                        pv.value.is_some()
                    } else {
                        false
                    };

                    if !type_valid {
                        all_valid = false;
                    }

                    validation_results.push(proto::InputValidationResult {
                        path: path.to_string(),
                        valid: type_valid,
                        error_message: if type_valid {
                            None
                        } else {
                            Some(format!("Invalid value for type {}", expected_type))
                        },
                        expected_type: Some(expected_type),
                        provided_value: provided_value.cloned(),
                    });
                } else {
                    validation_results.push(proto::InputValidationResult {
                        path: path.to_string(),
                        valid: true,
                        error_message: None,
                        expected_type: None,
                        provided_value: req.input_values.get(path).cloned(),
                    });
                }
            }
        }

        // If validate_only, return early
        if req.validate_only {
            return Ok(Response::new(proto::EvaluateFunctionalityResponse {
                success: all_valid,
                output_values: std::collections::HashMap::new(),
                validation_results,
                errors: if all_valid {
                    vec![]
                } else {
                    vec![proto::EvaluationError {
                        rule_id: String::new(),
                        error_type: "ValidationError".into(),
                        message: "One or more inputs failed validation".into(),
                    }]
                },
                metrics: Some(proto::ExecutionMetrics {
                    total_time_ns: start_time.elapsed().as_nanos() as i64,
                    rules_executed: 0,
                    rules_skipped: 0,
                    cache_hits: 0,
                    levels: vec![],
                }),
            }));
        }

        // If validation failed, return errors
        if !all_valid {
            return Ok(Response::new(proto::EvaluateFunctionalityResponse {
                success: false,
                output_values: std::collections::HashMap::new(),
                validation_results,
                errors: vec![proto::EvaluationError {
                    rule_id: String::new(),
                    error_type: "ValidationError".into(),
                    message: "One or more inputs failed validation".into(),
                }],
                metrics: Some(proto::ExecutionMetrics {
                    total_time_ns: start_time.elapsed().as_nanos() as i64,
                    rules_executed: 0,
                    rules_skipped: 0,
                    cache_hits: 0,
                    levels: vec![],
                }),
            }));
        }

        // Get rules for the product
        let rules: Vec<Rule> = store
            .get_rules_for_product(&req.product_id)
            .into_iter()
            .filter(|r| r.enabled)
            .cloned()
            .collect();

        drop(store);

        // Build input context
        let mut input_map = std::collections::HashMap::new();
        for (k, v) in &req.input_values {
            input_map.insert(k.clone(), try_proto_to_core_value(v)?);
        }

        // Execute rules
        let mut executor = self.executor.write().await;
        let mut context = ExecutionContext::new(input_map.into_iter().collect());

        match executor.execute(&rules, &mut context) {
            Ok(result) => {
                let outputs: std::collections::HashMap<String, proto::Value> = result
                    .context
                    .computed_values()
                    .iter()
                    .map(|(k, v)| (k.clone(), core_to_proto_value(v)))
                    .collect();

                let levels: Vec<proto::ExecutionLevel> = result
                    .levels
                    .iter()
                    .enumerate()
                    .map(|(i, ids)| proto::ExecutionLevel {
                        level: i as i32,
                        rule_ids: ids.iter().map(|id| id.to_string()).collect(),
                    })
                    .collect();

                Ok(Response::new(proto::EvaluateFunctionalityResponse {
                    success: true,
                    output_values: outputs,
                    validation_results,
                    errors: vec![],
                    metrics: Some(proto::ExecutionMetrics {
                        total_time_ns: result.total_time_ns as i64,
                        rules_executed: result.rule_results.len() as i32,
                        rules_skipped: 0,
                        cache_hits: 0,
                        levels,
                    }),
                }))
            }
            Err(e) => Ok(Response::new(proto::EvaluateFunctionalityResponse {
                success: false,
                output_values: std::collections::HashMap::new(),
                validation_results,
                errors: vec![proto::EvaluationError {
                    rule_id: String::new(),
                    error_type: "ExecutionError".into(),
                    message: e.to_string(),
                }],
                metrics: Some(proto::ExecutionMetrics {
                    total_time_ns: start_time.elapsed().as_nanos() as i64,
                    rules_executed: 0,
                    rules_skipped: 0,
                    cache_hits: 0,
                    levels: vec![],
                }),
            })),
        }
    }

    async fn submit_functionality(
        &self,
        request: Request<proto::SubmitFunctionalityRequest>,
    ) -> Result<Response<proto::ProductFunctionality>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        let key = EntityStore::functionality_key(&req.product_id, &req.name);
        let func = store.functionalities.get_mut(&key).ok_or_else(|| {
            Status::not_found(format!(
                "Functionality '{}' not found for product '{}'",
                req.name, req.product_id
            ))
        })?;

        func.submit()
            .map_err(|e| Status::failed_precondition(e.to_string()))?;

        Ok(Response::new(core_to_proto_functionality(func)))
    }

    async fn approve_functionality(
        &self,
        request: Request<proto::ApproveFunctionalityRequest>,
    ) -> Result<Response<proto::ProductFunctionality>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        let key = EntityStore::functionality_key(&req.product_id, &req.name);
        let func = store.functionalities.get_mut(&key).ok_or_else(|| {
            Status::not_found(format!(
                "Functionality '{}' not found for product '{}'",
                req.name, req.product_id
            ))
        })?;

        func.approve()
            .map_err(|e| Status::failed_precondition(e.to_string()))?;

        Ok(Response::new(core_to_proto_functionality(func)))
    }

    async fn reject_functionality(
        &self,
        request: Request<proto::RejectFunctionalityRequest>,
    ) -> Result<Response<proto::ProductFunctionality>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        let key = EntityStore::functionality_key(&req.product_id, &req.name);
        let func = store.functionalities.get_mut(&key).ok_or_else(|| {
            Status::not_found(format!(
                "Functionality '{}' not found for product '{}'",
                req.name, req.product_id
            ))
        })?;

        func.reject()
            .map_err(|e| Status::failed_precondition(e.to_string()))?;

        Ok(Response::new(core_to_proto_functionality(func)))
    }
}
