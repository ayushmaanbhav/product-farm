//! Server configuration constants
//!
//! Default values for gRPC and HTTP server configuration.

/// Default gRPC server port
pub const DEFAULT_GRPC_PORT: u16 = 50051;

/// Default HTTP/REST server port
pub const DEFAULT_HTTP_PORT: u16 = 8080;

/// Keep-alive interval in seconds
///
/// How often to send HTTP/2 PING frames to keep connections alive.
pub const KEEP_ALIVE_INTERVAL_SECS: u64 = 10;

/// Keep-alive timeout in seconds
///
/// How long to wait for a PING response before considering the connection dead.
pub const KEEP_ALIVE_TIMEOUT_SECS: u64 = 20;

/// Maximum message size for gRPC (4MB)
///
/// Limits the size of individual gRPC messages to prevent memory exhaustion.
pub const MAX_MESSAGE_SIZE: usize = 4 * 1024 * 1024;

/// Default cache TTL in seconds (5 minutes)
///
/// How long cached items remain valid.
pub const DEFAULT_CACHE_TTL_SECS: u64 = 300;
