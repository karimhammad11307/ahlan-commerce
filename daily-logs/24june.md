# Daily Log: June 24th

## Progress & Tasks Completed
Today was a massive day for performance and operational maturity in the Ahlan-Commerce project. We successfully completed three major chapters:

### **Chapter 12: Cache Layer Infrastructure**
- **Tasks**: Added the foundational Redis caching infrastructure (`CacheClient`) to the Axum API. Implemented methods to `cache_get`, `cache_set`, and `cache_delete`.
- **Implementation**: We used the `redis` crate to establish multiplexed asynchronous connections to our local Redis server. We injected the `CacheClient` securely into the `AppState` so all endpoints can interact with Redis safely.
- **Obstacles**: The biggest hurdle was ensuring the entire platform doesn't crash if Redis goes offline. We implemented safe fallbacks in `cache_get` and `cache_set` that catch connection errors, log a `WARN` trace, and return `None` rather than panicking.

### **Chapter 13: Simple Storefront Rendering**
- **Tasks**: Built a blazing-fast, server-rendered storefront read path utilizing the Cache-Aside Pattern.
- **Implementation**: We created the `GET /products/{handle}` route. On a cache miss, the system fetches product data from PostgreSQL via Cornucopia, verifies the `published == true` business policy, dynamically renders the HTML payload, and returns it to the user while asynchronously storing it in Redis with a 300-second TTL.
- **Obstacles**: Coordinating cache invalidation during write operations was tricky. We ensured that whenever a product is updated or its publication status changes (e.g., via `PATCH /api/products/{id}/publication`), the cache key is explicitly deleted to prevent serving stale data. Automated tests simulating Redis outages were written to guarantee the fallback logic worked perfectly.

### **Chapter 14: Generated And Written Docs**
- **Tasks**: Bridged the gap between machine-generated truth (OpenAPI/GraphQL SDL) and human-written context.
- **Implementation**: We removed loosely typed `json!({})` macros and replaced them with strict DTO structs (`ListProductsResponse`, `SingleProductResponse`, etc.) annotated with `#[utoipa::ToSchema]`. We added the beautiful Scalar API Explorer UI at `GET /docs/scalar`. Finally, we created a background script (`apps/api/src/bin/generate_docs.rs`) and a CI validation loop (`make docs-api-check`) to ensure docs never drift from the codebase.
- **Obstacles**: The primary challenge was properly integrating the `utoipa-scalar` router and managing complex generic traits without causing compilation errors.

---

##  What I Learned Today

### Redis & Caching Architecture
- Caching isn't just about speed; it's about defining architectural boundaries. The cache sits strictly around the *rendered* output, meaning we bypass both the database and the rendering engine entirely on a cache hit.
- A cache is an optimization, not a hard dependency. Building explicit fallbacks for Redis outages makes the system deeply resilient.

### Tracing the "Silent Bugs" (The Cloudflare Hyper Bug)
I read an incredible post-mortem from Cloudflare about how a silent bug caused truncated responses without triggering any server-side errors. This taught me a profound lesson for our architecture:

> **"A 200 OK with no logged error is not proof that a request succeeded."**

In our project, after adding the cache layer in Chapter 12, we could theoretically encounter a situation where a product response is partially written, the cache stores that broken HTML, and every subsequent request serves a corrupt cached value — all while the Axum logs show a perfect `200 OK`. 

**Takeaway**: We must always validate response completeness in tests, not just status codes. Our `TraceLayer` from `tower-http` gives us latency and status, but it does not give us the response body size. Adding a `Content-Length` check or logging response sizes in our handlers would give us an early signal if something like this ever happened in our API. We'd see "expected 3MB, received 200KB" instead of just a naive "200 OK."

---

## 🔗 Reading & Resources

Here are the links to what I read and watched today:

- **Article**: [Cloudflare Hyper Bug Post-Mortem](https://blog.cloudflare.com/hyper-bug/) - A deep dive into how silent truncation bugs occur and how to trace them.
- **Video**: [Redis Crash Course](https://youtu.be/z_NbVtbgBJw?si=FGZ3WiDrX6kjRegi) - Essential concepts on Redis caching, TTLs, and data structures.
