use std::sync::Arc;
use tokio::net::TcpListener;
use api::{config, AppState, create_app};

#[tokio::main]
async fn main() {
    // initialize tracing (from the observability contract)
    tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "api=debug,tower_http=debug".into())
    )
    .init();
    tracing::info!("starting Ahlan commerce API..");

    let config = config::Config::load();

    // Establish connection pool to PostgreSQL
    let db_pool = db::create_pool(&config.database_url);

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let cache_client = api::cache::CacheClient::new(&redis_url).expect("failed to connect to Redis");

    let shared_state = AppState {
        config: Arc::new(config.clone()),
        db_pool,
        cache: Arc::new(cache_client),
    };

    let app = create_app(shared_state);

    // start server
    let addr = format!("127.0.0.1:{}", config.port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
