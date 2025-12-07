//! AttributeGrpcService - Concrete attribute management
//!
//! Provides create, read, update, delete, list, and query endpoints
//! for concrete attributes including by-tag and by-functionality queries.

use tonic::{Request, Response, Status};

use product_farm_core::{AbstractPath, Attribute, AttributeDisplayName, ConcretePath, RuleId};

use crate::converters::{core_to_proto_attribute, proto_to_core_display_format, try_proto_to_core_value};
use crate::store::SharedStore;

use super::helpers::parse_page_token;
use super::proto;

/// gRPC service for concrete attribute management
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
            Attribute::new_just_definition(path.clone(), abstract_path, req.product_id.as_str())
        };

        // Validate
        attr.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

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

        attr.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

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
        let abstract_attrs =
            store.get_abstract_attrs_by_functionality(&req.product_id, &req.functionality_name);
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
