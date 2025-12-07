//! ProductFarmGrpcService - Rule evaluation service
//!
//! Provides rule evaluation, streaming evaluation, batch evaluation,
//! validation, execution plan, and health check endpoints.

use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

use product_farm_core::Rule;
use product_farm_rule_engine::{ExecutionContext, RuleDag, RuleExecutor};

use crate::converters::{core_to_proto_value, try_proto_to_core_value};
use crate::store::SharedStore;

use super::proto;

/// gRPC service for rule evaluation
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
