# Data Access Layer (DAL) Architecture

This document answers the core architectural questions regarding database persistence in Ahlan-commerce, specifically covering our SQL-first approach with Cornucopia.

### 1. What crate owns the raw database queries?
The **`packages/db`** crate owns the raw database queries. 
Raw SQL queries are stored in `db/queries/products/*.sql` files. Using the SQL-first approach with Cornucopia, these files are compiled into type-safe Rust code within the `packages/db/src/cornucopia.rs` module. The `db` crate then exposes a Data Access Layer boundary (`packages/db/src/products.rs`) that wraps these generated functions, ensuring all persistence logic is centralized and isolated from the rest of the application.

### 2. Why don't we use an ORM?
We avoid traditional ORMs (like Diesel or SeaORM) to maintain **SQL visibility** and avoid **abstraction leaks**.
ORMs hide the exact SQL executed behind Rust macros and DSLs, which can make it difficult to review, optimize, or debug query performance (e.g., N+1 query problems). By using Cornucopia with isolated `.sql` files:
* SQL remains perfectly reviewable as pure SQL.
* The Rust compiler still enforces strict type safety by checking the schema and generated code.
* We prevent "raw SQL drift" without paying the cognitive overhead of learning a complex DSL.

### 3. Where is the domain boundary between the HTTP layer and the persistence layer?
The domain boundary sits between the **HTTP Handlers (`apps/api/src/handlers.rs`)** and the **Data Access Layer (`packages/db/src/products.rs`)**.
The HTTP layer never interacts with raw SQL strings or the generated Cornucopia bindings directly. Instead, handlers:
1. Parse incoming HTTP requests into domain shapes.
2. Call `catalog` business logic to create or validate domain models.
3. Pass those domain models to the DAL (`db::products::*`).
4. Receive domain structures back from the DAL and serialize them into HTTP responses.

This ensures that the HTTP layer doesn't know *how* data is stored, and the persistence layer doesn't know *how* data is served.
