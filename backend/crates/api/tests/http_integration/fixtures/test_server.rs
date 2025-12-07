//! HTTP Test Server Setup
//!
//! Provides a test HTTP server backed by DGraph storage.

use std::net::SocketAddr;
use std::time::Duration;

use product_farm_api::rest::{create_router_with_state, AppState};
use product_farm_api::storage::StorageProvider;
use product_farm_api::store::create_shared_store;
use reqwest::{Client, Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::oneshot;

/// Test HTTP server with DGraph backend
pub struct TestServer {
    /// Base URL for API requests (e.g., "http://127.0.0.1:8081")
    pub base_url: String,
    /// HTTP client for making requests
    pub client: Client,
    /// Shutdown signal sender
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// Server address
    pub addr: SocketAddr,
}

impl TestServer {
    /// Create a new test server with DGraph backend
    pub async fn new_with_dgraph(dgraph_endpoint: &str) -> Result<Self, TestServerError> {
        let storage = StorageProvider::dgraph(dgraph_endpoint)
            .map_err(|e| TestServerError::StorageError(e.to_string()))?;

        Self::with_storage(storage).await
    }

    /// Create a new test server with hybrid storage (DGraph + cache)
    pub async fn new_with_hybrid(dgraph_endpoint: &str, cache_size: usize) -> Result<Self, TestServerError> {
        let storage = StorageProvider::hybrid(dgraph_endpoint, cache_size)
            .map_err(|e| TestServerError::StorageError(e.to_string()))?;

        Self::with_storage(storage).await
    }

    /// Create a new test server with in-memory storage (for quick tests)
    pub async fn new_memory() -> Result<Self, TestServerError> {
        let storage = StorageProvider::memory();
        Self::with_storage(storage).await
    }

    /// Create test server with given storage provider
    async fn with_storage(storage: StorageProvider) -> Result<Self, TestServerError> {
        // Find available port
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| TestServerError::BindError(e.to_string()))?;
        let addr = listener.local_addr()
            .map_err(|e| TestServerError::BindError(e.to_string()))?;

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        // Create app state with storage
        let store = create_shared_store();
        let state = AppState::with_storage(store, storage);

        // Create router
        let app = create_router_with_state(state);

        // Spawn server task
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    shutdown_rx.await.ok();
                })
                .await
                .ok();
        });

        // Wait for server to be ready
        tokio::time::sleep(Duration::from_millis(50)).await;

        let base_url = format!("http://{}", addr);
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| TestServerError::ClientError(e.to_string()))?;

        Ok(Self {
            base_url,
            client,
            shutdown_tx: Some(shutdown_tx),
            addr,
        })
    }

    /// Shutdown the test server
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        // Give server time to shutdown gracefully
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    /// Build full URL for a path
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    // =========================================================================
    // HTTP Methods
    // =========================================================================

    /// GET request expecting JSON response
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, TestServerError> {
        let response = self.client
            .get(self.url(path))
            .send()
            .await
            .map_err(|e| TestServerError::RequestError(e.to_string()))?;

        Self::handle_response(response).await
    }

    /// GET request returning raw response for status checking
    pub async fn get_response(&self, path: &str) -> Result<Response, TestServerError> {
        self.client
            .get(self.url(path))
            .send()
            .await
            .map_err(|e| TestServerError::RequestError(e.to_string()))
    }

    /// POST request expecting JSON response
    pub async fn post<R: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &R,
    ) -> Result<T, TestServerError> {
        let response = self.client
            .post(self.url(path))
            .json(body)
            .send()
            .await
            .map_err(|e| TestServerError::RequestError(e.to_string()))?;

        Self::handle_response(response).await
    }

    /// POST request returning raw response
    pub async fn post_response<R: Serialize>(
        &self,
        path: &str,
        body: &R,
    ) -> Result<Response, TestServerError> {
        self.client
            .post(self.url(path))
            .json(body)
            .send()
            .await
            .map_err(|e| TestServerError::RequestError(e.to_string()))
    }

    /// PUT request expecting JSON response
    pub async fn put<R: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &R,
    ) -> Result<T, TestServerError> {
        let response = self.client
            .put(self.url(path))
            .json(body)
            .send()
            .await
            .map_err(|e| TestServerError::RequestError(e.to_string()))?;

        Self::handle_response(response).await
    }

    /// PUT request returning raw response
    pub async fn put_response<R: Serialize>(
        &self,
        path: &str,
        body: &R,
    ) -> Result<Response, TestServerError> {
        self.client
            .put(self.url(path))
            .json(body)
            .send()
            .await
            .map_err(|e| TestServerError::RequestError(e.to_string()))
    }

    /// DELETE request expecting JSON response
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T, TestServerError> {
        let response = self.client
            .delete(self.url(path))
            .send()
            .await
            .map_err(|e| TestServerError::RequestError(e.to_string()))?;

        Self::handle_response(response).await
    }

    /// DELETE request returning raw response
    pub async fn delete_response(&self, path: &str) -> Result<Response, TestServerError> {
        self.client
            .delete(self.url(path))
            .send()
            .await
            .map_err(|e| TestServerError::RequestError(e.to_string()))
    }

    /// Get status code for a request
    pub async fn get_status(&self, path: &str) -> Result<StatusCode, TestServerError> {
        let response = self.get_response(path).await?;
        Ok(response.status())
    }

    /// POST and expect a specific error status
    pub async fn post_expect_status<R: Serialize>(
        &self,
        path: &str,
        body: &R,
        expected_status: StatusCode,
    ) -> Result<String, TestServerError> {
        let response = self.post_response(path, body).await?;
        let status = response.status();
        let body = response.text().await
            .map_err(|e| TestServerError::ResponseError(e.to_string()))?;

        if status != expected_status {
            return Err(TestServerError::UnexpectedStatus {
                expected: expected_status,
                actual: status,
                body,
            });
        }

        Ok(body)
    }

    /// Handle response - parse JSON or return error
    async fn handle_response<T: DeserializeOwned>(response: Response) -> Result<T, TestServerError> {
        let status = response.status();
        let body = response.text().await
            .map_err(|e| TestServerError::ResponseError(e.to_string()))?;

        if !status.is_success() {
            return Err(TestServerError::HttpError { status, body });
        }

        serde_json::from_str(&body)
            .map_err(|e| TestServerError::ParseError {
                error: e.to_string(),
                body,
            })
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// Errors that can occur during test server operations
#[derive(Debug, thiserror::Error)]
pub enum TestServerError {
    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Failed to bind server: {0}")]
    BindError(String),

    #[error("Failed to create HTTP client: {0}")]
    ClientError(String),

    #[error("Request failed: {0}")]
    RequestError(String),

    #[error("Response error: {0}")]
    ResponseError(String),

    #[error("HTTP error {status}: {body}")]
    HttpError { status: StatusCode, body: String },

    #[error("Failed to parse response: {error}\nBody: {body}")]
    ParseError { error: String, body: String },

    #[error("Unexpected status: expected {expected}, got {actual}\nBody: {body}")]
    UnexpectedStatus {
        expected: StatusCode,
        actual: StatusCode,
        body: String,
    },
}
