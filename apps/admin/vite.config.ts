import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { TanStackRouterVite } from '@tanstack/router-plugin/vite'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    // TanStackRouterVite MUST come before the react plugin so the route
    // tree is generated before Vite processes source files that import it.
    TanStackRouterVite({
      routesDirectory: './src/routes',
      generatedRouteTree: './src/routeTree.gen.ts',
    }),
    react(),
  ],
  server: {
    port: 5173,
    proxy: {
      // Proxy all /graphql requests to the Rust API during development.
      // This avoids CORS issues without needing a wildcard CORS policy on the backend.
      '/graphql': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
})
