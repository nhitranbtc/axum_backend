# Rust / Axum Backend Security Checklist

Run through this list when scoring Security dimension for this Axum backend project.

---

## Unsafe Code

```bash
# Find all unsafe blocks
grep -rn "unsafe" --include="*.rs" . | grep -v target | grep -v "//.*unsafe"
```

For each occurrence:
- Is there a `// SAFETY:` comment explaining the invariant? If not → flag
- Is the unsafe truly necessary or can it be replaced with safe API? → flag if avoidable

Score impact: Each unjustified `unsafe` block → -0.5 (up to -2)

---

## Secret Management

```bash
# Find hardcoded secrets or fallback defaults
grep -rn "secret\|password\|token\|key" --include="*.rs" . | grep -v target | grep -v test | grep -v "//"
# Check for fallback defaults on sensitive config
grep -rn "unwrap_or\|unwrap_or_else\|unwrap_or_default" --include="*.rs" src/config/ | grep -v test
```

Flags:
- `JWT_SECRET` with fallback default value → CRITICAL
- `DATABASE_URL` with fallback default → CRITICAL
- Any secret hardcoded in source code → CRITICAL
- Secrets in docker-compose without `${VAR}` references → HIGH
- `.env` file committed to git → CRITICAL

---

## Authentication & Authorization

```bash
# Check auth middleware application
grep -rn "auth_middleware\|layer\|route_layer" --include="*.rs" src/presentation/routes/ | grep -v test
# Check RBAC in handlers
grep -rn "claims\|role\|can_read\|can_write\|can_delete" --include="*.rs" src/presentation/handlers/ | grep -v test
```

Flags:
- Privileged endpoint without auth middleware → CRITICAL
- Handler performing write/delete without checking `claims.role` → HIGH
- Token type not validated (access vs refresh) → HIGH
- Missing `secure` flag on auth cookies → HIGH
- Missing `HttpOnly` flag on auth cookies → HIGH
- Missing `SameSite` attribute on cookies → MEDIUM

---

## Input Validation

```bash
# Check DTO validation
grep -rn "#\[validate\]\|\.validate()" --include="*.rs" . | grep -v target | grep -v test
# Check handler boundary validation
grep -rn "Json(\|Query(\|Path(" --include="*.rs" src/presentation/handlers/ | grep -v test
```

Flags:
- Handler accepting user input without validation → HIGH
- Missing `#[validate]` on DTOs with user-supplied fields → MEDIUM
- Path parameter parsed without error handling → MEDIUM
- Pagination without upper bound (unlimited page_size) → MEDIUM

---

## Async Safety

```bash
# Find blocking I/O in async context
grep -rn "std::fs::\|std::sync::Mutex\|std::thread::sleep\|std::process::Command" --include="*.rs" src/ | grep -v target | grep -v test
# Check for spawn_blocking usage
grep -rn "spawn_blocking" --include="*.rs" . | grep -v target | wc -l
```

Flags:
- `std::fs::read/write` in async function → HIGH (use `tokio::fs`)
- `std::sync::Mutex` in async code → HIGH (use `tokio::sync::Mutex`)
- Argon2 hashing without `spawn_blocking` → HIGH (CPU-heavy, blocks runtime)
- `PgConnection::establish` without `spawn_blocking` → HIGH
- Diesel migrations without `spawn_blocking` → HIGH

---

## SQL Injection

```bash
# Diesel enforces parameterized queries by default — check for raw SQL
grep -rn "sql_query\|execute_returning\|raw\|format!.*SELECT\|format!.*INSERT" --include="*.rs" . | grep -v target | grep -v test
```

Flags:
- Raw SQL with string interpolation → CRITICAL
- `format!` used to build SQL strings → CRITICAL
- Note: Standard Diesel queries are safe by design

---

## Error Information Leakage

```bash
# Check API responses for internal error details
grep -rn "to_string()\|Debug\|format!.*error\|format!.*err" --include="*.rs" src/presentation/ | grep -v test
# Check error response types
grep -rn "IntoResponse" --include="*.rs" src/shared/errors/ src/presentation/
```

Flags:
- DB error messages returned to client → HIGH
- File paths in error responses → HIGH
- Stack traces in API responses → HIGH
- `Debug` formatting of internal errors in response body → MEDIUM

---

## Token Security

```bash
# Check refresh token storage
grep -rn "refresh_token\|token_hash" --include="*.rs" src/infrastructure/ | grep -v test
# Check token hashing
grep -rn "sha256\|sha2\|digest\|hash" --include="*.rs" . | grep -v target | grep -v test
```

Flags:
- Refresh tokens stored as plaintext (not hashed) → HIGH
- JWT secret under 32 characters → HIGH
- Token expiry too long (>24h for access, >30d for refresh) → MEDIUM
- No token revocation mechanism → HIGH

---

## Rate Limiting

```bash
grep -rn "rate_limit\|RateLimit\|throttle\|Throttle" --include="*.rs" . | grep -v target
grep -rn "tower.*rate\|governor" Cargo.toml
```

Flags:
- No rate limiting on `/api/auth/login` → HIGH (brute force)
- No rate limiting on `/api/auth/register` → MEDIUM (spam)
- No rate limiting on `/api/auth/forgot-password` → MEDIUM (email spam)

---

## Confirmation Code Security

```bash
grep -rn "confirmation_code\|OsRng\|rand::\|thread_rng" --include="*.rs" . | grep -v target | grep -v test
```

Flags:
- Confirmation codes using `thread_rng` instead of `OsRng` → MEDIUM
- Codes shorter than 8 characters → MEDIUM
- No expiry on confirmation codes → HIGH
- Code not cleared after successful verification → MEDIUM

---

## Panic / Unwrap in Production

```bash
# Find unwrap/expect/panic outside test modules
grep -rn "\.unwrap()\|\.expect(\|panic!" --include="*.rs" src/ | grep -v target | grep -v "#\[cfg(test)\]\|mod tests"
```

Flags:
- `unwrap()` on user-supplied data → HIGH
- `panic!` in handler or use case logic → HIGH
- `expect()` without `// SAFETY:` comment → MEDIUM
- `unwrap_or()` silently swallowing Result errors → MEDIUM

---

## Summary Command

Run this quick scan for a fast security signal:

```bash
echo "=== unsafe blocks ===" && grep -rn "unsafe {" --include="*.rs" src/ | grep -v target | wc -l
echo "=== unwrap in prod ===" && grep -rn "\.unwrap()" --include="*.rs" src/ | grep -v target | grep -v "mod tests\|#\[test\]\|#\[cfg(test)\]" | wc -l
echo "=== hardcoded secrets ===" && grep -rn "secret.*=.*\"" --include="*.rs" src/config/ | grep -v test | grep -v "//"
echo "=== blocking in async ===" && grep -rn "std::fs::\|std::sync::Mutex\|std::thread::sleep" --include="*.rs" src/ | grep -v target | wc -l
echo "=== raw SQL ===" && grep -rn "sql_query\|format!.*SELECT" --include="*.rs" src/ | grep -v target | wc -l
echo "=== spawn_blocking ===" && grep -rn "spawn_blocking" --include="*.rs" src/ | grep -v target | wc -l
echo "=== rate limiting ===" && grep -rn "rate_limit\|governor\|throttle" --include="*.rs" src/ Cargo.toml | wc -l
```
