# Local Runtime Environment

This document describes the local runtime setup for **Ahlan Commerce** using `mprocs`.

## Why mprocs?

As the microservices stack grows (comprising the API, PostgreSQL database, and eventually frontend, workers, and Redis), managing each process in a separate terminal tab becomes cumbersome.

We use `mprocs` to:
- Provide a single, consolidated process board for local development.
- View and debug logs across process boundaries in real-time.
- Easily start and stop the entire development stack with a single command.

> [!IMPORTANT]
> **mprocs is a local workflow tool, not a production orchestrator.**
> In production, process management, log aggregation, and service scaling are handled by production-ready orchestrators (like Docker Compose, Kubernetes, or Coolify) and dedicated log collectors. `mprocs` is strictly for local visibility and convenience during development.

## Running the Stack

To start both the Axum API and the PostgreSQL database:
```bash
make start
```
This opens the `mprocs` interactive terminal interface.

Depending on your environment, the setup adapts dynamically:
- **Docker Setup**: If Docker is installed, running `make start` boots a PostgreSQL Docker container and tails its logs inside `mprocs`. Running `make stop` halts the container.
- **Native Setup**: If Docker is not installed, it checks if a local native PostgreSQL instance is running on port 5432 (using `pg_isready`). Inside `mprocs`, it tails the native PostgreSQL logs (e.g. `/var/log/postgresql/postgresql-16-main.log`). Running `make stop` will not shut down the system service since it is managed natively.

## Logs Locations

- **API Logs**: Displayed in the `api` pane of the `mprocs` window. These logs capture HTTP requests, application logic, and tracing messages from the Axum/Tokio server.
- **Database Logs**: Displayed in the `postgres` pane of the `mprocs` window. It tails either:
  - The PostgreSQL Docker container (`docker logs -f ahlan_db`).
  - The native PostgreSQL cluster logs (`/var/log/postgresql/postgresql-16-main.log` or similar).
