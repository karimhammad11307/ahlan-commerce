interface ErrorAlertProps {
  /** Error object from TanStack Query — message displayed without raw internals. */
  error: Error
}

/**
 * Error alert that surfaces the server message to the user.
 *
 * Security note: We display `error.message` which is already the
 * joined list of GraphQL error messages (see GraphQLError constructor).
 * The server must NOT leak SQL errors or stack traces in GraphQL error
 * extensions — that is enforced on the backend by the Chapter 03A error
 * handling contract. This component trusts the server to be correct.
 * React JSX auto-escapes the string, preventing XSS.
 */
export function ErrorAlert({ error }: ErrorAlertProps) {
  return (
    <p role="alert" aria-live="assertive">
      Error: {error.message}
    </p>
  )
}
