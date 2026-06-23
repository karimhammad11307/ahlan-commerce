--! list_products() *
SELECT id, title, handle, description, price_cents, inventory_quantity, published, published_at, created_at, updated_at 
FROM products
ORDER BY created_at ASC, id ASC;
