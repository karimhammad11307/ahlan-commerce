# Product Requirements Document (PRD): Products Management

## 1. User Story
As a store administrator,
I want to create, manage, and list products,
So that customers can view the store's inventory and purchase items.

## 2. Acceptance Criteria

### 2.1 Valid Create
- **Scenario:** The admin submits a valid product creation payload.
- **Expected Outcome:** The system successfully persists the product in the database, generates a unique UUID, assigns the current timestamp to `created_at` and `updated_at`, and returns a `201 Created` status code with the serialized product object.

### 2.2 Duplicate Handle
- **Scenario:** The admin attempts to create a product using a `handle` that is already assigned to another existing product.
- **Expected Outcome:** The system rejects the creation attempt and returns an appropriate error indicating a duplicate handle. The database remains unchanged.

### 2.3 List Empty Products
- **Scenario:** A user requests to list products, but the database currently contains zero products.
- **Expected Outcome:** The system returns a `200 OK` status with an empty array `[]` for the products list. It should not fail or return an error.

### 2.4 List Persisted Products
- **Scenario:** A user requests to list products, and the database contains one or more products.
- **Expected Outcome:** The system returns a `200 OK` status along with a JSON array containing the correctly serialized products, including all their fields (ID, title, handle, etc.). The products should be sorted properly.

### 2.5 Invalid Create Input Rejected
- **Scenario:** The admin submits a product creation payload with invalid data (e.g., negative price, empty title, invalid handle format).
- **Expected Outcome:** The system rejects the request before hitting the database, returning a `400 Bad Request` or validation error response. The database remains unchanged.
