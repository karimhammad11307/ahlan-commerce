use axum::{Json, extract::{State, Path}, http::StatusCode};
use serde_json::{Value, json};
use crate::errors::AppError;
use crate::{AppState, dtos::{CreateProductRequest, ProductResponse, UpdateProductPublicationRequest}};


pub async fn health_handler() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

pub async fn list_products_handler(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let client = state.db_pool.get().await.map_err(|e| AppError::Internal(e.to_string()))?;
    
    let products = db::products::list_products(&**client)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let responses: Vec<ProductResponse> = products
        .into_iter()
        .map(|p| ProductResponse {
            id: p.id.0.to_string(),
            title: p.title,
            handle: p.handle,
            description: p.description,
            price_cents: p.price_cents,
            inventory_quantity: p.inventory_quantity,
            published: p.published,
            published_at: p.published_at.map(|t| t.to_rfc3339()),
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
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

    // Get a database connection from the pool
    let client = state.db_pool.get().await.map_err(|e| AppError::Internal(e.to_string()))?;

    // call domain logic to get the initial product
    let new_product = match catalog::create_product(domain_input) {
        Ok(p) => p,
        Err(e) => return Err(AppError::ValidationFailed(e)),
    };
    match db::products::create_product(&**client, new_product).await {
        Ok(created_product) => {
            tracing::info!(
                product_id = %created_product.id.0,
                product_handle = %created_product.handle,
                "product created"
            );

            let response_dto = ProductResponse {
                id: created_product.id.0.to_string(),
                title: created_product.title,
                handle: created_product.handle,
                description: created_product.description,
                price_cents: created_product.price_cents,
                inventory_quantity: created_product.inventory_quantity,
                published: created_product.published,
                published_at: created_product.published_at.map(|t| t.to_rfc3339()),
                created_at: created_product.created_at.to_rfc3339(),
                updated_at: created_product.updated_at.to_rfc3339(),
            };

            Ok((StatusCode::CREATED, Json(json!({"product": response_dto}))))
        }
        Err(e) => {
            if let Some(db_err) = e.as_db_error() {
                if db_err.code().code() == "23505" {
                    tracing::warn!(
                        error_code = "duplicate_product_handle",
                        "failed to create product: handle already exists"
                    );
                    return Err(AppError::DuplicateHandle("Handle already exists".to_string()));
                }
            }
            Err(AppError::Internal(e.to_string()))
        }
    }
}

pub async fn list_published_products_handler(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let client = state.db_pool.get().await.map_err(|e| AppError::Internal(e.to_string()))?;

    let products = db::products::list_published_products(&**client)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let responses: Vec<ProductResponse> = products
        .into_iter()
        .map(|p| ProductResponse {
            id: p.id.0.to_string(),
            title: p.title,
            handle: p.handle,
            description: p.description,
            price_cents: p.price_cents,
            inventory_quantity: p.inventory_quantity,
            published: p.published,
            published_at: p.published_at.map(|t| t.to_rfc3339()),
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(json!({"products" : responses})))
}

pub async fn update_product_publication_handler(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<UpdateProductPublicationRequest>,
) -> Result<Json<Value>, AppError> {
    let published_at_parsed = match payload.published_at {
        Some(t) => Some(chrono::DateTime::parse_from_rfc3339(&t)
            .map_err(|_| AppError::ValidationFailed("invalid published_at timestamp".to_string()))?
            .with_timezone(&chrono::Utc)),
        None => None,
    };

    let updated_at = chrono::Utc::now();

    let client = state.db_pool.get().await.map_err(|e| AppError::Internal(e.to_string()))?;

    let updated_product = db::products::update_product_publication(&**client, id, payload.published, published_at_parsed, updated_at)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let response_dto = ProductResponse {
        id: updated_product.id.0.to_string(),
        title: updated_product.title,
        handle: updated_product.handle,
        description: updated_product.description,
        price_cents: updated_product.price_cents,
        inventory_quantity: updated_product.inventory_quantity,
        published: updated_product.published,
        published_at: updated_product.published_at.map(|t| t.to_rfc3339()),
        created_at: updated_product.created_at.to_rfc3339(),
        updated_at: updated_product.updated_at.to_rfc3339(),
    };

    Ok(Json(json!({"product": response_dto})))
}
