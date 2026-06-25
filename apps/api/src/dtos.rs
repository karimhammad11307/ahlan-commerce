// what the user is allowed to send in the Post body

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProductRequest {
    pub title: String,
    pub handle: String,
    pub description: Option<String>,
    pub price_cents: u32,
    pub inventory_quantity: u32,
    pub published: bool,
}

#[derive(Serialize, ToSchema)]
pub struct ProductResponse {
    pub id: String, // NOT UUID
    pub title: String,
    pub handle: String,
    pub description: Option<String>,
    pub price_cents: u32,
    pub inventory_quantity: u32,
    pub published: bool,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProductPublicationRequest {
    pub published: bool,
    pub published_at: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EnqueueImportJobRequest {
    pub input_path: String,
}

#[derive(Serialize, ToSchema)]
pub struct ImportJobResponse {
    pub id: String,
    pub status: String,
}

#[derive(Serialize, ToSchema)]
pub struct EnqueueImportJobResponse {
    pub job: ImportJobResponse,
}

#[derive(Serialize, ToSchema)]
pub struct ListProductsResponse {
    pub products: Vec<ProductResponse>,
}

#[derive(Serialize, ToSchema)]
pub struct SingleProductResponse {
    pub product: ProductResponse,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}
