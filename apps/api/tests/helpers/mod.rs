use api::{AppState, config, create_app};
use std::sync::Arc;
use tokio::net::TcpListener;

#[allow(dead_code)]
pub async fn spawn_test_server() -> (String, reqwest::Client) {
    let (addr, client, _) = spawn_test_server_with_cache().await;
    (addr, client)
}

pub async fn spawn_test_server_with_cache()
-> (String, reqwest::Client, Arc<api::cache::CacheClient>) {
    let config = config::Config::load();
    let db_pool = db::create_pool(&config.database_url);

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
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
