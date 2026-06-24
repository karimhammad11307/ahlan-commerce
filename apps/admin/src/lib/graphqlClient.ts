/**
 * Thin GraphQL transport wrapper using native browser fetch.
 *
 * Design contract (from client-choice.md):
 *   - TanStack Query owns server state (caching, loading, errors, retries).
 *   - This module is responsible ONLY for the HTTP transport and GraphQL
 *     protocol formatting (query string + variables -> POST -> data or errors).
 *   - Do NOT use Apollo Client or any heavy third-party GraphQL client.
 *
 * Security notes:
 *   - The endpoint is read from VITE_GRAPHQL_URL env var (set in .env).
 *     Falls back to '/graphql' (relative URL) which Vite proxies to
 *     http://localhost:3000/graphql in dev. The proxy avoids a wildcard CORS
 *     policy on the backend.
 *   - No auth tokens are stored in localStorage or sessionStorage.
 *     TODO(security): When auth is introduced, pass the Bearer token from a
 *     secure HttpOnly cookie via the BFF pattern — not from client-side JS.
 *   - GraphQL error messages from the server are surfaced as-is. The server
 *     must not leak SQL errors or stack traces in GraphQL error extensions
 *     (see Chapter 03A error handling contract). The frontend displays
 *     whatever the server returns without further transformation.
 */

/** Shape of a GraphQL error object from the server. */
export interface GraphQLErrorItem {
  message: string
  extensions?: Record<string, unknown>
}

/** Thrown when the server responds with a `errors` array. */
export class GraphQLError extends Error {
  public readonly errors: GraphQLErrorItem[]

  constructor(errors: GraphQLErrorItem[]) {
    // Join messages for the base Error.message — never log raw error objects
    // that may contain structured server data (secure logging practice).
    super(errors.map((e) => e.message).join('; '))
    this.name = 'GraphQLError'
    this.errors = errors
  }
}

const GRAPHQL_URL = import.meta.env['VITE_GRAPHQL_URL'] ?? '/graphql'

/**
 * Execute a GraphQL query or mutation against the API.
 *
 * @param query     The GraphQL document string (query or mutation).
 * @param variables Optional variables object matching the document signature.
 * @returns         The typed `data` payload from the GraphQL response.
 * @throws          `GraphQLError`  when the server returns `errors`.
 * @throws          `Error`         when the HTTP request itself fails.
 */
export async function gqlFetch<TData, TVariables = Record<string, unknown>>(
  query: string,
  variables?: TVariables,
): Promise<TData> {
  const response = await fetch(GRAPHQL_URL, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      // TODO(security): Add 'Authorization': `Bearer ${token}` here once the
      // BFF auth layer is introduced. Token must NOT come from localStorage.
    },
    body: JSON.stringify({ query, variables }),
  })

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }

  const json = (await response.json()) as {
    data?: TData
    errors?: GraphQLErrorItem[]
  }

  // Surface GraphQL application-layer errors distinctly from HTTP errors.
  if (json.errors && json.errors.length > 0) {
    throw new GraphQLError(json.errors)
  }

  // The GraphQL spec guarantees `data` is present when `errors` is absent.
  return json.data as TData
}
