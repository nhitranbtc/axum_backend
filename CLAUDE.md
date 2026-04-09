# CLAUDE.md — axum_backend

Hooks handle verification mechanically. This file handles everything hooks
can't enforce: how to think, plan, and manage context in this project.

---

## Agent Behavior

### Planning

- When asked to plan: output only the plan. No code until told to proceed.
- When given a plan: follow it exactly. Flag real problems and wait.
- For non-trivial features (3+ steps or architectural decisions): interview
  about implementation, UX, and tradeoffs before writing code.
- Never attempt multi-file refactors in one response. Break into phases of
  max 5 files. Complete, verify, get approval, then continue.

### Code Quality

- If architecture is flawed, state is duplicated, or patterns are
  inconsistent: propose the structural fix. Ask: "What would a senior
  perfectionist dev reject in code review?" Fix that.
- No robotic comment blocks. Default to no comments. Only comment when the
  WHY is non-obvious.
- Don't build for imaginary scenarios. Simple and correct beats elaborate
  and speculative.

### Context Management

- Before ANY structural refactor on a file >300 LOC: first remove dead
  props, unused exports, unused imports, debug logs. Commit cleanup
  separately.
- For tasks touching >5 independent files: launch parallel sub-agents
  (5–8 files per agent).
- After 10+ messages: re-read any file before editing it. Compaction may
  have destroyed your memory of its contents.
- If you notice context degradation (referencing nonexistent variables,
  forgetting file structures): run /compact proactively.

### Edit Safety

- Before every file edit: re-read the file. The Edit tool fails silently
  on stale old_string matches.
- On any rename or signature change, search for: direct calls, type
  references, string literals, re-exports, barrel files, test mocks.
  Assume grep missed something.
- Never delete a file without verifying nothing references it.

### Self-Correction

- If a fix doesn't work after two attempts: stop. Read the entire relevant
  section top-down. State where your mental model was wrong.
- When pointed to existing code as reference: study it, match its patterns
  exactly. Working code is a better spec than a description.
- Work from raw error data. Don't guess. If a bug report has no output,
  ask for it.

### Communication

- When told "yes", "do it", or "push": execute. Don't repeat the plan.
- Keep responses short. No trailing summaries of what you just did.

---

## Project Overview

Production-ready Axum backend with DDD architecture, CQRS pattern, JWT auth, and Diesel ORM (async via `diesel-async` + `deadpool`).

| Aspect | Detail |
|--------|--------|
| Language | Rust (edition 2021) |
| Framework | Axum 0.7 |
| Database | PostgreSQL via Diesel 2.1 + diesel-async |
| Auth | JWT (jsonwebtoken 9.0) + Argon2 |
| Async | Tokio |
| API docs | utoipa + Swagger UI (`swagger` feature flag) |

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

| Layer | Depends on | NEVER imports |
|-------|-----------|---------------|
| **domain/** | `uuid`, `chrono`, `thiserror`, `serde` only | Diesel, Axum, infrastructure, presentation |
| **application/** | domain traits only | concrete infrastructure types, Diesel, Axum |
| **infrastructure/** | domain traits (implements them) | presentation |
| **presentation/** | application use cases via `Arc<dyn Trait>` | repository methods directly |
| **shared/** | nothing project-internal | domain, application, infrastructure, presentation |

### CQRS Migration (In Progress)

- New writes → `application/commands/`
- New reads → `application/queries/`
- Do NOT create new files in `application/use_cases/` (legacy)
- When touching a use case, migrate it to command/query

## Build & Run

```bash
cargo run                                    # Starts on SERVER_HOST:SERVER_PORT
cargo watch -x run                           # Auto-reload

# Docker
cd docker/postgres && docker compose up -d   # PostgreSQL
cd docker/backend && docker compose up -d    # Full stack

# Check & lint
cargo fmt -- --check
cargo clippy -- -D warnings
cargo check

# Tests (Docker must be running for non-unit tests)
cargo test                                   # All tests
cargo test --lib                             # Unit tests only
cargo test --test api_tests                  # API integration
cargo test --test integration_tests          # Integration
```

## Testing

Tests use `testcontainers` for ephemeral PostgreSQL containers.

| Type | Location | Notes |
|------|----------|-------|
| Unit | `#[cfg(test)]` in source files | `mockall` for trait mocking |
| API | `tests/api/` | `axum-test` + `TestServer`, `#[serial]` |
| Integration | `tests/integration/` | email, DB |
| Benchmarks | `tests/benchmarks/` | Criterion |
| Helpers | `tests/common/` | `TestServer`, `MockPostgres`, factories |

## Environment Variables

Copy `.env.example` → `.env`. Required:

| Variable | Notes |
|----------|-------|
| `DATABASE_URL` | PostgreSQL connection string (no fallback) |
| `JWT_SECRET` | 32+ characters (no fallback) |
| `SERVER_HOST` / `SERVER_PORT` | defaults `127.0.0.1:3000` |
| `SMTP_*` | falls back to localhost in dev |
| `CONFIRMATION_CODE_EXPIRY` | seconds, default 60 |

## Code Conventions

### Error Handling (CRITICAL)

- **NEVER** `unwrap()`, `expect()`, or `panic!()` in production code. Only in tests or with `// SAFETY:` comment.
- Propagate with `?`. No silent swallowing via `unwrap_or()`.
- `thiserror` for typed errors. `anyhow` only in `main.rs`.
- Map repo errors explicitly with `From` impls or match arms — never `.to_string()`.
- Never expose internal errors in API responses. Log with `tracing::error!`.

### Async Safety (CRITICAL)

**NEVER block the Tokio runtime.** Wrap in `spawn_blocking`:
- `std::fs` → `tokio::fs`
- `PgConnection::establish` → `spawn_blocking`
- `diesel_migrations::run_pending_migrations` → `spawn_blocking`
- Argon2 hashing → `spawn_blocking`
- `std::sync::Mutex` → `tokio::sync::Mutex`

### Security

- No hardcoded secrets. `JWT_SECRET` and `DATABASE_URL` fail at startup if missing.
- No credentials in docker-compose — use `${VAR}` with `.env`.
- Cookie `secure` flag driven by runtime config, never hardcoded `false`.
- Hash refresh tokens (SHA-256+) before DB storage.
- Rate limit all auth endpoints.
- RBAC: every privileged handler checks `claims.role`. Auth ≠ authz.
- Confirmation codes: `OsRng` (CSPRNG), 8+ alphanumeric chars.
- Swagger UI and `/metrics` gated behind auth or feature flags in prod.

### Domain Layer

- Value objects enforce invariants at construction. Trust after creation.
- Repository traits use domain types (`&Email`, `&UserId`), not primitives.
- `User::new()` is the only constructor. No struct literals outside domain.
- `FromStr` for value objects (not custom `parse`).
- No `Default` for ID types that generate random values.

### Application Layer

- Use cases take repos via `Arc<dyn Trait>`.
- No duplicated logic — extract shared ops to `shared::utils`.
- DTOs validate with `validator` derives. Domain types validate at construction.

### Infrastructure Layer

- Repos return domain entities, not Diesel models. Convert in the repo.
- After `INSERT`, use `.get_result::<Model>()` — don't hand-build entities.
- `From<diesel::result::Error>` belongs here, not in domain.

### Presentation Layer

- `State<T>` over `Extension<T>`.
- Pass `&str` to use cases — avoid unnecessary `String` allocation.
- Routes grouped by domain: `/api/auth/*`, `/api/users/*`.
- Auth middleware on groups, RBAC checks in individual handlers.

### Diesel & Database

- `src/infrastructure/database/schema.rs` is auto-generated — do NOT edit.
- Migrations: `diesel migration generate <name>`. Include both `up.sql` and `down.sql`.
- Parameterized queries only (Diesel enforces this).
- Pool: `deadpool` + `diesel-async`. Access via `pool.get().await?`.

### Formatting & Style

- `cargo fmt` with `rustfmt.toml` (max_width=100, edition=2021, 4-space indent).
- `cargo clippy -- -D warnings` — zero warnings.
- `snake_case` functions, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants.
- `&str` over `String` in params. `impl Into<String>` for constructors.
- Iterator chains over manual loops.

## Known Technical Debt

1. CQRS migration incomplete — `use_cases/` and `commands/` coexist
2. `UserService::get_user_count()` returns hardcoded `0`
3. `delete_all` on `UserRepository` — remove or restrict to test-only
4. No graceful shutdown handler in `main.rs`
5. `reqwest` in `[dependencies]` — move to `[dev-dependencies]` if unused in prod
6. `Email::parse` validation weaker than DTO-level `#[validate(email)]`

<!-- code-review-graph MCP tools -->
## MCP Tools: code-review-graph

**ALWAYS use code-review-graph MCP tools BEFORE Grep/Glob/Read.** The graph
is faster, cheaper, and gives structural context that file scanning cannot.

| Tool | Use when |
|------|----------|
| `detect_changes` | Code review — risk-scored analysis |
| `get_review_context` | Source snippets — token-efficient |
| `get_impact_radius` | Blast radius of a change |
| `get_affected_flows` | Impacted execution paths |
| `query_graph` | Tracing callers, callees, imports, tests |
| `semantic_search_nodes` | Finding functions/classes by keyword |
| `get_architecture_overview` | High-level codebase structure |
| `refactor_tool` | Planning renames, dead code |

### Workflow

1. Graph auto-updates on file changes (via hooks).
2. `detect_changes` for code review.
3. `get_affected_flows` to understand impact.
4. `query_graph` pattern="tests_for" to check coverage.

Fall back to Grep/Glob/Read only when the graph doesn't cover what you need.
