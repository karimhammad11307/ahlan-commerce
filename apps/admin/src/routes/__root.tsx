import { createRootRoute, Link, Outlet } from '@tanstack/react-router'

export const Route = createRootRoute({
  component: RootLayout,
})

function RootLayout() {
  return (
    <>
      <header>
        <strong>Ahlan Commerce Admin</strong>
      </header>
      <nav>
        <Link to="/products" activeProps={{ style: { fontWeight: 'bold' } }}>
          Products
        </Link>
      </nav>
      <main>
        <Outlet />
      </main>
    </>
  )
}
