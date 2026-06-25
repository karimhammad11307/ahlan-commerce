# Architecture Boundaries

Ahlan-Commerce enforces strict architectural boundaries to separate concerns, ensure testability, and provide a fast user experience.

## Persistence vs. Handlers

**Handlers do not own persistence.**
In our Axum server, HTTP handlers inside `apps/api/src/handlers.rs` are responsible *only* for:
1. Parsing requests and validating shapes.
2. Converting requests into Core Domain models.
3. Passing those domain models to the database logic layer (`packages/db/src/`).
4. Returning the formatted HTTP response.

By separating the database logic from the HTTP logic, we can easily test handlers and replace the persistence layer without rewriting HTTP validation.

## Atlas vs. Cornucopia

We use two distinct tools for database management:
- **Atlas (Schema Migrations)**: Owns the shape of the database. It applies `.sql` migrations located in `db/migrations/` to manage tables, indices, and constraints.
- **Cornucopia (Typed Queries)**: Owns data access. It reads our custom `.sql` queries located in `db/queries/` and generates type-safe Rust functions to interact with the database. It does not create tables.

## Cache-Aside Pattern (Storefront UI)

For the public storefront, we use a strict **Cache-Aside Pattern** powered by Redis:
1. The Axum route `GET /products/{handle}` generates a deterministic cache key.
2. It attempts a Redis `GET`. On a hit, it serves the cached HTML immediately, completely bypassing Postgres.
3. On a miss, it fetches the data from Postgres, applies business policy (e.g., `published == true`), renders the HTML string natively, and asynchronously saves the payload into Redis with a 300-second TTL.

*(See `operations.md` for runtime failure behaviors).*

## REST and GraphQL Coexistence

Our API exposes both REST and GraphQL interfaces securely via the Axum Router (`apps/api/src/lib.rs`):
- **REST**: Best for strict operational endpoints like `/health`, import jobs, and simple data pipelines. Fully documented via OpenAPI/Scalar.
- **GraphQL**: Best for flexible frontend data querying. The schema is statically typed and mounted to `/api/graphql`.
Both interfaces rely on the exact same underlying `AppState` containing the `PgPool` and `CacheClient`, ensuring consistent data access regardless of the transport protocol.
