# Daily Log — Chapter 03: In-Memory Product API
 
**Date:** 2026-06-17
**Chapter:** 03 — In-Memory Product API (Axum + Serde)
 
---
 
## What I Learned Today
 
### Axum Concepts
 
**Extractors** — the way Axum pulls data out of an HTTP request automatically.
Instead of manually reading the request body, Axum does it for you by looking
at your handler's parameters and extracting what's needed:
 
- `Json<T>` — extracts and deserializes a JSON body into a Rust struct
- `Path<T>` — extracts a segment from the URL (e.g. `/products/:id`)
- `State<T>` — extracts shared application state (like `AppState`) injected at startup
**Handlers** — async functions that receive extracted inputs and return a response.
They are thin transport adapters: parse the request, call a domain function,
return the result. No business logic lives inside a handler.
 
**AppState** — a struct shared across all handlers via `Arc`. Holds shared
resources like the config and the in-memory product list.
 
**Routing** — mapping URL + method pairs to handler functions using
`Router::new().route(ROUTE_CONSTANT, get(handler).post(handler))`.
 
### Serde Concepts
 
**`#[derive(Serialize, Deserialize)]`** — macros that auto-generate the code
to convert a Rust struct to/from JSON. No manual parsing needed.
 
**DTOs (Data Transfer Objects)** — separate structs for what comes in over HTTP
(`CreateProductRequest`) and what goes out (`ProductResponse`). They are the
public contract. The internal domain model can change without breaking the API.
 
---
 
## What I Built Today
 
- Multiple API routes: `GET /health`, `GET /products`, `POST /products`
- A `CreateProductRequest` DTO (deserialized from the request body with Serde)
- A `ProductResponse` DTO (serialized back to JSON with Serde)
- Handler functions that convert request DTOs → domain models → response DTOs
- UUIDv7 product IDs generated in Rust at creation time
- `created_at` / `updated_at` timestamps set by the application, not a database
- Route strings moved into constants so no handler hardcodes a path string
- A central `Config` struct loaded once at startup and passed through `AppState`
- Basic error handling using `unwrap()` and `match` on `Option`/`Result`
- Async handler functions (`async fn`) that work correctly inside Axum's runtime
---
 
## New Concepts Highlighted
 
| Concept | What it does |
|---|---|
| `Json<T>` extractor | Deserializes request body into a typed Rust struct |
| `State<T>` extractor | Gives handlers access to shared `AppState` |
| `#[derive(Serialize)]` | Makes a struct serializable to JSON automatically |
| `#[derive(Deserialize)]` | Makes a struct deserializable from JSON automatically |
| Route constants | Prevent string drift across a growing codebase |
| DTO boundary | Decouples HTTP shape from internal domain model |
| Domain function | Business logic that lives outside the handler |
| `async fn` in Axum | Handlers must be async; Tokio runs them concurrently |
 
---
 
## Handler vs Domain Logic — What I Can Now Explain
 
A handler's only job is to translate HTTP into a domain call and back.
The domain function does the real work and knows nothing about HTTP.
 
```
route constant → handler (extracts, converts) → domain function → response DTO
```
 
This separation means the same domain function can later be called from a
GraphQL resolver, a background worker, or a test — without touching the handler.
 
---
 
## Resources I Used
 
- [Axum extractors docs](https://docs.rs/axum/latest/axum/extract/index.html)
- [Serde JSON guide](https://serde.rs/json.html)
- Chapter 03 README and `id-time-contract.md`
