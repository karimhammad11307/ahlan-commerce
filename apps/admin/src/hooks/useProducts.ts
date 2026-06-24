/**
 * TanStack Query hooks for the product domain.
 *
 * Ownership contract (README §What Engineers Should Notice):
 *   - TanStack Query is the SINGLE source of truth for server state.
 *   - Components MUST NOT copy query results into local useState.
 *   - Form field state (local UI) lives in useState inside components — that
 *     is appropriate because it is not server state.
 *
 * The GraphQL field names here are camelCase because async-graphql
 * (the Rust library) auto-converts snake_case struct fields to camelCase
 * in the generated schema (e.g. price_cents -> priceCents).
 * This is confirmed in apps/api/src/graphql/types.rs and schema.graphql.
 */

import {
  useQuery,
  useMutation,
  useQueryClient,
  type UseQueryResult,
  type UseMutationResult,
} from '@tanstack/react-query'
import { gqlFetch, GraphQLError } from '../lib/graphqlClient'

// ---------------------------------------------------------------------------
// Domain types — mirror the Chapter 09 GraphQL schema exactly (camelCase)
// ---------------------------------------------------------------------------

/** ISO-8601 UTC timestamp string as returned by the DateTime scalar. */
type ISODateString = string

export interface Product {
  id: string
  title: string
  handle: string
  description: string | null
  priceCents: number
  inventoryQuantity: number
  published: boolean
  publishedAt: ISODateString | null
  createdAt: ISODateString
  updatedAt: ISODateString
}

export interface ProductCreateInput {
  title: string
  handle: string
  description: string | null
  priceCents: number
  inventoryQuantity: number
  published: boolean
}

// ---------------------------------------------------------------------------
// Cache key — a single string tuple shared across query and invalidation.
// ---------------------------------------------------------------------------

export const PRODUCTS_QUERY_KEY = ['products'] as const

// ---------------------------------------------------------------------------
// GraphQL documents
// ---------------------------------------------------------------------------

const PRODUCTS_QUERY = /* GraphQL */ `
  query Products {
    products {
      id
      title
      handle
      description
      priceCents
      inventoryQuantity
      published
      publishedAt
      createdAt
      updatedAt
    }
  }
`

const PRODUCT_CREATE_MUTATION = /* GraphQL */ `
  mutation ProductCreate($input: ProductCreateInput!) {
    productCreate(input: $input) {
      id
      title
      handle
      description
      priceCents
      inventoryQuantity
      published
      publishedAt
      createdAt
      updatedAt
    }
  }
`

// ---------------------------------------------------------------------------
// Response envelope types for the gqlFetch generic parameter
// ---------------------------------------------------------------------------

interface ProductsQueryResponse {
  products: Product[]
}

interface ProductCreateMutationResponse {
  productCreate: Product
}

// ---------------------------------------------------------------------------
// Hooks
// ---------------------------------------------------------------------------

/**
 * Fetches the full product list.
 *
 * Exposes TanStack Query's `isLoading`, `isError`, `error`, `data`, and
 * `isSuccess` states directly — no server data is duplicated into component
 * state.
 */
export function useProducts(): UseQueryResult<Product[], Error> {
  return useQuery<Product[], Error>({
    queryKey: PRODUCTS_QUERY_KEY,
    queryFn: async () => {
      const response = await gqlFetch<ProductsQueryResponse>(PRODUCTS_QUERY)
      return response.products
    },
  })
}

/**
 * Creates a product and automatically invalidates the product list cache on
 * success, triggering a background refetch without a full page reload.
 *
 * Variable shape matches ProductCreateInput which aligns with the
 * Chapter 04 persisted product contract (see apps/api/src/graphql/types.rs).
 */
export function useCreateProduct(): UseMutationResult<
  Product,
  Error,
  ProductCreateInput
> {
  const queryClient = useQueryClient()

  return useMutation<Product, Error, ProductCreateInput>({
    mutationFn: async (input: ProductCreateInput) => {
      const response = await gqlFetch<
        ProductCreateMutationResponse,
        { input: ProductCreateInput }
      >(PRODUCT_CREATE_MUTATION, { input })
      return response.productCreate
    },
    onSuccess: () => {
      // Invalidate the product list cache key so TanStack Query schedules a
      // background refetch. Components subscribed to PRODUCTS_QUERY_KEY
      // will re-render with fresh data without a page reload.
      void queryClient.invalidateQueries({ queryKey: PRODUCTS_QUERY_KEY })
    },
    onError: (error: Error) => {
      // Log a safe, non-sensitive status message only. Do NOT log the full
      // error object or any user-submitted form values (secure logging rule).
      if (error instanceof GraphQLError) {
        console.error('Product creation failed (GraphQL):', error.message)
      } else {
        console.error('Product creation failed (network):', error.message)
      }
    },
  })
}
