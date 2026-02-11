# Chef stage: Install cargo-chef for caching dependencies
ARG RUST_VERSION=bookworm
FROM rust:${RUST_VERSION} AS chef
WORKDIR /app
# Install build dependencies mostly needed for cargo-chef or other crates
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef --locked

# Planner stage: Prepare recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage: Build dependencies and application
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .

# Step 1: Cargo Check
# Ensure the code compiles without errors before proceeding
RUN cargo check

# Step 2: Cargo Format
# Check format, if it fails, fix it.
# We use `||` to run `cargo fmt` only if `cargo fmt -- --check` fails.
# Note: cargo fmt modifies files in place, so the subsequent build will use formatted code.
RUN cargo fmt -- --check || cargo fmt

# Step 3: Build Release
RUN cargo build --release --bin axum_backend

# Runtime stage: Minimal runtime image
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies (OpenSSL, PostgreSQL client lib)
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl libssl-dev ca-certificates libpq-dev \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/axum_backend /usr/local/bin/

# Copy configuration and migrations if needed at runtime
# (Though we embedded migrations, it's good practice to keep structure)
COPY --from=builder /app/migrations ./migrations

# Don't run as root
RUN useradd -ms /bin/bash appuser
USER appuser

# Expose port
EXPOSE 3000

# Entrypoint
CMD ["axum_backend"]
