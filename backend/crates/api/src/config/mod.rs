//! Centralized configuration for the Product-FARM API
//!
//! This module provides a single source of truth for all configuration
//! constants, limits, and settings used across the API crate.
//!
//! # Modules
//!
//! - [`limits`]: Size and length limits for validation and DoS prevention
//! - [`server`]: Server configuration (ports, timeouts, message sizes)
//! - [`storage`]: Storage backend configuration (endpoints, cache settings)

pub mod limits;
pub mod server;
pub mod storage;

// Re-export commonly used items for convenience
pub use limits::*;
pub use server::{
    DEFAULT_GRPC_PORT, DEFAULT_HTTP_PORT, KEEP_ALIVE_INTERVAL_SECS, KEEP_ALIVE_TIMEOUT_SECS,
    MAX_MESSAGE_SIZE,
};
pub use storage::{DEFAULT_CACHE_SIZE, DEFAULT_DGRAPH_ENDPOINT};
