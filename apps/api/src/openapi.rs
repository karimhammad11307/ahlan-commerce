use utoipa::OpenApi;

use crate::dtos::{
    CreateProductRequest, EnqueueImportJobRequest, EnqueueImportJobResponse, ErrorDetail,
    ErrorResponse, ImportJobResponse, ListProductsResponse, ProductResponse,
    SingleProductResponse, UpdateProductPublicationRequest,
};

use crate::handlers::{
    create_product_handler, enqueue_import_job_handler, health_handler,
    list_products_handler, list_published_products_handler, update_product_publication_handler,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health_handler,
        crate::handlers::list_products_handler,
        crate::handlers::create_product_handler,
        crate::handlers::list_published_products_handler,
        crate::handlers::update_product_publication_handler,
        crate::handlers::enqueue_import_job_handler
    ),
    components(
        schemas(
            CreateProductRequest,
            EnqueueImportJobRequest,
            EnqueueImportJobResponse,
            ErrorDetail,
            ErrorResponse,
            ImportJobResponse,
            ListProductsResponse,
            ProductResponse,
            SingleProductResponse,
            UpdateProductPublicationRequest
        )
    ),
    tags(
        (name = "ahlan-commerce", description = "Ahlan-Commerce API")
    )
)]
pub struct ApiDoc;
