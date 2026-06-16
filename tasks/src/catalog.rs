use serde::{Deserialize, Serialize};



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: ProductId,
    pub title: String,
    pub handle: String,
    pub price_cents: u32,
    pub inventory_quantity: u32,
    pub published: bool,

}
#[derive(Debug, serde::Deserialize)]

pub struct ProductCreate{
    pub title: String,
    pub handle: String,
    pub  price_cents: u32,
// tasks
    pub inventory_quantity: u32,
    pub published: bool,

}


pub fn create_product(input: ProductCreate) -> Product{
    Product {
        // hardcode as we will jot use DB
        id: ProductId("".to_string()),
        title: input.title,
        handle: input.handle,
        price_cents: input.price_cents,
        inventory_quantity: 0,
        published: false,
    }
}

// return list

pub fn list_product() -> Vec<Product> {
    Vec::new()
}

// test units
#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_create_product(){
        let input = ProductCreate{
            title: "test Product".to_string(),
            handle: "test-product".to_string(),
            price_cents: 100,
            inventory_quantity: todo!(),
            published: todo!(),
        };
        let product = create_product(input);

        assert_eq!(product.title, "test Product");
        assert_eq!(product.price_cents, 100);
        assert_eq!(product.inventory_quantity, 0); // default
        assert_eq!(product.published, false); 
   
    }
}