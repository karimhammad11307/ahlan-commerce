# Deployment

## Status

CI is configured and passing on GitHub Actions.
Production deployment target: Coolify on a VPS.
Current simulation: docker-compose.prod.yml runs the full stack locally.

## Service topology

| Service    | Image / source                  | Port (prod) | Health check           |
|------------|--------------------------------|-------------|------------------------|
| API        | apps/api/Dockerfile            | 3000        | GET /health → 200      |
| Worker     | apps/api/Dockerfile (./api worker) | none   | visible in logs        |
| Admin      | apps/admin/Dockerfile.admin    | 80          | HTTP 200 on /          |
| PostgreSQL | postgres:16                    | 5432        | pg_isready             |
| Redis      | redis:7                        | 6379        | redis-cli ping         |

## Environment variables

| Variable           | Services         | Required | Notes                          |
|--------------------|-----------------|----------|--------------------------------|
| DATABASE_URL       | API, Worker     | yes      | Full postgres connection string |
| REDIS_URL          | API, Worker     | yes      | Redis connection string        |
| API_BIND_ADDR      | API             | no       | Default: 0.0.0.0:3000         |
| VITE_API_URL       | Admin (build)   | yes      | Baked into JS bundle at build  |

No secret values are committed to Git.
See .env.example for the required variable names.

## Build commands

API:     cargo build --release --bin api
Admin:   cd apps/admin && npm ci && npm run build
Worker:  uses the same binary as the API

## Start commands

API:     ./api
Worker:  ./api worker

## Migration command

Run BEFORE starting or restarting the API after any schema change:

  atlas migrate apply --env production

This applies pending Atlas migrations to the PostgreSQL database.

## Health endpoint

GET /health → 200 OK → {"status": "ok"}

## Local simulation

To run the full production-equivalent stack locally:

  make prod-up
  make prod-migrate
  make prod-health
  # API: http://localhost:3000
  # Admin: http://localhost:8080

## Coolify deployment plan

When a VPS is available:
1. SSH to server, run: curl -fsSL https://cdn.coollabs.io/coolify/install.sh | bash
2. Open http://SERVER_IP:8000 and create admin account
3. Connect GitHub repo as a source
4. Add PostgreSQL service → copy internal DATABASE_URL
5. Add Redis service → copy internal REDIS_URL
6. Add API application → Dockerfile: apps/api/Dockerfile → env vars → port 3000
7. Add Worker application → same Dockerfile → CMD override: ./api worker
8. Add Admin application → build command: cd apps/admin && npm ci && npm run build
   → publish dir: apps/admin/dist → set VITE_API_URL to the API's public URL
9. Run atlas migrate apply --env production
10. Verify GET /health returns 200

## Atlas blocker note

The Atlas migration CI step uses continue-on-error because GitHub Actions does
not have a production database to migrate. To run migrations manually:

  atlas migrate apply --env local   # local development
  atlas migrate apply --env production  # production (set DATABASE_URL first)

## Rollback

In Coolify: select service → Deployments tab → click previous deployment
→ Rollback to this deployment.

## Known blockers

- Coolify deployment is pending VPS access.
- The full production stack is verified locally via docker-compose.prod.yml.
- CI passes on GitHub Actions for all Rust and frontend checks.
