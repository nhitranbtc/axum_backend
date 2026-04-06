# Code Style Rules — axum_backend

These rules are derived from the project's actual codebase patterns and review findings. They supplement the global Rust rules in `~/.claude/rules/rust/`.

## Formatting

- `cargo fmt` with `rustfmt.toml` — enforced by PostToolUse hook
- `cargo clippy -- -D warnings` — enforced by PostToolUse hook, zero warnings allowed
- Max line width: 100 (per `rustfmt.toml`)
- 4-space indent, Unix line endings
- `match_block_trailing_comma = true`

## File & Function Size

- Files: 800 lines max. Split into submodules when approaching limit.
- Functions: 50 lines max. Extract helpers for complex logic.
- Nesting: 4 levels max. Use early returns and `?` to flatten.

## Naming

- `snake_case` for functions, methods, variables, modules
- `PascalCase` for types, traits, enums
- `SCREAMING_SNAKE_CASE` for constants
- Test names: descriptive without `test_` prefix — `creates_user_with_valid_email`, not `test_create_user`
- Repository impl names: `RepositoryImpl` (not `PostgresUserRepository`) to avoid coupling to DB tech
- Use case names: `<Verb><Noun>UseCase` — `LoginUseCase`, `CreateUserUseCase`
- Command names: `<Verb><Noun>Command` — `CreateUserCommand`
- Query names: `<Verb><Noun>Query` — `ListUsersQuery`, `UserStatisticsQuery`
- Error enums: `<UseCase>Error` — `LoginError`, `RegisterError`

## Error Handling

### Production Code — NEVER use:

```rust
// BANNED in production code paths:
.unwrap()                    // Use .map_err()? or .context()?
.expect("msg")               // Use .map_err()? or .context()?
panic!("msg")                // Return Result with typed error
.unwrap_or(fallback)         // on Result — silently swallows errors. Use ? instead
.unwrap_or_else(|_| panic!()) // same as unwrap()
```

### Allowed exceptions:
- `unwrap()` / `expect()` in `#[cfg(test)]` modules
- `expect()` with comment `// SAFETY: <reason>` for truly unreachable states

### Error Propagation Patterns

```rust
// CORRECT — preserve structured error with From impl
self.auth_repo
    .find_by_email(&email)
    .await
    .map_err(LoginError::from)?;

// CORRECT — explicit variant mapping
self.auth_repo
    .find_by_email(&email)
    .await
    .map_err(|e| match e {
        AuthRepositoryError::UserNotFound => LoginError::InvalidCredentials,
        e => LoginError::RepositoryError(e.to_string()),
    })?;

// WRONG — erases structured error into opaque string
.map_err(|e| LoginError::RepositoryError(e.to_string()))?;
```

### Error Types

- `thiserror` for all typed errors (domain, application, infrastructure)
- `anyhow` only in `main.rs` for top-level orchestration
- Each use case / command defines its own error enum
- Never expose DB errors, file paths, or stack traces in API responses — log with `tracing::error!`, return generic message

### Constructor Pattern

```rust
// CORRECT — constructors return Result
pub fn new(secret: String) -> Result<Self, ConfigError> {
    if secret.len() < 32 {
        return Err(ConfigError::WeakSecret);
    }
    Ok(Self { secret })
}

// WRONG — panic in constructor
pub fn new(secret: String) -> Self {
    if secret.len() < 32 {
        panic!("Secret too short"); // Never do this
    }
    Self { secret }
}
```

## Ownership & Borrowing

```rust
// CORRECT — borrow when you don't need ownership
pub async fn execute(&self, email: &str) -> Result<Response, Error> { ... }

// WRONG — unnecessary String allocation at handler boundary
pub async fn execute(&self, email: String) -> Result<Response, Error> { ... }
```

Rules:
- Function params: `&str` over `String`, `&Email` over `Email`, `&[T]` over `Vec<T>`
- Constructors: `impl Into<String>` for fields that need owned data
- Avoid `.clone()` on values that can be moved — especially `Arc` at the end of a chain
- Never clone to satisfy the borrow checker without understanding the root cause

## Async Safety

### NEVER block the Tokio runtime:

```rust
// BANNED in async context:
std::fs::read(path)                        // Use tokio::fs::read(path).await
std::fs::write(path, data)                 // Use tokio::fs::write(path, data).await
PgConnection::establish(url)               // Wrap in spawn_blocking
conn.run_pending_migrations(MIGRATIONS)    // Wrap in spawn_blocking
PasswordManager::hash(password)            // Wrap in spawn_blocking (Argon2 is CPU-heavy)
std::sync::Mutex::lock()                   // Use tokio::sync::Mutex in async code
std::process::Command::new()               // Use tokio::process::Command
std::thread::sleep()                       // Use tokio::time::sleep().await
```

### spawn_blocking pattern:

```rust
let hash = tokio::task::spawn_blocking(move || {
    PasswordManager::hash(&password)
}).await
    .map_err(|e| InternalError(e.to_string()))?
    .map_err(|e| InternalError(e.to_string()))?;
```

## Architecture Layer Rules

### domain/ — Pure business logic

```rust
// ALLOWED imports:
use uuid, chrono, serde, thiserror;
use crate::domain::*;

// BANNED imports:
use diesel::*;           // No ORM in domain
use axum::*;             // No web framework in domain
use crate::infrastructure::*;
use crate::presentation::*;
```

- Value objects enforce invariants at construction: `Email::parse()`, `UserId::new()`
- Repository traits live here, defined with domain types only
- `From<diesel::result::Error>` belongs in `infrastructure/`, NOT here

### application/ — Use cases, commands, queries

```rust
// ALLOWED imports:
use crate::domain::*;
use crate::shared::*;
use std::sync::Arc;

// BANNED imports:
use crate::infrastructure::database::*;  // Use trait bounds, not concrete types
use diesel::*;
use axum::*;
```

- Depend on repository traits via `Arc<R>` with trait bounds: `R: AuthRepository`
- New write operations: `application/commands/`
- New read operations: `application/queries/`
- Do NOT add to `application/use_cases/` (legacy, being migrated)

### infrastructure/ — Concrete implementations

```rust
// This is where Diesel, lettre, reqwest, etc. live
// Implement domain traits here
// Convert between Diesel models and domain entities in the repo
```

- Repository methods return domain entities (convert from `UserModel` → `User`)
- After INSERT, use `.get_result::<Model>()` — do NOT hand-build the return entity
- `model_to_entity()` must return `Result`, never panic on bad DB data

### presentation/ — HTTP layer

```rust
// Handler pattern:
pub async fn handler_name<R: AuthRepository>(
    State(use_case): State<Arc<SomeUseCase<R>>>,
    Json(payload): Json<RequestDto>,
) -> Result<Json<ApiResponse<ResponseDto>>, HandlerError> {
    payload.validate().map_err(|e| HandlerError::Validation(e.to_string()))?;
    let result = use_case.execute(...).await.map_err(HandlerError::from)?;
    Ok(Json(ApiResponse::success(result)))
}
```

- Use `State<T>` over `Extension<T>` for type-checked extraction
- Validate DTOs with `validator` at the handler boundary
- Never call repository methods directly — always go through use cases
- Auth middleware verifies token → handlers check RBAC (caller's role)

## API Response Envelope

All responses use `ApiResponse<T>`:

```rust
// Success:
{ "success": true, "data": { ... } }

// Error:
{ "success": false, "error": "Human-readable message" }
```

Handlers return `Result<Json<ApiResponse<T>>, HandlerError>`. The `HandlerError` `IntoResponse` impl formats the error envelope.

## Value Object Patterns

```rust
// Newtype wrapper with parse constructor
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn parse(email: impl Into<String>) -> Result<Self, DomainError> { ... }
    pub fn as_str(&self) -> &str { &self.0 }
    pub fn into_string(self) -> String { self.0 }
}

impl fmt::Display for Email { ... }
impl AsRef<str> for Email { ... }
```

- Implement `FromStr` (not custom `parse`) for standard `"value".parse::<Type>()` support
- Implement `Display` for string representation
- Implement `AsRef<str>` for transparent borrowing
- Do NOT implement `Default` for ID types that generate random values

## Use Case / Command Structure

```rust
#[derive(Debug, thiserror::Error)]
pub enum SomeUseCaseError {
    #[error("Not found")]
    NotFound,
    #[error("Repository error: {0}")]
    RepositoryError(String),
}

pub struct SomeUseCase<R: SomeRepository> {
    repo: Arc<R>,
}

impl<R: SomeRepository> SomeUseCase<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, ...) -> Result<Response, SomeUseCaseError> {
        // 1. Validate business rules
        // 2. Call repository
        // 3. Return response DTO
    }
}
```

## Repository Implementation Pattern

```rust
#[async_trait]
impl UserRepository for RepositoryImpl {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepositoryError::Internal(format!("Failed to get connection: {}", e)))?;

        let result = users::table
            .filter(users::id.eq(id.as_uuid()))
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| RepositoryError::Internal(e.to_string()))?;

        result.map(Self::model_to_entity).transpose()
    }
}
```

Pattern:
1. Get connection from pool with `map_err`
2. Build Diesel query (always parameterized)
3. Use `.optional()` for find operations
4. Convert model → entity via `model_to_entity()` (returns `Result`, never panics)

## Security Invariants

- `JWT_SECRET` and `DATABASE_URL`: no fallback defaults. Fail at startup if missing.
- Secrets never in docker-compose, scripts, or source — use `${VAR}` references
- Cookie `secure` flag driven by runtime config, never hardcoded `false`
- Rate limit all auth endpoints before production exposure
- RBAC: every privileged handler checks `claims.role` before executing
- Refresh tokens: hash (SHA-256) before storing in DB
- Confirmation codes: use `OsRng` (CSPRNG), minimum 8 alphanumeric chars
- Never expose internal errors in API responses

## Imports

Order (enforced by `cargo fmt`):
1. `std` / `core`
2. External crates (`axum`, `diesel`, `serde`, etc.)
3. `crate::` imports

Group related imports in a single `use` block with braces:

```rust
use crate::{
    application::use_cases::LoginUseCase,
    domain::repositories::AuthRepository,
    shared::utils::jwt::JwtManager,
};
```
