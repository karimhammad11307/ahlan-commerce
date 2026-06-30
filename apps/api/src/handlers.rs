use crate::errors::AppError;
use crate::{
    AppState,
    dtos::{
        CreateProductRequest, EnqueueImportJobRequest, ProductResponse,
        UpdateProductPublicationRequest,
    },
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde_json::{Value, json};

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "API is healthy", body = serde_json::Value)
    )
)]
pub async fn health_handler() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

#[utoipa::path(
    get,
    path = "/api/products",
    responses(
        (status = 200, description = "List all products", body = crate::dtos::ListProductsResponse),
        (status = 500, description = "Internal server error", body = crate::dtos::ErrorResponse)
    )
)]
pub async fn list_products_handler(
    State(state): State<AppState>,
) -> Result<Json<crate::dtos::ListProductsResponse>, AppError> {
    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

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

    tracing::debug!(count = responses.len(), "products listed");

    Ok(Json(crate::dtos::ListProductsResponse {
        products: responses,
    }))
}

#[utoipa::path(
    post,
    path = "/api/products",
    request_body = CreateProductRequest,
    responses(
        (status = 201, description = "Product created successfully", body = crate::dtos::SingleProductResponse),
        (status = 400, description = "Validation failed", body = crate::dtos::ErrorResponse),
        (status = 409, description = "Duplicate handle", body = crate::dtos::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::dtos::ErrorResponse)
    )
)]
pub async fn create_product_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<crate::dtos::SingleProductResponse>), AppError> {
    // validate inputs
    if payload.title.trim().is_empty() {
        tracing::warn!(
            error_code = "validation_failed",
            "request rejected: validation failed"
        );
        return Err(AppError::ValidationFailed(
            "title cannot be empty".to_string(),
        ));
    }
    if payload.handle.trim().is_empty() {
        tracing::warn!(
            error_code = "validation_failed",
            "request rejected: validation failed"
        );
        return Err(AppError::ValidationFailed(
            "handle cannot be empty".to_string(),
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
    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

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

            let cache_key = crate::cache::keys::product_page(&created_product.handle);
            state.cache.cache_delete(&cache_key).await;

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

            Ok((
                StatusCode::CREATED,
                Json(crate::dtos::SingleProductResponse {
                    product: response_dto,
                }),
            ))
        }
        Err(e) => {
            if let Some(db_err) = e.as_db_error() {
                if db_err.code().code() == "23505" {
                    tracing::warn!(
                        error_code = "duplicate_product_handle",
                        "failed to create product: handle already exists"
                    );
                    return Err(AppError::DuplicateHandle(
                        "Handle already exists".to_string(),
                    ));
                }
            }
            Err(AppError::Internal(e.to_string()))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/products/published",
    responses(
        (status = 200, description = "List published products", body = crate::dtos::ListProductsResponse),
        (status = 500, description = "Internal server error", body = crate::dtos::ErrorResponse)
    )
)]
pub async fn list_published_products_handler(
    State(state): State<AppState>,
) -> Result<Json<crate::dtos::ListProductsResponse>, AppError> {
    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

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

    Ok(Json(crate::dtos::ListProductsResponse {
        products: responses,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/products/{id}/publication",
    request_body = UpdateProductPublicationRequest,
    params(
        ("id" = String, Path, description = "Product ID UUID")
    ),
    responses(
        (status = 200, description = "Product publication updated", body = crate::dtos::SingleProductResponse),
        (status = 400, description = "Validation failed", body = crate::dtos::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::dtos::ErrorResponse)
    )
)]
pub async fn update_product_publication_handler(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<UpdateProductPublicationRequest>,
) -> Result<Json<crate::dtos::SingleProductResponse>, AppError> {
    let published_at_parsed = match payload.published_at {
        Some(t) => Some(
            chrono::DateTime::parse_from_rfc3339(&t)
                .map_err(|_| {
                    AppError::ValidationFailed("invalid published_at timestamp".to_string())
                })?
                .with_timezone(&chrono::Utc),
        ),
        None => None,
    };

    let updated_at = chrono::Utc::now();

    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let updated_product = db::products::update_product_publication(
        &**client,
        id,
        payload.published,
        published_at_parsed,
        updated_at,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let cache_key = crate::cache::keys::product_page(&updated_product.handle);
    state.cache.cache_delete(&cache_key).await;

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

    Ok(Json(crate::dtos::SingleProductResponse {
        product: response_dto,
    }))
}

#[utoipa::path(
    post,
    path = "/api/import-jobs",
    request_body = EnqueueImportJobRequest,
    responses(
        (status = 202, description = "Import job enqueued", body = crate::dtos::EnqueueImportJobResponse),
        (status = 400, description = "Validation failed", body = crate::dtos::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::dtos::ErrorResponse)
    )
)]
pub async fn enqueue_import_job_handler(
    State(state): State<AppState>,
    Json(payload): Json<crate::dtos::EnqueueImportJobRequest>,
) -> Result<(StatusCode, Json<crate::dtos::EnqueueImportJobResponse>), AppError> {
    if payload.input_path.trim().is_empty() {
        tracing::warn!(
            error_code = "validation_failed",
            "request rejected: validation failed"
        );
        return Err(AppError::ValidationFailed(
            "input_path cannot be empty".to_string(),
        ));
    }

    let job_id = uuid::Uuid::now_v7();
    let now = chrono::Utc::now();

    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let job = db::import_jobs::insert_import_job(
        &**client,
        job_id,
        db::import_jobs::ImportJobStatus::Queued,
        &payload.input_path,
        0,
        None,
        now,
        now,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    tracing::info!(
        job_id = %job.id,
        "import job enqueued"
    );

    let response_dto = crate::dtos::EnqueueImportJobResponse {
        job: crate::dtos::ImportJobResponse {
            id: job.id.to_string(),
            status: job.status.as_str().to_string(),
        },
    };

    Ok((StatusCode::ACCEPTED, Json(response_dto)))
}
