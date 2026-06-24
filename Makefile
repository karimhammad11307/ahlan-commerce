.PHONY: run-api test db-start db-migrate health start stop

run-api:
	cargo run -p api

run-worker:
	cargo run -p worker

test:
	cargo test

db-start:
	@if command -v docker >/dev/null 2>&1; then \
		docker start ahlan_db || docker run -d --name ahlan_db -p 5432:5432 -e POSTGRES_USER=ahlan -e POSTGRES_PASSWORD=ahlan_dev -e POSTGRES_DB=ahlan_commerce postgres:15; \
	else \
		echo "Docker not found. Checking native PostgreSQL on port 5432..."; \
		pg_isready -h localhost -p 5432 || (echo "Error: Native PostgreSQL is not running on port 5432!" && exit 1); \
	fi

db-migrate:
	.bin/atlas migrate apply --env local --allow-dirty

health:
	curl -f http://localhost:3000/health

start:
	mprocs

stop:
	@if command -v docker >/dev/null 2>&1; then \
		docker stop ahlan_db; \
	else \
		echo "Using native PostgreSQL. No container to stop."; \
	fi

cornucopia-generate:
	cornucopia generate -d packages/db/src/cornucopia.rs -q db/queries live -u postgres://ahlan:ahlan_dev@localhost:5432/ahlan_commerce
