# ── Stage 1: Chef (dependency caching layer) ────────────────────────────────
ARG RUST_VERSION=bookworm
FROM rust:${RUST_VERSION} AS chef
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    build-essential \
    protobuf-compiler \
    libprotobuf-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef --locked

# ── Stage 2: Planner ─────────────────────────────────────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 3: Builder ─────────────────────────────────────────────────────────
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Cache dependencies separately from application code
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

# Validate compilation
RUN cargo check

# Build release binary
RUN cargo build --release --bin axum_backend

# ── Stage 4: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Only OpenSSL + CA certs needed at runtime; no Postgres libs
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
    openssl \
    libssl3 \
    ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary
COPY --from=builder /app/target/release/axum_backend /usr/local/bin/axum_backend

# Run as non-root
RUN useradd -ms /bin/bash appuser
USER appuser

# Expose HTTP API port
EXPOSE 3000

ENTRYPOINT ["axum_backend"]
