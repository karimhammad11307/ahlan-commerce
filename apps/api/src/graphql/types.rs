use async_graphql::{SimpleObject, InputObject, ID};
use chrono::{DateTime, Utc};
use catalog::Product;

#[derive(SimpleObject)]
#[graphql(name = "Product")]
pub struct ProductGql {
    pub id: ID,
    pub title: String,
    pub handle: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub inventory_quantity: i32,
    pub published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Product> for ProductGql {
    fn from(p: Product) -> Self {
        Self {
            id: ID(p.id.0.to_string()),
            title: p.title,
            handle: p.handle,
            description: p.description,
            price_cents: p.price_cents as i32,
            inventory_quantity: p.inventory_quantity as i32,
            published: p.published,
            published_at: p.published_at,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

#[derive(InputObject)]
#[graphql(name = "ProductCreateInput")]
pub struct ProductCreateInput {
    pub title: String,
    pub handle: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub inventory_quantity: i32,
    pub published: bool,
}
