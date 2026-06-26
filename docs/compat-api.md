# Compatibility API Documentation

## Route
`POST /api/compat/products`

## Description
Creates a new product using an external payload format (e.g., from an external eCommerce platform). This endpoint acts as an adapter, translating the external format into the native `Ahlan-Commerce` product format.

## Request Body Example
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

### Field Mapping

| External Field | Type | Native Field | Rule / Transformation |
|---|---|---|---|
| `name` | String | `title` | Must not be empty. |
| `slug` | String | `handle` | Must not be empty. Must be globally unique. |
| `body_html` | String (Optional) | `description` | Empty strings or whitespace-only are normalized to `null`. |
| `price` | Float (USD) | `price_cents` | Multiplied by 100 and rounded to the nearest integer. Must be >= 0. |
| `qty` | Integer | `inventory_quantity` | Maps directly. |
| `is_active` | Boolean (Optional) | `published` | If omitted, defaults to `false`. |

## Responses

### 201 Created
Returned when the product is successfully created.

```json
{
  "product": {
    "id": "01940984-c8c0-7cf1-9b16-5bcab7967b02",
    "title": "Coffee Mug",
    "handle": "coffee-mug",
    "description": "Ceramic mug for daily coffee.",
    "price_cents": 2500,
    "inventory_quantity": 12,
    "published": true,
    "published_at": "2026-06-25T12:00:00Z",
    "created_at": "2026-06-25T12:00:00Z",
    "updated_at": "2026-06-25T12:00:00Z"
  }
}
```

### 400 Bad Request
Returned when validation fails (e.g., empty `name`, empty `slug`, negative `price`).

```json
{
  "error": {
    "code": "validation_failed",
    "message": "price cannot be negative"
  }
}
```

### 409 Conflict
Returned when a product with the same `slug` (handle) already exists.

```json
{
  "error": {
    "code": "duplicate_product_handle",
    "message": "Handle already exists"
  }
}
```

### 500 Internal Server Error
Returned when an unexpected database or server issue occurs.

```json
{
  "error": {
    "code": "internal_error",
    "message": "Internal server error"
  }
}
```
