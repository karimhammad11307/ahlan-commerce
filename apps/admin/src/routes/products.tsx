import { createFileRoute } from '@tanstack/react-router'
import { useProducts, useCreateProduct } from '../hooks/useProducts'
import type { ProductCreateInput } from '../hooks/useProducts'
import { ProductTable } from '../components/ProductTable'
import { ProductCreateForm } from '../components/ProductCreateForm'
import { LoadingSpinner } from '../components/LoadingSpinner'
import { EmptyState } from '../components/EmptyState'
import { ErrorAlert } from '../components/ErrorAlert'

// TanStack Router owns the /products URL and screen state.
// TanStack Query (via useProducts / useCreateProduct) owns the server state.
export const Route = createFileRoute('/products')({
  component: ProductsPage,
})

function ProductsPage() {
  // Server state from TanStack Query — NOT duplicated into local useState.
  const { data: products, isLoading, isError, error } = useProducts()
  const createProduct = useCreateProduct()

  function handleCreate(input: ProductCreateInput) {
    createProduct.mutate(input)
  }

  return (
    <section>
      <h1>Products</h1>

      {/* ── List section ──────────────────────────────────────────────── */}
      <div id="products-content">
        {isLoading && <LoadingSpinner />}

        {isError && error && <ErrorAlert error={error} />}

        {!isLoading && !isError && products && products.length === 0 && (
          <EmptyState />
        )}

        {!isLoading && !isError && products && products.length > 0 && (
          <ProductTable products={products} />
        )}
      </div>

      <hr />

      {/* ── Create section ────────────────────────────────────────────── */}
      <div id="product-create-section">
        {createProduct.isError && createProduct.error && (
          <ErrorAlert error={createProduct.error} />
        )}

        {createProduct.isSuccess && (
          <p role="status" aria-live="polite">
            Product created successfully. The list above has been updated.
          </p>
        )}

        <ProductCreateForm
          onSubmit={handleCreate}
          isPending={createProduct.isPending}
        />
      </div>
    </section>
  )
}
