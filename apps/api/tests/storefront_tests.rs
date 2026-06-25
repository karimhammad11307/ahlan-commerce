use api::{AppState, config, create_app};
use std::sync::Arc;
use tokio::net::TcpListener;
use serde_json::{json, Value};
use uuid::Uuid;

async fn spawn_test_server() -> (String, reqwest::Client, Arc<api::cache::CacheClient>) {
    let config = config::Config::load();
    let db_pool = db::create_pool(&config.database_url);

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let cache_client = api::cache::CacheClient::new(&redis_url).unwrap();
    let cache_arc = Arc::new(cache_client);

    let state = AppState {
        config: Arc::new(config),
        db_pool,
        cache: cache_arc.clone(),
    };

    let app = create_app(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let address = format!("http://127.0.0.1:{}", port);

    (address, client, cache_arc)
}

#[tokio::test]
async fn test_storefront_missing_product_returns_404() {
    let (addr, client, _) = spawn_test_server().await;

    let response = client
        .get(format!("{}/products/missing-handle-123", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_storefront_unpublished_product_returns_404() {
    let (addr, client, _) = spawn_test_server().await;

    let handle = format!("test-handle-{}", Uuid::now_v7());
    let payload = json!({
        "title": "Unpublished Product",
        "handle": handle,
        "description": "Test description",
        "price_cents": 1000,
        "inventory_quantity": 10,
        "published": false
    });

    client.post(format!("{}/api/products", addr)).json(&payload).send().await.unwrap();

    let response = client
        .get(format!("{}/products/{}", addr, handle))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_storefront_cache_hit_miss_and_invalidation() {
    let (addr, client, cache) = spawn_test_server().await;

    let handle = format!("test-handle-{}", Uuid::now_v7());
    let payload = json!({
        "title": "Storefront Product",
        "handle": handle,
        "description": "Test description",
        "price_cents": 1500,
        "inventory_quantity": 10,
        "published": true
    });

    // 1. Create product
    let create_res = client.post(format!("{}/api/products", addr)).json(&payload).send().await.unwrap();
    assert_eq!(create_res.status(), 201);
    let created_product: Value = create_res.json().await.unwrap();
    let product_id = created_product["product"]["id"].as_str().unwrap();

    let cache_key = api::cache::keys::product_page(&handle);

    // Initial cache state (empty)
    assert!(cache.cache_get(&cache_key).await.is_none());

    // 2. First request (Miss -> DB -> Set Cache)
    let response1 = client.get(format!("{}/products/{}", addr, handle)).send().await.unwrap();
    assert_eq!(response1.status(), 200);
    let html1 = response1.text().await.unwrap();
    assert!(html1.contains("Storefront Product"));
    assert!(html1.contains("$15.00"));

    // Wait a brief moment to ensure Redis SET async task finishes if any, wait cache_set is actually awaited in handler.
    let cached_str = cache.cache_get(&cache_key).await.expect("Cache should be set");
    assert!(cached_str.contains("Storefront Product"));

    // 3. Second request (Hit)
    let response2 = client.get(format!("{}/products/{}", addr, handle)).send().await.unwrap();
    assert_eq!(response2.status(), 200);

    // 4. Update publication status
    let patch_payload = json!({ "published": false });
    client.patch(format!("{}/api/products/{}/publication", addr, product_id))
        .json(&patch_payload).send().await.unwrap();

    // Cache should be deleted
    assert!(cache.cache_get(&cache_key).await.is_none());

    // 5. Request unpublished product returns 404
    let response3 = client.get(format!("{}/products/{}", addr, handle)).send().await.unwrap();
    assert_eq!(response3.status(), 404);
}

#[tokio::test]
async fn test_storefront_redis_outage_fallback() {
    let config = config::Config::load();
    let db_pool = db::create_pool(&config.database_url);

    // Give it a bad redis port to simulate outage
    let cache_client = api::cache::CacheClient::new("redis://127.0.0.1:9999").unwrap();

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
    let addr = format!("http://127.0.0.1:{}", port);

    let handle = format!("test-handle-{}", Uuid::now_v7());
    let payload = json!({
        "title": "Fallback Product",
        "handle": handle,
        "description": "Test description",
        "price_cents": 1000,
        "inventory_quantity": 10,
        "published": true
    });

    client.post(format!("{}/api/products", addr)).json(&payload).send().await.unwrap();

    let response = client
        .get(format!("{}/products/{}", addr, handle))
        .send()
        .await
        .unwrap();

    // Should gracefully fallback to DB and return 200
    assert_eq!(response.status(), 200);
}
