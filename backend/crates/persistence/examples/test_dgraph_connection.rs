//! Minimal test of dgraph-tonic connection

use dgraph_tonic::{Client, Query};

#[tokio::main]
async fn main() {
    println!("Testing dgraph-tonic connection...");

    let endpoint = "http://127.0.0.1:9080";
    println!("Endpoint: {}", endpoint);

    match Client::new(endpoint) {
        Ok(client) => {
            println!("Client created successfully");
            let mut txn = client.new_read_only_txn();
            println!("Created read-only txn");

            match txn.query("{ q(func: uid(0x1)) { uid } }").await {
                Ok(response) => {
                    println!("Query succeeded!");
                    println!("Response: {:?}", String::from_utf8_lossy(&response.json));
                }
                Err(e) => {
                    println!("Query failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create client: {:?}", e);
        }
    }
}
