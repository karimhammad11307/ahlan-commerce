# Manual Schema Change Documentation & Reflection (Task 04.2)

In this task, we manually modified our database schema by adding `description` and `published_at` columns, and then updated our Rust backend application to match these changes.

## 1. SQL Scripts Executed
We connected directly to our local PostgreSQL database instance and executed:
```sql
ALTER TABLE products ADD COLUMN description text;
ALTER TABLE products ADD COLUMN published_at timestamptz;
```

---

## 2. Codebase Parts Updated
To support the database alterations, we had to update:
1. **Domain Models (`packages/catalog/src/lib.rs`):** Updated the business logic `Product` and `ProductCreate` structs to represent `description: Option<String>` and `published_at: Option<DateTime<Utc>>`.
2. **DTO Contracts (`apps/api/src/dtos.rs`):** Expanded `CreateProductRequest` and `ProductResponse` to support reading and returning the fields.
3. **Database Handlers (`apps/api/src/handlers.rs`):** Adapted `INSERT INTO` and `SELECT` query strings and bind operations.

---

## 3. Reflection Questions

### Q1: What did you have to remember?
* **Alignment across layers:** We had to trace the new fields through three completely separate layers: the raw PostgreSQL database table, the Rust business domain library (`catalog`), the web router DTOs (`api::dtos`), and the database query binders (`api::handlers`). 
* **Type mapping:** Mapping Postgres data types (`text`, `timestamptz`) to correct Rust optionals (`Option<String>`, `Option<DateTime<Utc>>`) and serializable strings (`Option<String>`).
* **Correct SQL syntax:** Typing columns exactly correctly (e.g. `price_cents`, `published_at`) and matching parameter indices `$1, $2, ...` to the values in `.bind(...)` in the correct order.

### Q2: What broke when Rust code and DB schema disagreed?
* If the Rust code compiled but the database columns had not yet been added manually, any queries selecting or inserting those columns would fail at **runtime** with a `DatabaseError` (e.g. `column "description" does not exist`).
* Conversely, if the database had the columns but our Rust code was querying a subset of them or binding the wrong number of parameters, the SQLx driver would throw a deserialization or binding error, resulting in an `AppError::Internal` status code 500.

### Q3: Why would this be dangerous in production?
* **Schema Drift & Downtime:** In production, if code is deployed that expects new columns (like `description`) before the production database tables are altered, the app will immediately crash for users whenever those queries run.
* **Coordination Overhead:** If multiple developers work concurrently on the same codebase, one developer running manual updates locally doesn't notify others. Their teammates' local setups will break when pulling the new code.
* **Lack of Repeatability:** Manual operations cannot be safely tested in Continuous Integration (CI), cannot be reviewed in Pull Requests (PRs), and are extremely difficult to roll back cleanly if something goes wrong.

---

## 4. Atlas Migration Workflow & Commands (Task 04.3)

After experiencing manual schema pain, we introduced **Atlas** to manage our database schema changes safely and declaratively.

### Core Atlas Files Added
1. **`db/schema/products.sql`:** The single source of truth for our desired table state.
2. **`atlas.hcl`:** The configuration specifying local connection strings, migrations directory, and development database paths.
3. **`db/migrations/`:** The folder housing version-controlled SQL migration scripts generated automatically by Atlas.

### Essential Atlas Commands

#### A. Generating Migrations (Schema Diffing)
To calculate the difference between the schema files (`db/schema/`) and generate migration scripts in `db/migrations/`, we run:
```bash
.bin/atlas migrate diff initial_products --env local
```
* **Why it matters:** This command automatically generates a new SQL file representing the transition from the previous schema version to the target.

#### B. Applying Migrations
To run all outstanding migration scripts and update our target database to the latest schema version:
```bash
.bin/atlas migrate apply --env local --allow-dirty
```
* **Why it matters:** This is executed during deployments to safely update the database schema without losing existing customer data. We use the `--allow-dirty` flag since our PostgreSQL instance contains a dev schema (`atlas_dev`) used for baseline comparisons.

---

## 5. The Limitations of Declarative Migrations (Task 04.4)

Although **Atlas** has successfully solved our database schema migration discipline (we now have versioned, automated schema updates that we can review and test), **it does not solve Rust query safety**.

### Why the next pain is Query/Code Drift:
* **The Problem:** The SQL query strings inside our Rust code (like `"SELECT title, price_cents FROM products"`) are simple strings. The Rust compiler treats them as arbitrary characters.
* **The Drift:** If we use Atlas to alter a column name (e.g. renaming `price_cents` to `price`), Atlas will generate the migration script and apply it cleanly to PostgreSQL. However, **our Rust code will still compile successfully!** The compiler has no way of checking that `"price_cents"` no longer exists in the database.
* **The Danger:** This mismatch (drift) will only reveal itself at **runtime** when a user attempts to call the API, causing a crash or database error.
* **The Next Step:** In a later chapter, we will introduce tools to generate compile-time checked database queries (such as SQLx compile-time query macros or ORMs) to ensure that code and schema are verified safe at compile-time.


