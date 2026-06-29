FROM rust:1.89-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY apps/api/Cargo.toml apps/api/
COPY apps/worker/Cargo.toml apps/worker/
COPY packages/ packages/

RUN mkdir -p apps/api/src apps/worker/src && \
    echo "fn main() {}" > apps/api/src/main.rs && \
    echo "fn main() {}" > apps/worker/src/main.rs && \
    cargo build --release --bin api && \
    rm apps/api/src/main.rs apps/worker/src/main.rs

COPY apps/api/src apps/api/src
COPY apps/worker/src apps/worker/src
RUN touch apps/api/src/main.rs && \
    cargo build --release --bin api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/api ./api

ENV API_BIND_ADDR=0.0.0.0:3000

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["./api"]
