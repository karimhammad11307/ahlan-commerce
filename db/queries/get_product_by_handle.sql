--! get_product_by_handle(handle) ?
SELECT id, title, handle, description, price_cents, inventory_quantity, published, published_at, created_at, updated_at
FROM products
WHERE handle = $1
LIMIT 1;
