# Make Commands Reference

The `Makefile` serves as the primary task runner for the project. Below is a list of the key commands and their purposes.

## Application Lifecycle
- **`make start`**: Starts the API, Background Worker, and a Redis monitoring pane simultaneously using `mprocs`.
- **`make stop`**: Gracefully stops the PostgreSQL docker container (if running via Docker).
- **`make run-api`**: Runs only the `apps/api` Axum server natively via `cargo run`.
- **`make run-worker`**: Runs only the `apps/worker` background processor natively.

## Infrastructure & Database
- **`make db-start`**: Spins up the PostgreSQL database in Docker if it isn't running natively.
- **`make db-migrate`**: Applies schema changes to the database using the Atlas CLI (`.bin/atlas migrate apply`).
- **`make cornucopia-generate`**: Generates type-safe Rust structs and functions from the raw SQL queries defined in `db/queries/`. Must be run whenever a `.sql` file is added or modified.

## Documentation & Validation
- **`make docs-api`**: Executes the documentation generation script (`apps/api/src/bin/generate_docs.rs`). This outputs the OpenAPI JSON and GraphQL SDL to `docs/generated/`.
- **`make docs-api-check`**: Runs `make docs-api` and verifies that there are no uncommitted changes in the `docs/generated/` folder. This ensures our API definitions are perfectly in sync with the codebase. Used by our CI pipelines.

## Testing & Health
- **`make test`**: Runs all unit and integration tests across the workspace.
- **`make health`**: Pings the `/health` REST endpoint to verify the API is responsive.
- **`make redis-health`**: Pings the local Redis server to ensure it is accepting connections.
