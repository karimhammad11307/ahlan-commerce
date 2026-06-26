# Compatibility PRD — External Product Import Adapter

## Status
Draft — awaiting mentor approval

## Problem statement
Merchants migrating from external platforms (for example Shopify, WooCommerce,
or custom CSV pipelines) send product data in shapes that differ from the native
Ahlan CreateProductRequest format. Without a compatibility layer, merchants must
manually reformat every product before importing. This creates friction and
errors during migration.

## User story
As a merchant migrating from an external platform,
I want to POST a product in my existing payload format to a compatibility endpoint,
So that my products are imported into Ahlan without me reformatting each field.

## Scope
One external payload format is supported in this chapter.
The external format used for this spec is defined below.
Only the product create operation is in scope.

## External payload format (the format being adapted)
```json
{
  "name": "Coffee Mug",
  "slug": "coffee-mug",
  "body_html": "Ceramic mug for daily coffee.",
  "price": 25.00,
  "qty": 12,
  "is_active": true
}
```

Field mapping to native:
- `name` → `title`
- `slug` → `handle`
- `body_html` → `description` (optional — null if absent)
- `price` (float, USD) → `price_cents` (integer, multiply by 100 and round)
- `qty` → `inventory_quantity`
- `is_active` → `published`

## Acceptance criteria

AC-1: A POST to `/api/compat/products` with a valid external payload returns
      HTTP 201 and a product response matching the Chapter 04 API contract.

AC-2: The created product is retrievable via GET /api/products and via the
      GraphQL products query.

AC-3: A payload with a missing `name` field returns HTTP 400 with
      `{"error":{"code":"validation_failed","message":"..."}}`.

AC-4: A payload with a duplicate `slug` returns HTTP 409 with
      `{"error":{"code":"duplicate_product_handle","message":"..."}}`.

AC-5: A payload with `price` as a negative number returns HTTP 400 with
      `{"error":{"code":"validation_failed","message":"..."}}`.

AC-6: A payload with `body_html` absent is accepted; `description` is stored
      as null.

AC-7: The native POST /api/products endpoint continues to work unchanged.
      The compatibility adapter does not touch native handlers or domain logic.

AC-8: Cache invalidation runs after a successful compat create, same as native
      create: `storefront:product-page:{handle}` is deleted from Redis.

## Out of scope
- Bulk import (multiple products in one request)
- Authentication or API key enforcement on the compat endpoint
- Support for more than one external payload format
- Shopify-specific webhook format
- Product update or delete via compat
- GraphQL mutation for compat create
- Any schema change (no new database columns)

## Edge cases
- `price` is sent as an integer (e.g. 25 instead of 25.00): accept, treat as dollars.
- `price` has more than 2 decimal places (e.g. 25.999): round to nearest cent.
- `slug` contains uppercase letters: accept as-is; handle uniqueness check applies normally.
- `body_html` is present but empty string: store as null (treat empty as absent).
- `qty` is 0: accept; inventory_quantity of 0 is valid.
- `is_active` is absent: default to false (unpublished).

## Non-goals
This PRD does not describe the adapter implementation, the module structure,
or the Rust types involved. Those belong in the ADR and plan.
