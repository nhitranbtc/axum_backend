# Claude AI Instructions

## Project Overview

Production-ready Axum backend service with Domain-Driven Design (DDD) architecture, CQRS pattern, JWT authentication, and Diesel ORM (async via `diesel-async` + `deadpool`).

- **Language:** Rust (edition 2021)
- **Framework:** Axum 0.7
- **Database:** PostgreSQL via Diesel 2.1 + diesel-async
- **Auth:** JWT (jsonwebtoken 9.0) + Argon2 password hashing
- **Async runtime:** Tokio
- **API docs:** utoipa + Swagger UI (behind `swagger` feature flag)

## Architecture

```
src/
├── domain/           # Entities, value objects, repository traits, domain errors
├── application/      # Use cases, commands, queries, DTOs, services, actors
├── infrastructure/   # Database repos, email, cache, monitoring
├── presentation/     # Axum handlers, middleware, routes, responses
├── shared/           # Cross-cutting: errors, telemetry, utilities (JWT, password)
├── config/           # AppConfig, database pool config
├── lib.rs            # Module re-exports
└── main.rs           # Entrypoint: config → pool → migrations → router → serve
```

### Layer Rules

- **domain/** has ZERO infrastructure dependencies. No Diesel, no Axum, no external crates beyond `uuid`, `chrono`, `thiserror`, `serde`.
- **application/** depends on domain traits only. Never import concrete infrastructure types.
- **infrastructure/** implements domain traits. This is where Diesel, lettre, and external APIs live.
- **presentation/** depends on application use cases via `Arc<dyn Trait>` or `Arc<UseCase<R>>`. Never call repository methods directly from handlers.
- **shared/** is available to all layers but must not depend on domain, application, infrastructure, or presentation.

### CQRS Migration (In Progress)

The codebase is migrating from `use_cases/` to `commands/` + `queries/`. During migration:
- New write operations go in `application/commands/`
- New read operations go in `application/queries/`
- Do NOT create new files in `application/use_cases/` — that directory is legacy
- When touching a use case, migrate it to the appropriate command/query

## Build & Run

```bash
# Development
cargo run                        # Starts on SERVER_HOST:SERVER_PORT
cargo watch -x run               # Auto-reload on changes

# Docker (local development)
cd docker/postgres && docker compose up -d   # Start PostgreSQL
cd docker/backend && docker compose up -d    # Start full stack

# Check & lint
cargo fmt -- --check             # Formatting check
cargo clippy -- -D warnings      # Lint (treat warnings as errors)
cargo check                      # Fast compilation check

# Tests
cargo test                       # All tests (requires Docker PostgreSQL)
cargo test --lib                 # Unit tests only (no Docker needed)
cargo test --test api_tests      # API integration tests
cargo test --test integration_tests  # Integration tests
```

## Testing

Tests use `testcontainers` to spin up ephemeral PostgreSQL containers. Docker must be running.

- **Unit tests:** `#[cfg(test)]` modules in source files. Use `mockall` for trait mocking.
- **API tests:** `tests/api/` — uses `axum-test` with `TestServer`. Annotate with `#[serial]`.
- **Integration tests:** `tests/integration/` — email, DB integration.
- **Benchmarks:** `tests/benchmarks/` — Criterion benchmarks.
- **Test helpers:** `tests/common/` — `TestServer`, `MockPostgres`, factories, assertions.

## Environment Variables

Copy `.env.example` to `.env`. Required variables:
- `DATABASE_URL` — PostgreSQL connection string (required, no fallback)
- `JWT_SECRET` — Must be 32+ characters (required, no fallback)
- `SERVER_HOST` / `SERVER_PORT` — defaults `127.0.0.1:3000`
- `SMTP_*` — Email config (falls back to localhost in dev)
- `CONFIRMATION_CODE_EXPIRY` — seconds (default: 60)

## Code Conventions

### Error Handling (CRITICAL)

- **NEVER use `unwrap()` or `expect()` in production code paths.** Only in tests or truly unreachable states with a `// SAFETY:` comment.
- **NEVER use `panic!()` in library/service constructors.** Return `Result` and let the caller handle it.
- Propagate errors with `?` — do not silently swallow with `unwrap_or()` or `unwrap_or_default()`.
- Use `thiserror` for typed errors in domain/application layers.
- Use `anyhow` only in `main.rs` and top-level orchestration.
- Map repository errors to use-case errors explicitly — do NOT erase structured errors with `.to_string()`. Use `From` impls or match arms.
- **Never expose internal error details in API responses.** Log with `tracing::error!`, return generic messages to clients.

### Async Safety (CRITICAL)

- **NEVER call blocking I/O inside async functions** without `spawn_blocking`:
  - `std::fs::read/write` → use `tokio::fs` instead
  - `PgConnection::establish` → wrap in `tokio::task::spawn_blocking`
  - `diesel_migrations::run_pending_migrations` → wrap in `spawn_blocking`
  - `Argon2` hashing → wrap in `spawn_blocking`
  - `std::sync::Mutex` → use `tokio::sync::Mutex` in async contexts
- This applies to ALL layers, not just handlers.

### Security Rules

- **No hardcoded secrets.** Every secret must come from env vars. No fallback defaults for `JWT_SECRET` or `DATABASE_URL`. Fail at startup if missing.
- **No hardcoded credentials in docker-compose or scripts.** Use `${VAR}` references with `.env` files.
- **Cookie `secure` flag** must be driven by runtime config, never hardcoded `false`.
- **Hash refresh tokens** before storing in DB (SHA-256 minimum).
- **Rate limit all auth endpoints** before exposing to production.
- **RBAC enforcement:** Every handler that performs a privileged operation (create, update, delete users, change roles, import) MUST check the caller's role from JWT claims. Authentication alone is not authorization.
- **Confirmation codes:** Use `OsRng` (CSPRNG), minimum 8 alphanumeric characters or 32-byte hex tokens.
- **Swagger UI and `/metrics`** must be gated behind auth or feature flags in production.

### Domain Layer

- Value objects (`Email`, `UserId`, `UserRole`) enforce invariants at construction. Trust them after creation — no re-validation needed.
- Repository traits use domain types (`&Email`, `&UserId`), not raw primitives (`&str`, `Uuid`).
- `User::new()` is the only way to create a user entity. Never construct `User` via struct literal outside the domain.
- Implement `std::str::FromStr` (not custom `parse`) for value objects that parse from strings.
- `Default` must not be implemented for ID types that generate random values.

### Application Layer

- Use cases / commands / queries are the public API of the application layer.
- Each use case takes repository traits as constructor params (via `Arc<dyn Trait>`).
- Use cases must NOT duplicate logic — extract shared operations (e.g., confirmation code generation) into `shared::utils`.
- DTOs validate at the boundary with `validator` derive macros. Domain value objects validate at construction.

### Infrastructure Layer

- Repository implementations return domain entities, not Diesel models. Convert in the repo.
- After `INSERT`, use `.get_result::<Model>()` to return the actual DB row, not a hand-built entity.
- `From<diesel::result::Error>` conversions belong here, NOT in the domain layer.

### Presentation Layer

- Handlers extract use cases from Axum `State` (preferred) or `Extension`.
- Use `State<T>` over `Extension<T>` for type-checked extraction.
- Pass `&str` to use cases when possible — avoid unnecessary `String` allocation at the handler boundary.
- Group routes by domain: `/api/auth/*`, `/api/users/*`.
- Auth middleware goes on route groups, RBAC checks go in individual handlers.

### Diesel & Database

- Schema file is auto-generated at `src/infrastructure/database/schema.rs` — do NOT edit manually.
- Migrations live in `migrations/` — create with `diesel migration generate <name>`.
- Always include both `up.sql` and `down.sql`.
- Use parameterized queries exclusively — Diesel enforces this by default.
- Connection pool: `deadpool` with `diesel-async`. Access via `pool.get().await?`.

### Formatting & Style

- `cargo fmt` with `rustfmt.toml` (max_width=100, edition=2021, 4-space indent).
- `cargo clippy -- -D warnings` must pass with zero warnings.
- Follow Rust naming: `snake_case` functions, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants.
- Prefer `&str` over `String` in function params. Use `impl Into<String>` for constructors.
- Prefer iterator chains over manual loops for transformations.

## Known Technical Debt

These items are tracked and should be addressed incrementally:

1. CQRS migration incomplete — `use_cases/` and `commands/` coexist with duplicated logic
2. `UserService::get_user_count()` returns hardcoded `0` — wire to `UserRepository::count`
3. `delete_all` on `UserRepository` trait — remove or restrict to test-only
4. No graceful shutdown handler in `main.rs`
5. `reqwest` in `[dependencies]` — move to `[dev-dependencies]` if not used in production code
6. Email validation in `Email::parse` is weaker than DTO-level `#[validate(email)]`