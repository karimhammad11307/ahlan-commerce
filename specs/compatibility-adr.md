# Compatibility ADR — Adapter Pattern for External Product Import

## Status
Approved

## Context
The compatibility PRD (specs/compatibility-prd.md) requires accepting an external
product payload format. A decision is needed on where and how the format
translation happens without introducing external format knowledge into native
handlers or domain logic.

## Decision
Use a thin compatibility adapter in a dedicated `compat` module.

The adapter:
- Accepts the external payload as its own deserialized struct (`ExternalProductPayload`)
- Converts it to the native `catalog::ProductCreate` domain struct
- Calls the existing `catalog::create_product()` and `dal::insert_product()`
- Returns a native `ProductResponse` DTO

The compat handler is a thin Axum handler, identical in shape to the native
create handler, but pointing at the adapter instead of reading native DTOs.

The compat route `/api/compat/products` is added to the existing Axum router.
No existing routes, handlers, domain functions, or DAL functions are modified.

## Rejected alternatives

### Public API clone
Duplicate the entire native product create endpoint under a compat namespace
and teach it to parse both native and external shapes.

Rejected because: every future change to the native API requires a matching
change to the clone. Maintenance cost doubles. The two surfaces drift.

### Raw passthrough
Accept the external payload and pass it directly to native handlers without
transformation, relying on the native handler to be flexible about field names.

Rejected because: native handlers depend on `CreateProductRequest` which is the
public API contract. Changing it to accept external shapes breaks the native DTO
boundary and the GraphQL input type.

### Native handlers that understand external shapes
Add conditional logic inside `create_product_handler` to detect whether the
incoming payload is native or external and branch accordingly.

Rejected because: handlers are transport adapters only. They must not contain
format-detection logic. This violates the Chapter 03 handler/domain boundary.

### Separate domain function for compat
Write a new `catalog::create_product_from_external()` that accepts the external
struct directly.

Rejected because: the domain function already exists and works correctly.
The adapter pattern reuses it without modification. A second domain function
for the same operation duplicates behavior.

## Consequences
- A new `src/compat/` module is introduced. It is self-contained.
- `src/compat/mod.rs` and `src/compat/product_adapter.rs` are new files.
- `src/main.rs` gains one new route: `POST /api/compat/products`.
- `src/routes.rs` gains one new constant: `COMPAT_PRODUCTS`.
- `src/handlers.rs` or a new `src/compat/handler.rs` gains one new handler.
- No existing files are structurally changed. Domain and DAL are untouched.
- Cache invalidation (`storefront:product-page:{handle}`) runs after compat
  create, same as native create, because both call the same DAL path.
- Error mapping for the compat handler uses the existing `AppError` enum.
  No new error types are introduced.
- Future external formats require only a new adapter function and a new route.
  All existing code remains unchanged.
