# MEMORY.md
<!-- index only — max 200 lines — pointers not explanations -->

## Project
- type: axum-backend
- stack: Rust 2021, Axum 0.7, Diesel 2.1 + diesel-async + deadpool, PostgreSQL
- entry: src/main.rs
- architecture: DDD with CQRS (migration in progress)
- author: Nhi Tran
- license: MIT

## Build & Test
- build: `cargo build --release`
- dev: `cargo run` or `cargo watch -x run`
- test: `cargo test` (requires Docker for testcontainers)
- unit: `cargo test --lib`
- api: `cargo test --test api_tests`
- integration: `cargo test --test integration_tests`
- lint: `cargo clippy -- -D warnings`
- fmt: `cargo fmt -- --check`
- docker-pg: `cd docker/postgres && docker compose up -d`

## Architecture
- domain: entities (User, RefreshToken), value objects (Email, UserId, UserRole), repo traits → src/domain/
- application: commands/ + queries/ (CQRS) + use_cases/ (legacy) + DTOs + services + actors → src/application/
- infrastructure: DB repos (UserRepositoryImpl, AuthRepositoryImpl), email (lettre+askama), cache, monitoring → src/infrastructure/
- presentation: handlers (auth, user, role, monitoring), auth middleware, ApiResponse envelope → src/presentation/
- shared: JwtManager, PasswordManager (Argon2), telemetry (tracing) → src/shared/
- config: AppConfig (env vars), DbPool config (deadpool) → src/config/

## Domain Entities
- User: id, email, name, password_hash, role, is_active, is_email_verified, confirmation_code, timestamps
- RefreshToken: id, user_id, token_hash, expires_at, revoked_at

## CQRS Status
- commands/user/: CreateUserCommand, UpdateUserCommand
- queries/user/: GetUserQuery, ListUsersQuery, UserStatisticsQuery
- use_cases/auth/ (legacy): register, login, logout, verify_email, set_password, forgot_password, resend_code
- use_cases/user/ (legacy): create, get, list, import, update, roles

## API Endpoints (17 total)
- Public: GET /health, POST /api/auth/{register,login,verify,password,forgot-password,resend-code}
- Auth required: POST /api/auth/logout
- Auth required: POST /api/users/, GET /api/users/, POST /api/users/import
- Auth required: GET/PUT /api/users/:id, GET/PUT /api/users/:id/role
- Internal: GET /metrics, GET /system-health

## Database
- Tables: users, refresh_tokens (1 migration: 2026-02-10-120000_init_user_schema)
- Pool: deadpool + diesel-async (PostgreSQL)
- Schema: auto-generated at src/infrastructure/database/schema.rs

## Key Crates
- axum 0.7, tower-http 0.5 (trace, cors, gzip), axum-extra 0.9 (cookies)
- diesel 2.1 + diesel-async 0.4.1 + deadpool 0.12
- jsonwebtoken 9.0, argon2 0.5
- utoipa 4 + utoipa-swagger-ui 7 (`swagger` feature flag)
- lettre 0.11 + askama 0.15 (email templates)
- ractor 0.13 (actor model for bulk import)
- axum-prometheus 0.10, sysinfo 0.38

## Test Infrastructure
- tests/api/ — auth, cookie_auth, health, monitoring, preflight (axum-test + serial_test)
- tests/integration/ — email_tests
- tests/common/ — TestServer, MockPostgres, factories, assertions
- tests/benchmarks/ — Criterion benchmarks
- tests/load/ — load tests
- testcontainers-modules 0.14 for ephemeral PostgreSQL

## Decisions
- [init] CQRS migration: new writes → commands/, new reads → queries/, do NOT add to use_cases/
- [init] Error handling: thiserror everywhere, anyhow only in main.rs
- [init] Async safety: never block tokio runtime, use spawn_blocking for Argon2/migrations
- [init] Security: no hardcoded secrets, RBAC in handlers, hash refresh tokens
- [init] Email: LettreEmailService for prod, NoopEmailService for dev/test

## Known Issues / Workarounds
- CQRS migration incomplete — use_cases/ and commands/ coexist with duplicated logic
- UserService::get_user_count() returns hardcoded 0 — needs wiring to UserRepository::count
- UserStatisticsQuery fields mostly return 0 (placeholders)
- ListUsersQuery.execute_with_filters() — UserFilters struct exists but filtering not wired
- GetUserQuery has TODO comment for caching layer (300s TTL planned)
- delete_all on UserRepository trait — remove or restrict to test-only
- No graceful shutdown handler in main.rs
- reqwest in [dependencies] — should be [dev-dependencies] if not used in prod
- Email validation in Email::parse weaker than DTO-level #[validate(email)]
- cache/ and external_apis/ are placeholder modules (not yet implemented)
- jwt_secret has fallback default "dev-secret-change-in-production" — CLAUDE.md says no fallback (mismatch)
- Deprecated module aliases (auth_repository, user_repository, auth_dto, etc.) still present

## Topic Files
- [Architecture](memory/architecture.md) — full DDD layer inventory with entities, traits, handlers, infra
- [API](memory/api.md) — endpoint table, auth flow, database schema details
- [Testing](memory/testing.md) — test structure, helpers, key crates, runner commands
