# /review — Axum Backend Code Review

Perform a structured, project-specific code review of this Axum DDD backend.

**Argument:** `$ARGUMENTS` (optional: `all`, `security`, `async`, `architecture`, `tests`, or a file/directory path)

---

## Step 0: Determine Scope

- If `$ARGUMENTS` is empty or `all`: review all uncommitted changes (`git diff HEAD`). If working tree is clean, review the full codebase.
- If `$ARGUMENTS` is `security`, `async`, `architecture`, or `tests`: run only that specific gate (see below).
- If `$ARGUMENTS` is a file or directory path: review only that path.

Get the list of files to review. For uncommitted changes, run `git diff --name-only HEAD` and `git diff --cached --name-only`. For full codebase, scan `src/`, `tests/`, `migrations/`, `docker/`, `Cargo.toml`.

---

## Step 1: Build Gate (always runs first)

Run these checks and report any failures before proceeding:

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo check
```

If any fail, report the errors and stop. Do not proceed to domain review with a broken build.

---

## Step 2: Security Gate

Check every changed file for:

### CRITICAL (blocks approval)
- [ ] Hardcoded secrets, API keys, tokens, or passwords in source or docker files
- [ ] `unwrap()` or `expect()` in production code paths (non-test, non-unreachable)
- [ ] `panic!()` in any library/service constructor or request handler
- [ ] Missing RBAC check on privileged endpoints (create/update/delete users, role changes, import)
- [ ] JWT: fallback secrets, missing expiry, algorithm confusion, weak secret length
- [ ] Raw SQL or string-interpolated queries (bypass Diesel parameterization)
- [ ] Secrets in API error responses (database errors, stack traces, internal paths)

### HIGH (must fix before merge)
- [ ] Cookie `secure` flag not driven by runtime config
- [ ] Refresh tokens stored without hashing
- [ ] Missing rate limiting on auth endpoints (`/login`, `/verify`, `/forgot-password`, `/resend-code`)
- [ ] Swagger UI or `/metrics` exposed without authentication in production
- [ ] Confirmation codes using weak PRNG or fewer than 8 alphanumeric characters
- [ ] SMTP credentials silently defaulting to empty strings

---

## Step 3: Async Safety Gate

Check every changed `.rs` file for:

- [ ] `std::fs::read` / `std::fs::write` in async functions — must use `tokio::fs`
- [ ] `PgConnection::establish` or `diesel_migrations` outside `spawn_blocking`
- [ ] `Argon2` hashing (`PasswordManager::hash`) in async context without `spawn_blocking`
- [ ] `std::sync::Mutex` used in async code — must use `tokio::sync::Mutex`
- [ ] `std::process::Command` in async code — must use `tokio::process::Command`
- [ ] Any `.lock().unwrap()` that ignores mutex poisoning

---

## Step 4: Architecture Gate

### Layer Boundary Violations
- [ ] `domain/` importing anything from `infrastructure/`, `presentation/`, or `diesel`
- [ ] `application/` importing concrete infrastructure types (should use trait bounds only)
- [ ] `presentation/` calling repository methods directly (must go through use cases)
- [ ] `shared/` importing from domain, application, infrastructure, or presentation

### Domain Integrity
- [ ] Repository traits accepting raw `&str` where domain value objects (`&Email`, `&UserId`) should be used
- [ ] Entity construction via struct literal outside the domain layer (bypass `User::new()`)
- [ ] `From<diesel::result::Error>` impl in `domain/` (belongs in `infrastructure/`)
- [ ] New code in `application/use_cases/` (should go in `commands/` or `queries/`)

### Error Handling Patterns
- [ ] `.map_err(|e| SomeError(e.to_string()))` — must preserve structured error variants
- [ ] `.unwrap_or(0)` or `.unwrap_or(false)` silencing `Result` errors — propagate with `?`
- [ ] `AppError` variants leaking internal details (DB errors, file paths) to clients

### Code Quality
- [ ] Functions exceeding 50 lines
- [ ] Files exceeding 800 lines
- [ ] Nesting depth exceeding 4 levels
- [ ] Unnecessary `.clone()` on values that could be moved or borrowed
- [ ] `String` parameters where `&str` would suffice
- [ ] Duplicated logic across multiple use cases (extract to shared utility)

---

## Step 5: Test Gate

### Coverage Check
- [ ] New use case / command / query has corresponding unit test with `mockall`
- [ ] New handler has corresponding API test in `tests/api/`
- [ ] New repository method has integration test
- [ ] Error paths and edge cases are tested (not just happy path)

### Test Quality
- [ ] No `unwrap()` in test assertions that could silently mask failures — use `assert!`, `assert_eq!`, or `.expect("descriptive message")`
- [ ] API tests use `#[serial]` when sharing state
- [ ] Mock expectations use `.times(1)` or explicit counts
- [ ] Test names are descriptive (`creates_user_with_valid_email`, not `test_1`)
- [ ] No `println!` debug output left in tests

---

## Step 6: Report

Generate a structured report with this format:

```
## Review Summary

| Severity | Count |
|----------|-------|
| CRITICAL | X     |
| HIGH     | X     |
| MEDIUM   | X     |
| LOW      | X     |

## Findings

### [SEVERITY] Short description
- **File:** `path/to/file.rs`, line(s) X-Y
- **Rule:** Which gate/check this violates
- **Impact:** What could go wrong
- **Fix:** Specific remediation

## Verdict: PASS / BLOCK
```

**BLOCK** if any CRITICAL or HIGH issues found. **PASS** otherwise.

---

## Dispatch Strategy

For full codebase review (`all` or clean working tree), dispatch **4 parallel agents**:

1. **rust-reviewer** — domain + application layers (Steps 4 domain integrity + code quality)
2. **security-reviewer** — full security gate (Step 2) + async safety (Step 3)
3. **rust-reviewer** — infrastructure + presentation layers (Steps 3-4 layer boundaries)
4. **code-reviewer** — test suite quality (Step 5)

For targeted reviews (single file or directory), run checks inline without agents.
