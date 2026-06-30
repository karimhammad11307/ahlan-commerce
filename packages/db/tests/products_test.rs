use catalog::{ProductCreate, create_product};
use db::create_pool;

// Since we run tests against the same db, it's a good idea to create a helper for db URL.
fn get_test_db_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://ahlan:ahlan_dev@localhost:5432/ahlan_commerce".to_string())
}

#[tokio::test]
async fn test_create_and_list_products() {
    let pool = create_pool(&get_test_db_url());
    let client = pool.get().await.expect("Failed to get db client");

    let create_req = ProductCreate {
        title: "Test Product".to_string(),
        handle: format!("test-product-{}", uuid::Uuid::now_v7()),
        description: Some("Test description".to_string()),
        price_cents: 1000,
        inventory_quantity: 10,
        published: true,
    };

    let domain_product = create_product(create_req).unwrap();

    // Test create
    let created = db::products::create_product(&**client, domain_product.clone())
        .await
        .expect("Failed to create product");
    assert_eq!(created.title, "Test Product");
    assert_eq!(created.published, true);
    assert!(created.published_at.is_some());

    // Test list
    let products = db::products::list_products(&**client)
        .await
        .expect("Failed to list products");
    assert!(products.len() > 0);

    // Test list published
    let published_products = db::products::list_published_products(&**client)
        .await
        .expect("Failed to list published products");
    assert!(published_products.len() > 0);
}
