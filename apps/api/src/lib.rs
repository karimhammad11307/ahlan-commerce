pub mod cache;
pub mod compat;
pub mod config;
pub mod dtos;
pub mod errors;
pub mod graphql;
pub mod handlers;
pub mod openapi;
pub mod routes;
pub mod storefront;

use axum::{
    Router,
    routing::{get, patch, post},
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<config::Config>,
    pub db_pool: db::Pool,
    pub cache: Arc<cache::CacheClient>,
}

pub fn create_app(state: AppState) -> Router {
    let schema = graphql::build_schema(state.clone());

    Router::new()
        .merge(Scalar::with_url("/docs/scalar", openapi::ApiDoc::openapi()))
        .route(routes::HEALTH, get(handlers::health_handler))
        .route(
            routes::STOREFRONT_PRODUCT,
            get(storefront::storefront_handler),
        )
        .route(routes::PRODUCTS, get(handlers::list_products_handler))
        .route(
            routes::PUBLISHED_PRODUCTS,
            get(handlers::list_published_products_handler),
        )
        .route(routes::PRODUCTS, post(handlers::create_product_handler))
        .route(
            routes::PRODUCT_PUBLICATION,
            patch(handlers::update_product_publication_handler),
        )
        .route(
            routes::IMPORT_JOBS,
            post(handlers::enqueue_import_job_handler),
        )
        .route(
            routes::COMPAT_PRODUCTS,
            post(compat::handler::compat_create_product_handler),
        )
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
