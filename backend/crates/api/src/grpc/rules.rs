//! RuleGrpcService - Rule management
//!
//! Provides create, read, update, delete, and list endpoints for rules.

use tonic::{Request, Response, Status};

use product_farm_core::{Rule, RuleId, RuleInputAttribute, RuleOutputAttribute};

use crate::converters::core_to_proto_rule;
use crate::store::SharedStore;

use super::helpers::parse_page_token;
use super::proto;

/// gRPC service for rule management
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
        rule.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

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

        rule.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

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
