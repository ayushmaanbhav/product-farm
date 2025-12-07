//! ProductGrpcService - Product CRUD and lifecycle management
//!
//! Provides create, read, update, delete, list, clone, and approval
//! workflow endpoints for products.

use chrono::{TimeZone, Utc};
use tonic::{Request, Response, Status};

use product_farm_core::{
    CloneProductRequest as CoreCloneRequest, CloneSelections, Product, ProductCloneService,
    ProductId,
};

use crate::converters::{core_to_proto_product, proto_to_core_product_status};
use crate::store::{EntityStore, SharedStore};

use super::helpers::parse_page_token;
use super::proto;

/// gRPC service for product management
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

        // Store product
        use hashbrown::hash_map::Entry;
        match store.products.entry(new_product_id.clone()) {
            Entry::Occupied(_) => {
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
