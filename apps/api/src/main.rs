mod config;
mod dtos;
mod handlers;
mod routes;

use axum::{
    Router,
    routing::{get, post},
};
use catalog::Product;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<config::Config>,
    pub products_db: Arc<Mutex<Vec<Product>>>,
}

#[tokio::main]
async fn main() {
    // initialize tracing (from the observability contract)
    tracing_subscriber::fmt().init();
    tracing::info!("starting Ahlan commerce API..");

    let config = config::Config::load();
    let shared_state = AppState {
        config: Arc::new(config.clone()),
        products_db: Arc::new(Mutex::new(vec![])),
    };

    let app = Router::new()
        .route(routes::HEALTH, get(handlers::health_handler))
        .route(routes::PRODUCTS, get(handlers::list_products_handler))
        .route(routes::PRODUCTS, post(handlers::create_product_handler))
        .with_state(shared_state);

    // start server
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
