//! Test dgraph operations that match what repositories do

use dgraph_tonic::{Client, Mutate, Mutation, Query};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    println!("Testing dgraph-tonic operations...\n");

    let endpoint = "http://127.0.0.1:9080";
    println!("Endpoint: {}", endpoint);

    let client = match Client::new(endpoint) {
        Ok(c) => {
            println!("✓ Client created successfully");
            c
        }
        Err(e) => {
            println!("✗ Failed to create client: {:?}", e);
            return;
        }
    };

    // Test 1: Simple query without variables (should work)
    println!("\n--- Test 1: Simple query without variables ---");
    let mut txn = client.new_read_only_txn();
    match txn.query("{ q(func: uid(0x1)) { uid } }").await {
        Ok(response) => {
            println!("✓ Simple query succeeded");
            println!("  Response: {:?}", String::from_utf8_lossy(&response.json));
        }
        Err(e) => {
            println!("✗ Simple query failed: {:?}", e);
        }
    }

    // Test 2: Query with variables (what repos use)
    println!("\n--- Test 2: Query with variables ---");
    let mut txn2 = client.new_read_only_txn();
    let query = r#"
        query GetProduct($id: string) {
            products(func: eq(Product.id, $id)) {
                uid
            }
        }
    "#;
    let mut vars = HashMap::new();
    vars.insert("$id".to_string(), "test-product".to_string());

    match txn2.query_with_vars(query, vars).await {
        Ok(response) => {
            println!("✓ Query with vars succeeded");
            println!("  Response: {:?}", String::from_utf8_lossy(&response.json));
        }
        Err(e) => {
            println!("✗ Query with vars failed: {:?}", e);
        }
    }

    // Test 3: Mutation with NQuads (more reliable than JSON)
    println!("\n--- Test 3: Mutation with NQuads ---");
    let mut txn3 = client.new_mutated_txn();
    let mut mu = Mutation::new();
    let nquads = r#"
_:new <dgraph.type> "TestNode" .
_:new <TestNode.name> "test-nquad" .
"#;
    mu.set_set_nquads(nquads);

    match txn3.mutate(mu).await {
        Ok(response) => {
            println!("✓ NQuads mutation succeeded");
            println!("  UIDs: {:?}", response.uids);

            match txn3.commit().await {
                Ok(_) => println!("✓ Commit succeeded"),
                Err(e) => println!("✗ Commit failed: {:?}", e),
            }
        }
        Err(e) => {
            println!("✗ NQuads mutation failed: {:?}", e);
        }
    }

    // Test 4: Mutation with JSON (try the format DGraph expects)
    println!("\n--- Test 4: Mutation with JSON array ---");
    let mut txn4 = client.new_mutated_txn();
    let mut mu4 = Mutation::new();
    // Try JSON without the set wrapper - just raw JSON objects as array
    let json_data = r#"[{"uid":"_:new2","dgraph.type":"Product","Product.id":"test-json-product","Product.status":"DRAFT","Product.template_type":"TRADING","Product.version":1}]"#;

    if let Err(e) = mu4.set_set_json(json_data.as_bytes()) {
        println!("✗ Failed to set JSON: {:?}", e);
    } else {
        match txn4.mutate(mu4).await {
            Ok(response) => {
                println!("✓ JSON mutation succeeded");
                println!("  UIDs: {:?}", response.uids);

                match txn4.commit().await {
                    Ok(_) => println!("✓ Commit succeeded"),
                    Err(e) => println!("✗ Commit failed: {:?}", e),
                }
            }
            Err(e) => {
                println!("✗ JSON mutation failed: {:?}", e);
            }
        }
    }

    println!("\nDone!");
}
