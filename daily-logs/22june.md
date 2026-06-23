# Daily Log - June 22, 2026

## What I Implemented Today
Today was a massive day for building out the backend architecture for Ahlan-Commerce. I completed the implementation for three full chapters (Chapters 7, 8, and 9), moving the project from basic REST endpoints to a fully tested, database-connected, multi-protocol (REST & GraphQL) API.

### 1. Chapter 07: SQL-First DAL (Data Access Layer)
- **Database Connection:** Set up a highly concurrent connection pool using `deadpool-postgres` and `tokio-postgres`.
- **SQL-First Approach:** Moved away from traditional ORMs. Instead, I wrote pure SQL queries in `.sql` files (`list_products.sql`, `insert_product.sql`, etc.).
- **Code Generation:** Used `Cornucopia` to automatically generate type-safe Rust functions from those raw SQL files, entirely eliminating SQL-injection risks while maintaining maximum database performance.
- **Data Mapping:** Created a mapping layer in `packages/db/src/products.rs` to convert Cornucopia tuples into the pure domain `catalog::Product` model.

### 2. Chapter 08: Specs and Tests (Behavior-Driven Development)
- **Product Specifications:** Built out a strict Product Requirements Document (`product-prd.md`) containing User Stories and Acceptance Criteria.
- **Scenario Mapping:** Translated the PRD into strict `Given/When/Then` scenarios (`product-scenarios.md`), mapping each behavior to a specific ID (e.g., `PRD-PROD-001`).
- **Domain Testing:** Added unit tests inside the `catalog` domain to ensure pure business rules (like rejecting empty titles) are enforced before reaching the database.
- **API Integration Testing:** Built an automated, in-memory testing server in `apps/api/tests/api_tests.rs`. Wrote tests covering `POST` and `GET` requests that hit the real database, explicitly proving the scenarios we defined.

### 3. Chapter 09: GraphQL Slice
- **GraphQL Schema Setup:** Added `async-graphql` and `async-graphql-axum` to support GraphQL natively inside our Axum router.
- **Type Safety & Mapping:** Leveraged macros like `#[derive(SimpleObject)]` and `#[derive(InputObject)]` to automatically generate our GraphQL schema from Rust structs. Handled CamelCase conversions (e.g., `price_cents` to `priceCents`) seamlessly.
- **Resolvers:** Built completely isolated `QueryRoot` and `MutationRoot` resolvers that leverage the exact same underlying Domain logic and Data Access Layer as the REST endpoints.
- **Error Extensions:** Extended our custom `AppError` to natively parse into `async-graphql::Error`, allowing us to inject custom extension codes (like `extensions: { code: "validation_failed" }`) while safely hiding internal database panics (`500 Internal Error`).
- **Omnichannel Support:** Configured the `axum` router to handle standard REST traffic at `/api/*` and GraphQL traffic at `/graphql` simultaneously without conflicts.

---

## What I Learned
1. **The Power of SQL-First:** Rather than fighting an ORM (like SeaORM or Diesel) to construct complex queries, writing pure SQL and letting Cornucopia generate the Rust bindings is significantly faster, safer, and easier to debug.
2. **Behavior-Driven Specifications:** Tests shouldn't just be arbitrary code; they should map directly to human-readable business requirements. By tagging tests with Scenario IDs, non-engineers can confidently read the specs and trust they are mathematically proven.
3. **GraphQL with Rust is Incredible:** `async-graphql` is a game-changer. The way it automatically converts `snake_case` to `camelCase`, maps Rust `Result`s into GraphQL errors, and infers schema types from Rust structs makes building a GraphQL layer incredibly clean and type-safe.
4. **Integration Testing in Axum:** Using `tokio::spawn` to boot up a temporary `axum::serve` instance on a random port makes full end-to-end HTTP testing incredibly fast and realistic compared to mocking everything.

---

## Important Links & Docs
Here are the core technologies and concepts I heavily utilized today:

*   **Cornucopia (SQL-First Rust Code Generator):** 
    *   [https://cornucopia-rs.netlify.app/](https://cornucopia-rs.netlify.app/)
*   **async-graphql (GraphQL Server Library for Rust):** 
    *   [https://async-graphql.github.io/async-graphql/en/index.html](https://async-graphql.github.io/async-graphql/en/index.html)
*   **SeaORM (For Comparison with SQL-First):**
    *   [https://www.sea-ql.org/SeaORM/](https://www.sea-ql.org/SeaORM/)
