.PHONY: run-api test db-start db-migrate health start stop redis-health docker-build docker-run-api prod-up prod-down prod-logs prod-migrate prod-health

# When inside a Flatpak sandbox, delegate docker/pg_isready/redis-cli to the host.
HOST := $(shell if command -v host-spawn >/dev/null 2>&1; then echo "host-spawn"; fi)

run-api:
	cargo run -p api --bin api

run-worker:
	cargo run -p worker

test:
	cargo test

redis-health:
	@$(HOST) redis-cli ping

db-start:
	@if $(HOST) docker ps >/dev/null 2>&1; then \
		$(HOST) docker start ahlan_db 2>/dev/null || \
		$(HOST) docker run -d --name ahlan_db -p 5432:5432 \
			-e POSTGRES_USER=ahlan -e POSTGRES_PASSWORD=ahlan_dev \
			-e POSTGRES_DB=ahlan_commerce postgres:15; \
		echo "PostgreSQL ready (Docker)."; \
	elif $(HOST) pg_isready -h localhost -p 5432 >/dev/null 2>&1; then \
		:; \
	else \
		echo "Error: PostgreSQL is not running on port 5432!"; \
		if [ -n "$(HOST)" ]; then \
			echo "Start it with: host-spawn service postgresql start"; \
		fi; \
		exit 1; \
	fi

db-migrate:
	.bin/atlas migrate apply --env local --allow-dirty

health:
	curl -f http://localhost:3000/health

start:
	mprocs

stop:
	@if $(HOST) docker ps >/dev/null 2>&1; then \
		$(HOST) docker stop ahlan_db; \
	else \
		echo "Using native PostgreSQL. No container to stop."; \
	fi

cornucopia-generate:
	cornucopia generate -d packages/db/src/cornucopia.rs -q db/queries live -u postgres://ahlan:ahlan_dev@localhost:5432/ahlan_commerce

docs-api:
	mkdir -p docs/generated
	cargo run -p api --bin generate_docs

docs-api-check: docs-api
	git diff --exit-code docs/generated/ || (echo "Error: Generated docs are out of sync. Run 'make docs-api' and commit the changes." && exit 1)

docker-build:
	$(HOST) docker build -f apps/api/Dockerfile -t ahlan-api:latest .

docker-run-api:
	$(HOST) docker run \
		-e DATABASE_URL=$(DATABASE_URL) \
		-e REDIS_URL=$(REDIS_URL) \
		-p 3000:3000 \
		ahlan-api:latest

admin-build:
	cd apps/admin && npm ci && npm run build

admin-preview:
	cd apps/admin && npm run preview

prod-up:
	$(HOST) docker compose -f docker-compose.prod.yml up --build -d

prod-down:
	$(HOST) docker compose -f docker-compose.prod.yml down

prod-logs:
	$(HOST) docker compose -f docker-compose.prod.yml logs -f

prod-migrate:
	$(HOST) docker compose -f docker-compose.prod.yml exec api \
		sh -c "atlas migrate apply --env production" 2>/dev/null || \
		echo "Run atlas migrate apply manually with production DATABASE_URL"

prod-health:
	curl -f http://localhost:3000/health && echo " API healthy"
