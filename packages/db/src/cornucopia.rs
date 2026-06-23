// This file was generated with `cornucopia`. Do not modify.

pub mod types {  }

pub mod queries { pub mod create_product { use cornucopia_client::GenericClient;
use tokio_postgres::Error;

    pub async fn create_product<T: GenericClient>(client:&T, id : &uuid::Uuid,title : &str,handle : &str,description : &str,price_cents : &i32,inventory_quantity : &i32,published : &bool,published_at : &time::OffsetDateTime,created_at : &time::OffsetDateTime,updated_at : &time::OffsetDateTime) -> Result<(uuid::Uuid,String,String,String,i32,i32,bool,time::OffsetDateTime,time::OffsetDateTime,time::OffsetDateTime),Error> {let stmt = client.prepare("INSERT INTO products (id, title, handle, description, price_cents, inventory_quantity, published, published_at, created_at, updated_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
RETURNING id, title, handle, description, price_cents, inventory_quantity, published, published_at, created_at, updated_at;
").await?;
let res = client.query_one(&stmt, &[&id,&title,&handle,&description,&price_cents,&inventory_quantity,&published,&published_at,&created_at,&updated_at]).await?;

let return_value={ let return_value_0: uuid::Uuid = res.get(0); let return_value_1: String = res.get(1); let return_value_2: String = res.get(2); let return_value_3: String = res.get(3); let return_value_4: i32 = res.get(4); let return_value_5: i32 = res.get(5); let return_value_6: bool = res.get(6); let return_value_7: time::OffsetDateTime = res.get(7); let return_value_8: time::OffsetDateTime = res.get(8); let return_value_9: time::OffsetDateTime = res.get(9); (return_value_0,return_value_1,return_value_2,return_value_3,return_value_4,return_value_5,return_value_6,return_value_7,return_value_8,return_value_9) }; Ok(return_value)} }

pub mod list_published_products { use cornucopia_client::GenericClient;
use tokio_postgres::Error;

    pub async fn list_published_products<T: GenericClient>(client:&T, ) -> Result<Vec<(uuid::Uuid,String,String,String,i32,i32,bool,time::OffsetDateTime,time::OffsetDateTime,time::OffsetDateTime)>,Error> {let stmt = client.prepare("SELECT id, title, handle, description, price_cents, inventory_quantity, published, published_at, created_at, updated_at
FROM products
WHERE published = true
ORDER BY published_at DESC NULLS LAST, created_at ASC, id ASC;
").await?;
let res = client.query(&stmt, &[]).await?;

let return_value = res.iter().map(|res| { let return_value_0: uuid::Uuid = res.get(0); let return_value_1: String = res.get(1); let return_value_2: String = res.get(2); let return_value_3: String = res.get(3); let return_value_4: i32 = res.get(4); let return_value_5: i32 = res.get(5); let return_value_6: bool = res.get(6); let return_value_7: time::OffsetDateTime = res.get(7); let return_value_8: time::OffsetDateTime = res.get(8); let return_value_9: time::OffsetDateTime = res.get(9); (return_value_0,return_value_1,return_value_2,return_value_3,return_value_4,return_value_5,return_value_6,return_value_7,return_value_8,return_value_9) }).collect::<Vec<(uuid::Uuid,String,String,String,i32,i32,bool,time::OffsetDateTime,time::OffsetDateTime,time::OffsetDateTime)>>(); Ok(return_value)} }

pub mod list_products { use cornucopia_client::GenericClient;
use tokio_postgres::Error;

    pub async fn list_products<T: GenericClient>(client:&T, ) -> Result<Vec<(uuid::Uuid,String,String,String,i32,i32,bool,time::OffsetDateTime,time::OffsetDateTime,time::OffsetDateTime)>,Error> {let stmt = client.prepare("SELECT id, title, handle, description, price_cents, inventory_quantity, published, published_at, created_at, updated_at
FROM products
ORDER BY created_at ASC, id ASC;
").await?;
let res = client.query(&stmt, &[]).await?;

let return_value = res.iter().map(|res| { let return_value_0: uuid::Uuid = res.get(0); let return_value_1: String = res.get(1); let return_value_2: String = res.get(2); let return_value_3: String = res.get(3); let return_value_4: i32 = res.get(4); let return_value_5: i32 = res.get(5); let return_value_6: bool = res.get(6); let return_value_7: time::OffsetDateTime = res.get(7); let return_value_8: time::OffsetDateTime = res.get(8); let return_value_9: time::OffsetDateTime = res.get(9); (return_value_0,return_value_1,return_value_2,return_value_3,return_value_4,return_value_5,return_value_6,return_value_7,return_value_8,return_value_9) }).collect::<Vec<(uuid::Uuid,String,String,String,i32,i32,bool,time::OffsetDateTime,time::OffsetDateTime,time::OffsetDateTime)>>(); Ok(return_value)} }

pub mod update_product_publication { use cornucopia_client::GenericClient;
use tokio_postgres::Error;

    pub async fn update_product_publication<T: GenericClient>(client:&T, published : &bool,published_at : &time::OffsetDateTime,updated_at : &time::OffsetDateTime,id : &uuid::Uuid) -> Result<(uuid::Uuid,String,String,String,i32,i32,bool,time::OffsetDateTime,time::OffsetDateTime,time::OffsetDateTime),Error> {let stmt = client.prepare("UPDATE products
SET published = $1, published_at = $2, updated_at = $3
WHERE id = $4
RETURNING id, title, handle, description, price_cents, inventory_quantity, published, published_at, created_at, updated_at;
").await?;
let res = client.query_one(&stmt, &[&published,&published_at,&updated_at,&id]).await?;

let return_value={ let return_value_0: uuid::Uuid = res.get(0); let return_value_1: String = res.get(1); let return_value_2: String = res.get(2); let return_value_3: String = res.get(3); let return_value_4: i32 = res.get(4); let return_value_5: i32 = res.get(5); let return_value_6: bool = res.get(6); let return_value_7: time::OffsetDateTime = res.get(7); let return_value_8: time::OffsetDateTime = res.get(8); let return_value_9: time::OffsetDateTime = res.get(9); (return_value_0,return_value_1,return_value_2,return_value_3,return_value_4,return_value_5,return_value_6,return_value_7,return_value_8,return_value_9) }; Ok(return_value)} } }