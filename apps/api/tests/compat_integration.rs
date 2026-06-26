use axum::http::StatusCode;
use serde_json::json;

use api::{AppState, config, create_app};
use std::sync::Arc;
use tokio::net::TcpListener;

async fn spawn_test_server() -> (String, reqwest::Client) {
    let config = config::Config::load();
    let db_pool = db::create_pool(&config.database_url);

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let cache_client = api::cache::CacheClient::new(&redis_url).unwrap();

    let state = AppState {
        config: Arc::new(config),
        db_pool,
        cache: Arc::new(cache_client),
    };

    let app = create_app(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let address = format!("http://127.0.0.1:{}", port);

    (address, client)
}

#[tokio::test]
async fn test_compat_create_valid_payload_returns_201() {
    let (addr, client) = spawn_test_server().await;

    let payload = json!({
        "name": "Integration Coffee Mug",
        "slug": format!("coffee-mug-compat-{}", uuid::Uuid::new_v4()),
        "body_html": "Ceramic mug",
        "price": "25.00",
        "stock": 12,
        "is_visible": true
    });

    let response = client
        .post(&format!("{}/api/compat/products", addr))
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::CREATED);

    let body: serde_json::Value = response.json().await.unwrap();
    let product = &body["product"];
    
    assert_eq!(product["title"], "Integration Coffee Mug");
    assert_eq!(product["description"], "Ceramic mug");
    assert_eq!(product["price_cents"], 2500);
    assert_eq!(product["inventory_quantity"], 12);
    assert_eq!(product["published"], true);
}

#[tokio::test]
async fn test_compat_create_blank_name_returns_400() {
    let (addr, client) = spawn_test_server().await;

    let payload = json!({
        "name": "   ",
        "slug": "some-slug",
        "price": "25.00",
        "stock": 12
    });

    let response = client
        .post(&format!("{}/api/compat/products", addr))
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn test_compat_create_duplicate_slug_returns_409() {
    let (addr, client) = spawn_test_server().await;

    let slug = format!("duplicate-compat-{}", uuid::Uuid::new_v4());
    
    let payload = json!({
        "name": "First Product",
        "slug": slug,
        "price": "25.00",
        "stock": 12
    });

    // First request should succeed
    let response1 = client
        .post(&format!("{}/api/compat/products", addr))
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(response1.status(), StatusCode::CREATED);

    // Second request with same slug should fail
    let payload2 = json!({
        "name": "Second Product",
        "slug": slug,
        "price": "30.00",
        "stock": 5
    });

    let response2 = client
        .post(&format!("{}/api/compat/products", addr))
        .json(&payload2)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response2.status(), StatusCode::CONFLICT);
    let body: serde_json::Value = response2.json().await.unwrap();
    assert_eq!(body["error"]["code"], "duplicate_product_handle");
}

#[tokio::test]
async fn test_compat_create_negative_price_returns_400() {
    let (addr, client) = spawn_test_server().await;

    let payload = json!({
        "name": "Bad Price",
        "slug": "bad-price",
        "price": "-10.00",
        "stock": 12
    });

    let response = client
        .post(&format!("{}/api/compat/products", addr))
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["error"]["code"], "validation_failed");
}

#[tokio::test]
async fn test_native_create_still_works_regression() {
    let (addr, client) = spawn_test_server().await;

    let payload = json!({
        "title": "Native Product",
        "handle": format!("native-{}", uuid::Uuid::new_v4()),
        "description": "Native description",
        "price_cents": 1500,
        "inventory_quantity": 50,
        "published": true
    });

    let response = client
        .post(&format!("{}/api/products", addr))
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["product"]["title"], "Native Product");
    assert_eq!(body["product"]["price_cents"], 1500);
}
