use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    http::StatusCode,
};
use crate::AppState;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

// --- CACHE LAYER SHAPE ---
#[derive(Serialize, Deserialize)]
pub struct CachedProductPage {
    pub html: String,
    pub product_id: String,
    pub product_updated_at: String,
    pub rendered_at: String,
}

// --- CONTEXT BUILDER ---
struct RenderContext {
    product_id: String,
    title: String,
    price_cents: u32,
    inventory_quantity: u32,
    updated_at: chrono::DateTime<chrono::Utc>,
}

async fn build_context(state: &AppState, handle: &str) -> Result<Option<RenderContext>, crate::errors::AppError> {
    let client = state.db_pool.get().await.map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;

    let product = match db::products::get_product_by_handle(&**client, handle).await {
        Ok(Some(p)) => p,
        Ok(None) => return Ok(None),
        Err(e) => return Err(crate::errors::AppError::Internal(e.to_string())),
    };

    if !product.published {
        return Ok(None);
    }

    Ok(Some(RenderContext {
        product_id: product.id.0.to_string(),
        title: product.title,
        price_cents: product.price_cents,
        inventory_quantity: product.inventory_quantity,
        updated_at: product.updated_at,
    }))
}

// --- RENDERER ---
fn render_html(ctx: &RenderContext) -> String {
    let inventory_msg = if ctx.inventory_quantity > 0 {
        format!("{} in stock", ctx.inventory_quantity)
    } else {
        "Out of stock".to_string()
    };

    let price_dollars = ctx.price_cents as f64 / 100.0;

    format!(
        "<!doctype html>\n<html>\n<head><title>{title}</title></head>\n<body>\n    <h1>{title}</h1>\n    <p>Price: ${price:.2}</p>\n    <p>{inventory}</p>\n</body>\n</html>",
        title = ctx.title,
        price = price_dollars,
        inventory = inventory_msg
    )
}

// --- CACHE LAYER & HANDLER ---
pub async fn storefront_handler(
    State(state): State<AppState>,
    Path(handle): Path<String>,
) -> axum::response::Response {
    let cache_key = crate::cache::keys::product_page(&handle);

    // 1. Try Redis GET
    if let Some(cached_json) = state.cache.cache_get(&cache_key).await {
        if let Ok(cached_page) = serde_json::from_str::<CachedProductPage>(&cached_json) {
            return Html(cached_page.html).into_response();
        } else {
            // Invalid JSON in cache, we will fall through to db read
            warn!(cache_key = %cache_key, "invalid json in cache, falling back to db");
        }
    }

    // 2. Cache Miss or Fallback -> Build Context
    let ctx_result = build_context(&state, &handle).await;
    match ctx_result {
        Ok(Some(ctx)) => {
            // 3. Render HTML
            let html = render_html(&ctx);
            
            let cache_payload = CachedProductPage {
                html: html.clone(),
                product_id: ctx.product_id,
                product_updated_at: ctx.updated_at.to_rfc3339(),
                rendered_at: chrono::Utc::now().to_rfc3339(),
            };

            // 4. Save to Redis (fire and forget)
            if let Ok(json_str) = serde_json::to_string(&cache_payload) {
                state.cache.cache_set(&cache_key, &json_str, crate::cache::PRODUCT_PAGE_TTL).await;
            }

            Html(html).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Product not found").into_response(),
        Err(e) => {
            error!(error = ?e, "internal server error during storefront render");
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        }
    }
}
