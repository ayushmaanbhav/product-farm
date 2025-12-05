//! gRPC and REST API for Product-FARM
//!
//! Provides:
//! - gRPC services for all Product-FARM operations:
//!   - ProductFarmService: Rule evaluation (evaluate, batch evaluate, streaming)
//!   - ProductService: Product CRUD + clone + approval workflow
//!   - AbstractAttributeService: Abstract attribute CRUD + queries by component/tag/functionality
//!   - AttributeService: Concrete attribute CRUD + queries
//!   - RuleService: Rule CRUD
//!   - DatatypeService: Datatype CRUD
//!   - ProductFunctionalityService: Functionality CRUD + evaluate + approval workflow
//!   - ProductTemplateService: Enumeration CRUD
//!
//! # Quick Start
//!
//! ```ignore
//! use product_farm_api::{create_all_services, create_shared_store};
//! use tonic::transport::Server;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let store = create_shared_store();
//!     let services = create_all_services(store);
//!
//!     Server::builder()
//!         .add_service(product_farm_service_server::ProductFarmServiceServer::new(services.product_farm))
//!         .add_service(product_service_server::ProductServiceServer::new(services.product))
//!         // ... add more services
//!         .serve("[::1]:50051".parse()?)
//!         .await?;
//!     Ok(())
//! }
//! ```

pub mod converters;
pub mod error;
pub mod grpc;
pub mod rest;
pub mod server;
pub mod service;
pub mod store;
pub mod validation;

// Re-export key types
pub use converters::*;
pub use error::*;
pub use grpc::proto;
pub use grpc::{
    create_all_services, AbstractAttributeGrpcService, AllServices, AttributeGrpcService,
    DatatypeGrpcService, ProductFarmGrpcService, ProductFunctionalityGrpcService,
    ProductGrpcService, ProductTemplateGrpcService, RuleGrpcService,
};
pub use server::*;
pub use service::*;
pub use store::{create_shared_store, EntityStore, SharedStore};
pub use validation::*;
