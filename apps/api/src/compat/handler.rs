use axum::{Json, extract::State, http::StatusCode};

use super::product_adapter::{ExternalProductPayload, adapt_external_product};
use crate::errors::AppError;
use crate::{AppState, dtos::ProductResponse};

pub async fn compat_create_product_handler(
    State(state): State<AppState>,
    Json(payload): Json<ExternalProductPayload>,
) -> Result<(StatusCode, Json<crate::dtos::SingleProductResponse>), AppError> {
    let domain_input = adapt_external_product(payload)?;

    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let new_product = match catalog::create_product(domain_input) {
        Ok(p) => p,
        Err(e) => return Err(AppError::ValidationFailed(e)),
    };

    match db::products::create_product(&**client, new_product).await {
        Ok(created_product) => {
            tracing::info!(
                product_id = %created_product.id.0,
                product_handle = %created_product.handle,
                "product created via compat adapter"
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
                        "failed to create compat product: handle already exists"
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
