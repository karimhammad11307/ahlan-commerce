# Operational Behaviors

Ahlan-Commerce is designed to be resilient in production. This document highlights key runtime behaviors to be aware of during operations and outages.

## Redis Outages & Fallbacks

The Cache-Aside pattern in the storefront (`apps/api/src/storefront.rs`) is strictly an optimization, not a hard dependency. 

**If Redis crashes or becomes unreachable:**
1. The `CacheClient` (`apps/api/src/cache.rs`) will attempt to fetch a connection from the `deadpool-redis` pool.
2. The connection attempt will fail.
3. Instead of panicking or returning an HTTP 500 error to the user, the cache logic logs a `WARN` via `tracing` ("redis get failed — cache miss") and gracefully returns `None`.
4. The storefront handler treats this exactly like a standard Cache Miss. It queries PostgreSQL directly, renders the HTML, and returns a successful `200 OK` response to the user.
5. The subsequent async attempt to `cache_set` the HTML will similarly fail, log a `WARN`, and continue.

**Operational Impact:**
During a Redis outage, your Postgres database will experience significantly higher load as 100% of storefront requests will fall back to it. However, the platform will remain functionally online for customers.

## Cache Invalidation Workflow

To ensure product data remains fresh, cache invalidation is proactively managed during write operations:
- When a product is created (`POST /api/products`), the cache key `storefront:product-page:{handle}` is cleared. (This handles scenarios where a product is re-created with a previously used handle).
- When a product is updated or published/unpublished (`PATCH /api/products/{id}/publication`), the cache key is explicitly deleted.
- If an admin forgets to clear a cache key, or a background process updates a product manually, the Storefront keys have a hard **300-second TTL** (Time To Live). They will automatically expire and self-heal after 5 minutes.
