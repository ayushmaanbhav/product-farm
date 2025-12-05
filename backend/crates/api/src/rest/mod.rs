//! REST API module for Product-FARM
//!
//! Provides HTTP/JSON endpoints alongside the gRPC server

use axum::{
    http::{header, Method},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::store::SharedStore;

pub mod error;
pub mod types;
pub mod constants;
pub mod helpers;
pub mod validation;
pub mod products;
pub mod attributes;
pub mod rules;
pub mod datatypes;
pub mod functionalities;
pub mod templates;
pub mod evaluation;

mod converters;

pub use error::{ApiError, ApiResult};

/// Create the REST API router with all endpoints
pub fn create_router(store: SharedStore) -> Router {
    // Configure CORS for browser requests
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
        ]);

    Router::new()
        // Product routes
        .merge(products::routes())
        // Attribute routes (abstract + concrete)
        .merge(attributes::routes())
        // Rule routes
        .merge(rules::routes())
        // Datatype routes
        .merge(datatypes::routes())
        // Functionality routes
        .merge(functionalities::routes())
        // Template enumeration routes
        .merge(templates::routes())
        // Evaluation routes
        .merge(evaluation::routes())
        // Shared state
        .with_state(store)
        // Apply CORS
        .layer(cors)
}
