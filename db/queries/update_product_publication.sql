--! update_product_publication(published, published_at, updated_at, id)
UPDATE products 
SET published = $1, published_at = $2, updated_at = $3
WHERE id = $4
RETURNING id, title, handle, description, price_cents, inventory_quantity, published, published_at, created_at, updated_at;
