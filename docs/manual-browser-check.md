# Manual Browser Verification — Chapter 10 Admin UI

## Automation Blocker Note

End-to-end browser automation (Playwright or Cypress) was not set up in this
chapter. The Chapter 10 README instructs implementors to document a manual
check if automation is blocked. The blockers are:

1. **No E2E framework scaffolded** — Adding Playwright requires a separate
   `npm init playwright@latest` step that falls outside the Chapter 10 scope
   per the README rule: *"Do not add storefront assets, worker components, or
   structural design polish yet."* A browser test runner is infrastructure, not
   a core logic slice.
2. **No CI environment defined** — There is no `.github/workflows` or CI runner
   configured in this repository yet, so headless browser tests have no
   execution target.

A mentor may add Playwright as a follow-up task after the manual steps below
pass. The `docs/manual-browser-check.md` file (this document) serves as the
repeatable verification record until then.

---

## Prerequisites

Ensure the following services are running before starting:

| Service     | Command                       | Port  |
|-------------|-------------------------------|-------|
| PostgreSQL  | `make db-start`               | 5432  |
| Migrations  | `make db-migrate`             | —     |
| Rust API    | `make run-api`                | 3000  |
| Admin UI    | `cd apps/admin && npm run dev`| 5173  |

Or use `mprocs` to start API + Admin together:

```bash
mprocs
```

---

## Step-by-Step Verification

### Step 1 — Verify the API is healthy

```bash
curl -s http://localhost:3000/health
```

**Expected:** `200 OK` with a JSON or text body confirming the API is alive.

---

### Step 2 — Open the Admin UI

Navigate to:

```
http://localhost:5173/products
```

**Expected:**
- The browser redirects from `/` to `/products` automatically.
- Page title: **Ahlan Commerce — Admin**
- Header shows: **Ahlan Commerce Admin**
- Navigation shows: **Products** link (bold/active).
- Heading `<h1>`: **Products**
- Content area shows either:
  - `"No products yet. Use the form below to create the first one."` (empty state), OR
  - A populated `<table>` if products already exist in the DB.
- Below a `<hr>`, a **Create Product** form is rendered.

---

### Step 3 — Verify GraphQL list query in Network tab

1. Open **DevTools → Network** tab.
2. Filter by `graphql`.
3. Reload the page (`Ctrl+R`).

**Expected request:**
- **Method:** `POST`
- **URL:** `/graphql` (proxied to `http://localhost:3000/graphql`)
- **Request body:**
  ```json
  {
    "query": "query Products { products { id title handle description priceCents inventoryQuantity published publishedAt createdAt updatedAt } }",
    "variables": undefined
  }
  ```
- **Response:**
  ```json
  {
    "data": {
      "products": [ /* array of product objects */ ]
    }
  }
  ```

---

### Step 4 — Create a product via the form

Fill in the form fields with these exact test values:

| Field                 | Value            |
|-----------------------|------------------|
| Title                 | `Test Widget`    |
| Handle                | `test-widget`    |
| Description           | `A test product` |
| Price (cents)         | `999`            |
| Inventory Quantity    | `50`             |
| Published (checkbox)  | ✅ checked       |

Click **Create Product**.

**Expected immediately:**
- The button changes to `"Creating…"` while the mutation is pending.
- The form is disabled (grayed out) during the request.

**Expected after success:**
- Success message appears: `"Product created successfully. The list above has been updated."`
- The product table above **updates without a full page reload** — TanStack Query invalidates the `['products']` cache key and fires a background refetch.
- The new product row `Test Widget` appears in the table.

---

### Step 5 — Verify mutation request in Network tab

In **DevTools → Network**, check the second `POST /graphql` request (the mutation):

**Expected request body:**
```json
{
  "query": "mutation ProductCreate($input: ProductCreateInput!) { productCreate(input: $input) { id title handle description priceCents inventoryQuantity published publishedAt createdAt updatedAt } }",
  "variables": {
    "input": {
      "title": "Test Widget",
      "handle": "test-widget",
      "description": "A test product",
      "priceCents": 999,
      "inventoryQuantity": 50,
      "published": true
    }
  }
}
```

**Expected response:**
```json
{
  "data": {
    "productCreate": {
      "id": "<uuid>",
      "title": "Test Widget",
      "handle": "test-widget",
      "description": "A test product",
      "priceCents": 999,
      "inventoryQuantity": 50,
      "published": true,
      "publishedAt": "<ISO timestamp>",
      "createdAt": "<ISO timestamp>",
      "updatedAt": "<ISO timestamp>"
    }
  }
}
```

---

### Step 6 — Verify database persistence

In a separate terminal, confirm the product was persisted:

```bash
psql postgresql://ahlan:ahlan_dev@localhost:5432/ahlan_commerce \
  -c "SELECT id, title, handle, price_cents, inventory_quantity, published FROM products WHERE handle = 'test-widget';"
```

**Expected:** one row with the values entered in Step 4.

Alternatively, query the REST API:

```bash
curl -s http://localhost:3000/products | jq '.[] | select(.handle == "test-widget")'
```

---

### Step 7 — Verify duplicate handle rejection

Try submitting the form again with the same handle `test-widget`.

**Expected:**
- An error alert appears: `Error: Handle already exists` (or similar message from the server).
- The product list does **not** change.
- No duplicate row is added to the table.

---

### Step 8 — Verify empty state on fresh DB

To test the empty state:

```bash
psql postgresql://ahlan:ahlan_dev@localhost:5432/ahlan_commerce \
  -c "DELETE FROM products WHERE handle = 'test-widget';"
```

Reload `http://localhost:5173/products`.

**Expected:** `"No products yet. Use the form below to create the first one."`

---

## Trace: Browser → DB

```
Browser (React)
  └─ useProducts() / useCreateProduct()        [TanStack Query — server state]
       └─ gqlFetch()                           [thin native fetch wrapper]
            └─ POST http://localhost:3000/graphql
                 └─ Axum /graphql route
                      └─ async-graphql schema
                           └─ QueryRoot::products / MutationRoot::product_create
                                └─ db::products::list_products / create_product
                                     └─ PostgreSQL (ahlan_commerce.products)
```

## Why TanStack Query Owns Server State

- `useProducts()` returns `isLoading`, `isError`, `data` — components never
  call `useState` to store the product list.
- `useCreateProduct()` calls `queryClient.invalidateQueries(['products'])` in
  `onSuccess`. This triggers a background refetch. The UI updates automatically.
- If a component re-mounts or the window regains focus, TanStack Query
  re-fetches stale data. No manual `setState` is required.
- Apollo Client is deliberately excluded per `client-choice.md` — using two
  server-state cache owners in one exercise adds cognitive load without
  teaching benefit.
