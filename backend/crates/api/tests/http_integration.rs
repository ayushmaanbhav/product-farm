//! HTTP API Integration Tests
//!
//! Entry point for all HTTP API integration tests.
//!
//! These tests require the `dgraph` feature flag to be enabled.
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all HTTP integration tests
//! cargo test -p product-farm-api --features dgraph --test http_integration
//!
//! # Run specific test module
//! cargo test -p product-farm-api --features dgraph --test http_integration products::
//!
//! # Run a specific test
//! cargo test -p product-farm-api --features dgraph --test http_integration test_create_product
//! ```

#![cfg(feature = "dgraph")]

// Include modules from the http_integration directory using path attribute
#[path = "http_integration/fixtures/mod.rs"]
mod fixtures;

#[path = "http_integration/products/mod.rs"]
mod products;

#[path = "http_integration/datatypes/mod.rs"]
mod datatypes;

#[path = "http_integration/enumerations/mod.rs"]
mod enumerations;

#[path = "http_integration/abstract_attributes/mod.rs"]
mod abstract_attributes;

#[path = "http_integration/concrete_attributes/mod.rs"]
mod concrete_attributes;

#[path = "http_integration/rules/mod.rs"]
mod rules;

#[path = "http_integration/functionalities/mod.rs"]
mod functionalities;

#[path = "http_integration/evaluation/mod.rs"]
mod evaluation;

#[path = "http_integration/workflows/mod.rs"]
mod workflows;
