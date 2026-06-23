# Raw SQL Drift

This document answers the questions from Task 07.1 regarding the experience of adding new SQL queries directly in application code.

### Which SQL strings had to change?
We had to write completely new `SELECT` and `UPDATE` strings for `list_published_products_handler` and `update_product_publication_handler`. If we had changed the `products` table schema, we would have had to manually hunt down these strings in `handlers.rs` and update them. 

### Which Rust structs had to change?
We had to create a new DTO `UpdateProductPublicationRequest`. We also mapped the SQL results into `ProductResponse`. When parsing the rows returned from the raw query, we had to manually map every single field (e.g., `let id: uuid::Uuid = row.get("id");`).

### What could compile while still being wrong?
Because the SQL queries are just raw strings passed to `sqlx::query()`, the Rust compiler does not check if the columns actually exist in the database, if the types match, or if the syntax is valid. 
For example, if we mistyped `SELECT inventory_quantity` as `SELECT inventory_qty`, the code would compile perfectly fine but fail at runtime. Also, if we requested a string instead of an integer from `row.get("price_cents")`, it would compile but crash.

### What could leak to production if a query was missed?
If a schema migration renamed a column (like `price_cents` to `price`), and we missed updating one of the SQL strings in `handlers.rs`, the application would deploy successfully. However, when a user hits the endpoint with the missed query, the API would crash with a database error, leading to a production outage.

This highlights the painful "drift" between database schemas and embedded SQL strings, and justifies the need for typed code generation and a DAL.
