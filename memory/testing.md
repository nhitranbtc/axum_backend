---
name: testing
description: Test structure, helpers, and test organization patterns
type: project
---

# Testing Structure

## Test Directories
- `tests/api/` — API integration tests (axum-test with TestServer)
  - auth.rs, cookie_auth.rs, health.rs, monitoring.rs, preflight.rs
- `tests/integration/` — service integration tests
  - email_tests.rs
- `tests/benchmarks/` — Criterion benchmarks
  - backend_benchmarks.rs
- `tests/load/` — load tests
  - load_tests.rs
- `tests/common/` — shared test utilities
  - server.rs (TestServer setup), mock.rs (MockPostgres), factories.rs, assertions.rs

## Test Entry Points
- `tests/api_tests.rs` → includes tests/api/ modules
- `tests/integration_tests.rs` → includes tests/integration/ modules
- `tests/load_tests.rs` → includes tests/load/ modules

## Key Test Crates
- axum-test 14.0 — HTTP testing with TestServer
- mockall 0.12 — trait mocking
- serial_test 3.0 — `#[serial]` for DB-dependent tests
- testcontainers-modules 0.14 — ephemeral PostgreSQL containers
- criterion 0.5 — benchmarks
- tokio-test 0.4 — async test utilities

## Running Tests
- `cargo test` — all tests (Docker required for testcontainers)
- `cargo test --lib` — unit tests only (no Docker)
- `cargo test --test api_tests` — API tests
- `cargo test --test integration_tests` — integration tests
- `tests/run_tests.sh` — test runner script
