//! Test fixtures for HTTP API integration tests
//!
//! Provides reusable test infrastructure:
//! - `TestServer` - HTTP test server setup
//! - `DgraphTestContext` - DGraph connection and cleanup
//! - Data builders for creating test entities
//! - Custom assertion helpers

pub mod test_server;
pub mod dgraph_setup;
pub mod data_builders;
pub mod assertions;

pub use test_server::*;
pub use dgraph_setup::*;
pub use data_builders::*;
pub use assertions::*;
