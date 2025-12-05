//! Minimal test of dgraph-tonic connection
//! Run with: cargo run --example test_dgraph_connection --features dgraph

use dgraph_tonic::Client;

#[tokio::main]
async fn main() {
    println!("Testing dgraph-tonic connection...");

    let endpoints = vec![
        "http://127.0.0.1:9080",
        "http://localhost:9080",
    ];

    for endpoint in endpoints {
        println!("\n--- Testing endpoint: {} ---", endpoint);

        match Client::new(endpoint) {
            Ok(client) => {
                println!("  Client created successfully");

                // Try a simple read-only query
                let mut txn = client.new_read_only_txn();
                println!("  Created read-only txn");

                match txn.query("{ q(func: uid(0x1)) { uid } }").await {
                    Ok(response) => {
                        println!("  Query succeeded!");
                        println!("  Response JSON: {:?}", String::from_utf8_lossy(&response.json));
                    }
                    Err(e) => {
                        println!("  Query failed: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("  Failed to create client: {:?}", e);
            }
        }
    }
}
