# Daily Log — June 17, 2026: The Journey Through Chapter 03 & Chapter 04

## Date: June 17, 2026
## Chapters Covered:
* **Chapter 03:** In-Memory Product API (Axum, Serde, DTOs, and Error Handling)
* **Chapter 04:** PostgreSQL Basics (SQLx, Connection Pools, Manual Schema Changes, and Atlas Migrations)

---

## 1. Chapter 03: In-Memory Product API

Today we transitioned from a monolithic directory to a clean, modular **monorepo layout**. This taught us the core principles of Rust service architecture:

### Key Concepts & Implementations
* **Monorepo Restructuring:** We split the codebase into:
  * `apps/api/`: The web API crate containing Axum routing, configuration, HTTP handlers, and DTOs.
  * `packages/catalog/`: The domain business logic library.
* **Separation of Concerns (Handler vs. Domain Logic):**
  * Handlers (`apps/api/src/handlers.rs`) act as thin transport adapters. Their only job is to deserialize HTTP request data, call a domain function, and serialize the result back to HTTP.
  * Domain functions (`packages/catalog/src/lib.rs`) contain the core business rules. They know nothing about HTTP, request headers, or status codes.
* **Data Transfer Objects (DTOs):**
  * We implemented `CreateProductRequest` and `ProductResponse` in `dtos.rs` to decouple what the API client sends/receives from how the product is stored internally. This prevents changes to our internal fields from breaking our public API contract.
* **Axum Extractors:**
  * `State(state)`: Injects shared server state (like connection pools or in-memory vectors) across async handlers.
  * `Json(payload)`: Parses the incoming HTTP request JSON body directly into a Rust struct.
* **Robust Error Handling:**
  * Created a custom `AppError` enum that implements Axum's `IntoResponse` trait.
  * This allows us to intercept errors and map them to standard HTTP status codes:
    * `400 Bad Request` for validation failures (e.g., empty titles).
    * `409 Conflict` for duplicate handles.
    * `500 Internal Server Error` for system crashes.
  * *Security Aspect:* Internal error messages (like database connection issues) are logged internally but stripped from the JSON body returned to the client to prevent leaking sensitive infrastructure info.

---

## 2. Chapter 04: PostgreSQL Basics & Database Persistence

We evolved our API from storing products in a temporary memory vector (`Vec<Product>`) to storing them in a real relational database: **PostgreSQL**.

### A. Installing & Initializing PostgreSQL on Linux
* **Service Boundary:** We learned that PostgreSQL is not an in-process library (like SQLite) but a separate network service running on port `5432`.
* **Database Setup:** 
  * Configured host, port, credentials, and connection strings inside the `.env` file:
    ```env
    DATABASE_URL=postgresql://ahlan:ahlan_dev@localhost:5432/ahlan_commerce
    ```
  * Checked connection health using the command-line utility `pg_isready`.
  * Initialized the table schema using the `db/schema/initial-products.sql` file via `psql`.

### B. The Connection Pool (`PgPool`)
* **What it does:** It maintains a cache of active TCP connections to the database.
* **Why it is critical:** Opening a fresh connection to a remote database for every single API request requires expensive network handshakes and authentication. A pool allows our async web server threads to reuse connections instantly, processing parallel queries concurrently without exhausting database resources.

### C. Plain SQL Queries with SQLx
* We used `sqlx::query` to run plain SQL statements directly in Rust.
* **Reading Products:** Fetched rows and deserialized individual columns using `row.get("column_name")`.
* **Writing Products:** Executed `INSERT INTO` statements using parameterized placeholders (`$1, $2, ...`) and bound variables safely to prevent SQL injection.

### D. Manual Schema Changes (The Painful Way)
* **What we did:** We ran raw SQL `ALTER TABLE` statements directly on the database to add a `description` column and a `published_at` timestamp. We then had to manually update our domain structs, our request/response DTOs, and rewrite every single SQL insert and select query to match.
* **Reflection & Hard Truths:**
  * **What we had to remember:** Field alignment across 4 separate places (Postgres columns, Domain models, DTO structs, and SQL bind strings).
  * **What broke on disagreement:** Stale code querying new columns or missing parameters crashed instantly at runtime with database errors.
  * **Production Danger:** Doing manual schema changes in production leads to **schema drift** (mismatch between code and live database), causing deployments to crash the application, and makes rollback or team sync impossible.

### E. Introducing Atlas (Declarative Migrations)
To solve schema drift, we introduced **Atlas**:
* **Declarative Schema File (`db/schema/products.sql`):** Our single source of truth describing what the database *should* look like.
* **Atlas Config (`atlas.hcl`):** Set up to manage our environments. We bypassed sandbox Docker limits by allocating a secondary schema (`atlas_dev`) inside our existing database for dev comparisons.
* **Version-Controlled migrations:**
  * We ran `atlas migrate diff initial_products --env local` to automatically generate our baseline migration scripts in `db/migrations/`.
  * We reset the database and ran `atlas migrate apply --env local --allow-dirty` to apply migrations and rebuild our table automatically.

### F. Query Safety & Code Drift
* **The Limit of Migrations:** We realized that while Atlas automates database schema changes, it does not check if our Rust SQL string queries match the database. If we drop a column in our schema, our Rust code still compiles fine but will crash at runtime. We will address this query-to-code drift in a future chapter.

---

## 3. Important Commands We Used Today

```bash
# Verify PostgreSQL server is running
pg_isready -h localhost -p 5432 -U ahlan

# Run SQL scripts to initialize schema
PGPASSWORD=ahlan_dev psql -h localhost -U ahlan -d ahlan_commerce -f "db/schema/initial-products.sql"

# Generate schema migrations using Atlas
.bin/atlas migrate diff initial_products --env local

# Apply migrations using Atlas
.bin/atlas migrate apply --env local --allow-dirty

# Verify compilation status
cargo check --all-targets

# Run test suite
cargo test
```

---

## 4. Resources and Reference Links

*Use this section to save links to documentation, tutorials, and articles you read during today's journey.*

* [Axum official documentation](https://docs.rs/axum/latest/axum/)
* [SQLx PostgreSQL driver docs](https://docs.rs/sqlx/latest/sqlx/struct.PgPool.html)
* [Atlas Declarative Migrations Guide](https://atlasgo.io/concepts/declarative-vs-versioned)
* [PostgreSQL Tutorial on SELECT and ALTER TABLE](https://www.postgresql.org/docs/current/sql-altertable.html)
* [Ahmed Farghal - Rust Al Ghalaba](https://www.youtube.com/watch?v=4KP1E_btdrI&list=PLald6EODoOJU0GMuYHlkS9MLhTPE7HiaT&index=4)
