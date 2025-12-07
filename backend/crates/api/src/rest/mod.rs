//! REST API module for Product-FARM
//!
//! Provides HTTP/JSON endpoints alongside the gRPC server

use axum::{
    http::{header, Method},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::storage::StorageProvider;
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

/// Application state for REST handlers
///
/// Contains both the legacy SharedStore (for backward compatibility during migration)
/// and the new StorageProvider (for repository-based access).
#[derive(Clone)]
pub struct AppState {
    /// Legacy shared store (to be deprecated)
    pub store: SharedStore,
    /// New storage provider with repository access
    pub storage: StorageProvider,
}

impl AppState {
    /// Create new app state with default memory storage
    pub fn new(store: SharedStore) -> Self {
        Self {
            store,
            storage: StorageProvider::memory(),
        }
    }

    /// Create app state with custom storage provider
    pub fn with_storage(store: SharedStore, storage: StorageProvider) -> Self {
        Self { store, storage }
    }
}

/// Create the REST API router with all endpoints (legacy SharedStore only)
pub fn create_router(store: SharedStore) -> Router {
    create_router_with_state(AppState::new(store))
}

/// Create the REST API router with full AppState
pub fn create_router_with_state(state: AppState) -> Router {
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
        .with_state(state)
        // Apply CORS
        .layer(cors)
}
