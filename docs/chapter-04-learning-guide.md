# Chapter 04 - PostgreSQL Basics: Comprehensive Learning Guide & Strategy

Welcome to Chapter 04! This chapter transitions our **Ahlan Commerce** product API from a transient, in-memory store to a persistent, relational database using **PostgreSQL**. 

This guide serves as a pedagogical reference and step-by-step roadmap. It explains what we are doing, why we are doing it, the core architectural concepts, and the exact steps we will take to implement each task.

---

## 1. What the Chapter Teaches & How It Benefits You

### The Shift to Real Persistence
Up until now, when the Rust server restarted, all created products disappeared because they were held in a memory vector (`Vec<Product>` wrapped in `Arc<Mutex<>>`). In a production-grade system, applications must maintain state across restarts, crashes, deployments, and scaling events. This requires a separate persistence service—a database.

### Understanding the Service-Database Boundary
By using PostgreSQL (rather than SQLite), you learn that a database is not just a library running inside your application's memory space. It is a **separate service** with its own:
1. **Network Boundary**: Connections are established over TCP/IP or Unix domain sockets.
2. **Failure Modes**: The database might start slower than the app, run out of connections, have network hiccups, or fail mid-transaction.
3. **Concurrent Access Rules**: Multiple app instances might write to the same rows concurrently, requiring transaction isolation.

### The Migration Lifecycle (The "Why" of Atlas)
Schema design is never static. As features get added, tables must evolve. This chapter guides you through the progression of database schema discipline:
1. **Plain SQL Setup**: Getting the first tables up and running quickly.
2. **Manual Schema Evolution**: Making raw changes directly in the database (`ALTER TABLE`). This teaches you the **pain and fragility** of manual alterations (drift between code and database, forgetting where changes were applied, breaking active environments).
3. **Atlas Declarative Migrations**: Transitioning to a version-controlled migration workflow. This ensures that every database instance (local, staging, production) is updated in a repeatable, reviewable, and automated way.

---

## 2. Core Architectural Concepts Explained

### A. What is a Connection Pool?
A **Connection Pool** (managed by SQLx as `PgPool`) is a cache of database connections maintained by the application. 

#### Why do we need it?
1. **High Overhead of Connection Establishment**: Setting up a new TCP connection, performing the PostgreSQL handshake, and authenticating for *every* incoming HTTP request is extremely slow and resource-intensive.
2. **Concurrent Request Handling**: Since Axum handles requests concurrently across multiple threads, a single connection would bottleneck the entire application (only one query could run at a time). A pool allows multiple worker threads to checkout connections, execute queries in parallel, and return them to the pool.
3. **Resource Capping**: It prevents the application from overwhelming the database with too many concurrent connections, which could exhaust database memory.

### B. Why Plain SQL first?
We use raw, plain SQL queries (e.g., `SELECT ... FROM products`, `INSERT INTO products ...`) instead of query builders or ORMs (Object-Relational Mappings) to:
* Avoid hiding the relational database model behind abstractions.
* See the direct mapping between Rust structs and database rows.
* Feel the compilation feedback loops and where runtime drift can occur before we introduce code-generation tools.

### C. The Danger of Manual Schema Changes
In Task 04.2, we modify the schema manually using raw SQL statements directly against our local database instance. 
* **The Danger**: In a real team, if developer A runs `ALTER TABLE products ADD COLUMN description text` locally, developer B's local app will crash because they do not have the column. If we deploy the code to production without running that exact same SQL command first, the production app will crash instantly.
* **Schema Drift**: When the database structure and the application's expected structure disagree, queries fail at runtime. Manual changes cannot be easily reviewed in pull requests, tested in CI/CD, or rolled back safely.

### D. What Atlas Solves (and What It Doesn't)
**Atlas** is a modern database schema migration tool. It allows you to define your desired database schema in a declarative format (SQL or HCL) and automatically calculates the migration steps (diffs) to transition the database from its current state to the desired state.
* **What it solves**: Schema migration discipline. It generates migration scripts, tracks which migrations have been applied to which database using a metadata table, and ensures consistency across environments.
* **What it does NOT solve**: Query type safety in Rust. Even if Atlas manages your tables perfectly, Rust query code can still drift from the database structure (e.g., querying a column that was renamed or dropped) unless we introduce query code generation (which we will do in a later chapter).

---

## 3. Step-by-Step Implementation Roadmap

We will complete Chapter 04 by moving systematically through the four tasks:

### Task 04.1: Persist with Plain SQL
* **Goal**: Replace the in-memory `products_db` with a PostgreSQL connection pool (`PgPool`) and read/write products using plain SQL queries.
1. **Add Dependencies**: Scan and add `sqlx` (with `postgres`, `runtime-tokio-rustls`, `chrono`, `uuid` features) and `dotenvy` to `Cargo.toml`.
2. **Configure Database Connection**: Add loading of `DATABASE_URL` in `config.rs` from `.env`.
3. **Initialize Schema**: Drop the existing boilerplate `products` table and execute `initial-products.sql` on the local database.
4. **Update Shared State**: Replace the `Arc<Mutex<Vec<Product>>>` in `AppState` with a `sqlx::PgPool`.
5. **Update Repository Logic**: Replace vectors with `sqlx` query operations in `packages/catalog` (or in handlers if query code is directly in the app). We will write SQL queries like:
   * `SELECT id, title, handle, price_cents, inventory_quantity, published, created_at, updated_at FROM products`
   * `INSERT INTO products (id, title, handle, price_cents, inventory_quantity, published, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)`
6. **Verify API Behavior**: Confirm that `POST /api/products` and `GET /api/products` work exactly as before, and that restarting the API preserves the product data.

### Task 04.2: Change the Schema the Painful Way
* **Goal**: Manually alter the database schema to add `description` and `published_at` columns, modify our Rust models/APIs, and observe the complexity of keeping them in sync.
1. **Run Manual ALTER TABLE**:
   ```sql
   ALTER TABLE products ADD COLUMN description text;
   ALTER TABLE products ADD COLUMN published_at timestamptz;
   ```
2. **Update Rust Models**:
   * Add optional `description: Option<String>` and `published_at: Option<DateTime<Utc>>` to the domain `Product` and `ProductCreate` structs in `packages/catalog/src/lib.rs`.
   * Update the DTOs (`CreateProductRequest` and `ProductResponse`) in `apps/api/src/dtos.rs`.
3. **Update Queries**: Adjust SQL queries to include `description` and `published_at` in both reading and writing operations.
4. **Document Reflection**: Write `docs/manual-schema-change.md` to log the change details and answer the reflective questions about what broke and why this is dangerous in production.
5. **Verify**: Ensure the API matches the new `product-api-contract.md` contract.

### Task 04.3: Introduce Atlas
* **Goal**: Bring in Atlas to manage schema migrations declaratively.
1. **Create Atlas Schema & Config**:
   * Create `db/schema/products.sql` containing the desired `products` table structure.
   * Write `atlas.hcl` to define the environment settings (database connection URL, migration directory, schema source).
2. **Generate Migrations**:
   * Run `atlas migrate diff initial_products --env local` to generate the baseline migrations under `db/migrations/`.
3. **Verify Migration Application**:
   * Clean/drop the database, and verify that `atlas migrate apply --env local` successfully sets up the database schema from scratch.
4. **Document Atlas Commands**: Write a brief instruction section in `docs/manual-schema-change.md` or a new file outlining the exact Atlas command flags.

### Task 04.4: Keep Plain SQL for Now
* **Goal**: Retain plain SQL queries for our database operations, resisting the urge to add query generators or ORMs.
1. **Write Reflection Notes**: Record in `docs/manual-schema-change.md` that while Atlas handles migrations, the Rust query code remains manually written, meaning that query-to-schema drift is still possible.
2. **Final Compilation & Test**: Verify that the entire system compiles cleanly and runs correctly.

---

## 4. How to Think & Debug During Implementation
* **Keep Dates UTC**: In Postgres, we use `timestamptz`. In Rust, we use `DateTime<Utc>`. Make sure to store and read dates as UTC to avoid timezone issues.
* **UUIDv7 Hand-off**: We generate UUIDs in Rust using `uuid::Uuid::now_v7()`. We insert them into Postgres as `uuid` columns. Ensure the database driver correctly serializes and deserializes `uuid::Uuid`.
* **Errors & Connections**: If the database server is not reachable, SQLx will return a `sqlx::Error::Database` or connection error. Ensure we log these errors properly using our `AppError::Internal` variant so that internal connection strings are not leaked to HTTP clients.
