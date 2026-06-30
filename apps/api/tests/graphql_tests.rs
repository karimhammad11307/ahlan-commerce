mod helpers;

use serde_json::{Value, json};
use uuid::Uuid;

#[tokio::test]
async fn test_products_query() {
    let (addr, client) = helpers::spawn_test_server().await;

    // First ensure there is at least one product
    let handle = format!("graphql-query-{}", Uuid::now_v7());
    let _ = client
        .post(format!("{}/api/products", addr))
        .json(&json!({
            "title": "GraphQL Product",
            "handle": handle,
            "description": "Test description",
            "price_cents": 1000,
            "inventory_quantity": 10,
            "published": true
        }))
        .send()
        .await
        .unwrap();

    let query = json!({
        "query": "{ products { id title handle priceCents published createdAt } }"
    });

    let response = client
        .post(format!("{}/graphql", addr))
        .json(&query)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    // Check there are no errors
    assert!(body.get("errors").is_none());

    let products = body["data"]["products"].as_array().unwrap();
    assert!(products.len() > 0);

    let found = products.iter().any(|p| p["handle"] == handle);
    assert!(found, "Product should be retrievable via GraphQL");
}

#[tokio::test]
async fn test_product_create_mutation() {
    let (addr, client) = helpers::spawn_test_server().await;

    let handle = format!("graphql-mutation-{}", Uuid::now_v7());
    let query = json!({
        "query": "mutation($input: ProductCreateInput!) { productCreate(input: $input) { id title handle priceCents publishedAt createdAt updatedAt } }",
        "variables": {
            "input": {
                "title": "Mutation Created Product",
                "handle": handle,
                "description": "Created via GraphQL",
                "priceCents": 2000,
                "inventoryQuantity": 50,
                "published": true
            }
        }
    });

    let response = client
        .post(format!("{}/graphql", addr))
        .json(&query)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert!(body.get("errors").is_none());

    let product = &body["data"]["productCreate"];
    assert_eq!(product["title"], "Mutation Created Product");
    assert_eq!(product["handle"], handle);
    assert_eq!(product["priceCents"], 2000);
    assert!(product["id"].is_string());
    assert!(product["createdAt"].is_string());
    assert!(product["updatedAt"].is_string());
    assert!(product["publishedAt"].is_string());
}

#[tokio::test]
async fn test_validation_error_returns_error_code() {
    let (addr, client) = helpers::spawn_test_server().await;

    let handle = format!("graphql-validation-{}", Uuid::now_v7());
    let query = json!({
        "query": "mutation($input: ProductCreateInput!) { productCreate(input: $input) { id } }",
        "variables": {
            "input": {
                "title": "   ", // Empty title after trim
                "handle": handle,
                "priceCents": 1000,
                "inventoryQuantity": 10,
                "published": false
            }
        }
    });

    let response = client
        .post(format!("{}/graphql", addr))
        .json(&query)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200); // GraphQL typically returns 200 for errors too
    let body: Value = response.json().await.unwrap();

    let errors = body["errors"].as_array().expect("Expected errors array");
    assert!(errors.len() > 0);

    let error = &errors[0];
    assert_eq!(error["message"], "Title cannot be empty");
    assert_eq!(error["extensions"]["code"], "validation_failed");
}
