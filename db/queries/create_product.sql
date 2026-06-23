--! create_product(id, title, handle, description, price_cents, inventory_quantity, published, published_at, created_at, updated_at)
INSERT INTO products (id, title, handle, description, price_cents, inventory_quantity, published, published_at, created_at, updated_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
RETURNING id, title, handle, description, price_cents, inventory_quantity, published, published_at, created_at, updated_at;
