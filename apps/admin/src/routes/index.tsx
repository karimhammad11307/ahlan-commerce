import { createFileRoute, redirect } from '@tanstack/react-router'

// Index route: redirect / -> /products immediately.
// TanStack Router owns this navigation — no component state involved.
export const Route = createFileRoute('/')({
  beforeLoad: () => {
    throw redirect({ to: '/products' })
  },
})
