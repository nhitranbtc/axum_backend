# Scoring Rubrics

Detailed criteria for each dimension. Use these to calibrate scores consistently.

---

## 1. Structure (Directory Layout, Modularity, Naming)

| Score | Criteria |
|---|---|
| 9–10 | Clear separation of concerns, intuitive directory layout, consistent naming, each module has single responsibility, no God files |
| 7–8 | Generally clean, minor inconsistencies, one or two oversized files or ambiguous names |
| 5–6 | Flat or inconsistent structure, some modules doing too much, naming is mixed convention |
| 3–4 | Hard to navigate, unclear what lives where, modules highly coupled, naming is confusing |
| 0–2 | Everything in one file or random structure, no discernible organization |

**Axum/DDD specifics:**
- DDD layers correctly isolated (domain/, application/, infrastructure/, presentation/, shared/) — good
- Domain layer has zero infrastructure imports — verify
- Application layer depends only on domain traits, never concrete repos — verify
- `lib.rs` or any file over 800 lines — flag
- No separation of types, errors, DTOs into submodules in large layers — flag

---

## 2. Architecture & Design

| Score | Criteria |
|---|---|
| 9–10 | Appropriate patterns for domain, low coupling, high cohesion, abstractions at right level, scales without major rework |
| 7–8 | Solid design with minor over-engineering or under-abstraction, a few tight couplings |
| 5–6 | Pattern used but inconsistently, some coupling issues, abstractions leak details |
| 3–4 | Ad-hoc design, significant coupling, will require major refactor to extend |
| 0–2 | No discernible design, everything calls everything |

**Axum/DDD specifics:**
- Handlers calling repository methods directly (bypassing use cases) — deduct 1–2
- Repository traits using domain types (`&Email`, `UserId`) not primitives — good (+0.5)
- CQRS pattern consistently applied — check if commands/queries coexist cleanly with legacy use_cases
- `Arc<dyn Trait>` or `Arc<UseCase<R>>` for dependency injection — good
- Business logic in handlers instead of use cases — flag
- Value objects enforcing invariants at construction — good

---

## 3. Security

| Score | Criteria |
|---|---|
| 9–10 | No critical findings, all inputs validated, auth on all privileged paths, secrets from env vars only, no unsafe without justification |
| 7–8 | Minor issues (e.g., some missing rate limiting), no exploitable vulnerabilities found |
| 5–6 | Moderate issues present, some unchecked inputs, potential DoS vectors, unsafe usage unclear |
| 3–4 | Multiple clear vulnerabilities, missing auth checks, hardcoded secrets, dangerous patterns |
| 0–2 | Critical exploitable vulnerabilities, no input validation, secrets in code |

**Automatic score caps:**
- Any hardcoded secret/key in source → cap at 3
- Missing auth middleware on privileged endpoints → cap at 5
- Blocking I/O in async context without spawn_blocking → deduct 1 per instance (up to -3)
- `unwrap()` on user-supplied data in handler → deduct 0.5 each (up to -2)

**Axum-specific checks:**
- JWT secret from env var with no fallback default — verify
- Auth middleware applied to all `/api/users/*` routes — verify
- RBAC checks in handlers for privileged operations (role changes, imports) — verify
- Refresh tokens hashed before DB storage — verify
- Cookie `secure` flag driven by runtime config — verify
- Rate limiting on `/api/auth/*` endpoints — verify
- Error responses don't leak DB errors or file paths — verify

---

## 4. Code Quality

| Score | Criteria |
|---|---|
| 9–10 | Readable, consistent style, proper error types, no `unwrap` in prod paths, low complexity, DRY |
| 7–8 | Generally clean, a few `unwrap`s or minor duplication, style mostly consistent |
| 5–6 | Noticeable duplication, inconsistent error handling, some complex functions, mixed conventions |
| 3–4 | Hard to read, significant duplication, `unwrap` / `panic` in prod paths, no consistent style |
| 0–2 | Unreadable, copy-pasted blocks everywhere, no error handling |

**Rust/Axum specifics:**
- `unwrap()`/`expect()` outside `#[cfg(test)]` — deduct 0.5 each (up to -2)
- `thiserror` for typed errors, `anyhow` only in main.rs — good
- `cargo clippy -- -D warnings` passes — verify
- Deeply nested match arms — flag
- Functions over 50 lines — flag
- Files over 800 lines — flag
- CQRS duplication (use_cases/ and commands/ doing same thing) — flag as tech debt

---

## 5. Testing

| Score | Criteria |
|---|---|
| 9–10 | Unit + integration + API tests, edge cases covered, mock repos present, benchmarks present, CI runs tests |
| 7–8 | Good unit test coverage, some integration tests, benchmarks present or not needed |
| 5–6 | Some tests but gaps in coverage, happy-path only, no benchmarks |
| 3–4 | Minimal tests, only smoke tests, no edge cases |
| 0–2 | No tests or tests that don't assert anything meaningful |

**Axum backend specifics:**
- `#[cfg(test)]` modules in source files for unit tests — expected
- API tests using `axum-test` with `TestServer` — good
- `mockall` for trait mocking in unit tests — good
- `testcontainers` for DB integration tests — good
- `#[serial]` on DB-dependent tests — expected
- Benchmark suite present (Criterion) — good
- `cargo test` passes — verify if possible
- Auth error paths tested (invalid token, expired, wrong role) — verify

---

## 6. Dependencies

| Score | Criteria |
|---|---|
| 9–10 | All deps pinned, lock file committed, minimal deps, no known-bad versions |
| 7–8 | Mostly pinned, lock file present, one or two loose version ranges |
| 5–6 | Some unpinned deps, lock file may be missing, slightly bloated |
| 3–4 | Many unpinned, outdated versions, redundant deps doing same thing |
| 0–2 | No lock file, wildcard versions, known vulnerable deps |

**Rust specifics:**
- `Cargo.lock` committed — check
- Deps using exact or compatible version ranges (e.g., `"1.0"` not `"*"`) — verify
- `cargo audit` signal (check if present in CI)
- `reqwest` in `[dependencies]` vs `[dev-dependencies]` — check if used in production
- Duplicate transitive deps doing same job — flag

---

## 7. Documentation

| Score | Criteria |
|---|---|
| 9–10 | README with setup/build/test, CLAUDE.md with conventions, inline doc comments on public API, Swagger/OpenAPI, CHANGELOG |
| 7–8 | Good README, CLAUDE.md present, most public items documented, Swagger present |
| 5–6 | README exists but thin, some inline docs, no CHANGELOG |
| 3–4 | README minimal or outdated, little to no inline docs |
| 0–2 | No README, no docs, code is the only documentation |

**Axum backend specifics:**
- `CLAUDE.md` with architecture, conventions, known tech debt — good
- `MEMORY.md` with codebase index — good
- Swagger UI at `/swagger-ui/` with all endpoints documented — verify
- `//!` crate-level docs in `lib.rs` — check
- Public trait methods documented with `///` — check
- `.env.example` present with all required variables — check
