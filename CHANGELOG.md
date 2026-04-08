# Changelog

All notable changes to axum_backend are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project uses [Conventional Commits](https://www.conventionalcommits.org/) and [Semantic Versioning](https://semver.org/).

## Structure

Each release section is organized by change type:

- **Added** — new features, endpoints, infrastructure
- **Changed** — modifications to existing functionality
- **Fixed** — bug fixes, security patches
- **Deprecated** — features marked for removal
- **Removed** — deleted features or code
- **Security** — vulnerability fixes, hardening

Within each type, entries follow:

```text
- <description> (<scope>) [#issue] — <commit>
```

---

## [Unreleased]

### Fixed

- Update `TestServer` for `create_router` API change (missing `is_production` param) (tests) [#20] — ce398c1
- Update confirmation code length assertions from 6 to 8 chars after security hardening (tests) [#20] — f286304
- Remove unused imports: `Value` in cookie_auth, `AppConfig` in preflight, `mock::*` re-export (tests) [#20] — f286304
- Suppress unused import warning in integration_tests entry point (tests) [#20] — f286304

### Security

- Remove hardcoded `JWT_SECRET` fallback — fail at startup if missing or < 32 chars (config) [#11] — 050d760
- Replace `thread_rng` with `OsRng` (CSPRNG) for confirmation code generation (shared) [#13] — 050d760
- Replace `std::fs::read` with `tokio::fs::read` in async user handler (presentation) [#16] — 050d760
- Wrap Argon2 hashing in `spawn_blocking` in import use case (application) [#16] — 050d760
- `JwtManager::new()` returns `Result` instead of panicking on weak secret (shared) [#18] — 050d760
- `model_to_entity` returns `Result` instead of panicking on invalid DB data (infrastructure) [#18] — 050d760
- Add `is_production` flag to `create_router` for environment-aware security controls (presentation) — 050d760

### Changed

- CI: Add `build-and-test` job (fmt, clippy, unit tests, API tests with Postgres) as gate before Claude code review [#25] — 2dbc5e0
- CI: Filter review workflow to Rust-relevant paths (`src/**/*.rs`, `migrations/**`, `Cargo.toml`) — 2dbc5e0
- CI: Skip draft PRs, use `fetch-depth: 0` for full diff context — 2dbc5e0

---

## [0.5.0] — 2026-04-07

### Added

- Codebase audit report for 2026-04-07 (docs) — b23bc76
- Codebase rater skill for Axum backend quality scoring (tooling) — 829b209
- Claude memory skill and code-reviewer agent configs (tooling) — 8f5913e
- MEMORY.md and topic files for codebase knowledge index (docs) — a638c02

### Changed

- Copy `.claude` config in GitHub Actions workflow (ci) — 97d99ff
- Add PreToolUse hooks to block `.env` file access (tooling) — cda2a0c
- Enable issue creation permission in Claude Code GitHub Action (ci) — f64d1f3

---

## [0.4.0] — 2026-04-06

### Added

- Claude Code project rules, review command, and security skill (tooling) — 3642e2e
- Claude PR Assistant and Code Review GitHub Actions workflows (ci) — eb8b62c, 675241f

### Changed

- Update Claude workflow actions configuration (ci) — f2d5ef0, b4eb4be, 872abba

---

## [0.3.0] — 2026-03-06

### Added

- Post feature with full CRUD (domain) — b269dae
- ScyllaDB operations layer (infrastructure) — 8f3f784
- ScyllaDB single and cluster Docker configurations (infrastructure) — da4435e
- ScyllaDB stack implementation (database) — 1ba3719

---

## [0.2.0] — 2026-02-14

### Added

- NATS messaging implementation (infrastructure) — 8f4e017
- Async Redis Cluster support (cache) — 5aa088b
- Redis caching layer (cache) — e474706
- gRPC services and infrastructure (infrastructure) — 9f0fcb7

### Changed

- Refactor and enhance testing framework (tests) — fc40c5f

---

## [0.1.0] — 2026-02-10

### Added

- Initial release of production-ready Axum backend service — 15d3e5b
- DDD architecture with domain, application, infrastructure, presentation layers
- JWT authentication with Argon2 password hashing
- Diesel ORM with async PostgreSQL via `diesel-async` + `deadpool`
- API integration tests with testcontainers — d10358a
- Docker setup for local development — a87e588
- Swagger UI behind feature flag (`utoipa`)
- Health check and monitoring endpoints

---

<!-- Links -->
[Unreleased]: https://github.com/nhitranbtc/axum_backend/compare/main...fix/test-and-ci-updates
[0.5.0]: https://github.com/nhitranbtc/axum_backend/compare/3642e2e...050d760
[0.4.0]: https://github.com/nhitranbtc/axum_backend/compare/b269dae...3642e2e
[0.3.0]: https://github.com/nhitranbtc/axum_backend/compare/5aa088b...b269dae
[0.2.0]: https://github.com/nhitranbtc/axum_backend/compare/15d3e5b...5aa088b
[0.1.0]: https://github.com/nhitranbtc/axum_backend/releases/tag/15d3e5b
