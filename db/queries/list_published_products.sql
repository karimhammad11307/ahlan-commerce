--! list_published_products() *
SELECT id, title, handle, description, price_cents, inventory_quantity, published, published_at, created_at, updated_at 
FROM products 
WHERE published = true 
ORDER BY published_at DESC NULLS LAST, created_at ASC, id ASC;
