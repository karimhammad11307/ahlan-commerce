mod config;
mod dtos;
mod handlers;
mod routes;
mod errors;

use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<config::Config>,
    pub db_pool: sqlx::PgPool,
}

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
    let db_pool = sqlx::PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let shared_state = AppState {
        config: Arc::new(config.clone()),
        db_pool,
    };

    let app = Router::new()
        .route(routes::HEALTH, get(handlers::health_handler))
        .route(routes::PRODUCTS, get(handlers::list_products_handler))
        .route(routes::PRODUCTS, post(handlers::create_product_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);

    // start server
    let addr = format!("127.0.0.1:{}", config.port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
