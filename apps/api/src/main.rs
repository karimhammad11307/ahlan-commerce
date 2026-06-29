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

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set — see .env.example");

    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let bind_addr = std::env::var("API_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    let config = config::Config::load();

    // Establish connection pool to PostgreSQL
    let db_pool = db::create_pool(&database_url);

    let cache_client = api::cache::CacheClient::new(&redis_url).expect("failed to connect to Redis");

    let shared_state = AppState {
        config: Arc::new(config.clone()),
        db_pool,
        cache: Arc::new(cache_client),
    };

    let app = create_app(shared_state);

    // start server
    let listener = TcpListener::bind(&bind_addr).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
