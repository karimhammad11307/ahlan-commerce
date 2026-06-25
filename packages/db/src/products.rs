use crate::cornucopia::queries;
use catalog::{Product, ProductId};
use tokio_postgres::Error as PostgresError;
use time::OffsetDateTime;
use chrono::{DateTime, Utc};

fn convert_time_to_chrono(t: OffsetDateTime) -> DateTime<Utc> {
    DateTime::from_timestamp(t.unix_timestamp(), t.nanosecond()).unwrap_or_default()
}

// Convert from Cornucopia's tuple to our Product struct
fn map_product_row(row: (uuid::Uuid, String, String, String, i32, i32, bool, OffsetDateTime, OffsetDateTime, OffsetDateTime)) -> Product {
    Product {
        id: ProductId(row.0),
        title: row.1,
        handle: row.2,
        description: if row.3.is_empty() { None } else { Some(row.3) },
        price_cents: row.4 as u32,
        inventory_quantity: row.5 as u32,
        published: row.6,
        published_at: if row.6 { Some(convert_time_to_chrono(row.7)) } else { None },
        created_at: convert_time_to_chrono(row.8),
        updated_at: convert_time_to_chrono(row.9),
    }
}

pub async fn create_product(client: &impl cornucopia_client::GenericClient, product: Product) -> Result<Product, PostgresError> {
    let published_at = match product.published_at {
        Some(t) => OffsetDateTime::from_unix_timestamp(t.timestamp()).unwrap(),
        None => OffsetDateTime::from_unix_timestamp(0).unwrap(),
    };
    let created_at = OffsetDateTime::from_unix_timestamp(product.created_at.timestamp()).unwrap();
    let updated_at = OffsetDateTime::from_unix_timestamp(product.updated_at.timestamp()).unwrap();
    
    let description = product.description.unwrap_or_default();
    
    let res = queries::create_product::create_product(
        client, 
        &product.id.0, 
        &product.title, 
        &product.handle, 
        &description, 
        &(product.price_cents as i32), 
        &(product.inventory_quantity as i32), 
        &product.published, 
        &published_at, 
        &created_at, 
        &updated_at
    ).await?;
    
    Ok(map_product_row(res))
}

pub async fn list_products(client: &impl cornucopia_client::GenericClient) -> Result<Vec<Product>, PostgresError> {
    let rows = queries::list_products::list_products(client).await?;
    Ok(rows.into_iter().map(map_product_row).collect())
}

pub async fn list_published_products(client: &impl cornucopia_client::GenericClient) -> Result<Vec<Product>, PostgresError> {
    let rows = queries::list_published_products::list_published_products(client).await?;
    Ok(rows.into_iter().map(map_product_row).collect())
}

pub async fn get_product_by_handle(client: &impl cornucopia_client::GenericClient, handle: &str) -> Result<Option<Product>, PostgresError> {
    let row = queries::get_product_by_handle::get_product_by_handle(client, &handle).await?;
    Ok(row.map(map_product_row))
}

pub async fn update_product_publication(
    client: &impl cornucopia_client::GenericClient, 
    id: uuid::Uuid, 
    published: bool, 
    published_at: Option<DateTime<Utc>>, 
    updated_at: DateTime<Utc>
) -> Result<Product, PostgresError> {
    let published_at_time = match published_at {
        Some(t) => OffsetDateTime::from_unix_timestamp(t.timestamp()).unwrap(),
        None => OffsetDateTime::from_unix_timestamp(0).unwrap(),
    };
    let updated_at_time = OffsetDateTime::from_unix_timestamp(updated_at.timestamp()).unwrap();
    
    let res = queries::update_product_publication::update_product_publication(
        client, 
        &published, 
        &published_at_time, 
        &updated_at_time, 
        &id
    ).await?;
    
    Ok(map_product_row(res))
}
