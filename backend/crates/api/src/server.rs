//! gRPC and REST Server for Product-FARM
//!
//! Provides server configuration and startup functionality for both
//! gRPC and REST/HTTP servers.

use std::future::IntoFuture;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tonic::transport::Server;
use tracing::info;

use crate::config::server::{
    DEFAULT_GRPC_PORT, DEFAULT_HTTP_PORT, KEEP_ALIVE_INTERVAL_SECS, KEEP_ALIVE_TIMEOUT_SECS,
    MAX_MESSAGE_SIZE,
};

use crate::grpc::proto::{
    abstract_attribute_service_server::AbstractAttributeServiceServer,
    attribute_service_server::AttributeServiceServer,
    datatype_service_server::DatatypeServiceServer,
    product_farm_service_server::ProductFarmServiceServer,
    product_functionality_service_server::ProductFunctionalityServiceServer,
    product_service_server::ProductServiceServer,
    product_template_service_server::ProductTemplateServiceServer,
    rule_service_server::RuleServiceServer,
};
use crate::grpc::create_all_services;
use crate::rest;
use crate::store::{create_shared_store, SharedStore};

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to bind the gRPC server to
    pub grpc_addr: SocketAddr,
    /// Address to bind the HTTP/REST server to
    pub http_addr: SocketAddr,
    /// Maximum message size (default: 4MB)
    pub max_message_size: usize,
    /// Whether to enable HTTP/2 keep-alive
    pub enable_keepalive: bool,
    /// Keep-alive interval in seconds
    pub keepalive_interval_secs: u64,
    /// Keep-alive timeout in seconds
    pub keepalive_timeout_secs: u64,
    /// Whether to enable HTTP REST server
    pub enable_http: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            grpc_addr: format!("0.0.0.0:{}", DEFAULT_GRPC_PORT).parse().unwrap(),
            http_addr: format!("0.0.0.0:{}", DEFAULT_HTTP_PORT).parse().unwrap(),
            max_message_size: MAX_MESSAGE_SIZE,
            enable_keepalive: true,
            keepalive_interval_secs: KEEP_ALIVE_INTERVAL_SECS,
            keepalive_timeout_secs: KEEP_ALIVE_TIMEOUT_SECS,
            enable_http: true,
        }
    }
}

impl ServerConfig {
    /// Create a new server configuration with gRPC address
    pub fn new(addr: impl Into<SocketAddr>) -> Self {
        Self {
            grpc_addr: addr.into(),
            ..Default::default()
        }
    }

    /// Set the gRPC bind address (legacy - use with_grpc_addr instead)
    pub fn with_addr(mut self, addr: impl Into<SocketAddr>) -> Self {
        self.grpc_addr = addr.into();
        self
    }

    /// Set the gRPC bind address
    pub fn with_grpc_addr(mut self, addr: impl Into<SocketAddr>) -> Self {
        self.grpc_addr = addr.into();
        self
    }

    /// Set the HTTP/REST bind address
    pub fn with_http_addr(mut self, addr: impl Into<SocketAddr>) -> Self {
        self.http_addr = addr.into();
        self
    }

    /// Set maximum message size
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// Set keep-alive configuration
    pub fn with_keepalive(mut self, interval_secs: u64, timeout_secs: u64) -> Self {
        self.enable_keepalive = true;
        self.keepalive_interval_secs = interval_secs;
        self.keepalive_timeout_secs = timeout_secs;
        self
    }

    /// Disable keep-alive
    pub fn without_keepalive(mut self) -> Self {
        self.enable_keepalive = false;
        self
    }

    /// Enable HTTP REST server
    pub fn with_http(mut self) -> Self {
        self.enable_http = true;
        self
    }

    /// Disable HTTP REST server (gRPC only)
    pub fn without_http(mut self) -> Self {
        self.enable_http = false;
        self
    }
}

/// The Product-FARM gRPC and REST server
pub struct ProductFarmServer {
    config: ServerConfig,
    store: SharedStore,
}

impl ProductFarmServer {
    /// Create a new server with default configuration
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
            store: create_shared_store(),
        }
    }

    /// Create a new server with custom configuration
    pub fn with_config(config: ServerConfig) -> Self {
        Self {
            config,
            store: create_shared_store(),
        }
    }

    /// Create a new server with a shared store (for testing)
    pub fn with_store(store: SharedStore) -> Self {
        Self {
            config: ServerConfig::default(),
            store,
        }
    }

    /// Get the gRPC server address
    pub fn addr(&self) -> SocketAddr {
        self.config.grpc_addr
    }

    /// Get the HTTP server address
    pub fn http_addr(&self) -> SocketAddr {
        self.config.http_addr
    }

    /// Build the gRPC router with all services
    fn build_grpc_router(&self, store: SharedStore) -> tonic::transport::server::Router {
        let services = create_all_services(store);
        let max_size = self.config.max_message_size;

        let mut builder = Server::builder();

        if self.config.enable_keepalive {
            use std::time::Duration;
            builder = builder
                .http2_keepalive_interval(Some(Duration::from_secs(
                    self.config.keepalive_interval_secs,
                )))
                .http2_keepalive_timeout(Some(Duration::from_secs(
                    self.config.keepalive_timeout_secs,
                )));
        }

        builder
            .add_service(
                ProductFarmServiceServer::new(services.product_farm)
                    .max_decoding_message_size(max_size)
                    .max_encoding_message_size(max_size),
            )
            .add_service(
                ProductServiceServer::new(services.product)
                    .max_decoding_message_size(max_size)
                    .max_encoding_message_size(max_size),
            )
            .add_service(
                AbstractAttributeServiceServer::new(services.abstract_attribute)
                    .max_decoding_message_size(max_size)
                    .max_encoding_message_size(max_size),
            )
            .add_service(
                AttributeServiceServer::new(services.attribute)
                    .max_decoding_message_size(max_size)
                    .max_encoding_message_size(max_size),
            )
            .add_service(
                RuleServiceServer::new(services.rule)
                    .max_decoding_message_size(max_size)
                    .max_encoding_message_size(max_size),
            )
            .add_service(
                DatatypeServiceServer::new(services.datatype)
                    .max_decoding_message_size(max_size)
                    .max_encoding_message_size(max_size),
            )
            .add_service(
                ProductFunctionalityServiceServer::new(services.functionality)
                    .max_decoding_message_size(max_size)
                    .max_encoding_message_size(max_size),
            )
            .add_service(
                ProductTemplateServiceServer::new(services.template)
                    .max_decoding_message_size(max_size)
                    .max_encoding_message_size(max_size),
            )
    }

    /// Start the server (both gRPC and HTTP)
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let grpc_addr = self.config.grpc_addr;
        let http_addr = self.config.http_addr;
        let enable_http = self.config.enable_http;
        let store = self.store.clone();

        // Build gRPC router
        let grpc_router = self.build_grpc_router(store.clone());

        if enable_http {
            // Run both servers concurrently
            info!("Starting Product-FARM gRPC server on {}", grpc_addr);
            info!("Starting Product-FARM REST server on {}", http_addr);

            let http_router = rest::create_router(store);
            let listener = TcpListener::bind(http_addr).await?;

            tokio::select! {
                result = grpc_router.serve(grpc_addr) => {
                    result?;
                }
                result = axum::serve(listener, http_router).into_future() => {
                    result?;
                }
            }
        } else {
            // gRPC only
            info!("Starting Product-FARM gRPC server on {}", grpc_addr);
            grpc_router.serve(grpc_addr).await?;
        }

        Ok(())
    }

    /// Start the server with graceful shutdown
    pub async fn run_with_shutdown(
        self,
        shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let grpc_addr = self.config.grpc_addr;
        let http_addr = self.config.http_addr;
        let enable_http = self.config.enable_http;
        let store = self.store.clone();

        // Build gRPC router
        let grpc_router = self.build_grpc_router(store.clone());

        if enable_http {
            // Run both servers concurrently with shutdown
            info!("Starting Product-FARM gRPC server on {}", grpc_addr);
            info!("Starting Product-FARM REST server on {}", http_addr);

            let http_router = rest::create_router(store);
            let listener = TcpListener::bind(http_addr).await?;

            // Create a shutdown signal that can be cloned
            let shutdown = async move {
                shutdown_signal.await;
            };

            // Use tokio::select to run both servers and handle shutdown
            tokio::select! {
                result = grpc_router.serve_with_shutdown(grpc_addr, async {
                    tokio::signal::ctrl_c().await.ok();
                }) => {
                    result?;
                }
                result = axum::serve(listener, http_router)
                    .with_graceful_shutdown(async {
                        tokio::signal::ctrl_c().await.ok();
                    })
                    .into_future() => {
                    result?;
                }
                _ = shutdown => {
                    info!("Shutdown signal received");
                }
            }
        } else {
            // gRPC only
            info!("Starting Product-FARM gRPC server on {}", grpc_addr);
            grpc_router.serve_with_shutdown(grpc_addr, shutdown_signal).await?;
        }

        info!("Server shutdown complete");
        Ok(())
    }
}

impl Default for ProductFarmServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Start a simple server on the default ports (gRPC: 50051, HTTP: 8080)
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    ProductFarmServer::new().run().await
}

/// Start a server on a specific gRPC address (HTTP on default port 8080)
pub async fn run_server_on(addr: impl Into<SocketAddr>) -> Result<(), Box<dyn std::error::Error>> {
    ProductFarmServer::with_config(ServerConfig::new(addr))
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(
            config.grpc_addr,
            format!("0.0.0.0:{}", DEFAULT_GRPC_PORT).parse::<SocketAddr>().unwrap()
        );
        assert_eq!(
            config.http_addr,
            format!("0.0.0.0:{}", DEFAULT_HTTP_PORT).parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.max_message_size, MAX_MESSAGE_SIZE);
        assert!(config.enable_keepalive);
        assert!(config.enable_http);
    }

    #[test]
    fn test_config_builder() {
        let config = ServerConfig::default()
            .with_grpc_addr("127.0.0.1:9000".parse::<SocketAddr>().unwrap())
            .with_http_addr("127.0.0.1:9001".parse::<SocketAddr>().unwrap())
            .with_max_message_size(8 * 1024 * 1024)
            .without_keepalive()
            .without_http();

        assert_eq!(
            config.grpc_addr,
            "127.0.0.1:9000".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(
            config.http_addr,
            "127.0.0.1:9001".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.max_message_size, 8 * 1024 * 1024);
        assert!(!config.enable_keepalive);
        assert!(!config.enable_http);
    }

    #[test]
    fn test_server_creation() {
        let server = ProductFarmServer::new();
        assert_eq!(
            server.addr(),
            format!("0.0.0.0:{}", DEFAULT_GRPC_PORT).parse::<SocketAddr>().unwrap()
        );
        assert_eq!(
            server.http_addr(),
            format!("0.0.0.0:{}", DEFAULT_HTTP_PORT).parse::<SocketAddr>().unwrap()
        );
    }
}
