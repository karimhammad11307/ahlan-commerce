# Product Scenarios

## PRD-PROD-001 - Valid Product Create

Version: 1 - 2026-06-22

Intent:
Ensure the system correctly persists new valid products to the database and returns the generated state.

Given:
- An authenticated user (currently implicit/bypassed).
- A valid JSON payload containing title, handle, description, price, inventory quantity, and published status.

When:
- The user issues a `POST /api/products` request with the payload.

Then:
- The system persists the product.
- The system returns a `201 Created` status code.
- The response body contains the persisted product, including the generated `id`, `created_at`, and `updated_at`.

Verification:
Automated by: test_api_valid_product_create

Review:
Status: Pending
Reviewed version: none
Reviewed by: none
Reviewed at: none

---

## PRD-PROD-002 - Duplicate Handle Rejected

Version: 1 - 2026-06-22

Intent:
Prevent the creation of products with non-unique handles, as handles act as unique identifiers for URLs.

Given:
- An existing product in the database with the handle `test-handle`.
- A valid JSON payload for a new product, also using the handle `test-handle`.

When:
- The user issues a `POST /api/products` request with the duplicate payload.

Then:
- The system rejects the creation.
- The database remains unchanged.
- The system returns an appropriate error (e.g. `400 Bad Request` or `409 Conflict`) indicating a duplicate handle.

Verification:
Automated by: test_api_duplicate_handle_rejected

Review:
Status: Pending
Reviewed version: none
Reviewed by: none
Reviewed at: none

---

## PRD-PROD-003 - List Empty Products

Version: 1 - 2026-06-22

Intent:
Ensure the API safely handles listing products when the catalog is completely empty, rather than throwing an error.

Given:
- A clean database with zero products.

When:
- The user issues a `GET /api/products` request.

Then:
- The system returns a `200 OK` status code.
- The response body contains `{"products": []}`.

Verification:
Automated by: test_api_list_empty_products

Review:
Status: Pending
Reviewed version: none
Reviewed by: none
Reviewed at: none

---

## PRD-PROD-004 - List Persisted Products

Version: 1 - 2026-06-22

Intent:
Ensure the API successfully retrieves and serializes all persisted products from the database.

Given:
- A database containing at least one valid product.

When:
- The user issues a `GET /api/products` request.

Then:
- The system returns a `200 OK` status code.
- The response body contains a `products` JSON array populated with the existing products, including their full attributes.

Verification:
Automated by: test_api_list_persisted_products

Review:
Status: Pending
Reviewed version: none
Reviewed by: none
Reviewed at: none

---

## PRD-PROD-005 - Invalid Create Input Rejected

Version: 1 - 2026-06-22

Intent:
Ensure that business domain validation rules catch invalid properties before database execution.

Given:
- A JSON payload containing structurally invalid data (e.g., negative price, empty title, or invalid handle format).

When:
- The user issues a `POST /api/products` request or invokes the domain logic.

Then:
- The domain validation fails.
- The system returns an error.
- The database is completely untouched.

Verification:
Automated by: test_domain_invalid_create_input_rejected

Review:
Status: Pending
Reviewed version: none
Reviewed by: none
Reviewed at: none
