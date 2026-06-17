use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

pub struct ProductId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: ProductId,
    pub title: String,
    pub handle: String,
    pub price_cents: u32,
    pub inventory_quantity: u32,
    pub published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// domain input shae
pub struct ProductCreate {
    pub title: String,
    pub handle: String,
    pub price_cents: u32,
    pub inventory_quantity: u32,
    pub published: bool,
}

// business logic

pub fn create_product(input: ProductCreate) -> Product {
    let now = Utc::now();
    Product {
        id: ProductId(Uuid::now_v7()),
        title: input.title,
        handle: input.handle,
        price_cents: input.price_cents,
        inventory_quantity: input.inventory_quantity,
        published: input.published,
        created_at: now,
        updated_at: now,
    }
}
