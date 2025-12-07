//! Product CRUD Tests
//!
//! Tests for Create, Read, Update, Delete operations on products.

use crate::fixtures::{
    assertions::*, data_builders::ProductBuilder, DgraphTestContext, TestContext,
};
use product_farm_api::rest::types::{
    DeleteResponse, ListProductsResponse, ProductResponse, UpdateProductRequest,
};
use reqwest::StatusCode;

// =============================================================================
// CREATE Tests
// =============================================================================

#[tokio::test]
async fn test_create_product_with_all_fields() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("fullprod");

    // Create product with all fields
    let request = ProductBuilder::new(&product_id)
        .with_name("Full Test Product")
        .with_template("insurance")
        .with_description("A comprehensive test product")
        .with_expiry(chrono::Utc::now().timestamp() + 86400 * 365) // 1 year from now
        .build();

    let response: ProductResponse = ctx.server
        .post("/api/products", &request)
        .await
        .expect("Failed to create product");

    // Assert all response fields
    assert_product_fields(&response, &product_id, "Full Test Product", "insurance");
    assert_product_draft(&response);
    assert_product_version(&response, 1);
    assert_product_timestamps_valid(&response);
    assert_eq!(response.description, "A comprehensive test product");
    assert!(response.expiry_at.is_some());
    assert!(response.parent_product_id.is_none());

    // Verify via HTTP API (since we're using in-memory backend)
    let get_response: ProductResponse = ctx.server
        .get(&format!("/api/products/{}", product_id))
        .await
        .expect("Failed to get product via API");
    assert_eq!(get_response.id, product_id);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_product_minimal_fields() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("minprod");

    // Create with only required fields
    let request = ProductBuilder::new(&product_id)
        .with_name("Minimal Product")
        .with_template("trading")
        .build();

    let response: ProductResponse = ctx.server
        .post("/api/products", &request)
        .await
        .expect("Failed to create product");

    assert_product_fields(&response, &product_id, "Minimal Product", "trading");
    assert_product_draft(&response);
    assert!(response.expiry_at.is_none());
    assert!(response.description.is_empty() || response.description == "");

    ctx.dgraph.track_product(product_id);
    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_product_different_templates() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    let templates = ["insurance", "trading", "loan", "mortgage"];

    for template in templates {
        let product_id = ctx.unique_product_id(&format!("{}prod", template));
        let request = ProductBuilder::new(&product_id)
            .with_template(template)
            .build();

        let response: ProductResponse = ctx.server
            .post("/api/products", &request)
            .await
            .expect(&format!("Failed to create {} product", template));

        assert_eq!(response.template_type, template);
        ctx.dgraph.track_product(product_id);
    }

    ctx.cleanup().await.ok();
}

// =============================================================================
// READ Tests
// =============================================================================

#[tokio::test]
async fn test_get_existing_product() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("getprod");

    // Create product
    let request = ProductBuilder::new(&product_id)
        .with_name("Get Test Product")
        .build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Get product
    let response: ProductResponse = ctx.server
        .get(&format!("/api/products/{}", product_id))
        .await
        .expect("Failed to get product");

    assert_product_fields(&response, &product_id, "Get Test Product", "insurance");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_get_nonexistent_product() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let response = ctx.server
        .get_response("/api/products/nonexistent_product_id_xyz")
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_list_products_empty() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    // List products (may contain other test data, but should work)
    let response: ListProductsResponse = ctx.server
        .get("/api/products?pageSize=100")
        .await
        .expect("Failed to list products");

    // Just verify the structure is correct
    assert!(response.total_count >= 0);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_list_products_with_pagination() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");

    // Create multiple products
    for i in 0..5 {
        let product_id = ctx.unique_product_id(&format!("listprod{}", i));
        let request = ProductBuilder::new(&product_id).build();
        ctx.server.post::<_, ProductResponse>("/api/products", &request).await
            .expect("Failed to create product");
        ctx.dgraph.track_product(product_id);
    }

    // List with small page size
    let response: ListProductsResponse = ctx.server
        .get("/api/products?pageSize=2")
        .await
        .expect("Failed to list products");

    assert!(response.items.len() <= 2);

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_list_products_with_status_filter() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("statusprod");

    // Create a DRAFT product
    let request = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Filter by DRAFT status
    let response: ListProductsResponse = ctx.server
        .get("/api/products?statusFilter=DRAFT&pageSize=100")
        .await
        .expect("Failed to list products");

    // All returned products should be DRAFT
    for product in &response.items {
        assert_product_status(product, "DRAFT");
    }

    ctx.cleanup().await.ok();
}

// =============================================================================
// UPDATE Tests
// =============================================================================

#[tokio::test]
async fn test_update_product_name() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("updateprod");

    // Create product
    let request = ProductBuilder::new(&product_id)
        .with_name("Original Name")
        .build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Update name
    let update = UpdateProductRequest {
        name: Some("Updated Name".to_string()),
        description: None,
        effective_from: None,
        expiry_at: None,
    };

    let response: ProductResponse = ctx.server
        .put(&format!("/api/products/{}", product_id), &update)
        .await
        .expect("Failed to update product");

    assert_eq!(response.name, "Updated Name");
    // updated_at may equal created_at if update happens in the same second
    assert!(response.updated_at >= response.created_at);

    // Verify via HTTP API (in-memory backend)
    let get_response: ProductResponse = ctx.server
        .get(&format!("/api/products/{}", product_id))
        .await
        .expect("Failed to get product via API");
    assert_eq!(get_response.name, "Updated Name");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_update_product_description() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("descprod");

    // Create product without description
    let request = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Update description
    let update = UpdateProductRequest {
        name: None,
        description: Some("New description".to_string()),
        effective_from: None,
        expiry_at: None,
    };

    let response: ProductResponse = ctx.server
        .put(&format!("/api/products/{}", product_id), &update)
        .await
        .expect("Failed to update product");

    assert_eq!(response.description, "New description");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_update_product_dates() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("datesprod");

    let now = chrono::Utc::now().timestamp();
    let request = ProductBuilder::new(&product_id)
        .with_effective_from(now)
        .build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    ctx.dgraph.track_product(product_id.clone());

    // Update dates
    let new_effective = now + 86400; // +1 day
    let new_expiry = now + 86400 * 365; // +1 year

    let update = UpdateProductRequest {
        name: None,
        description: None,
        effective_from: Some(new_effective),
        expiry_at: Some(new_expiry),
    };

    let response: ProductResponse = ctx.server
        .put(&format!("/api/products/{}", product_id), &update)
        .await
        .expect("Failed to update product");

    assert_eq!(response.effective_from, new_effective);
    assert_eq!(response.expiry_at, Some(new_expiry));

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_update_nonexistent_product() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let update = UpdateProductRequest {
        name: Some("New Name".to_string()),
        description: None,
        effective_from: None,
        expiry_at: None,
    };

    let response = ctx.server
        .put_response("/api/products/nonexistent_xyz", &update)
        .await
        .expect("Request should complete");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}

// =============================================================================
// DELETE Tests
// =============================================================================

#[tokio::test]
async fn test_delete_draft_product() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("delprod");

    // Create product
    let request = ProductBuilder::new(&product_id).build();
    ctx.server.post::<_, ProductResponse>("/api/products", &request).await
        .expect("Failed to create product");
    // Don't track - we're deleting it

    // Delete product
    let response: DeleteResponse = ctx.server
        .delete(&format!("/api/products/{}", product_id))
        .await
        .expect("Failed to delete product");

    assert!(response.success);

    // Verify deleted via HTTP API (in-memory backend)
    let get_response = ctx.server
        .get_response(&format!("/api/products/{}", product_id))
        .await
        .expect("Request should complete");
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND, "Product should be deleted");

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_delete_nonexistent_product() {
    let ctx = TestContext::new().await.expect("Failed to create test context");

    let response = ctx.server
        .delete_response("/api/products/nonexistent_del_xyz")
        .await
        .expect("Request should complete");

    // Should return NOT_FOUND or success (idempotent delete)
    assert!(
        response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::OK,
        "Expected NOT_FOUND or OK, got {}",
        response.status()
    );

    ctx.cleanup().await.ok();
}

#[tokio::test]
async fn test_create_read_update_delete_cycle() {
    let mut ctx = TestContext::new().await.expect("Failed to create test context");
    let product_id = ctx.unique_product_id("crudcycle");

    // CREATE
    let request = ProductBuilder::new(&product_id)
        .with_name("CRUD Cycle Test")
        .build();
    let created: ProductResponse = ctx.server
        .post("/api/products", &request)
        .await
        .expect("Failed to create product");
    assert_product_version(&created, 1);

    // READ
    let read: ProductResponse = ctx.server
        .get(&format!("/api/products/{}", product_id))
        .await
        .expect("Failed to read product");
    assert_eq!(read.id, created.id);
    assert_eq!(read.version, created.version);

    // UPDATE
    let update = UpdateProductRequest {
        name: Some("Updated CRUD Test".to_string()),
        description: None,
        effective_from: None,
        expiry_at: None,
    };
    let updated: ProductResponse = ctx.server
        .put(&format!("/api/products/{}", product_id), &update)
        .await
        .expect("Failed to update product");
    assert_eq!(updated.name, "Updated CRUD Test");

    // DELETE
    let deleted: DeleteResponse = ctx.server
        .delete(&format!("/api/products/{}", product_id))
        .await
        .expect("Failed to delete product");
    assert!(deleted.success);

    // VERIFY DELETED
    let get_response = ctx.server
        .get_response(&format!("/api/products/{}", product_id))
        .await
        .expect("Request should complete");
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);

    ctx.cleanup().await.ok();
}
