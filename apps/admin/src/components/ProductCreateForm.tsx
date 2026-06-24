import { useState } from 'react'
import type { ProductCreateInput } from '../hooks/useProducts'

interface ProductCreateFormProps {
  onSubmit: (input: ProductCreateInput) => void
  isPending: boolean
}

/**
 * Controlled form for creating a product.
 *
 * Ownership contract:
 *   - Form field state (title, handle, etc.) is LOCAL useState — correct,
 *     because these are local UI concerns, not server state.
 *   - The submitted data travels to the server via the useMutation hook
 *     in the parent component. TanStack Query owns the server state result.
 *
 * Input field mapping to GraphQL ProductCreateInput (camelCase):
 *   title              -> String!
 *   handle             -> String!
 *   description        -> String  (optional)
 *   priceCents         -> Int!    (parsed as integer)
 *   inventoryQuantity  -> Int!    (parsed as integer)
 *   published          -> Boolean!
 *
 * Security: all values rendered via JSX; no dangerouslySetInnerHTML.
 * The server validates field types and business rules (e.g. unique handle).
 * parseInt guards priceCents and inventoryQuantity against non-integer input.
 */
export function ProductCreateForm({ onSubmit, isPending }: ProductCreateFormProps) {
  const [title, setTitle] = useState('')
  const [handle, setHandle] = useState('')
  const [description, setDescription] = useState('')
  const [priceCents, setPriceCents] = useState('')
  const [inventoryQuantity, setInventoryQuantity] = useState('')
  const [published, setPublished] = useState(false)

  function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault()

    const parsedPriceCents = parseInt(priceCents, 10)
    const parsedInventory = parseInt(inventoryQuantity, 10)

    // Basic client-side guard: reject obviously invalid integers before
    // sending. The server is the authoritative validator.
    if (isNaN(parsedPriceCents) || isNaN(parsedInventory)) {
      return
    }

    onSubmit({
      title: title.trim(),
      handle: handle.trim(),
      description: description.trim() !== '' ? description.trim() : null,
      priceCents: parsedPriceCents,
      inventoryQuantity: parsedInventory,
      published,
    })
  }

  return (
    <form onSubmit={handleSubmit}>
      <fieldset disabled={isPending}>
        <legend>Create Product</legend>

        <div>
          <label htmlFor="product-title">Title</label>
          <input
            id="product-title"
            type="text"
            required
            value={title}
            onChange={(e) => setTitle(e.target.value)}
          />
        </div>

        <div>
          <label htmlFor="product-handle">Handle</label>
          <input
            id="product-handle"
            type="text"
            required
            value={handle}
            onChange={(e) => setHandle(e.target.value)}
          />
        </div>

        <div>
          <label htmlFor="product-description">Description</label>
          <textarea
            id="product-description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={3}
          />
        </div>

        <div>
          <label htmlFor="product-price-cents">Price (cents)</label>
          <input
            id="product-price-cents"
            type="number"
            required
            min={0}
            step={1}
            value={priceCents}
            onChange={(e) => setPriceCents(e.target.value)}
          />
        </div>

        <div>
          <label htmlFor="product-inventory-quantity">Inventory Quantity</label>
          <input
            id="product-inventory-quantity"
            type="number"
            required
            min={0}
            step={1}
            value={inventoryQuantity}
            onChange={(e) => setInventoryQuantity(e.target.value)}
          />
        </div>

        <div>
          <label htmlFor="product-published">
            <input
              id="product-published"
              type="checkbox"
              checked={published}
              onChange={(e) => setPublished(e.target.checked)}
            />
            {' '}Published
          </label>
        </div>

        <button type="submit" disabled={isPending}>
          {isPending ? 'Creating…' : 'Create Product'}
        </button>
      </fieldset>
    </form>
  )
}
