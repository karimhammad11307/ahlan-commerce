use async_graphql::{Context, Object, Result as GqlResult};
use crate::AppState;
use super::types::{ProductGql, ProductCreateInput};
use crate::errors::AppError;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn product_create(&self, ctx: &Context<'_>, input: ProductCreateInput) -> GqlResult<ProductGql> {
        let state = ctx.data::<AppState>()?;
        
        let domain_input = catalog::ProductCreate {
            title: input.title,
            handle: input.handle,
            description: input.description,
            price_cents: input.price_cents as u32,
            inventory_quantity: input.inventory_quantity as u32,
            published: input.published,
        };
        
        let new_product = match catalog::create_product(domain_input) {
            Ok(p) => p,
            Err(e) => return Err(AppError::ValidationFailed(e).into()),
        };
        
        let client = state.db_pool.get().await.map_err(|e| AppError::Internal(e.to_string()))?;
        
        match db::products::create_product(&**client, new_product).await {
            Ok(created_product) => {
                tracing::info!(
                    product_id = %created_product.id.0,
                    product_handle = %created_product.handle,
                    "product created via graphql"
                );
                
                let cache_key = crate::cache::keys::product_page(&created_product.handle);
                state.cache.cache_delete(&cache_key).await;

                Ok(ProductGql::from(created_product))
            }
            Err(e) => {
                if let Some(db_err) = e.as_db_error() {
                    if db_err.code().code() == "23505" {
                        tracing::warn!(
                            error_code = "duplicate_product_handle",
                            "failed to create product: handle already exists"
                        );
                        return Err(AppError::DuplicateHandle("Handle already exists".to_string()).into());
                    }
                }
                Err(AppError::Internal(e.to_string()).into())
            }
        }
    }
}
