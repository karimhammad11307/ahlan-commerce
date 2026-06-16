// task 0.1.1
mod catalog;

use axum::{
    extract::State,
    routing::{get,post},
    Json, Router,
};

use serde_json::{json,Value};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;


#[derive(Clone)]
struct  AppState{
    products_db: Arc<Mutex<Vec<catalog::Product>>>,
}

#[tokio::main]
async fn main() {
    // database initialization
    let shared_state = AppState{
        products_db: Arc::new(Mutex::new(Vec::new())),
    };

    // Explicitly typed `app` as `Router` (which defaults to Router<()>) to help the compiler with type inference.
    // This solves the E0282 "type annotations needed for Router<_>" error.
    let app: Router = Router::new()
        .route("/health", get(health_handler))
        .route("/api/products", get(list_products_handler))
        .route("/api/products", post(create_product_handler))
        .with_state(shared_state); // injection into axum

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Ahlan commerce: Server is listening on port 3000");

    // `axum::serve` to actually start the web server and resolve the type constraints for `app`.
    axum::serve(listener, app).await.unwrap();
}

// task 02.
// handlers routes
async fn health_handler() -> Json<Value> {
    Json(json!({"status": "healthy"}))
}
    
async fn list_products_handler(State(state): State<AppState>) -> Json<Value> {
    // lock mutex
    let products = state.products_db.lock().unwrap();

    Json(json!({"products": *products}))
}

async fn create_product_handler(
    State(state): State<AppState>,
    Json(payload): Json<catalog::ProductCreate>, // axum parses the request body
) -> Json<Value> {
    // call domain logic
    let new_product = catalog::create_product(payload);

    // lock mutex
    state.products_db.lock().unwrap().push(new_product.clone());

    // reponse json
    Json(json!({
        "product" : new_product
    }))

}


