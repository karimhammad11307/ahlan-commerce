# Deployment preparation

## Required environment variables

| Variable | Required | Default | Service | Description |
|---|---|---|---|---|
| DATABASE_URL | yes | none | API, Worker | PostgreSQL connection string |
| REDIS_URL | no | redis://127.0.0.1:6379 | API, Worker | Redis connection string |
| API_BIND_ADDR | no | 0.0.0.0:3000 | API | Listen address |
| ADMIN_PUBLIC_API_URL | yes at build | none | Admin (build time) | API URL baked into admin bundle |

## Missing var behavior

If DATABASE_URL is missing, the API exits immediately at startup with:
  thread 'main' panicked at 'DATABASE_URL must be set — see .env.example'

No request is ever served with a missing DATABASE_URL.

## Build commands

### API
```
cargo build --release --bin api
```
Output: `target/release/api`
This is the binary that both the API service and worker service run.

### Admin frontend
```
cd apps/admin && npm ci && npm run build
```
Output: `apps/admin/dist/`
Static files. Served by a static file server (nginx or Coolify static deploy).
No Node.js process at runtime.

## Start commands

### API service
```
./target/release/api
```
Reads DATABASE_URL, REDIS_URL, API_BIND_ADDR from environment.
Listens on API_BIND_ADDR.

### Worker service
```
./target/release/api worker
```
Same binary, different subcommand. Reads same env vars.
Does not listen on HTTP — processes background jobs only.

## Migration command

Run BEFORE starting or restarting the API service:
```
atlas migrate apply --env production
```
This applies any pending migrations to the production PostgreSQL database.
Never restart the API after a schema change without running this first.

## Health check

URL: GET /health
Expected response: 200 OK with body {"status":"ok"}
Coolify should poll this URL after deploying to confirm the service is up.

## Independent restart rules

| Service | Can restart independently? | Notes |
|---|---|---|
| Redis | Yes | Cache is ephemeral. TTL handles staleness. |
| API | Yes, after migrations | Must run atlas migrate apply if schema changed |
| Worker | Yes | Stateless. Safe to restart anytime. |
| PostgreSQL | With care | Is the source of truth. Coordinate with team. |
| Admin | Yes | Static files. No state. |

## Docker build

### Build the API image
```
docker build -f apps/api/Dockerfile -t ahlan-api:latest .
```
Run from the project root (so Cargo.toml workspace is accessible).

### Run locally to test
```
docker run \
  -e DATABASE_URL=postgres://ahlan:ahlan_dev@host.docker.internal:5432/ahlan_commerce \
  -e REDIS_URL=redis://host.docker.internal:6379 \
  -e API_BIND_ADDR=0.0.0.0:3000 \
  -p 3000:3000 \
  ahlan-api:latest
```

### Run the worker from the same image
```
docker run \
  -e DATABASE_URL=postgres://ahlan:ahlan_dev@host.docker.internal:5432/ahlan_commerce \
  -e REDIS_URL=redis://host.docker.internal:6379 \
  ahlan-api:latest \
  ./api worker
```

## Frontend build config

Framework: Vite + React + TypeScript
Build output: apps/admin/dist/
Build command: npm ci && npm run build
Runtime: static file serving only — no Node.js process at runtime
The ADMIN_PUBLIC_API_URL env var must be set at BUILD TIME (baked into the JS bundle).

In apps/admin/.env.production:
  VITE_API_URL=https://api.your-domain.com

In apps/admin/vite.config.ts:
  define: { 'import.meta.env.VITE_API_URL': JSON.stringify(process.env.VITE_API_URL) }

## Runtime contract summary

| Service | Image/source | Start | Health check | Needs migration |
|---|---|---|---|---|
| API | apps/api/Dockerfile | ./api | GET /health → 200 | Yes — before start |
| Worker | apps/api/Dockerfile | ./api worker | none (background) | No |
| Admin | apps/admin/dist/ | static server | HTTP 200 on / | No |
| PostgreSQL | managed/docker | docker service | pg_isready | N/A |
| Redis | managed/docker | docker service | redis-cli ping | No |
