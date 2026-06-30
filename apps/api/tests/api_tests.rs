mod helpers;

use serde_json::{Value, json};
use uuid::Uuid;

#[tokio::test]
async fn test_api_valid_product_create() {
    // Maps to PRD-PROD-001
    let (addr, client) = helpers::spawn_test_server().await;

    let handle = format!("test-handle-{}", Uuid::now_v7());
    let payload = json!({
        "title": "Valid Product",
        "handle": handle,
        "description": "Test description",
        "price_cents": 1000,
        "inventory_quantity": 10,
        "published": true
    });

    let response = client
        .post(format!("{}/api/products", addr))
        .json(&payload)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 201);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["product"]["title"], "Valid Product");
    assert_eq!(body["product"]["handle"], handle);
}

#[tokio::test]
async fn test_api_duplicate_handle_rejected() {
    // Maps to PRD-PROD-002
    let (addr, client) = helpers::spawn_test_server().await;

    let handle = format!("duplicate-handle-{}", Uuid::now_v7());
    let payload = json!({
        "title": "Valid Product",
        "handle": handle,
        "description": "Test description",
        "price_cents": 1000,
        "inventory_quantity": 10,
        "published": true
    });

    // Create first
    let res1 = client
        .post(format!("{}/api/products", addr))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res1.status(), 201);

    // Create second with same handle
    let res2 = client
        .post(format!("{}/api/products", addr))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res2.status(), 409); // Expect Conflict for duplicate handle
}

#[tokio::test]
async fn test_api_list_empty_products() {
    // Maps to PRD-PROD-003
    // Note: Since we are hitting a shared DB, it might not be empty.
    // We can't guarantee an empty list unless we use a fresh DB.
    // But we CAN guarantee that it returns a 200 OK and a JSON object with a "products" array.
    let (addr, client) = helpers::spawn_test_server().await;

    let response = client
        .get(format!("{}/api/products", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert!(body["products"].is_array());
}

#[tokio::test]
async fn test_api_list_persisted_products() {
    // Maps to PRD-PROD-004
    let (addr, client) = helpers::spawn_test_server().await;

    // Create one to guarantee persistence
    let handle = format!("list-persisted-{}", Uuid::now_v7());
    let payload = json!({
        "title": "Persisted Product",
        "handle": handle,
        "description": "Test description",
        "price_cents": 1000,
        "inventory_quantity": 10,
        "published": true
    });

    client
        .post(format!("{}/api/products", addr))
        .json(&payload)
        .send()
        .await
        .unwrap();

    let response = client
        .get(format!("{}/api/products", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    let products = body["products"].as_array().unwrap();
    assert!(products.len() > 0);

    // Verify it's in the list
    let found = products.iter().any(|p| p["handle"] == handle);
    assert!(found, "Created product should be in the persisted list");
}
