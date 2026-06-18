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
    pub description: Option<String>,
    pub price_cents: u32,
    pub inventory_quantity: u32,
    pub published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// domain input shape
pub struct ProductCreate {
    pub title: String,
    pub handle: String,
    pub description: Option<String>,
    pub price_cents: u32,
    pub inventory_quantity: u32,
    pub published: bool,
}

// business logic
pub fn create_product(input: ProductCreate) -> Product {
    let now = Utc::now();
    let published_at = if input.published {
        Some(now)
    } else {
        None
    };

    Product {
        id: ProductId(Uuid::now_v7()),
        title: input.title,
        handle: input.handle,
        description: input.description,
        price_cents: input.price_cents,
        inventory_quantity: input.inventory_quantity,
        published: input.published,
        published_at,
        created_at: now,
        updated_at: now,
    }
}
