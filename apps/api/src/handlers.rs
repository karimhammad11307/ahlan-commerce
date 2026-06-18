use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};
use crate::errors::AppError;
use crate::{AppState, dtos::{CreateProductRequest, ProductResponse}};
use sqlx::Row;

pub async fn health_handler() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

pub async fn list_products_handler(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, title, handle, description, price_cents, inventory_quantity, published, published_at, created_at, updated_at FROM products"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let responses: Vec<ProductResponse> = rows
        .iter()
        .map(|row| {
            let id: uuid::Uuid = row.get("id");
            let title: String = row.get("title");
            let handle: String = row.get("handle");
            let description: Option<String> = row.get("description");
            let price_cents: i32 = row.get("price_cents");
            let inventory_quantity: i32 = row.get("inventory_quantity");
            let published: bool = row.get("published");
            let published_at: Option<chrono::DateTime<chrono::Utc>> = row.get("published_at");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

            ProductResponse {
                id: id.to_string(),
                title,
                handle,
                description,
                price_cents: price_cents as u32,
                inventory_quantity: inventory_quantity as u32,
                published,
                published_at: published_at.map(|t| t.to_rfc3339()),
                created_at: created_at.to_rfc3339(),
                updated_at: updated_at.to_rfc3339(),
            }
        })
        .collect();

    tracing::debug!(
        count = responses.len(),
        "products listed"
    );

    Ok(Json(json!({"products" : responses})))
}

pub async fn create_product_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    // validate inputs
    if payload.title.trim().is_empty() {
        tracing::warn!(
            error_code = "validation_failed",
            "request rejected: validation failed"
        );
        return Err(AppError::ValidationFailed(
            "title cannot be empty".to_string()
        ));
    }
    if payload.handle.trim().is_empty() {
        tracing::warn!(
            error_code = "validation_failed",
            "request rejected: validation failed"
        );
        return Err(AppError::ValidationFailed(
            "handle cannot be empty".to_string()
        ));
    }

    // convert request into domain models
    let domain_input = catalog::ProductCreate {
        title: payload.title,
        handle: payload.handle,
        description: payload.description,
        price_cents: payload.price_cents,
        inventory_quantity: payload.inventory_quantity,
        published: payload.published,
    };

    // call domain logic
    let new_product = catalog::create_product(domain_input);

    // Insert product into PostgreSQL
    let insert_result = sqlx::query(
        r#"
        INSERT INTO products (id, title, handle, description, price_cents, inventory_quantity, published, published_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#
    )
    .bind(new_product.id.0)
    .bind(&new_product.title)
    .bind(&new_product.handle)
    .bind(&new_product.description)
    .bind(new_product.price_cents as i32)
    .bind(new_product.inventory_quantity as i32)
    .bind(new_product.published)
    .bind(new_product.published_at)
    .bind(new_product.created_at)
    .bind(new_product.updated_at)
    .execute(&state.db_pool)
    .await;

    match insert_result {
        Ok(_) => {
            tracing::info!(
                product_id = %new_product.id.0,
                product_handle = %new_product.handle,
                "product created"
            );

            let response_dto = ProductResponse {
                id: new_product.id.0.to_string(),
                title: new_product.title,
                handle: new_product.handle,
                description: new_product.description,
                price_cents: new_product.price_cents,
                inventory_quantity: new_product.inventory_quantity,
                published: new_product.published,
                published_at: new_product.published_at.map(|t| t.to_rfc3339()),
                created_at: new_product.created_at.to_rfc3339(),
                updated_at: new_product.updated_at.to_rfc3339(),
            };

            Ok((StatusCode::CREATED, Json(json!({"product": response_dto}))))
        }
        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                // Check if unique key violation occurred (PostgreSQL code 23505)
                if db_err.code().as_deref() == Some("23505") {
                    tracing::warn!(
                        error_code = "duplicate_product_handle",
                        handle = %new_product.handle,
                        "failed to create product: handle already exists"
                    );
                    return Err(AppError::DuplicateHandle(new_product.handle));
                }
            }
            Err(AppError::Internal(e.to_string()))
        }
    }
}
