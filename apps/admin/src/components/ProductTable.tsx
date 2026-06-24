import type { Product } from '../hooks/useProducts'

interface ProductTableProps {
  products: Product[]
}

/**
 * Renders the product list in a semantic <table>.
 *
 * Column labels are human-readable strings. The underlying data fields are
 * the camelCase names returned by the Chapter 09 GraphQL schema
 * (priceCents, inventoryQuantity, publishedAt, createdAt, updatedAt).
 *
 * All values are rendered via React JSX interpolation — no dangerouslySetInnerHTML,
 * no innerHTML, no raw HTML strings. React auto-escapes every value.
 */
export function ProductTable({ products }: ProductTableProps) {
  return (
    <table>
      <caption>Product List</caption>
      <thead>
        <tr>
          <th scope="col">Title</th>
          <th scope="col">Handle</th>
          <th scope="col">Description</th>
          <th scope="col">Price (cents)</th>
          <th scope="col">Inventory</th>
          <th scope="col">Published</th>
          <th scope="col">Published At</th>
          <th scope="col">Created At</th>
          <th scope="col">Updated At</th>
        </tr>
      </thead>
      <tbody>
        {products.map((product) => (
          <tr key={product.id}>
            <td>{product.title}</td>
            <td>
              <code>{product.handle}</code>
            </td>
            <td>{product.description ?? '—'}</td>
            <td>{product.priceCents}</td>
            <td>{product.inventoryQuantity}</td>
            <td>{product.published ? 'Yes' : 'No'}</td>
            <td>{product.publishedAt ?? '—'}</td>
            <td>{product.createdAt}</td>
            <td>{product.updatedAt}</td>
          </tr>
        ))}
      </tbody>
    </table>
  )
}
