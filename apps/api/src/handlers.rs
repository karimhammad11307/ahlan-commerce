use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};

use crate::{AppState, dtos::{CreateProductRequest, ProductResponse}};

pub async fn health_handler() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

pub async fn list_products_handler(State(state): State<AppState>) -> Json<Value> {
    let products = state.products_db.lock().unwrap();

    //convert internal domain models into external DTOs
    let responses: Vec<ProductResponse> = products
        .iter()
        .map(|p| ProductResponse {
            id: p.id.0.to_string(),
            title: p.title.clone(),
            handle: p.handle.clone(),
            price_cents: p.price_cents,
            inventory_quantity: p.inventory_quantity,
            published: p.published,
            created_at: p.created_at.to_rfc3339(),
        })
        .collect();

    Json(json!({"products" : responses}))
}

pub async fn create_product_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateProductRequest>,
) -> (StatusCode, Json<Value>) {
    // convert request into domain models
    // map incoming DTO to Domain Input
    let domain_input = catalog::ProductCreate {
        title: payload.title,
        handle: payload.handle,
        price_cents: payload.price_cents,
        inventory_quantity: payload.inventory_quantity,
        published: payload.published,
    };

    // call domain logic
    let new_product = catalog::create_product(domain_input);

    // mutate in-memory DB
    state.products_db.lock().unwrap().push(new_product.clone());

    // map domain output back to outgoing DTO
    let response_dto = ProductResponse {
        id: new_product.id.0.to_string(),
        title: new_product.title.clone(),
        handle: new_product.handle.clone(),
        price_cents: new_product.price_cents,
        inventory_quantity: new_product.inventory_quantity,
        published: new_product.published,
        created_at: new_product.created_at.to_rfc3339(),
    };

    // return response with 201 Created status code
    (StatusCode::CREATED, Json(json!({"product": response_dto})))
}
