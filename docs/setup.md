# Project Setup

This guide walks through setting up the Ahlan-Commerce backend locally.

## Prerequisites

Ensure you have the following installed on your machine:
- **Rust**: Install via [rustup](https://rustup.rs/).
- **PostgreSQL**: Install native PostgreSQL (e.g., version 15) or Docker.
- **Redis**: Install native `redis-server` and `redis-cli`, or Docker.
- **Atlas CLI**: Install [Atlas](https://atlasgo.io/getting-started/) for schema migrations.
- **Cornucopia**: Install via `cargo install cornucopia` to generate typed SQL queries.
- **mprocs**: Install via `cargo install mprocs` for running all services concurrently.

## Database & Infrastructure

1. **Start the database and cache**:
   If using Docker, the Makefile simplifies this:
   ```bash
   make db-start
   ```
   *(Ensure Redis is also running locally on port 6379).*

2. **Run Migrations**:
   We use Atlas to manage the PostgreSQL schema.
   ```bash
   make db-migrate
   ```

3. **Generate DAL Queries**:
   Whenever you edit `.sql` files in `db/queries/`, run:
   ```bash
   make cornucopia-generate
   ```

## Running the Application

You can spin up the API, Worker, and Redis monitor all at once using `mprocs`:
```bash
make start
```

Or run the API individually:
```bash
make run-api
```

## API Documentation

- **Scalar UI (REST)**: Available at `http://localhost:3000/docs/scalar`
- **Generated Contracts**: Open `docs/generated/openapi.json` or `docs/generated/schema.graphql`. 
  Regenerate them using `make docs-api`.
