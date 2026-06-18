// what the user is allowed to send in the Post body

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub title: String,
    pub handle: String,
    pub description: Option<String>,
    pub price_cents: u32,
    pub inventory_quantity: u32,
    pub published: bool,
}

#[derive(Serialize)]
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
