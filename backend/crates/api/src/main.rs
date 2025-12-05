//! Product-FARM gRPC and REST Server binary
//!
//! This is the main entry point for running the Product-FARM server.
//!
//! # Usage
//!
//! ```bash
//! # Default ports: gRPC=50051, HTTP=8080
//! cargo run
//!
//! # Custom gRPC port (HTTP stays on 8080)
//! cargo run -- 50052
//!
//! # Custom gRPC and HTTP ports
//! cargo run -- 50052 9000
//!
//! # gRPC only (no HTTP server)
//! GRPC_ONLY=1 cargo run
//! ```

use product_farm_api::{ProductFarmServer, ServerConfig};
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "product_farm_api=info,tonic=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse command-line arguments for ports
    // arg 1: gRPC port (default: 50051)
    // arg 2: HTTP port (default: 8080)
    let grpc_port: u16 = env::args()
        .nth(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(50051);

    let http_port: u16 = env::args()
        .nth(2)
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let grpc_addr: std::net::SocketAddr = format!("0.0.0.0:{}", grpc_port).parse()?;
    let http_addr: std::net::SocketAddr = format!("0.0.0.0:{}", http_port).parse()?;

    // Check if HTTP should be disabled
    let enable_http = env::var("GRPC_ONLY").is_err();

    let mut config = ServerConfig::new(grpc_addr)
        .with_http_addr(http_addr)
        .with_max_message_size(16 * 1024 * 1024) // 16MB
        .with_keepalive(30, 60);

    if !enable_http {
        config = config.without_http();
    }

    let server = ProductFarmServer::with_config(config);

    tracing::info!(
        "Starting Product-FARM server (gRPC: {}, HTTP: {})",
        grpc_addr,
        if enable_http {
            http_addr.to_string()
        } else {
            "disabled".to_string()
        }
    );

    // Setup graceful shutdown
    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        tracing::info!("Received shutdown signal");
    };

    server.run_with_shutdown(shutdown).await?;

    Ok(())
}
