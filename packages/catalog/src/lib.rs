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
pub fn create_product(input: ProductCreate) -> Result<Product, String> {
    if input.title.trim().is_empty() {
        return Err("Title cannot be empty".to_string());
    }
    if input.handle.trim().is_empty() {
        return Err("Handle cannot be empty".to_string());
    }

    let now = Utc::now();
    let published_at = if input.published { Some(now) } else { None };

    Ok(Product {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_invalid_create_input_rejected() {
        let input = ProductCreate {
            title: "".to_string(),
            handle: "valid-handle".to_string(),
            description: None,
            price_cents: 1000,
            inventory_quantity: 10,
            published: false,
        };
        let result = create_product(input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Title cannot be empty");

        let input_handle = ProductCreate {
            title: "Valid Title".to_string(),
            handle: "".to_string(),
            description: None,
            price_cents: 1000,
            inventory_quantity: 10,
            published: false,
        };
        let result_handle = create_product(input_handle);
        assert!(result_handle.is_err());
        assert_eq!(result_handle.unwrap_err(), "Handle cannot be empty");
    }
}
