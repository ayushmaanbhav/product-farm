//! Storage configuration constants
//!
//! Default values for storage backend configuration.

/// Default DGraph Alpha gRPC endpoint
pub const DEFAULT_DGRAPH_ENDPOINT: &str = "http://localhost:9080";

/// Default cache size for hybrid storage (number of items)
///
/// Used for in-memory caching layer in hybrid mode.
pub const DEFAULT_CACHE_SIZE: usize = 10_000;

/// Test cache size (smaller for faster tests)
pub const TEST_CACHE_SIZE: usize = 5_000;
