//! AbstractAttributeGrpcService - Abstract attribute management
//!
//! Provides create, read, update, delete, list, and query endpoints
//! for abstract attributes including by-component, by-tag, and by-functionality queries.

use tonic::{Request, Response, Status};

use product_farm_core::{AbstractAttribute, AbstractAttributeTag, AbstractPath, AttributeDisplayName};

use crate::converters::{core_to_proto_abstract_attribute, proto_to_core_display_format};
use crate::store::SharedStore;

use super::helpers::parse_page_token;
use super::proto;

/// gRPC service for abstract attribute management
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
        attr.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

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
        attr.check_modifiable()
            .map_err(|e| Status::failed_precondition(e.to_string()))?;

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

        attr.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

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

    async fn get_abstract_attributes_by_tag(
        &self,
        request: Request<proto::GetByTagRequest>,
    ) -> Result<Response<proto::ListAbstractAttributesResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let attrs = store.get_abstract_attrs_by_tag(&req.product_id, &req.tag);
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

    async fn get_abstract_attributes_by_tags(
        &self,
        request: Request<proto::GetByTagsRequest>,
    ) -> Result<Response<proto::ListAbstractAttributesResponse>, Status> {
        let req = request.into_inner();
        let store = self.store.read().await;

        let attrs = store.get_abstract_attrs_by_tags(&req.product_id, &req.tags, req.match_all);
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

    async fn get_abstract_attributes_by_functionality(
        &self,
        request: Request<proto::GetByFunctionalityRequest>,
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
}
