use async_graphql::{Context, Object, Result as GqlResult};
use crate::AppState;
use super::types::ProductGql;
use crate::errors::AppError;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn products(&self, ctx: &Context<'_>) -> GqlResult<Vec<ProductGql>> {
        let state = ctx.data::<AppState>()?;
        
        let client = state.db_pool.get().await.map_err(|e| AppError::Internal(e.to_string()))?;
        
        let products = db::products::list_products(&**client)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
            
        Ok(products.into_iter().map(ProductGql::from).collect())
    }
}
