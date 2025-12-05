//! REST handlers for Rule Evaluation
//!
//! Provides HTTP endpoints for evaluating rules and attributes

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use hashbrown::HashMap as HashbrownHashMap;
use product_farm_core::{ProductId, Rule, Value};
use product_farm_rule_engine::{ExecutionContext, RuleExecutor};
use std::collections::HashMap;

use crate::store::SharedStore;

use super::error::{ApiError, ApiResult};
use super::types::*;

/// Create routes for evaluation endpoints
pub fn routes() -> Router<SharedStore> {
    Router::new()
        .route("/api/evaluate", post(evaluate))
        .route("/api/batch-evaluate", post(batch_evaluate))
        .route(
            "/api/products/:product_id/execution-plan",
            get(get_execution_plan),
        )
        .route("/api/products/:product_id/validate", get(validate_product))
        .route(
            "/api/products/:product_id/impact-analysis",
            post(impact_analysis),
        )
}

/// Evaluate rules for a product
async fn evaluate(
    State(store): State<SharedStore>,
    Json(req): Json<EvaluateRequest>,
) -> ApiResult<Json<EvaluateResponse>> {
    let store = store.read().await;

    // Verify product exists (store uses String keys)
    if !store.products.contains_key(&req.product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            req.product_id
        )));
    }

    let pid = ProductId::new(&req.product_id);

    // Get rules for this product
    let rules: Vec<Rule> = if req.rule_ids.is_empty() {
        // Get all enabled rules for the product
        store
            .rules
            .values()
            .filter(|r| r.product_id == pid && r.enabled)
            .cloned()
            .collect()
    } else {
        // Get specific rules by ID
        req.rule_ids
            .iter()
            .filter_map(|id| store.rules.get(id).cloned())
            .filter(|r| r.product_id == pid && r.enabled)
            .collect()
    };

    if rules.is_empty() {
        // No rules to execute - return empty success
        return Ok(Json(EvaluateResponse {
            success: true,
            outputs: HashMap::new(),
            rule_results: vec![],
            metrics: ExecutionMetricsJson {
                total_time_ns: 0,
                rules_executed: 0,
                rules_skipped: 0,
                cache_hits: 0,
                levels: vec![],
            },
            errors: vec![],
        }));
    }

    // Convert input data from JSON to Value (using hashbrown for ExecutionContext)
    let input_data: HashbrownHashMap<String, Value> = req
        .input_data
        .iter()
        .map(|(k, v)| (k.clone(), Value::from(v)))
        .collect();

    // Create execution context
    let mut context = ExecutionContext::new(input_data);

    // Execute rules
    let mut executor = RuleExecutor::new();
    let result = executor.execute(&rules, &mut context).map_err(|e| {
        ApiError::Internal(format!("Rule execution failed: {}", e))
    })?;

    // Convert outputs from Value to AttributeValueJson
    let outputs: HashMap<String, AttributeValueJson> = result
        .context
        .computed_values()
        .iter()
        .map(|(k, v)| (k.clone(), AttributeValueJson::from(v)))
        .collect();

    // Build rule results
    let rule_results: Vec<RuleResultJson> = result
        .rule_results
        .iter()
        .map(|rr| RuleResultJson {
            rule_id: rr.rule_id.to_string(),
            outputs: rr
                .outputs
                .iter()
                .map(|(path, value)| OutputValueJson {
                    path: path.clone(),
                    value: AttributeValueJson::from(value),
                })
                .collect(),
            execution_time_ns: rr.execution_time_ns as i64,
            skipped: false,
            skip_reason: None,
            error: None,
        })
        .collect();

    // Build level metrics
    let level_metrics: Vec<LevelMetricsJson> = result
        .levels
        .iter()
        .enumerate()
        .map(|(i, rule_ids)| LevelMetricsJson {
            level: i as i32,
            time_ns: 0, // Individual level timing not tracked
            rules_count: rule_ids.len() as i32,
        })
        .collect();

    Ok(Json(EvaluateResponse {
        success: true,
        outputs,
        rule_results,
        metrics: ExecutionMetricsJson {
            total_time_ns: result.total_time_ns as i64,
            rules_executed: result.rule_results.len() as i32,
            rules_skipped: 0,
            cache_hits: 0,
            levels: level_metrics,
        },
        errors: vec![],
    }))
}

/// Batch evaluate multiple requests for a product
async fn batch_evaluate(
    State(store): State<SharedStore>,
    Json(req): Json<BatchEvaluateRequest>,
) -> ApiResult<Json<BatchEvaluateResponse>> {
    let store = store.read().await;
    let start_time = std::time::Instant::now();

    // Verify product exists (store uses String keys)
    if !store.products.contains_key(&req.product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            req.product_id
        )));
    }

    let pid = ProductId::new(&req.product_id);

    // Get all enabled rules for the product (reused across batch)
    let rules: Vec<Rule> = store
        .rules
        .values()
        .filter(|r| r.product_id == pid && r.enabled)
        .cloned()
        .collect();

    // Create executor once and reuse (maintains compiled rule cache)
    let mut executor = RuleExecutor::new();

    // Pre-compile rules once for efficiency
    if !rules.is_empty() {
        executor.compile_rules(&rules).map_err(|e| {
            ApiError::Internal(format!("Failed to compile rules: {}", e))
        })?;
    }

    // Process each request in the batch
    let mut results: Vec<BatchResultItem> = Vec::with_capacity(req.requests.len());

    for batch_item in &req.requests {
        // Convert input data from JSON to Value (using hashbrown for ExecutionContext)
        let input_data: HashbrownHashMap<String, Value> = batch_item
            .input_data
            .iter()
            .map(|(k, v)| (k.clone(), Value::from(v)))
            .collect();

        // Create execution context
        let mut context = ExecutionContext::new(input_data);

        // Execute rules
        match executor.execute(&rules, &mut context) {
            Ok(exec_result) => {
                // Convert outputs to AttributeResult
                let attribute_results: Vec<AttributeResult> = exec_result
                    .context
                    .computed_values()
                    .iter()
                    .map(|(path, value)| AttributeResult {
                        path: path.clone(),
                        value: AttributeValueJson::from(value),
                        computed: true, // These are computed values
                    })
                    .collect();

                results.push(BatchResultItem {
                    request_id: batch_item.request_id.clone(),
                    results: attribute_results,
                    success: true,
                    error: None,
                });
            }
            Err(e) => {
                results.push(BatchResultItem {
                    request_id: batch_item.request_id.clone(),
                    results: vec![],
                    success: false,
                    error: Some(format!("Evaluation failed: {}", e)),
                });
            }
        }
    }

    let total_time_ns = start_time.elapsed().as_nanos() as i64;

    Ok(Json(BatchEvaluateResponse {
        results,
        total_time_ns,
    }))
}

/// Get the execution plan for a product's rules
async fn get_execution_plan(
    State(store): State<SharedStore>,
    Path(product_id): Path<String>,
) -> ApiResult<Json<ExecutionPlanResponse>> {
    let store = store.read().await;

    // Verify product exists (store uses String keys)
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    // Collect rules for this product
    let mut rules: Vec<_> = store
        .rules
        .values()
        .filter(|r| r.product_id == pid && r.enabled)
        .collect();

    // Sort by order_index
    rules.sort_by_key(|r| r.order_index);

    // Build execution levels (group rules by order)
    let mut level_map: HashMap<i32, Vec<String>> = HashMap::new();
    for rule in &rules {
        level_map
            .entry(rule.order_index)
            .or_default()
            .push(rule.id.to_string());
    }
    let mut levels: Vec<ExecutionLevelJson> = level_map
        .into_iter()
        .map(|(level, rule_ids)| ExecutionLevelJson { level, rule_ids })
        .collect();
    levels.sort_by_key(|l| l.level);

    // Build dependencies
    let dependencies: Vec<RuleDependencyJson> = rules
        .iter()
        .map(|rule| RuleDependencyJson {
            rule_id: rule.id.to_string(),
            depends_on: rule
                .input_attributes
                .iter()
                .map(|a| a.path.as_str().to_string())
                .collect(),
            produces: rule
                .output_attributes
                .iter()
                .map(|a| a.path.as_str().to_string())
                .collect(),
        })
        .collect();

    // Find missing inputs (inputs that no rule produces)
    let all_outputs: std::collections::HashSet<String> = rules
        .iter()
        .flat_map(|r| r.output_attributes.iter().map(|a| a.path.as_str().to_string()))
        .collect();

    let missing_inputs: Vec<MissingInputJson> = rules
        .iter()
        .flat_map(|rule| {
            rule.input_attributes.iter().filter_map(|input| {
                let path = input.path.as_str().to_string();
                if !all_outputs.contains(&path) {
                    Some(MissingInputJson {
                        rule_id: rule.id.to_string(),
                        input_path: path,
                    })
                } else {
                    None
                }
            })
        })
        .collect();

    // Generate simple text-based graphs (placeholders)
    let dot_graph = format!(
        "digraph ExecutionPlan {{ {} }}",
        rules
            .iter()
            .map(|r| format!("\"{}\"", r.id))
            .collect::<Vec<_>>()
            .join("; ")
    );

    let mermaid_graph = format!(
        "graph TD\n{}",
        rules
            .iter()
            .map(|r| format!("  {}[{}]", r.id, r.rule_type))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let ascii_graph = format!(
        "Execution Plan:\n{}",
        levels
            .iter()
            .map(|l| format!("  Level {}: {:?}", l.level, l.rule_ids))
            .collect::<Vec<_>>()
            .join("\n")
    );

    Ok(Json(ExecutionPlanResponse {
        levels,
        dependencies,
        missing_inputs,
        dot_graph,
        mermaid_graph,
        ascii_graph,
    }))
}

/// Validate a product's configuration
async fn validate_product(
    State(store): State<SharedStore>,
    Path(product_id): Path<String>,
) -> ApiResult<Json<ValidationResponse>> {
    let store = store.read().await;

    // Verify product exists (store uses String keys)
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);

    let mut errors: Vec<ValidationError> = Vec::new();
    let mut warnings: Vec<ValidationWarning> = Vec::new();

    // Check for missing required attributes in functionalities
    for functionality in store.functionalities.values().filter(|f| f.product_id == pid) {
        for required in &functionality.required_attributes {
            let has_concrete = store
                .attributes
                .values()
                .any(|a| a.abstract_path == required.abstract_path && a.product_id == pid);

            if !has_concrete {
                errors.push(ValidationError {
                    code: "MISSING_REQUIRED_ATTRIBUTE".to_string(),
                    message: format!(
                        "Functionality '{}' requires attribute '{}' which has no concrete implementation",
                        functionality.name,
                        required.abstract_path.as_str()
                    ),
                    path: Some(required.abstract_path.as_str().to_string()),
                    severity: "error".to_string(),
                });
            }
        }
    }

    // Check for attributes without values (when value_type is Static)
    for attr in store.attributes.values().filter(|a| a.product_id == pid) {
        if attr.value.is_none() && attr.rule_id.is_none() {
            warnings.push(ValidationWarning {
                code: "ATTRIBUTE_NO_VALUE".to_string(),
                message: format!(
                    "Attribute '{}' has no value and no associated rule",
                    attr.path.as_str()
                ),
                path: Some(attr.path.as_str().to_string()),
                suggestion: Some("Add a static value or associate a rule".to_string()),
            });
        }
    }

    // Check for orphaned rules (rules with no outputs)
    for rule in store.rules.values().filter(|r| r.product_id == pid) {
        if rule.output_attributes.is_empty() {
            warnings.push(ValidationWarning {
                code: "RULE_NO_OUTPUTS".to_string(),
                message: format!("Rule '{}' has no output attributes", rule.id),
                path: None,
                suggestion: Some("Add output attributes to make this rule useful".to_string()),
            });
        }
    }

    let valid = errors.is_empty();

    Ok(Json(ValidationResponse {
        product_id: product_id.clone(),
        valid,
        errors,
        warnings,
    }))
}

/// Analyze the impact of changing an attribute
async fn impact_analysis(
    State(store): State<SharedStore>,
    Path(product_id): Path<String>,
    Json(req): Json<ImpactAnalysisRequest>,
) -> ApiResult<Json<ImpactAnalysisResponse>> {
    let store = store.read().await;

    // Verify product exists
    if !store.products.contains_key(&product_id) {
        return Err(ApiError::NotFound(format!(
            "Product '{}' not found",
            product_id
        )));
    }

    let pid = ProductId::new(&product_id);
    let target_path = &req.target_path;

    // Collect all rules for this product
    let rules: Vec<_> = store
        .rules
        .values()
        .filter(|r| r.product_id == pid)
        .collect();

    // Build dependency maps
    // output_to_rules: maps output paths to rules that produce them
    // input_to_rules: maps input paths to rules that consume them
    let mut output_to_rules: HashMap<String, Vec<String>> = HashMap::new();
    let mut input_to_rules: HashMap<String, Vec<String>> = HashMap::new();

    for rule in &rules {
        for output in &rule.output_attributes {
            output_to_rules
                .entry(output.path.as_str().to_string())
                .or_default()
                .push(rule.id.to_string());
        }
        for input in &rule.input_attributes {
            input_to_rules
                .entry(input.path.as_str().to_string())
                .or_default()
                .push(rule.id.to_string());
        }
    }

    // Find direct dependencies
    let mut direct_dependencies: Vec<DependencyInfoJson> = Vec::new();
    let mut affected_rules: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Upstream: rules that produce this attribute (what this depends on)
    if let Some(producing_rules) = output_to_rules.get(target_path) {
        for rule_id in producing_rules {
            affected_rules.insert(rule_id.clone());
            // Find the rule to get its inputs
            if let Some(rule) = rules.iter().find(|r| r.id.to_string() == *rule_id) {
                for input in &rule.input_attributes {
                    let path = input.path.as_str().to_string();
                    // Check if abstract attribute is immutable
                    let is_immutable = store
                        .attributes
                        .values()
                        .find(|a| a.path.as_str() == path && a.product_id == pid)
                        .and_then(|attr| {
                            store.abstract_attributes.values().find(|aa| {
                                aa.abstract_path == attr.abstract_path && aa.product_id == pid
                            })
                        })
                        .map(|aa| aa.immutable)
                        .unwrap_or(false);

                    direct_dependencies.push(DependencyInfoJson {
                        path: path.clone(),
                        attribute_name: path.split('.').last().unwrap_or(&path).to_string(),
                        direction: "upstream".to_string(),
                        distance: 1,
                        is_immutable,
                    });
                }
            }
        }
    }

    // Downstream: rules that consume this attribute (what depends on this)
    if let Some(consuming_rules) = input_to_rules.get(target_path) {
        for rule_id in consuming_rules {
            affected_rules.insert(rule_id.clone());
            // Find the rule to get its outputs
            if let Some(rule) = rules.iter().find(|r| r.id.to_string() == *rule_id) {
                for output in &rule.output_attributes {
                    let path = output.path.as_str().to_string();
                    let is_immutable = store
                        .attributes
                        .values()
                        .find(|a| a.path.as_str() == path && a.product_id == pid)
                        .and_then(|attr| {
                            store.abstract_attributes.values().find(|aa| {
                                aa.abstract_path == attr.abstract_path && aa.product_id == pid
                            })
                        })
                        .map(|aa| aa.immutable)
                        .unwrap_or(false);

                    direct_dependencies.push(DependencyInfoJson {
                        path: path.clone(),
                        attribute_name: path.split('.').last().unwrap_or(&path).to_string(),
                        direction: "downstream".to_string(),
                        distance: 1,
                        is_immutable,
                    });
                }
            }
        }
    }

    // Find transitive dependencies (BFS up to depth 5)
    let mut transitive_dependencies: Vec<DependencyInfoJson> = Vec::new();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    visited.insert(target_path.clone());

    // Add direct dependency paths to visited
    for dep in &direct_dependencies {
        visited.insert(dep.path.clone());
    }

    let mut current_downstream: Vec<String> = direct_dependencies
        .iter()
        .filter(|d| d.direction == "downstream")
        .map(|d| d.path.clone())
        .collect();

    // Trace downstream transitively
    for distance in 2..=5 {
        let mut next_level: Vec<String> = Vec::new();
        for path in &current_downstream {
            if let Some(consuming_rules) = input_to_rules.get(path) {
                for rule_id in consuming_rules {
                    affected_rules.insert(rule_id.clone());
                    if let Some(rule) = rules.iter().find(|r| r.id.to_string() == *rule_id) {
                        for output in &rule.output_attributes {
                            let out_path = output.path.as_str().to_string();
                            if !visited.contains(&out_path) {
                                visited.insert(out_path.clone());
                                let is_immutable = store
                                    .attributes
                                    .values()
                                    .find(|a| a.path.as_str() == out_path && a.product_id == pid)
                                    .and_then(|attr| {
                                        store.abstract_attributes.values().find(|aa| {
                                            aa.abstract_path == attr.abstract_path && aa.product_id == pid
                                        })
                                    })
                                    .map(|aa| aa.immutable)
                                    .unwrap_or(false);

                                transitive_dependencies.push(DependencyInfoJson {
                                    path: out_path.clone(),
                                    attribute_name: out_path.split('.').last().unwrap_or(&out_path).to_string(),
                                    direction: "downstream".to_string(),
                                    distance,
                                    is_immutable,
                                });
                                next_level.push(out_path);
                            }
                        }
                    }
                }
            }
        }
        if next_level.is_empty() {
            break;
        }
        current_downstream = next_level;
    }

    // Find affected functionalities
    let affected_functionalities: Vec<String> = store
        .functionalities
        .values()
        .filter(|f| {
            f.product_id == pid
                && f.required_attributes
                    .iter()
                    .any(|ra| visited.contains(ra.abstract_path.as_str()))
        })
        .map(|f| f.name.clone())
        .collect();

    // Check for immutable dependents
    let immutable_paths: Vec<String> = direct_dependencies
        .iter()
        .chain(transitive_dependencies.iter())
        .filter(|d| d.is_immutable)
        .map(|d| d.path.clone())
        .collect();

    let has_immutable_dependents = !immutable_paths.is_empty();

    Ok(Json(ImpactAnalysisResponse {
        target_path: target_path.clone(),
        direct_dependencies,
        transitive_dependencies,
        affected_rules: affected_rules.into_iter().collect(),
        affected_functionalities,
        has_immutable_dependents,
        immutable_paths,
    }))
}
