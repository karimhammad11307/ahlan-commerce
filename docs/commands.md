# Project Commands

This document lists the available `make` commands to help run and manage the `ahlan-commerce` project easily. These commands replace the long, raw commands that are error-prone to type manually.

- `make run-api`: Starts the backend Rust API locally. Wraps the raw command: `cargo run -p api`.
- `make test`: Runs all automated tests. Wraps the raw command: `cargo test`.
- `make db-start`: Starts the local PostgreSQL database using Docker. Wraps the raw command: `docker start ahlan_db || docker run ...`.
- `make db-migrate`: Applies database schema migrations using Atlas. Wraps the raw command: `.bin/atlas migrate apply --env local --allow-dirty`.
- `make health`: Checks if the API is running and healthy by sending a simple HTTP request. Wraps the raw command: `curl -f http://localhost:3000/health`.

*Note: You can inspect the `Makefile` in the root directory to see the exact shell instructions.*
