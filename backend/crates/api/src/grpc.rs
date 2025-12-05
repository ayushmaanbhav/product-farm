//! gRPC service implementations for Product-FARM
//!
//! Implements all service traits generated from product_farm.proto

use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

use chrono::{TimeZone, Utc};
use product_farm_core::{
    AbstractAttribute, AbstractAttributeTag, AbstractPath, Attribute, AttributeDisplayName,
    CloneProductRequest as CoreCloneRequest, CloneSelections, ConcretePath, DataType, FunctionalityId,
    FunctionalityRequiredAttribute, Product, ProductCloneService, ProductFunctionality,
    ProductId, ProductTemplateEnumeration, Rule, RuleId, RuleInputAttribute,
    RuleOutputAttribute, TemplateEnumerationId,
};
use product_farm_rule_engine::{ExecutionContext, RuleDag, RuleExecutor};

use crate::converters::*;
use crate::store::{EntityStore, SharedStore};

// Include the generated protobuf code
pub mod proto {
    tonic::include_proto!("product_farm");
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Parse a page token into an offset, returning an error for invalid tokens.
/// Empty string returns 0 (start from beginning).
fn parse_page_token(page_token: &str) -> Result<usize, Status> {
    if page_token.is_empty() {
        return Ok(0);
    }
    page_token
        .parse()
        .map_err(|_| Status::invalid_argument(format!("Invalid page_token: '{}'", page_token)))
}

// ============================================================================
// PRODUCT FARM SERVICE (Rule Evaluation)
// ============================================================================

pub struct ProductFarmGrpcService {
    executor: Arc<RwLock<RuleExecutor>>,
    store: SharedStore,
    start_time: std::time::Instant,
}

impl ProductFarmGrpcService {
    pub fn new(store: SharedStore) -> Self {
        Self {
            executor: Arc::new(RwLock::new(RuleExecutor::new())),
            store,
            start_time: std::time::Instant::now(),
        }
    }
}

#[tonic::async_trait]
impl proto::product_farm_service_server::ProductFarmService for ProductFarmGrpcService {
    async fn evaluate(
        &self,
        request: Request<proto::EvaluateRequest>,
    ) -> Result<Response<proto::EvaluateResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        // Get rules for the product
        let rules: Vec<Rule> = store
            .get_rules_for_product(&req.product_id)
            .into_iter()
            .filter(|r| r.enabled)
            .filter(|r| req.rule_ids.is_empty() || req.rule_ids.contains(&r.id.to_string()))
            .cloned()
            .collect();
        drop(store);

        if rules.is_empty() {
            return Ok(Response::new(proto::EvaluateResponse {
                success: true,
                outputs: std::collections::HashMap::new(),
                rule_results: vec![],
                metrics: Some(proto::ExecutionMetrics {
                    total_time_ns: 0,
                    rules_executed: 0,
                    rules_skipped: 0,
                    cache_hits: 0,
                    levels: vec![],
                }),
                errors: vec![],
            }));
        }

        // Build input context
        let mut input_map = std::collections::HashMap::new();
        for (k, v) in &req.input_data {
            input_map.insert(k.clone(), try_proto_to_core_value(v)?);
        }

        // Execute rules
        let mut executor = self.executor.write().await;
        let mut context = ExecutionContext::new(input_map.into_iter().collect());

        let result = executor
            .execute(&rules, &mut context)
            .map_err(|e| Status::internal(format!("Execution error: {}", e)))?;

        // Build response
        let outputs: std::collections::HashMap<String, proto::Value> = result
            .context
            .computed_values()
            .iter()
            .map(|(k, v)| (k.clone(), core_to_proto_value(v)))
            .collect();

        let rule_results: Vec<proto::RuleResult> = result
            .rule_results
            .iter()
            .map(|r| proto::RuleResult {
                rule_id: r.rule_id.to_string(),
                outputs: r
                    .outputs
                    .iter()
                    .map(|(path, val)| proto::OutputValue {
                        attribute_path: path.clone(),
                        value: Some(core_to_proto_value(val)),
                    })
                    .collect(),
                execution_time_ns: r.execution_time_ns as i64,
                success: true,
                error_message: String::new(),
            })
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

        Ok(Response::new(proto::EvaluateResponse {
            success: true,
            outputs,
            rule_results,
            metrics: Some(proto::ExecutionMetrics {
                total_time_ns: result.total_time_ns as i64,
                rules_executed: result.rule_results.len() as i32,
                rules_skipped: 0,
                cache_hits: 0,
                levels,
            }),
            errors: vec![],
        }))
    }

    type EvaluateStreamStream =
        Pin<Box<dyn Stream<Item = Result<proto::EvaluateResponse, Status>> + Send>>;

    async fn evaluate_stream(
        &self,
        request: Request<tonic::Streaming<proto::EvaluateRequest>>,
    ) -> Result<Response<Self::EvaluateStreamStream>, Status> {
        let mut stream = request.into_inner();
        let store = self.store.clone();

        let output_stream = async_stream::try_stream! {
            // Create a dedicated executor for this stream to avoid lock contention
            // across concurrent streams. The executor is lightweight and caches
            // compiled rules only for the duration of this stream.
            let mut stream_executor = RuleExecutor::new();
            let mut cached_product_id: Option<String> = None;
            let mut cached_rules: Vec<Rule> = Vec::new();

            while let Some(req) = stream.message().await? {
                // Only re-fetch rules if product_id changed
                if cached_product_id.as_ref() != Some(&req.product_id) {
                    let store_guard = store.read().await;
                    cached_rules = store_guard
                        .get_rules_for_product(&req.product_id)
                        .into_iter()
                        .filter(|r| r.enabled)
                        .cloned()
                        .collect();
                    drop(store_guard);
                    cached_product_id = Some(req.product_id.clone());
                }

                let mut input_map = std::collections::HashMap::new();
                for (k, v) in &req.input_data {
                    input_map.insert(k.clone(), try_proto_to_core_value(v)?);
                }

                let mut context = ExecutionContext::new(input_map.into_iter().collect());

                match stream_executor.execute(&cached_rules, &mut context) {
                    Ok(result) => {
                        let outputs: std::collections::HashMap<String, proto::Value> = result
                            .context
                            .computed_values()
                            .iter()
                            .map(|(k, v)| (k.clone(), core_to_proto_value(v)))
                            .collect();

                        yield proto::EvaluateResponse {
                            success: true,
                            outputs,
                            rule_results: vec![],
                            metrics: Some(proto::ExecutionMetrics {
                                total_time_ns: result.total_time_ns as i64,
                                rules_executed: result.rule_results.len() as i32,
                                rules_skipped: 0,
                                cache_hits: 0,
                                levels: vec![],
                            }),
                            errors: vec![],
                        };
                    }
                    Err(e) => {
                        yield proto::EvaluateResponse {
                            success: false,
                            outputs: std::collections::HashMap::new(),
                            rule_results: vec![],
                            metrics: None,
                            errors: vec![proto::EvaluationError {
                                rule_id: String::new(),
                                error_type: "ExecutionError".into(),
                                message: e.to_string(),
                            }],
                        };
                    }
                }
            }
        };

        Ok(Response::new(Box::pin(output_stream)))
    }

    async fn batch_evaluate(
        &self,
        request: Request<proto::BatchEvaluateRequest>,
    ) -> Result<Response<proto::BatchEvaluateResponse>, Status> {
        let req = request.into_inner();
        let start_time = std::time::Instant::now();

        let store = self.store.read().await;
        let rules: Vec<Rule> = store
            .get_rules_for_product(&req.product_id)
            .into_iter()
            .filter(|r| r.enabled)
            .cloned()
            .collect();
        drop(store);

        let rules_per_input = rules.len() as i32;
        let total_inputs = req.inputs.len() as i32;
        let mut successful_inputs = 0i32;
        let mut failed_inputs = 0i32;
        let mut results = Vec::with_capacity(req.inputs.len());

        let mut executor = self.executor.write().await;
        for batch_input in &req.inputs {
            let mut input_map = std::collections::HashMap::new();
            for (k, v) in &batch_input.data {
                input_map.insert(k.clone(), try_proto_to_core_value(v)?);
            }

            let mut context = ExecutionContext::new(input_map.into_iter().collect());

            match executor.execute(&rules, &mut context) {
                Ok(result) => {
                    let outputs: std::collections::HashMap<String, proto::Value> = result
                        .context
                        .computed_values()
                        .iter()
                        .map(|(k, v)| (k.clone(), core_to_proto_value(v)))
                        .collect();

                    results.push(proto::BatchResult {
                        input_id: batch_input.input_id.clone(),
                        success: true,
                        outputs,
                        errors: vec![],
                    });
                    successful_inputs += 1;
                }
                Err(e) => {
                    results.push(proto::BatchResult {
                        input_id: batch_input.input_id.clone(),
                        success: false,
                        outputs: std::collections::HashMap::new(),
                        errors: vec![proto::EvaluationError {
                            rule_id: String::new(),
                            error_type: "ExecutionError".into(),
                            message: e.to_string(),
                        }],
                    });
                    failed_inputs += 1;
                }
            }
        }

        let total_time_ns = start_time.elapsed().as_nanos() as i64;
        let avg_time_per_input_ns = if total_inputs > 0 {
            total_time_ns / total_inputs as i64
        } else {
            0
        };

        Ok(Response::new(proto::BatchEvaluateResponse {
            success: failed_inputs == 0,
            results,
            metrics: Some(proto::BatchMetrics {
                total_time_ns,
                total_inputs,
                successful_inputs,
                failed_inputs,
                rules_per_input,
                avg_time_per_input_ns,
            }),
        }))
    }

    async fn validate_rules(
        &self,
        request: Request<proto::ValidateRulesRequest>,
    ) -> Result<Response<proto::ValidateRulesResponse>, Status> {
        let req = request.into_inner();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for rule in &req.rules {
            // Validate JSON expression
            if let Err(e) = serde_json::from_str::<serde_json::Value>(&rule.expression_json) {
                errors.push(proto::ValidationError {
                    rule_id: rule.id.clone(),
                    error_type: "InvalidJson".into(),
                    message: format!("Invalid JSON in expression: {}", e),
                    field: "expression_json".into(),
                });
                continue;
            }

            // Check for empty outputs
            if rule.output_attributes.is_empty() {
                errors.push(proto::ValidationError {
                    rule_id: rule.id.clone(),
                    error_type: "NoOutputs".into(),
                    message: "Rule must have at least one output attribute".into(),
                    field: "output_attributes".into(),
                });
            }

            // Check for missing inputs
            for input in &rule.input_attributes {
                let input_path = &input.path;
                if !req.available_inputs.contains(input_path) {
                    let has_producer = req.rules.iter().any(|r| {
                        r.output_attributes.iter().any(|o| &o.path == input_path)
                    });
                    if !has_producer {
                        warnings.push(proto::ValidationWarning {
                            rule_id: rule.id.clone(),
                            warning_type: "MissingInput".into(),
                            message: format!(
                                "Input '{}' is not in available_inputs and has no producer rule",
                                input_path
                            ),
                        });
                    }
                }
            }
        }

        Ok(Response::new(proto::ValidateRulesResponse {
            valid: errors.is_empty(),
            errors,
            warnings,
        }))
    }

    async fn get_execution_plan(
        &self,
        request: Request<proto::GetExecutionPlanRequest>,
    ) -> Result<Response<proto::ExecutionPlanResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let rules: Vec<Rule> = store
            .get_rules_for_product(&req.product_id)
            .into_iter()
            .filter(|r| r.enabled)
            .filter(|r| req.rule_ids.is_empty() || req.rule_ids.contains(&r.id.to_string()))
            .cloned()
            .collect();
        drop(store);

        let dag = RuleDag::from_rules(&rules)
            .map_err(|e| Status::internal(format!("Failed to build DAG: {}", e)))?;

        let levels = dag
            .execution_levels()
            .map_err(|e| Status::internal(format!("Failed to compute levels: {}", e)))?;

        let execution_levels: Vec<proto::ExecutionLevel> = levels
            .iter()
            .enumerate()
            .map(|(i, ids)| proto::ExecutionLevel {
                level: i as i32,
                rule_ids: ids.iter().map(|id| id.to_string()).collect(),
            })
            .collect();

        let dependencies: Vec<proto::RuleDependency> = rules
            .iter()
            .map(|r| proto::RuleDependency {
                rule_id: r.id.to_string(),
                depends_on: r.input_attributes.iter().map(|a| a.path.as_str().to_string()).collect(),
                produces: r.output_attributes.iter().map(|a| a.path.as_str().to_string()).collect(),
            })
            .collect();

        Ok(Response::new(proto::ExecutionPlanResponse {
            levels: execution_levels,
            dependencies,
            missing_inputs: vec![],
            dot_graph: dag.to_dot(),
            mermaid_graph: dag.to_mermaid(),
            ascii_graph: dag.to_ascii().unwrap_or_default(),
        }))
    }

    async fn health_check(
        &self,
        _request: Request<proto::HealthCheckRequest>,
    ) -> Result<Response<proto::HealthCheckResponse>, Status> {
        Ok(Response::new(proto::HealthCheckResponse {
            status: proto::HealthStatus::Serving as i32,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs() as i64,
            metadata: std::collections::HashMap::new(),
        }))
    }
}

// ============================================================================
// PRODUCT SERVICE
// ============================================================================

pub struct ProductGrpcService {
    store: SharedStore,
}

impl ProductGrpcService {
    pub fn new(store: SharedStore) -> Self {
        Self { store }
    }
}

#[tonic::async_trait]
impl proto::product_service_server::ProductService for ProductGrpcService {
    async fn create_product(
        &self,
        request: Request<proto::CreateProductRequest>,
    ) -> Result<Response<proto::Product>, Status> {
        let req = request.into_inner();

        // Validate product ID
        if !product_farm_core::validation::is_valid_product_id(&req.id) {
            return Err(Status::invalid_argument(format!(
                "Invalid product ID '{}': must match pattern",
                req.id
            )));
        }

        let effective_from = Utc
            .timestamp_opt(req.effective_from, 0)
            .single()
            .ok_or_else(|| Status::invalid_argument("Invalid effective_from timestamp"))?;

        let mut product = Product::new(
            req.id.as_str(),
            req.name.as_str(),
            req.template_type.as_str(),
            effective_from,
        );

        if let Some(desc) = req.description {
            product = product.with_description(desc);
        }

        if let Some(expiry) = req.expiry_at {
            let expiry_dt = Utc
                .timestamp_opt(expiry, 0)
                .single()
                .ok_or_else(|| Status::invalid_argument("Invalid expiry_at timestamp"))?;
            product = product.with_expiry(expiry_dt);
        }

        // Validate product
        product.validate().map_err(|e| Status::invalid_argument(e.to_string()))?;

        let mut store = self.store.write().await;

        // Use entry API to check and insert atomically
        use hashbrown::hash_map::Entry;
        let entry = match store.products.entry(req.id.clone()) {
            Entry::Occupied(_) => {
                return Err(Status::already_exists(format!(
                    "Product '{}' already exists",
                    req.id
                )));
            }
            Entry::Vacant(e) => e,
        };

        let proto_product = core_to_proto_product(&product);
        entry.insert(product);

        // Initialize related collections
        store.abstract_attrs_by_product.insert(req.id.clone(), vec![]);
        store.attrs_by_product.insert(req.id.clone(), vec![]);
        store.rules_by_product.insert(req.id.clone(), vec![]);
        store.funcs_by_product.insert(req.id, vec![]);

        Ok(Response::new(proto_product))
    }

    async fn get_product(
        &self,
        request: Request<proto::GetProductRequest>,
    ) -> Result<Response<proto::Product>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        store
            .products
            .get(&req.id)
            .map(|p| Response::new(core_to_proto_product(p)))
            .ok_or_else(|| Status::not_found(format!("Product '{}' not found", req.id)))
    }

    async fn update_product(
        &self,
        request: Request<proto::UpdateProductRequest>,
    ) -> Result<Response<proto::Product>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        let product = store
            .products
            .get_mut(&req.id)
            .ok_or_else(|| Status::not_found(format!("Product '{}' not found", req.id)))?;

        // Only allow updates in Draft status
        if !product.is_editable() {
            return Err(Status::failed_precondition(
                "Product can only be updated in DRAFT status",
            ));
        }

        if let Some(name) = req.name {
            product.name = name;
        }
        if let Some(desc) = req.description {
            product.description = Some(desc);
        }
        if let Some(eff) = req.effective_from {
            product.effective_from = Utc
                .timestamp_opt(eff, 0)
                .single()
                .ok_or_else(|| Status::invalid_argument("Invalid effective_from timestamp"))?;
        }
        if let Some(exp) = req.expiry_at {
            product.expiry_at = Some(
                Utc.timestamp_opt(exp, 0)
                    .single()
                    .ok_or_else(|| Status::invalid_argument("Invalid expiry_at timestamp"))?,
            );
        }

        product.updated_at = Utc::now();
        product.version += 1;

        product.validate().map_err(|e| Status::invalid_argument(e.to_string()))?;

        Ok(Response::new(core_to_proto_product(product)))
    }

    async fn delete_product(
        &self,
        request: Request<proto::DeleteProductRequest>,
    ) -> Result<Response<proto::DeleteResponse>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // Only allow deletion in Draft status
        if let Some(product) = store.products.get(&req.id) {
            if !product.is_editable() {
                return Err(Status::failed_precondition(
                    "Only DRAFT products can be deleted",
                ));
            }
        }

        let existed = store.products.remove(&req.id).is_some();

        // Clean up related data
        if existed {
            store.abstract_attrs_by_product.remove(&req.id);
            store.attrs_by_product.remove(&req.id);
            store.rules_by_product.remove(&req.id);
            store.funcs_by_product.remove(&req.id);
        }

        Ok(Response::new(proto::DeleteResponse {
            success: existed,
            message: None,
        }))
    }

    async fn list_products(
        &self,
        request: Request<proto::ListProductsRequest>,
    ) -> Result<Response<proto::ListProductsResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let mut products: Vec<_> = store.products.values().collect();

        // Apply filters
        if let Some(status) = req.status_filter {
            let filter_status = proto_to_core_product_status(status);
            products.retain(|p| p.status == filter_status);
        }
        if let Some(ref template) = req.template_type_filter {
            products.retain(|p| p.template_type.as_str() == template);
        }

        // Sort by created_at descending
        products.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let total_count = products.len() as i32;

        // Pagination
        let page_size = if req.page_size > 0 {
            req.page_size as usize
        } else {
            50
        };
        let offset = parse_page_token(&req.page_token)?;

        let paginated: Vec<proto::Product> = products
            .into_iter()
            .skip(offset)
            .take(page_size)
            .map(core_to_proto_product)
            .collect();

        let next_token = if paginated.len() == page_size {
            (offset + page_size).to_string()
        } else {
            String::new()
        };

        Ok(Response::new(proto::ListProductsResponse {
            products: paginated,
            next_page_token: next_token,
            total_count,
        }))
    }

    async fn clone_product(
        &self,
        request: Request<proto::CloneProductRequest>,
    ) -> Result<Response<proto::CloneProductResponse>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // Get source product
        let source_product = store
            .products
            .get(&req.source_product_id)
            .ok_or_else(|| {
                Status::not_found(format!("Source product '{}' not found", req.source_product_id))
            })?
            .clone();

        // Check new ID doesn't exist
        if store.products.contains_key(&req.new_product_id) {
            return Err(Status::already_exists(format!(
                "Product '{}' already exists",
                req.new_product_id
            )));
        }

        // Validate new product ID
        if !product_farm_core::validation::is_valid_product_id(&req.new_product_id) {
            return Err(Status::invalid_argument(format!(
                "Invalid product ID '{}'",
                req.new_product_id
            )));
        }

        // Gather source data
        let source_abstract_attrs: Vec<_> = store
            .get_abstract_attrs_for_product(&req.source_product_id)
            .into_iter()
            .cloned()
            .collect();
        let source_attrs: Vec<_> = store
            .get_attrs_for_product(&req.source_product_id)
            .into_iter()
            .cloned()
            .collect();
        let source_rules: Vec<_> = store
            .get_rules_for_product(&req.source_product_id)
            .into_iter()
            .cloned()
            .collect();
        let source_funcs: Vec<_> = store
            .get_funcs_for_product(&req.source_product_id)
            .into_iter()
            .cloned()
            .collect();

        let effective_from = Utc
            .timestamp_opt(req.effective_from, 0)
            .single()
            .ok_or_else(|| Status::invalid_argument("Invalid effective_from timestamp"))?;

        // Build selections from request (if any provided)
        let selections = if req.selected_components.is_empty()
            && req.selected_datatypes.is_empty()
            && req.selected_enumerations.is_empty()
            && req.selected_functionalities.is_empty()
            && req.selected_abstract_attributes.is_empty()
        {
            None // Full clone - no filtering
        } else {
            Some(CloneSelections {
                components: req.selected_components,
                datatypes: req.selected_datatypes,
                enumerations: req.selected_enumerations,
                functionalities: req.selected_functionalities,
                abstract_attributes: req.selected_abstract_attributes,
            })
        };

        // Perform deep clone
        let clone_request = CoreCloneRequest {
            new_product_id: ProductId::new(&req.new_product_id),
            new_name: req.new_name,
            new_description: req.new_description,
            effective_from,
            selections,
            // Default to true for backward compatibility (clone concrete attrs)
            clone_concrete_attributes: req.clone_concrete_attributes,
        };

        let result = ProductCloneService::clone_product(
            &source_product,
            &source_abstract_attrs,
            &source_attrs,
            &source_rules,
            &source_funcs,
            clone_request,
        )
        .map_err(|e| Status::internal(format!("Clone failed: {}", e)))?;

        // Store cloned entities
        let abstract_attrs_cloned = result.abstract_attributes.len() as i32;
        let attributes_cloned = result.attributes.len() as i32;
        let rules_cloned = result.rules.len() as i32;
        let functionalities_cloned = result.functionalities.len() as i32;

        let new_product_id = result.product.id.as_str().to_string();

        // Store product - use entry API to ensure atomicity
        // (defensive: the write lock is held throughout, but this makes intent clear)
        use hashbrown::hash_map::Entry;
        match store.products.entry(new_product_id.clone()) {
            Entry::Occupied(_) => {
                // This shouldn't happen since we checked earlier and hold the write lock,
                // but handle it defensively
                return Err(Status::already_exists(format!(
                    "Product '{}' was created concurrently",
                    new_product_id
                )));
            }
            Entry::Vacant(entry) => {
                entry.insert(result.product.clone());
            }
        }

        // Store abstract attributes
        let mut aa_paths = vec![];
        for attr in result.abstract_attributes {
            let path = attr.abstract_path.as_str().to_string();
            aa_paths.push(path.clone());
            store.abstract_attributes.insert(path, attr);
        }
        store.abstract_attrs_by_product.insert(new_product_id.clone(), aa_paths);

        // Store attributes
        let mut attr_paths = vec![];
        for attr in result.attributes {
            let path = attr.path.as_str().to_string();
            attr_paths.push(path.clone());
            store.attributes.insert(path, attr);
        }
        store.attrs_by_product.insert(new_product_id.clone(), attr_paths);

        // Store rules
        let mut rule_ids = vec![];
        for rule in result.rules {
            let id = rule.id.to_string();
            rule_ids.push(id.clone());
            store.rules.insert(id, rule);
        }
        store.rules_by_product.insert(new_product_id.clone(), rule_ids);

        // Store functionalities
        let mut func_keys = vec![];
        for func in result.functionalities {
            let key = EntityStore::functionality_key(&new_product_id, &func.name);
            func_keys.push(key.clone());
            store.functionalities.insert(key, func);
        }
        store.funcs_by_product.insert(new_product_id, func_keys);

        Ok(Response::new(proto::CloneProductResponse {
            product: Some(core_to_proto_product(&result.product)),
            abstract_attributes_cloned: abstract_attrs_cloned,
            attributes_cloned,
            rules_cloned,
            functionalities_cloned,
        }))
    }

    async fn submit_product(
        &self,
        request: Request<proto::SubmitProductRequest>,
    ) -> Result<Response<proto::Product>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        let product = store
            .products
            .get_mut(&req.id)
            .ok_or_else(|| Status::not_found(format!("Product '{}' not found", req.id)))?;

        product
            .submit()
            .map_err(|e| Status::failed_precondition(e.to_string()))?;

        Ok(Response::new(core_to_proto_product(product)))
    }

    async fn approve_product(
        &self,
        request: Request<proto::ApproveProductRequest>,
    ) -> Result<Response<proto::Product>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        let product = store
            .products
            .get_mut(&req.id)
            .ok_or_else(|| Status::not_found(format!("Product '{}' not found", req.id)))?;

        product
            .approve()
            .map_err(|e| Status::failed_precondition(e.to_string()))?;

        Ok(Response::new(core_to_proto_product(product)))
    }

    async fn reject_product(
        &self,
        request: Request<proto::RejectProductRequest>,
    ) -> Result<Response<proto::Product>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        let product = store
            .products
            .get_mut(&req.id)
            .ok_or_else(|| Status::not_found(format!("Product '{}' not found", req.id)))?;

        product
            .reject()
            .map_err(|e| Status::failed_precondition(e.to_string()))?;

        Ok(Response::new(core_to_proto_product(product)))
    }

    async fn discontinue_product(
        &self,
        request: Request<proto::DiscontinueProductRequest>,
    ) -> Result<Response<proto::Product>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        let product = store
            .products
            .get_mut(&req.id)
            .ok_or_else(|| Status::not_found(format!("Product '{}' not found", req.id)))?;

        product
            .discontinue()
            .map_err(|e| Status::failed_precondition(e.to_string()))?;

        Ok(Response::new(core_to_proto_product(product)))
    }
}

// ============================================================================
// ABSTRACT ATTRIBUTE SERVICE
// ============================================================================

pub struct AbstractAttributeGrpcService {
    store: SharedStore,
}

impl AbstractAttributeGrpcService {
    pub fn new(store: SharedStore) -> Self {
        Self { store }
    }
}

#[tonic::async_trait]
impl proto::abstract_attribute_service_server::AbstractAttributeService
    for AbstractAttributeGrpcService
{
    async fn create_abstract_attribute(
        &self,
        request: Request<proto::CreateAbstractAttributeRequest>,
    ) -> Result<Response<proto::AbstractAttribute>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // Check product exists and is editable
        let product = store
            .products
            .get(&req.product_id)
            .ok_or_else(|| Status::not_found(format!("Product '{}' not found", req.product_id)))?;

        if !product.is_editable() {
            return Err(Status::failed_precondition(
                "Product must be in DRAFT status to add attributes",
            ));
        }

        // Build abstract path
        let abstract_path = AbstractPath::build(
            &req.product_id,
            &req.component_type,
            req.component_id.as_deref(),
            &req.attribute_name,
        );

        // Create attribute
        let mut attr = AbstractAttribute::new(
            abstract_path.clone(),
            req.product_id.as_str(),
            req.component_type.as_str(),
            req.datatype_id.as_str(),
        );

        if let Some(ref cid) = req.component_id {
            attr = attr.with_component_id(cid.as_str());
        }
        if let Some(ref enum_name) = req.enum_name {
            attr = attr.with_enum(enum_name.as_str());
        }
        if req.immutable {
            attr = attr.immutable();
        }
        if let Some(ref desc) = req.description {
            attr = attr.with_description(desc.as_str());
        }
        if let Some(ref constraint) = req.constraint_expression {
            let expr: serde_json::Value = serde_json::from_str(constraint)
                .map_err(|e| Status::invalid_argument(format!("Invalid constraint JSON: {}", e)))?;
            attr = attr.with_constraint(expr);
        }

        // Add tags
        for (order, tag_name) in req.tags.iter().enumerate() {
            attr = attr.with_tag_name(tag_name.as_str(), order as i32);
        }

        // Add display names
        for dn in &req.display_names {
            attr.display_names.push(AttributeDisplayName::for_abstract(
                req.product_id.clone(),
                abstract_path.clone(),
                dn.display_name.as_str(),
                proto_to_core_display_format(dn.format),
                dn.order,
            ));
        }

        // Validate
        attr.validate().map_err(|e| Status::invalid_argument(e.to_string()))?;

        let proto_attr = core_to_proto_abstract_attribute(&attr);
        let path_str = abstract_path.as_str().to_string();

        // Use entry API to check and insert atomically
        use hashbrown::hash_map::Entry;
        match store.abstract_attributes.entry(path_str.clone()) {
            Entry::Occupied(_) => {
                return Err(Status::already_exists(format!(
                    "Abstract attribute '{}' already exists",
                    abstract_path.as_str()
                )));
            }
            Entry::Vacant(entry) => {
                entry.insert(attr);
            }
        }
        store
            .abstract_attrs_by_product
            .entry(req.product_id)
            .or_default()
            .push(path_str);

        Ok(Response::new(proto_attr))
    }

    async fn get_abstract_attribute(
        &self,
        request: Request<proto::GetAbstractAttributeRequest>,
    ) -> Result<Response<proto::AbstractAttribute>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        store
            .abstract_attributes
            .get(&req.abstract_path)
            .map(|a| Response::new(core_to_proto_abstract_attribute(a)))
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Abstract attribute '{}' not found",
                    req.abstract_path
                ))
            })
    }

    async fn update_abstract_attribute(
        &self,
        request: Request<proto::UpdateAbstractAttributeRequest>,
    ) -> Result<Response<proto::AbstractAttribute>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // Check product is editable
        let product = store
            .products
            .get(&req.product_id)
            .ok_or_else(|| Status::not_found(format!("Product '{}' not found", req.product_id)))?;

        if !product.is_editable() {
            return Err(Status::failed_precondition(
                "Product must be in DRAFT status to update attributes",
            ));
        }

        let attr = store
            .abstract_attributes
            .get_mut(&req.abstract_path)
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Abstract attribute '{}' not found",
                    req.abstract_path
                ))
            })?;

        // Check attribute is modifiable
        attr.check_modifiable().map_err(|e| Status::failed_precondition(e.to_string()))?;

        if let Some(desc) = req.description {
            attr.description = Some(desc);
        }

        if let Some(constraint) = req.constraint_expression {
            let expr: serde_json::Value = serde_json::from_str(&constraint)
                .map_err(|e| Status::invalid_argument(format!("Invalid constraint JSON: {}", e)))?;
            attr.constraint_expression = Some(expr);
        }

        // Update tags if provided
        if !req.tags.is_empty() {
            attr.tags = req
                .tags
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    AbstractAttributeTag::new(
                        attr.abstract_path.clone(),
                        name.as_str(),
                        attr.product_id.clone(),
                        i as i32,
                    )
                })
                .collect();
        }

        // Update display names if provided
        if !req.display_names.is_empty() {
            attr.display_names = req
                .display_names
                .iter()
                .map(|dn| {
                    AttributeDisplayName::for_abstract(
                        attr.product_id.clone(),
                        attr.abstract_path.clone(),
                        dn.display_name.as_str(),
                        proto_to_core_display_format(dn.format),
                        dn.order,
                    )
                })
                .collect();
        }

        attr.validate().map_err(|e| Status::invalid_argument(e.to_string()))?;

        Ok(Response::new(core_to_proto_abstract_attribute(attr)))
    }

    async fn delete_abstract_attribute(
        &self,
        request: Request<proto::DeleteAbstractAttributeRequest>,
    ) -> Result<Response<proto::DeleteResponse>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // Check product is editable
        if let Some(product) = store.products.get(&req.product_id) {
            if !product.is_editable() {
                return Err(Status::failed_precondition(
                    "Product must be in DRAFT status to delete attributes",
                ));
            }
        }

        // Check attribute is not immutable
        if let Some(attr) = store.abstract_attributes.get(&req.abstract_path) {
            if attr.immutable {
                return Err(Status::failed_precondition("Cannot delete immutable attribute"));
            }
        }

        let existed = store.abstract_attributes.remove(&req.abstract_path).is_some();

        if existed {
            if let Some(paths) = store.abstract_attrs_by_product.get_mut(&req.product_id) {
                paths.retain(|p| p != &req.abstract_path);
            }
        }

        Ok(Response::new(proto::DeleteResponse {
            success: existed,
            message: None,
        }))
    }

    async fn list_abstract_attributes(
        &self,
        request: Request<proto::ListAbstractAttributesRequest>,
    ) -> Result<Response<proto::ListAbstractAttributesResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let attrs: Vec<_> = store.get_abstract_attrs_for_product(&req.product_id);
        let total_count = attrs.len() as i32;

        // Pagination
        let page_size = if req.page_size > 0 {
            req.page_size as usize
        } else {
            50
        };
        let offset = parse_page_token(&req.page_token)?;

        let paginated: Vec<proto::AbstractAttribute> = attrs
            .into_iter()
            .skip(offset)
            .take(page_size)
            .map(core_to_proto_abstract_attribute)
            .collect();

        let next_token = if paginated.len() == page_size {
            (offset + page_size).to_string()
        } else {
            String::new()
        };

        Ok(Response::new(proto::ListAbstractAttributesResponse {
            attributes: paginated,
            next_page_token: next_token,
            total_count,
        }))
    }

    async fn get_abstract_attributes_by_component(
        &self,
        request: Request<proto::GetByComponentRequest>,
    ) -> Result<Response<proto::ListAbstractAttributesResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let attrs = store.get_abstract_attrs_by_component(
            &req.product_id,
            &req.component_type,
            req.component_id.as_deref(),
        );

        let proto_attrs: Vec<_> = attrs.into_iter().map(core_to_proto_abstract_attribute).collect();
        let total_count = proto_attrs.len() as i32;

        Ok(Response::new(proto::ListAbstractAttributesResponse {
            attributes: proto_attrs,
            next_page_token: String::new(),
            total_count,
        }))
    }

    async fn get_abstract_attributes_by_tag(
        &self,
        request: Request<proto::GetByTagRequest>,
    ) -> Result<Response<proto::ListAbstractAttributesResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let attrs = store.get_abstract_attrs_by_tag(&req.product_id, &req.tag);
        let proto_attrs: Vec<_> = attrs.into_iter().map(core_to_proto_abstract_attribute).collect();
        let total_count = proto_attrs.len() as i32;

        Ok(Response::new(proto::ListAbstractAttributesResponse {
            attributes: proto_attrs,
            next_page_token: String::new(),
            total_count,
        }))
    }

    async fn get_abstract_attributes_by_tags(
        &self,
        request: Request<proto::GetByTagsRequest>,
    ) -> Result<Response<proto::ListAbstractAttributesResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let attrs = store.get_abstract_attrs_by_tags(&req.product_id, &req.tags, req.match_all);
        let proto_attrs: Vec<_> = attrs.into_iter().map(core_to_proto_abstract_attribute).collect();
        let total_count = proto_attrs.len() as i32;

        Ok(Response::new(proto::ListAbstractAttributesResponse {
            attributes: proto_attrs,
            next_page_token: String::new(),
            total_count,
        }))
    }

    async fn get_abstract_attributes_by_functionality(
        &self,
        request: Request<proto::GetByFunctionalityRequest>,
    ) -> Result<Response<proto::ListAbstractAttributesResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let attrs = store.get_abstract_attrs_by_functionality(&req.product_id, &req.functionality_name);
        let proto_attrs: Vec<_> = attrs.into_iter().map(core_to_proto_abstract_attribute).collect();
        let total_count = proto_attrs.len() as i32;

        Ok(Response::new(proto::ListAbstractAttributesResponse {
            attributes: proto_attrs,
            next_page_token: String::new(),
            total_count,
        }))
    }
}

// ============================================================================
// ATTRIBUTE SERVICE (Concrete Attributes)
// ============================================================================

pub struct AttributeGrpcService {
    store: SharedStore,
}

impl AttributeGrpcService {
    pub fn new(store: SharedStore) -> Self {
        Self { store }
    }
}

#[tonic::async_trait]
impl proto::attribute_service_server::AttributeService for AttributeGrpcService {
    async fn create_attribute(
        &self,
        request: Request<proto::CreateAttributeRequest>,
    ) -> Result<Response<proto::Attribute>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // Check product exists and is editable
        let product = store
            .products
            .get(&req.product_id)
            .ok_or_else(|| Status::not_found(format!("Product '{}' not found", req.product_id)))?;

        if !product.is_editable() {
            return Err(Status::failed_precondition(
                "Product must be in DRAFT status to add attributes",
            ));
        }

        // Build concrete path
        let path = ConcretePath::build(
            &req.product_id,
            &req.component_type,
            &req.component_id,
            &req.attribute_name,
        );

        // Verify abstract path exists
        let abstract_path = AbstractPath::new(req.abstract_path.as_str());
        if !store.abstract_attributes.contains_key(abstract_path.as_str()) {
            return Err(Status::failed_precondition(format!(
                "Abstract attribute '{}' not found",
                abstract_path.as_str()
            )));
        }

        // Create attribute based on value type
        let attr = if let Some(ref v) = req.value {
            // Fixed value attribute
            Attribute::new_fixed_value(
                path.clone(),
                abstract_path,
                req.product_id.as_str(),
                try_proto_to_core_value(v)?,
            )
        } else if let Some(ref rule_id) = req.rule_id {
            // Rule-driven attribute
            Attribute::new_rule_driven(
                path.clone(),
                abstract_path,
                req.product_id.as_str(),
                RuleId::from_string(rule_id),
            )
        } else {
            // Just a definition
            Attribute::new_just_definition(
                path.clone(),
                abstract_path,
                req.product_id.as_str(),
            )
        };

        // Validate
        attr.validate().map_err(|e| Status::invalid_argument(e.to_string()))?;

        let proto_attr = core_to_proto_attribute(&attr);
        let path_str = path.as_str().to_string();

        // Use entry API to check and insert atomically
        use hashbrown::hash_map::Entry;
        match store.attributes.entry(path_str.clone()) {
            Entry::Occupied(_) => {
                return Err(Status::already_exists(format!(
                    "Attribute '{}' already exists",
                    path.as_str()
                )));
            }
            Entry::Vacant(entry) => {
                entry.insert(attr);
            }
        }
        store
            .attrs_by_product
            .entry(req.product_id)
            .or_default()
            .push(path_str);

        Ok(Response::new(proto_attr))
    }

    async fn get_attribute(
        &self,
        request: Request<proto::GetAttributeRequest>,
    ) -> Result<Response<proto::Attribute>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        store
            .attributes
            .get(&req.path)
            .map(|a| Response::new(core_to_proto_attribute(a)))
            .ok_or_else(|| Status::not_found(format!("Attribute '{}' not found", req.path)))
    }

    async fn update_attribute(
        &self,
        request: Request<proto::UpdateAttributeRequest>,
    ) -> Result<Response<proto::Attribute>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // Check product is editable
        let product = store
            .products
            .get(&req.product_id)
            .ok_or_else(|| Status::not_found(format!("Product '{}' not found", req.product_id)))?;

        if !product.is_editable() {
            return Err(Status::failed_precondition(
                "Product must be in DRAFT status to update attributes",
            ));
        }

        let attr = store
            .attributes
            .get_mut(&req.path)
            .ok_or_else(|| Status::not_found(format!("Attribute '{}' not found", req.path)))?;

        if let Some(ref v) = req.value {
            attr.value = Some(try_proto_to_core_value(v)?);
        }

        if let Some(ref rule_id) = req.rule_id {
            attr.rule_id = Some(RuleId::from_string(rule_id));
        }

        // Update display names if provided
        if !req.display_names.is_empty() {
            attr.display_names = req
                .display_names
                .iter()
                .map(|dn| {
                    AttributeDisplayName::for_concrete(
                        attr.product_id.clone(),
                        attr.path.clone(),
                        &dn.display_name,
                        proto_to_core_display_format(dn.format),
                        dn.order,
                    )
                })
                .collect();
        }

        attr.validate().map_err(|e| Status::invalid_argument(e.to_string()))?;

        Ok(Response::new(core_to_proto_attribute(attr)))
    }

    async fn delete_attribute(
        &self,
        request: Request<proto::DeleteAttributeRequest>,
    ) -> Result<Response<proto::DeleteResponse>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // Check product is editable
        if let Some(product) = store.products.get(&req.product_id) {
            if !product.is_editable() {
                return Err(Status::failed_precondition(
                    "Product must be in DRAFT status to delete attributes",
                ));
            }
        }

        let existed = store.attributes.remove(&req.path).is_some();

        if existed {
            if let Some(paths) = store.attrs_by_product.get_mut(&req.product_id) {
                paths.retain(|p| p != &req.path);
            }
        }

        Ok(Response::new(proto::DeleteResponse {
            success: existed,
            message: None,
        }))
    }

    async fn list_attributes(
        &self,
        request: Request<proto::ListAttributesRequest>,
    ) -> Result<Response<proto::ListAttributesResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let attrs: Vec<_> = store.get_attrs_for_product(&req.product_id);
        let total_count = attrs.len() as i32;

        // Pagination
        let page_size = if req.page_size > 0 {
            req.page_size as usize
        } else {
            50
        };
        let offset = parse_page_token(&req.page_token)?;

        let paginated: Vec<proto::Attribute> = attrs
            .into_iter()
            .skip(offset)
            .take(page_size)
            .map(core_to_proto_attribute)
            .collect();

        let next_token = if paginated.len() == page_size {
            (offset + page_size).to_string()
        } else {
            String::new()
        };

        Ok(Response::new(proto::ListAttributesResponse {
            attributes: paginated,
            next_page_token: next_token,
            total_count,
        }))
    }

    async fn get_attributes_by_tag(
        &self,
        request: Request<proto::GetAttributesByTagRequest>,
    ) -> Result<Response<proto::ListAttributesResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let attrs = store.get_attrs_by_tag(&req.product_id, &req.tag);
        let proto_attrs: Vec<_> = attrs.into_iter().map(core_to_proto_attribute).collect();
        let total_count = proto_attrs.len() as i32;

        Ok(Response::new(proto::ListAttributesResponse {
            attributes: proto_attrs,
            next_page_token: String::new(),
            total_count,
        }))
    }

    async fn get_attributes_by_functionality(
        &self,
        request: Request<proto::GetAttributesByFunctionalityRequest>,
    ) -> Result<Response<proto::ListAttributesResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        // Get abstract attributes for functionality
        let abstract_attrs = store.get_abstract_attrs_by_functionality(&req.product_id, &req.functionality_name);
        let abstract_paths: std::collections::HashSet<_> = abstract_attrs
            .iter()
            .map(|a| a.abstract_path.as_str())
            .collect();

        // Get concrete attributes matching those abstract paths
        let attrs: Vec<_> = store
            .get_attrs_for_product(&req.product_id)
            .into_iter()
            .filter(|a| abstract_paths.contains(a.abstract_path.as_str()))
            .map(core_to_proto_attribute)
            .collect();

        let total_count = attrs.len() as i32;

        Ok(Response::new(proto::ListAttributesResponse {
            attributes: attrs,
            next_page_token: String::new(),
            total_count,
        }))
    }
}

// ============================================================================
// RULE SERVICE
// ============================================================================

pub struct RuleGrpcService {
    store: SharedStore,
}

impl RuleGrpcService {
    pub fn new(store: SharedStore) -> Self {
        Self { store }
    }
}

#[tonic::async_trait]
impl proto::rule_service_server::RuleService for RuleGrpcService {
    async fn create_rule(
        &self,
        request: Request<proto::CreateRuleRequest>,
    ) -> Result<Response<proto::Rule>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // Check product exists and is editable
        let product = store
            .products
            .get(&req.product_id)
            .ok_or_else(|| Status::not_found(format!("Product '{}' not found", req.product_id)))?;

        if !product.is_editable() {
            return Err(Status::failed_precondition(
                "Product must be in DRAFT status to add rules",
            ));
        }

        // Validate JSON expression
        let _: serde_json::Value = serde_json::from_str(&req.expression_json)
            .map_err(|e| Status::invalid_argument(format!("Invalid expression JSON: {}", e)))?;

        // Create rule
        let rule_id = RuleId::new();
        let mut rule = Rule::new(
            req.product_id.as_str(),
            req.rule_type.as_str(),
            req.expression_json.as_str(),
        )
        .with_id(rule_id.clone())
        .with_display(req.display_expression.as_str())
        .with_order(req.order_index);

        if let Some(ref desc) = req.description {
            rule = rule.with_description(desc.as_str());
        }

        // Set inputs
        rule.input_attributes = req
            .input_attributes
            .iter()
            .enumerate()
            .map(|(i, path)| RuleInputAttribute::new(rule_id.clone(), path.as_str(), i as i32))
            .collect();

        // Set outputs
        rule.output_attributes = req
            .output_attributes
            .iter()
            .enumerate()
            .map(|(i, path)| RuleOutputAttribute::new(rule_id.clone(), path.as_str(), i as i32))
            .collect();

        // Validate
        rule.validate().map_err(|e| Status::invalid_argument(e.to_string()))?;

        let proto_rule = core_to_proto_rule(&rule);
        let id = rule_id.to_string();

        store.rules.insert(id.clone(), rule);
        store
            .rules_by_product
            .entry(req.product_id)
            .or_default()
            .push(id);

        Ok(Response::new(proto_rule))
    }

    async fn get_rule(
        &self,
        request: Request<proto::GetRuleRequest>,
    ) -> Result<Response<proto::Rule>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        store
            .rules
            .get(&req.id)
            .map(|r| Response::new(core_to_proto_rule(r)))
            .ok_or_else(|| Status::not_found(format!("Rule '{}' not found", req.id)))
    }

    async fn update_rule(
        &self,
        request: Request<proto::UpdateRuleRequest>,
    ) -> Result<Response<proto::Rule>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // First, check product is editable (immutable borrow)
        {
            let rule = store
                .rules
                .get(&req.id)
                .ok_or_else(|| Status::not_found(format!("Rule '{}' not found", req.id)))?;

            let product = store
                .products
                .get(rule.product_id.as_str())
                .ok_or_else(|| Status::not_found("Associated product not found"))?;

            if !product.is_editable() {
                return Err(Status::failed_precondition(
                    "Product must be in DRAFT status to update rules",
                ));
            }
        }

        // Now get mutable reference
        let rule = store
            .rules
            .get_mut(&req.id)
            .ok_or_else(|| Status::not_found(format!("Rule '{}' not found", req.id)))?;

        if let Some(rule_type) = req.rule_type {
            rule.rule_type = rule_type;
        }

        if let Some(display) = req.display_expression {
            rule.display_expression = display;
        }

        if let Some(expr) = req.expression_json {
            // Validate JSON
            let _: serde_json::Value = serde_json::from_str(&expr)
                .map_err(|e| Status::invalid_argument(format!("Invalid expression JSON: {}", e)))?;
            rule.compiled_expression = expr;
        }

        if let Some(desc) = req.description {
            rule.description = Some(desc);
        }

        if let Some(enabled) = req.enabled {
            rule.enabled = enabled;
        }

        if let Some(order) = req.order_index {
            rule.order_index = order;
        }

        // Update inputs if provided
        if !req.input_attributes.is_empty() {
            rule.input_attributes = req
                .input_attributes
                .iter()
                .enumerate()
                .map(|(i, path)| RuleInputAttribute::new(rule.id.clone(), path.as_str(), i as i32))
                .collect();
        }

        // Update outputs if provided
        if !req.output_attributes.is_empty() {
            rule.output_attributes = req
                .output_attributes
                .iter()
                .enumerate()
                .map(|(i, path)| RuleOutputAttribute::new(rule.id.clone(), path.as_str(), i as i32))
                .collect();
        }

        rule.validate().map_err(|e| Status::invalid_argument(e.to_string()))?;

        Ok(Response::new(core_to_proto_rule(rule)))
    }

    async fn delete_rule(
        &self,
        request: Request<proto::DeleteRuleRequest>,
    ) -> Result<Response<proto::DeleteResponse>, Status> {
        let req = request.into_inner();
        let mut store = self.store.write().await;

        // Check product is editable
        if let Some(rule) = store.rules.get(&req.id) {
            if let Some(product) = store.products.get(rule.product_id.as_str()) {
                if !product.is_editable() {
                    return Err(Status::failed_precondition(
                        "Product must be in DRAFT status to delete rules",
                    ));
                }
            }
        }

        let existed = if let Some(rule) = store.rules.remove(&req.id) {
            let product_id = rule.product_id.as_str().to_string();
            if let Some(ids) = store.rules_by_product.get_mut(&product_id) {
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

    async fn list_rules(
        &self,
        request: Request<proto::ListRulesRequest>,
    ) -> Result<Response<proto::ListRulesResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let mut rules: Vec<_> = store.get_rules_for_product(&req.product_id);

        // Apply filters
        if let Some(ref rule_type) = req.rule_type_filter {
            rules.retain(|r| &r.rule_type == rule_type);
        }
        if let Some(enabled) = req.enabled_filter {
            rules.retain(|r| r.enabled == enabled);
        }

        // Sort by order_index
        rules.sort_by_key(|r| r.order_index);

        let total_count = rules.len() as i32;

        // Pagination
        let page_size = if req.page_size > 0 {
            req.page_size as usize
        } else {
            50
        };
        let offset = parse_page_token(&req.page_token)?;

        let paginated: Vec<proto::Rule> = rules
            .into_iter()
            .skip(offset)
            .take(page_size)
            .map(core_to_proto_rule)
            .collect();

        let next_token = if paginated.len() == page_size {
            (offset + page_size).to_string()
        } else {
            String::new()
        };

        Ok(Response::new(proto::ListRulesResponse {
            rules: paginated,
            next_page_token: next_token,
            total_count,
        }))
    }
}

// ============================================================================
// DATATYPE SERVICE
// ============================================================================

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
            if let Some(abstract_attr) = store.abstract_attributes.get(attr.abstract_path.as_str()) {
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

// ============================================================================
// PRODUCT FUNCTIONALITY SERVICE
// ============================================================================

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
        func.validate().map_err(|e| Status::invalid_argument(e.to_string()))?;

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

        func.validate().map_err(|e| Status::invalid_argument(e.to_string()))?;

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

// ============================================================================
// PRODUCT TEMPLATE SERVICE
// ============================================================================

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
        enumeration.validate().map_err(|e| Status::invalid_argument(e.to_string()))?;

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

        enumeration.validate().map_err(|e| Status::invalid_argument(e.to_string()))?;

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
            if let Some(abstract_attr) = store.abstract_attributes.get(attr.abstract_path.as_str()) {
                // Check if abstract attribute uses this enumeration
                if let Some(ref enum_name) = abstract_attr.enum_name {
                    if enum_name == &req.id {
                        // Check if the concrete attribute has this value
                        if let Some(product_farm_core::Value::String(v)) = &attr.value {
                            if v == &req.value {
                                let value_type = match attr.value_type {
                                    product_farm_core::AttributeValueType::FixedValue => "FIXED_VALUE",
                                    product_farm_core::AttributeValueType::RuleDriven => "RULE_DRIVEN",
                                    product_farm_core::AttributeValueType::JustDefinition => "JUST_DEFINITION",
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
            if let Some(abstract_attr) = store.abstract_attributes.get(attr.abstract_path.as_str()) {
                if let Some(ref enum_name) = abstract_attr.enum_name {
                    if enum_name == &req.enumeration_id {
                        let value_type = match attr.value_type {
                            product_farm_core::AttributeValueType::FixedValue => "FIXED_VALUE",
                            product_farm_core::AttributeValueType::RuleDriven => "RULE_DRIVEN",
                            product_farm_core::AttributeValueType::JustDefinition => "JUST_DEFINITION",
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
        let enumeration = store
            .enumerations
            .get(&req.enumeration_id)
            .ok_or_else(|| {
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
            if let Some(abstract_attr) = store.abstract_attributes.get(attr.abstract_path.as_str()) {
                if let Some(ref enum_name) = abstract_attr.enum_name {
                    if enum_name == &req.enumeration_id {
                        // Check if the concrete attribute has this specific value
                        if let Some(product_farm_core::Value::String(v)) = &attr.value {
                            if v == &req.value {
                                let value_type = match attr.value_type {
                                    product_farm_core::AttributeValueType::FixedValue => "FIXED_VALUE",
                                    product_farm_core::AttributeValueType::RuleDriven => "RULE_DRIVEN",
                                    product_farm_core::AttributeValueType::JustDefinition => "JUST_DEFINITION",
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

// ============================================================================
// SERVICE CREATION HELPER
// ============================================================================

/// All gRPC services bundled together
pub struct AllServices {
    pub product_farm: ProductFarmGrpcService,
    pub product: ProductGrpcService,
    pub abstract_attribute: AbstractAttributeGrpcService,
    pub attribute: AttributeGrpcService,
    pub rule: RuleGrpcService,
    pub datatype: DatatypeGrpcService,
    pub functionality: ProductFunctionalityGrpcService,
    pub template: ProductTemplateGrpcService,
}

/// Create all gRPC services with a shared store
pub fn create_all_services(store: SharedStore) -> AllServices {
    AllServices {
        product_farm: ProductFarmGrpcService::new(store.clone()),
        product: ProductGrpcService::new(store.clone()),
        abstract_attribute: AbstractAttributeGrpcService::new(store.clone()),
        attribute: AttributeGrpcService::new(store.clone()),
        rule: RuleGrpcService::new(store.clone()),
        datatype: DatatypeGrpcService::new(store.clone()),
        functionality: ProductFunctionalityGrpcService::new(store.clone()),
        template: ProductTemplateGrpcService::new(store),
    }
}
