---
name: security-review
description: Project-specific security review for this Axum DDD backend. Covers JWT, auth, RBAC, async safety, secrets, and OWASP Top 10 checks against the actual codebase patterns.
---

# Security Review — axum_backend

Run a structured security review tailored to this project's architecture and known vulnerability patterns.

## When to Use

- Before any commit touching `src/shared/utils/`, `src/presentation/middleware/`, `src/config/`, `src/application/use_cases/auth/`, `src/infrastructure/database/repositories/auth.rs`, or `docker/`
- After implementing any new handler or endpoint
- Before creating a PR
- When the `/review security` command is invoked

## Review Procedure

Execute gates in order. Stop and report if any CRITICAL is found.

---

### Gate 1: Secrets & Configuration

Scan these files for hardcoded secrets:

**Files to check:**
- `src/config/app_config.rs`
- `docker/backend/docker-compose.yml`
- `docker/postgres/docker-compose.yml`
- `docker/backend/run_container.sh`
- `.env.example`
- Any new `.yml`, `.yaml`, `.toml`, `.sh` files

**Rules:**

| Check | Severity | How to Detect |
|-------|----------|---------------|
| JWT secret has fallback default | CRITICAL | `unwrap_or_else` on `JWT_SECRET` env var in `app_config.rs` |
| DATABASE_URL has fallback default | CRITICAL | `unwrap_or_else` on `DATABASE_URL` — must use `map_err` |
| Hardcoded passwords in docker-compose | HIGH | Grep for `POSTGRES_PASSWORD:`, `GF_SECURITY_ADMIN_PASSWORD=`, credential literals |
| Credentials in script output | LOW | `echo` statements containing `$DB_PASS`, `$DB_USER` |
| `.env` file committed to git | CRITICAL | Check `.gitignore` includes `.env` |

**Required pattern for security-critical env vars:**

```rust
// CORRECT — fail fast, no fallback
jwt_secret: env::var("JWT_SECRET")
    .map_err(|_| ConfigError::MissingEnvVar("JWT_SECRET".to_string()))?,

// WRONG — silently falls back to known value
jwt_secret: env::var("JWT_SECRET")
    .unwrap_or_else(|_| "dev-secret-change-in-production".to_string()),
```

**Acceptable fallback vars** (non-secret, reasonable defaults):
- `SERVER_HOST` → `"127.0.0.1"`
- `SERVER_PORT` → `"3000"`
- `RUST_LOG` → `"info"`
- `JWT_ISSUER` → `"axum-backend"`
- `JWT_AUDIENCE` → `"axum-backend-api"`

---

### Gate 2: JWT & Token Security

**Files to check:**
- `src/shared/utils/jwt.rs` — `JwtManager`
- `src/presentation/middleware/auth.rs` — `auth_middleware`
- `src/presentation/handlers/auth.rs` — cookie handling
- `src/infrastructure/database/repositories/auth.rs` — refresh token storage

**Rules:**

| Check | Severity | Details |
|-------|----------|---------|
| `JwtManager::new` panics instead of returning `Result` | CRITICAL | Line 46-48: `panic!` on weak secret. Must return `Result<Self, JwtError>` |
| JWT algorithm pinned to HS256 | OK | `Algorithm::HS256` — verify no `Algorithm::None` anywhere |
| All required claims validated | OK | `exp`, `sub`, `iat`, `jti`, `iss`, `aud` — verify `set_required_spec_claims` |
| Token type checked in middleware | OK | `claims.token_type != "access"` — verify refresh tokens rejected |
| JWT expiry detection uses typed enum | HIGH | Line 113: string matching `"ExpiredSignature"` is brittle. Must use `e.kind()` |
| Cookie `secure` flag hardcoded `false` | HIGH | Lines 121, 129 in `handlers/auth.rs`. Must be runtime config |
| Cookie `SameSite` set | OK | `SameSite::Lax` — verify not `SameSite::None` without `Secure` |
| Refresh token stored as plaintext | HIGH | `token_hash` field stores raw JWT, not hash. Must SHA-256 hash before INSERT |

**Refresh token storage fix pattern:**

```rust
use sha2::{Sha256, Digest};

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

// Store: hash_token(&refresh_token)
// Lookup: WHERE token_hash = hash_token(&incoming_token)
```

---

### Gate 3: Authentication & Authorization (RBAC)

**Files to check:**
- `src/presentation/middleware/auth.rs` — authentication layer
- `src/presentation/handlers/role.rs` — role management
- `src/presentation/handlers/user.rs` — user CRUD + import
- `src/presentation/routes/users.rs` — route guards
- `src/domain/value_objects/user_role.rs` — role definitions

**Rules:**

| Check | Severity | Details |
|-------|----------|---------|
| Privileged endpoint without RBAC check | CRITICAL | Any handler that creates/updates/deletes users or changes roles MUST check `claims.role` |
| Auth middleware only validates token, not role | BY DESIGN | Middleware does authentication; handlers do authorization |
| `Claims` struct missing `role` field | HIGH | Role must be in JWT claims to avoid DB round-trip on every request |
| `delete_all` exposed without admin guard | HIGH | `UserRepository::delete_all` — should be test-only or admin-gated |

**Endpoints requiring admin role:**
- `PUT /api/users/:id/role` — `update_user_role`
- `POST /api/users/import` — `import_users`
- `POST /api/users` — `create_user`
- `DELETE /api/users/:id` — (if exposed)

**Required RBAC pattern in handlers:**

```rust
pub async fn update_user_role<R: AuthRepository>(
    State(use_case): State<Arc<UpdateUserRoleUseCase<R>>>,
    claims: Claims,
    // ...
) -> Result<..., RoleApiError> {
    // RBAC check — MUST be first operation
    if claims.role != "admin" {
        return Err(RoleApiError::Forbidden);
    }
    // ... proceed with operation
}
```

---

### Gate 4: Cryptographic Operations

**Files to check:**
- `src/shared/utils/password.rs` — Argon2 hashing
- `src/application/use_cases/auth/register.rs` — confirmation code generation
- `src/application/use_cases/auth/forgot_password.rs` — same
- `src/application/use_cases/auth/resend_code.rs` — same

**Rules:**

| Check | Severity | Details |
|-------|----------|---------|
| Password hashing uses Argon2 with OsRng salt | OK | `SaltString::generate(&mut OsRng)` in `password.rs` |
| `PasswordManager::verify` swallows errors | MEDIUM | Line 41-44: `Err(_) => Ok(false)` hides hash corruption. Should distinguish wrong-password from parse-failure |
| Confirmation code uses `thread_rng()` not `OsRng` | HIGH | `rand::thread_rng()` in 3 files. Must use `OsRng` for security tokens |
| Confirmation code is only 6 numeric digits | HIGH | 10^6 = 1M combinations. Must be 8+ alphanumeric (62^8 = 218B combinations) |
| Code generation duplicated in 3 files | HIGH | `register.rs:85-90`, `forgot_password.rs:51-55`, `resend_code.rs:59-63`. Extract to shared util |
| Code expiry default is 60 seconds | MEDIUM | Email delivery can take longer. Should be 900+ seconds |

**Required confirmation code pattern:**

```rust
// In shared/utils/crypto.rs (new file)
use rand::{rngs::OsRng, Rng};

pub fn generate_confirmation_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789";
    let mut rng = OsRng;
    (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
```

---

### Gate 5: Input Validation & Injection

**Files to check:**
- `src/domain/value_objects/email.rs` — `Email::parse`
- `src/application/dto/` — all request DTOs
- `src/infrastructure/database/repositories/` — all repo files
- `src/presentation/handlers/user.rs` — CSV import

**Rules:**

| Check | Severity | Details |
|-------|----------|---------|
| SQL injection via raw queries | CRITICAL | Diesel enforces parameterization. Grep for `sql_query(format!` or string interpolation in queries |
| `Email::parse` too permissive | MEDIUM | Accepts `a@b` (no TLD). Domain validation weaker than DTO `#[validate(email)]` |
| CSV import reads hardcoded path | HIGH | `std::fs::read("import/users.csv")` — no upload, hardcoded relative path, no auth check |
| Missing `#[validate]` on request DTOs | HIGH | Every `Json<RequestDto>` extraction must call `.validate()` |
| Path traversal in file operations | HIGH | Any `std::fs::read` with user-controlled path (currently hardcoded but worth flagging) |

**Validation checklist for new DTOs:**

```rust
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SomeRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(length(min = 8, max = 128))]
    pub password: String,
}
```

---

### Gate 6: Error Information Leakage

**Files to check:**
- `src/shared/errors/mod.rs` — `AppError::into_response`
- `src/presentation/handlers/auth.rs` — `AuthError::into_response`
- All use case error enums

**Rules:**

| Check | Severity | Details |
|-------|----------|---------|
| Database errors in API response body | MEDIUM | `RepositoryError(String)` carrying raw Diesel error text. Must map to generic message |
| User enumeration via distinct errors | MEDIUM | `forgot_password` returns `UserNotFound` vs success. Both flows should return same response |
| Internal paths in error messages | MEDIUM | `Config(format!("Failed to read CSV file: {}", e))` leaks filesystem path |
| Stack traces in responses | OK | Rust doesn't include by default. Verify no `Debug` formatting in responses |

**Required pattern — error sanitization at handler boundary:**

```rust
// In handler, before returning error to client:
AuthError::LoginError(msg) => {
    tracing::error!("Login failed: {}", msg);  // Full detail server-side
    (StatusCode::UNAUTHORIZED, "Invalid credentials")  // Generic to client
}
```

---

### Gate 7: Async Safety (Security Impact)

**Files to check:** All `.rs` files in `src/`

Blocking the async runtime is a denial-of-service vector.

| Check | Severity | Details |
|-------|----------|---------|
| `std::fs::read` in async handler | CRITICAL | `handlers/user.rs:41-43` — blocks Tokio worker |
| `PgConnection::establish` in async fn | CRITICAL | `connection.rs:22-26` — blocks at startup |
| Argon2 hash without `spawn_blocking` | HIGH | `use_cases/user/import.rs:48` — CPU-intensive, blocks worker |
| `std::sync::Mutex::lock` in async code | HIGH | `monitoring.rs:26` — blocks worker, poison risk |
| `panic!` in production async path | CRITICAL | `repositories/auth.rs:35-38` — crashes Tokio worker |

---

### Gate 8: Infrastructure Exposure

**Files to check:**
- `src/presentation/routes/mod.rs` — route definitions
- `docker/backend/docker-compose.yml` — exposed ports

| Check | Severity | Details |
|-------|----------|---------|
| Swagger UI exposed without auth | HIGH | `/swagger-ui` mounted unconditionally. Gate behind feature flag + auth in prod |
| `/metrics` exposed without auth | HIGH | Prometheus endpoint leaks internal metrics. Gate behind admin auth |
| `/api/admin/system` exposed without auth | HIGH | System health endpoint. Gate behind admin auth |
| Grafana on public port with default password | HIGH | Port 3001 with `admin/admin`. Internal-only network |
| No rate limiting on auth endpoints | HIGH | `/login`, `/verify`, `/forgot-password`, `/resend-code` all unprotected |
| No CORS configuration | MEDIUM | No `CorsLayer`. Must explicitly configure allowed origins |
| No graceful shutdown | HIGH | SIGTERM kills in-flight requests. Use `.with_graceful_shutdown()` |

---

## Report Format

```
## Security Review — [date]

### Scope
Files reviewed: [list]

### Findings

| # | Severity | Gate | Issue | File:Line |
|---|----------|------|-------|-----------|
| 1 | CRITICAL | ... | ... | ... |

### OWASP Top 10 Coverage

| Category | Status | Notes |
|----------|--------|-------|
| A01 Broken Access Control | ... | ... |
| A02 Cryptographic Failures | ... | ... |
| A03 Injection | ... | ... |
| A04 Insecure Design | ... | ... |
| A05 Security Misconfiguration | ... | ... |
| A06 Vulnerable Components | ... | ... |
| A07 Auth Failures | ... | ... |
| A08 Data Integrity | ... | ... |
| A09 Logging/Monitoring | ... | ... |
| A10 SSRF | ... | ... |

### Verdict: PASS / BLOCK
BLOCK if any CRITICAL or HIGH. PASS otherwise.

### Recommended Actions
1. [Priority-ordered list of fixes]
```

## Dispatch

For full security review, use the **security-reviewer** agent with this prompt:

> Perform security review of the Axum backend at [project root]. Follow the 8-gate procedure in `.claude/skills/security-review/SKILL.md`. Check every file listed in each gate. Report all findings with severity, file path, and line number.

For targeted review (single file), run the relevant gates inline.
