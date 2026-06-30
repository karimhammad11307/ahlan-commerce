# Ahlan Commerce — Comprehensive Project Guide

## 1. What Is Ahlan Commerce?

Ahlan Commerce is a learning sandbox for a backend e-commerce platform. It is a **multi-service Rust monorepo** backed by PostgreSQL and Redis, with a React admin frontend.

The codebase explores a handful of architectural patterns:

- **Compile-time checked SQL** via Cornucopia (no ORM, no raw string queries)
- **Cache-aside with Redis** (graceful fallback when Redis is down)
- **Background job processing** with a database-backed queue and retry logic
- **Multi-protocol API** — REST, GraphQL, and a compatibility adapter sharing the same domain layer
- **Process manager** (mprocs) for one-command local development

---

## 2. Directory Tree (Annotated)

```
ahlan-commerce/
├── Cargo.toml                  # Workspace root — 4 crates, shared dependencies
├── Makefile                    # 20+ targets for dev, test, build, deploy
├── mprocs.yaml                 # Process manager — launches all services
├── atlas.hcl                   # Atlas migration tool config
├── .env                        # Local dev environment variables
├── docker-compose.prod.yml     # Production Docker Compose (5 services)
│
├── apps/
│   ├── api/                    # Axum REST + GraphQL server
│   │   ├── src/
│   │   │   ├── main.rs         # Entry point: config, pool, server start
│   │   │   ├── lib.rs          # create_app() — router builder
│   │   │   ├── config.rs       # Environment config loader
│   │   │   ├── errors.rs       # AppError enum → HTTP/GraphQL errors
│   │   │   ├── dtos.rs         # Request/response DTOs
│   │   │   ├── routes.rs       # Route path constants
│   │   │   ├── handlers.rs     # REST handlers
│   │   │   ├── storefront.rs   # HTML storefront with caching
│   │   │   ├── cache.rs        # Redis cache client wrapper
│   │   │   ├── openapi.rs      # Utoipa OpenAPI spec
│   │   │   ├── compat/         # External API compatibility adapter
│   │   │   └── graphql/        # async-graphql schema, query, mutation
│   │   ├── tests/              # Integration tests (helpers + 4 test files)
│   │   └── Dockerfile          # Multi-stage Rust build
│   │
│   ├── worker/                 # Background import-job processor
│   │   └── src/main.rs         # Poll loop: acquire → process → retry
│   │
│   └── admin/                  # React/Vite admin frontend
│       ├── src/
│       │   ├── main.tsx        # Entry — QueryClient + Router
│       │   ├── routes/         # TanStack Router routes
│       │   ├── hooks/          # TanStack Query hooks (useProducts)
│       │   ├── components/     # ProductTable, ProductCreateForm, etc.
│       │   └── lib/            # GraphQL client (thin fetch wrapper)
│       ├── Dockerfile.admin    # Multi-stage Node → nginx build
│       └── nginx.conf          # SPA fallback to index.html
│
├── packages/
│   ├── catalog/                # Domain logic — pure Rust, no I/O
│   │   └── src/lib.rs          # Product, ProductCreate, create_product()
│   │
│   └── db/                     # Persistence — Cornucopia + deadpool-postgres
│       ├── src/
│       │   ├── lib.rs          # create_pool(), convert_time_to_chrono()
│       │   ├── products.rs     # Product persistence queries
│       │   ├── import_jobs.rs  # Import job persistence
│       │   └── cornucopia.rs   # AUTO-GENERATED — never edit
│       └── tests/              # DB integration tests
│
├── db/
│   ├── schema/                 # Declarative schema (Atlas source of truth)
│   ├── migrations/             # Timestamped migrations
│   └── queries/                # Cornucopia SQL query files (6 files)
│
├── docs/                       # Documentation
│   ├── generated/              # Auto-generated OpenAPI + GraphQL schema
│   └── COMPREHENSIVE_PROJECT_GUIDE.md   # ← This file
│
├── .bin/                       # Local binaries (gitignored)
│   ├── atlas                   # Atlas migration tool
│   └── host-exec               # Transparent Flatpak sandbox wrapper
│
└── .github/workflows/          # CI: fmt, clippy, migrate, cornucopia, test, build
```

---

## 3. Technology Stack & Concepts

### Rust — Workspace, Crates, Edition 2024

The project is a **Cargo workspace** with 4 crates:

```
apps/api       — Axum server (REST + GraphQL). Binary: api.
apps/worker    — Background worker. Binary: worker.
packages/catalog — Domain logic. Library, no I/O.
packages/db    — Persistence. Library, Cornucopia + deadpool-postgres.
```

**Workspace dependencies** (`[workspace.dependencies]` in root `Cargo.toml`) centralize versions for shared deps like `serde`, `tokio`, `uuid`, `chrono`, etc. Crates inherit them with `dep.workspace = true`.

Edition 2024 requires a recent Rust toolchain. It enables new syntax and the `use<>` precise import syntax.

### Axum — HTTP Framework

Axum is built on top of **Tower** (the standard Rust service/middleware ecosystem) and **Tokio**. Key patterns used:

- **Extractors**: `State<AppState>`, `Path<String>`, `Json<T>` — handler parameters declare what they need
- **State sharing**: `AppState` (DB pool, Redis client, config) wrapped in `Arc` and injected via `.with_state(state)`
- **IntoResponse**: Every handler returns something that implements `IntoResponse` — tuples like `(StatusCode, Json<T>)` or `Result<T, AppError>`
- **Middleware**: `TraceLayer` for HTTP request logging
- **Router merge**: Routes are composed with `.route()` and `.merge()` (for Scalar UI)

### Tokio — Async Runtime

Tokio is the async runtime powering both the API server and the worker. Used for:
- `tokio::main` — async entry point
- `tokio::net::TcpListener` — binding the server
- `tokio::spawn` — running background tasks
- `tokio::test` — async test harness

### Cornucopia — Compile-Time Checked SQL

Cornucopia reads `.sql` files from `db/queries/` and generates **type-safe Rust functions**. The SQL is parsed at compile time against a **live database** to verify correctness. Generated code goes into `packages/db/src/cornucopia.rs` — never edited manually.

Each query file contains one or more named queries with a `--!` annotation:

```sql
--! create_product(id, title, ...)
INSERT INTO products (...) VALUES ($1, $2, ...) RETURNING *;
```

This generates a function like `cornucopia::create_product(client, id, title, ...)` with the exact parameter types and return tuple.

Regenerate after changing queries: `make cornucopia-generate`.

### PostgreSQL — Connection Pool & Queries

**Connection pooling** uses `deadpool-postgres` (not `sqlx` — sqlx was removed as dead weight). Pool creation is centralized in `packages/db/src/lib.rs`:

```rust
pub fn create_pool(url: &str) -> Pool {
    let config = Config::new(url);
    config.create_pool(/* ... */)
}
```

**Concurrency control**: The worker's job acquisition uses `FOR UPDATE SKIP LOCKED` — atomically claims a queued job without blocking other workers:

```sql
UPDATE import_jobs SET status = 'running'
WHERE id = (
    SELECT id FROM import_jobs WHERE status = 'queued'
    ORDER BY created_at ASC FOR UPDATE SKIP LOCKED LIMIT 1
);
```

**Domain timestamps** are set by the Rust application clock (`chrono::Utc::now()`), not by the database. This keeps the domain layer testable without a database.

A **bridge function** `convert_time_to_chrono()` handles the mismatch between the `time` crate (used by tokio-postgres) and the `chrono` crate (used by the domain layer).

### Redis — Cache-Aside Pattern

The storefront uses a **cache-aside** strategy:

```
GET /products/{handle}
  → 1. Check Redis (key: "storefront:product-page:{handle}")
  → 2. On HIT → return cached HTML immediately
  → 3. On MISS → query PostgreSQL → render HTML → set Redis with 300s TTL → return HTML
```

**Cache invalidation** happens on:
- Product creation (`POST /api/products`)
- Publication status update (`PATCH /api/products/{id}/publication`)
- Also through GraphQL and compat endpoints (any write path invalidates)

**Graceful degradation**: If Redis is unreachable, the cache client logs a warning and returns `None`, causing a fallback to the database. No 500 error.

### async-graphql — GraphQL

The GraphQL layer is built with `async-graphql` + `async-graphql-axum`. It sits alongside the REST handlers, sharing the same domain and persistence layers.

- **Query**: `products: [Product!]!` — lists all products
- **Mutation**: `productCreate(input: ProductCreateInput!): Product!` — creates a product
- **Error handling**: `AppError` converts to `async_graphql::Error` with extension `code` for structured error responses

The schema is defined declaratively with derive macros (`#[Object]`, `#[SimpleObject]`, `#[InputObject]`).

### Utoipa + Scalar — OpenAPI Documentation

`utoipa` generates an OpenAPI 3.0 spec from Rust code annotations (`#[utoipa::path(...)]` on handlers, `#[derive(ToSchema)]` on DTOs). The spec is served via **Scalar** (a modern OpenAPI UI alternative to Swagger) at `GET /docs/scalar`.

A separate binary (`generate_docs`) writes the OpenAPI JSON and GraphQL SDL to `docs/generated/` as static files.

### Tracing — Structured Logging

The `tracing` crate with `tracing-subscriber` provides structured, async-aware logging. The subscriber uses `env-filter`, so log level is controlled via the `RUST_LOG` environment variable.

### React + TanStack Router + TanStack Query

The admin frontend uses:
- **React 18** with TypeScript
- **Vite** for dev server and build
- **TanStack Router** for type-safe client-side routing with auto-generated route tree
- **TanStack Query (v5)** for server state management (data fetching, caching, mutations)

Data flow:

```
Component → useProducts() hook → GraphQL fetch → /graphql proxy (Vite) → API server
```

The route tree is auto-generated by a Vite plugin and committed as `routeTree.gen.ts` (gitignored — never edit manually).

### Docker — Multi-Stage Builds & Production Compose

**API Dockerfile** (multi-stage):
1. **Builder**: `rust:1.89-slim` — compiles the API binary (caches dependencies separately for fast rebuilds)
2. **Runtime**: `debian:bookworm-slim` — minimal image containing only the binary, SSL certs, and libssl3

**Admin Dockerfile**: Node 20 builder → nginx:alpine runtime serving static files.

**Production Docker Compose** runs 5 services on an internal bridge network:
- `postgres:16-alpine` + `redis:7-alpine` (with health checks)
- `api` and `worker` (same image, different commands)
- `admin` (static files via nginx on port 8080)

No PostgreSQL port is published to the host in production — containers communicate via Docker's internal network.

### Atlas — Database Migrations

Atlas manages the database schema using a **declarative** approach:
- Source of truth: `db/schema/products.sql` (contains the complete desired schema)
- `atlas migrate diff` generates migration files from schema changes
- `atlas migrate apply` applies pending migrations
- A checksum file (`atlas.sum`) tracks migration integrity

### mprocs — Local Process Manager

mprocs runs all services in a single terminal with a TUI that shows logs from each process side by side:

```
┌───────────────┬─────────────────┬───────────────┐
│   redis       │   api           │   postgres    │
│               │                 │               │
├───────────────┼─────────────────┼───────────────┤
│   admin       │   worker        │               │
│               │                 │               │
└───────────────┴─────────────────┴───────────────┘
```

Start everything with a single command: `make start` → runs `mprocs`.

### Flatpak / host-spawn — Sandbox Development

The development environment runs inside a **Flatpak sandbox** (Freedesktop SDK 25.08). Inside the sandbox:
- No Docker binary
- No `redis-cli`
- No `pg_isready`

These tools exist on the **host**, reached via `host-spawn`. The project handles this transparently:

1. **Makefile**: A `HOST` variable auto-detects `host-spawn` and prefixes all docker/redis-cli/pg_isready commands
2. **`.bin/host-exec`**: A wrapper script for static config files (mprocs.yaml) that detects `host-spawn` at runtime and delegates to the host

---

## 4. Architecture — Request Lifecycles

### REST Product Creation

```
Client                     API Server                    PostgreSQL
  │                          │                             │
  │  POST /api/products      │                             │
  │  {title, handle, ...}    │                             │
  │ ───────────────────────> │                             │
  │                          │                             │
  │     handlers.rs          │                             │
  │     ├── validate fields  │                             │
  │     ├── catalog::        │                             │
  │     │   create_product() │                             │
  │     │   (domain logic)   │                             │
  │     ├── db::products::   │                             │
  │     │   create_product() │                             │
  │     │                    │ ───── INSERT product ─────> │
  │     │                    │ <── RETURN product ─────── │
  │     ├── cache::delete()  │                             │
  │     │   (invalidate)     │                             │
  │     │                    │                             │
  │  <── 201 {product}       │                             │
```

### Storefront (Cache-Aside)

```
Client                     Axum Handler                  Redis              PostgreSQL
  │                          │                            │                    │
  │  GET /products/{handle}  │                            │                    │
  │ ───────────────────────> │                            │                    │
  │                          │                            │                    │
  │                          │  GET storefront:page:{h}   │                    │
  │                          │ ────────────────────────>  │                    │
  │                          │                            │                    │
  │                          │  <── HIT (cached HTML) ──  │                    │
  │  <── 200 HTML ─────────  │                            │                    │
  │     (cache hit, fast)    │                            │                    │
  │                          │                            │                    │
  │     OR on MISS:          │                            │                    │
  │                          │                            │                    │
  │                          │   query product by handle  │                    │
  │                          │ ────────────────────────────────────────────> │
  │                          │ <── product row ─────────────────────────────│
  │                          │                            │                    │
  │                          │   render HTML              │                    │
  │                          │   SET cache 300s           │                    │
  │                          │ ────────────────────────>  │                    │
  │  <── 200 HTML ─────────  │                            │                    │
│     (cache miss, slow)   │                            │                    │
```

When Redis is down: `cache_get()` returns `None` gracefully → fall through to DB → render HTML → return 200. No error.

### Compat Adapter

```
Client                     Compat Handler              Domain             DB
  │                          │                          │                  │
  │  POST /api/compat/       │                          │                  │
  │  products                │                          │                  │
  │  {name, slug, price:     │                          │                  │
  │   "25.99", qty, ...}     │                          │                  │
  │ ───────────────────────> │                          │                  │
  │                          │                          │                  │
  │  product_adapter.rs      │                          │                  │
  │  ├── name → title        │                          │                  │
  │  ├── slug → handle       │                          │                  │
  │  ├── body_html→desc      │                          │                  │
  │  │   (empty → None)      │                          │                  │
  │  ├── price ("25.99")→    │                          │                  │
  │  │   price_cents (2599)  │                          │                  │
  │  ├── qty→inventory_qty   │                          │                  │
  │  ├── is_active→published │                          │                  │
  │  │                          │                          │                  │
  │  → catalog::              │                          │                  │
  │    create_product()       │                          │                  │
  │    (same as REST)         │                          │                  │
  │                          │ ──── INSERT ────>        │                  │
```

Field mapping: `name→title`, `slug→handle`, `body_html→description` (empty → None), `price` is a JSON **string** like `"25.99"` parsed as `f64` and converted to cents, `qty` has alias `stock`, `is_active` has alias `is_visible`.

### Background Worker (Import Jobs)

```
Worker Poll Loop (every 2s)
  │
  ├── acquire_queued_job()
  │   └── UPDATE ... FOR UPDATE SKIP LOCKED
  │
  ├── Read JSON file from input_path
  │
  ├── For each product in file:
  │   ├── catalog::create_product() (domain validation)
  │   ├── db::products::create_product() (INSERT)
  │   └── On error → retry
  │
  ├── On success → status = "succeeded"
  │
  └── On failure:
      ├── attempts < 3 → status = "queued" (retry next poll)
      └── attempts >= 3 → status = "failed"
```

---

## 5. Data Layer

### Database Schema

```sql
-- products
id                  uuid PRIMARY KEY       -- UUIDv7 (time-ordered, unique)
title               text NOT NULL
handle              text NOT NULL UNIQUE   -- URL-friendly identifier
description         text                   -- nullable (optional)
price_cents         integer NOT NULL        -- stored in cents, not float
inventory_quantity  integer NOT NULL
published           boolean NOT NULL
published_at        timestamptz             -- set on first publication
created_at          timestamptz NOT NULL    -- set by Rust app clock
updated_at          timestamptz NOT NULL    -- set by Rust app clock

-- import_jobs
id                  uuid PRIMARY KEY       -- UUIDv7
status              text NOT NULL           -- queued|running|succeeded|failed
input_path          text NOT NULL           -- filesystem path to import JSON
attempts            integer NOT NULL        -- retry counter
last_error          text                    -- error message from last failure
created_at          timestamptz NOT NULL
updated_at          timestamptz NOT NULL
```

### Cornucopia Pipeline

```
db/queries/*.sql  ──→  cornucopia generate  ──→  cornucopia.rs
                                                      │
                                           packages/db/src/
                                           products.rs, import_jobs.rs
                                           call generated functions
```

Regenerate after SQL changes: `make cornucopia-generate` (requires live PostgreSQL).

### Time Crate Bridge

Postgres driver (tokio-postgres) returns timestamps as `time::OffsetDateTime`. The domain uses `chrono::DateTime<Utc>`. The bridge:

```rust
pub fn convert_time_to_chrono(t: OffsetDateTime) -> DateTime<Utc> {
    // OffsetDateTime → chrono::DateTime<Utc>
}
```

---

## 6. Caching Strategy

| Aspect | Detail |
|---|---|
| **Pattern** | Cache-aside (lazy population) |
| **Cache key** | `storefront:product-page:{handle}` |
| **TTL** | 300 seconds (5 minutes) |
| **Invalidation** | On every product write (create, update publication) |
| **Redis outage** | Graceful fallback — `cache_get()` returns `None` → DB query → 200 OK |
| **Cache client** | Thin wrapper over `redis::Client` with logging on hit/miss/error |

---

## 7. Background Worker

### Job State Machine

```
    ┌─────────┐
    │ QUEUED  │
    └────┬────┘
         │ acquire (FOR UPDATE SKIP LOCKED)
         ▼
    ┌──────────┐
    │ RUNNING  │
    └────┬─────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
 ┌────────┐ ┌──────┐
 │SUCCEED │ │FAILED│
 └────────┘ └──────┘
      ▲
      │ (< 3 attempts, retry)
      └── QUEUED ──┘
```

### Retry Logic

- On failure: increment attempts counter
- If attempts < 3: set status back to "queued" for re-acquisition
- If attempts >= 3: set status to "failed" with last error message

---

## 8. Error Handling

### Unified Error Format

Every error response follows:

```json
{
  "error": {
    "code": "validation_failed",
    "message": "title must not be empty"
  }
}
```

### Error Codes

| HTTP Status | Error Code | Source |
|---|---|---|
| 400 | `validation_failed` | Empty title/handle, negative price |
| 404 | `not_found` | Product handle not found (storefront) |
| 409 | `duplicate_product_handle` | Unique constraint violation (PostgreSQL 23505) |
| 500 | `internal_error` | DB connection failure, unexpected errors |

### AppError Enum

```rust
pub enum AppError {
    ValidationFailed(String),   // 400
    DuplicateHandle(String),    // 409
    NotFound(String),           // 404
    Internal(String),           // 500
}
```

It implements both `IntoResponse` (for REST) and `From<AppError> for async_graphql::Error` (for GraphQL). Internal errors are logged before returning a safe generic message.

---

## 9. API Reference

### REST Endpoints

| Method | Path | Handler | Description |
|---|---|---|---|
| GET | `/health` | `health_handler` | Health check → `{"status":"ok"}` |
| GET | `/api/products` | `list_products_handler` | All products (created_at ASC) |
| POST | `/api/products` | `create_product_handler` | Create product + invalidate cache |
| GET | `/api/products/published` | `list_published_products_handler` | Published products (published_at DESC) |
| PATCH | `/api/products/{id}/publication` | `update_product_publication_handler` | Toggle publish + invalidate cache |
| POST | `/api/import-jobs` | `enqueue_import_job_handler` | Enqueue a background import |
| GET | `/products/{handle}` | `storefront_handler` | HTML storefront page (cached) |
| POST | `/api/compat/products` | `compat_create_product_handler` | External format product creation |

### GraphQL

| Type | Field | Description |
|---|---|---|
| **Query** | `products: [Product!]!` | List all products |
| **Mutation** | `productCreate(input: ProductCreateInput!): Product!` | Create product |
| **Input** | `ProductCreateInput` | title, handle, description, priceCents, inventoryQuantity, published |
| **Type** | `Product` | id, title, handle, description, priceCents, inventoryQuantity, published, publishedAt, createdAt, updatedAt |

**Endpoint**: `POST /graphql`
**OpenAPI/Scalar UI**: `GET /docs/scalar`

---

## 10. Frontend (Admin)

### Component Tree

```
<App>
  <QueryClientProvider>
    <RouterProvider>
      <RootLayout>
        ├── Header + Navigation
        └── <Outlet>
              ├── / → redirects to /products
              └── /products
                    ├── LoadingSpinner (while fetching)
                    ├── ErrorAlert (on fetch error)
                    ├── ProductTable (list of products)
                    ├── EmptyState (when no products)
                    └── ProductCreateForm (create new product)
    </RouterProvider>
  </QueryClientProvider>
```

### Data Flow

```
ProductTable / ProductCreateForm
      │
      ▼
useProducts() / useCreateProduct()   ← TanStack Query hooks
      │
      ▼
gqlFetch<T>()                        ← Thin fetch-based GraphQL client
      │
      ▼
POST /graphql                        ← Vite proxy → localhost:3000/graphql
```

The Vite dev server proxies `/graphql` to the Rust API server (`localhost:3000`).

---

## 11. Development Workflow

### Quick Start

```bash
# Start PostgreSQL + Redis
make db-start           # Docker container or native PostgreSQL
make redis-health       # Verify Redis is up

# Run all services
make start              # Launches mprocs (redis, api, postgres, admin, worker)

# Or run individually
make run-api            # API server on :3000
make run-worker         # Background worker

# Run tests
make test               # cargo test (requires live PostgreSQL + Redis)
```

### Key Makefile Targets

| Target | What It Does |
|---|---|
| `make run-api` | `cargo run -p api --bin api` |
| `make run-worker` | `cargo run -p worker` |
| `make test` | `cargo test` (all workspace tests) |
| `make db-start` | Start PostgreSQL (Docker or native) |
| `make db-migrate` | Apply pending Atlas migrations |
| `make redis-health` | Ping Redis |
| `make start` | Launch mprocs (all services) |
| `make docker-build` | Build API Docker image |
| `make prod-up` | Docker Compose production (5 services) |
| `make cornucopia-generate` | Regenerate Cornucopia code from SQL |
| `make docs-api` | Write OpenAPI + GraphQL schema to docs/ |

### mprocs Services

When you run `make start` (mprocs), five processes start:

| Panel | Command | Purpose |
|---|---|---|
| redis | `.bin/host-exec redis-cli monitor` | Redis CLI monitor (live commands) |
| api | `cargo run -p api` | Axum server on port 3000 |
| postgres | `make db-start && ... docker logs ...` | PostgreSQL logs |
| admin | `cd apps/admin && npm run dev` | Vite dev server |
| worker | `cargo run -p worker` | Background job processor |

### Testing

- **All tests are integration tests** requiring live PostgreSQL + Redis (no mocking)
- **No test isolation** — tests share DB state. Use `Uuid::now_v7()` for unique handles
- **Helpers**: `spawn_test_server()` and `spawn_test_server_with_cache()` in `tests/helpers/mod.rs` start the server on a random port
- **30 tests** across 5 test files + unit tests

### Flatpak Sandbox

When running inside the Flatpak sandbox (Freedesktop SDK 25.08):

- **Docker** is not available inside the sandbox, but exists on the host via `host-spawn docker`
- **`redis-cli`** and **`pg_isready`** are not in the sandbox — they exist on the host
- The Makefile auto-detects `host-spawn` via the `HOST` variable and prefixes all relevant commands
- `mprocs.yaml` uses `.bin/host-exec` to transparently delegate to the host at runtime
- If you see "PostgreSQL is not running": `host-spawn service postgresql start` (on the host)

---

## 12. Production & CI

### Production Architecture

```
┌─ Docker Compose ───────────────────────────────────────┐
│                                                         │
│  postgres:16-alpine  (internal network, no host port)   │
│  redis:7-alpine      (internal network, no host port)   │
│                                                         │
│  api     (Dockerfile)  ← connects to postgres + redis   │
│  worker  (same image, command: ./api worker)            │
│                                                         │
│  admin   (Dockerfile.admin → nginx :8080)               │
│                                                         │
│  Network: ahlan (bridge driver)                         │
│  Volumes: postgres_data (persistent)                    │
└─────────────────────────────────────────────────────────┘
```

### CI Pipeline (GitHub Actions)

```
commit → push
  │
  ├── cargo fmt --check
  ├── cargo clippy
  ├── atlas migrate apply
  ├── cornucopia check (regeneration diff)
  ├── cargo test
  └── npm run build (admin)
```

PostgreSQL 16 and Redis 7 run as service containers during CI.

---

## 13. Key Architectural Decisions

| Decision | Rationale |
|---|---|
| **No ORM** | Cornucopia gives compile-time SQL safety without hiding the database |
| **Domain in separate crate** | `packages/catalog` has zero I/O — testable without a database |
| **Timestamps from app clock** | Domain is deterministic, testable, not coupled to DB `now()` |
| **Price in cents** | Avoids float rounding errors in financial calculations |
| **UUIDv7** | Time-ordered, unique, no auto-increment, no sequence |
| **Cache-aside (not write-through)** | Simple, works with any cache outage, cache is disposable |
| **One shared error type** | Unified JSON error format across REST, GraphQL, and compat |
| **Worker separate binary** | Can scale independently; same shared libs as API |
| **host-spawn wrapper** | Lets Flatpak sandbox users develop without installing native tools |
