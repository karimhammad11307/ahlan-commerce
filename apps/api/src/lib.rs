pub mod config;
pub mod dtos;
pub mod handlers;
pub mod routes;
pub mod errors;
pub mod graphql;

use axum::{
    Router,
    routing::{get, post, patch},
};
use tower_http::trace::TraceLayer;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<config::Config>,
    pub db_pool: db::Pool,
}

pub fn create_app(state: AppState) -> Router {
    let schema = graphql::build_schema(state.clone());

    Router::new()
        .route(routes::HEALTH, get(handlers::health_handler))
        .route(routes::PRODUCTS, get(handlers::list_products_handler))
        .route(routes::PUBLISHED_PRODUCTS, get(handlers::list_published_products_handler))
        .route(routes::PRODUCTS, post(handlers::create_product_handler))
        .route(routes::PRODUCT_PUBLICATION, patch(handlers::update_product_publication_handler))
        .route("/graphql", post(graphql_handler))
        .layer(axum::Extension(schema))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn graphql_handler(
    axum::Extension(schema): axum::Extension<graphql::AppSchema>,
    req: async_graphql_axum::GraphQLRequest,
) -> async_graphql_axum::GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
