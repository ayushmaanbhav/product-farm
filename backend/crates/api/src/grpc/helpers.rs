//! Shared helper functions for gRPC services

use tonic::Status;

/// Parse a page token into an offset, returning an error for invalid tokens.
/// Empty string returns 0 (start from beginning).
pub fn parse_page_token(page_token: &str) -> Result<usize, Status> {
    if page_token.is_empty() {
        return Ok(0);
    }
    page_token
        .parse()
        .map_err(|_| Status::invalid_argument(format!("Invalid page_token: '{}'", page_token)))
}
