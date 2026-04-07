---
name: architecture
description: DDD layer inventory — entities, value objects, repos, use cases, commands, queries, handlers, infrastructure
type: project
---

# Architecture Deep Dive

## Domain Layer (src/domain/)

### Entities
- **User** (`entities/user.rs`) — id, email, name, password_hash, role, is_active, is_email_verified, confirmation_code, timestamps
  - `new(email, name)` → unverified user; `from_existing(...)` → reconstruct from DB
  - `set_confirmation_code()`, `verify_email()`, `set_password()`, `update_name()`, `update_email()`
- **RefreshToken** (`entities/refresh_token.rs`) — id, user_id, token_hash, expires_at, revoked_at
  - `new()`, `is_valid()`, `revoke()`

### Value Objects
- **Email** (`value_objects/email.rs`) — parse constructor validates @ and length, normalizes lowercase
- **UserId** (`value_objects/user_id.rs`) — newtype over Uuid, `new()`, `from_uuid()`, `from_string()`, Copy
- **UserRole** (`value_objects/user_role.rs`) — enum Admin/Editor/Viewer with `can_read/write/delete()`, Default=Viewer

### Repository Traits
- **UserRepository** (`repositories/user.rs`) — save, update, find_by_id, find_by_email, exists_by_email, count, list_paginated, delete, delete_all
- **AuthRepository** (`repositories/auth.rs`) — find_by_email, create_user, update_last_login, update_user, save/find/revoke refresh tokens, cleanup_expired_tokens
  - Has `#[cfg_attr(test, mockall::automock)]`

### Errors
- **DomainError** — InvalidEmail, InvalidName, InvalidUserData (thiserror)
- **RepositoryError** — Database, NotFound, DuplicateEmail, Internal (+ From<diesel::result::Error>)
- **AuthRepositoryError** — DatabaseError, UserNotFound, TokenNotFound, EmailAlreadyExists

### Re-exports (domain/mod.rs)
- `User`, `DomainError`, `UserRepository`, `Email`, `UserId`
- Deprecated aliases: `auth_repository`, `user_repository`

---

## Application Layer (src/application/)

### Commands (CQRS — new writes)
- `commands/user/create.rs` — CreateUserCommand<R: UserRepository>
- `commands/user/update.rs` — UpdateUserCommand<R: UserRepository> (takes UserId, not String)

### Queries (CQRS — new reads)
- `queries/user/get.rs` — GetUserQuery<R: UserRepository> (takes UserId)
- `queries/user/list.rs` — ListUsersQuery<R: UserRepository> → (Vec<User>, i64 count); UserFilters struct (not yet wired)
- `queries/user/statistics.rs` — UserStatisticsQuery<R: UserRepository> → UserStatistics (mostly placeholders returning 0)

### Use Cases (legacy — do NOT add new files here)
- **Auth** (`use_cases/auth/`):
  - RegisterUseCase — creates user + sends confirmation email
  - LoginUseCase — password OR code auth, returns JWT pair
  - LogoutUseCase — single session or all sessions
  - VerifyEmailUseCase — validates code, activates user
  - SetPasswordUseCase — validates reset code, hashes password (spawn_blocking)
  - ForgotPasswordUseCase — generates reset code, sends email
  - ResendConfirmCodeUseCase — resends confirmation email
- **User** (`use_cases/user/`): create, get, list, import, update, roles (GetUserRoleUseCase, UpdateUserRoleUseCase)
- **Admin** (`use_cases/admin/`): empty module

### DTOs
- **Auth**: RegisterRequest, LoginRequest, VerifyEmailRequest, SetPasswordRequest, LogoutRequest, ForgotPasswordRequest, ResendConfirmCodeRequest, RegisterResponse, AuthResponse, UserInfo
- **User**: CreateUserDto, UpdateUserDto, UserResponseDto (From<User>)
- **Role**: UpdateRoleRequest, RoleResponse, RolePermissions

### Services
- `services/auth.rs` — AuthService: token pair creation, refresh token storage/verification/revocation
- `services/user.rs` — UserService: user_exists_by_email, get_user_by_id/email, can_delete_user, get_user_count (returns 0!)
- `services/email.rs` — EmailService trait (Send+Sync, automock): send(recipient, email_type)
  - EmailType: Welcome, Confirmation(code), PasswordReset(code)

### Actors
- `actors/import.rs` — UserCreationActor (ractor): one-shot actor per CSV record, checks duplicate then creates user

---

## Presentation Layer (src/presentation/)

### Routes
- `/health` — GET health_check
- `/metrics` — GET prometheus metrics (inline)
- `/api/admin/system` — GET system_health (Extension<SystemMonitor>)
- `/api/auth/register` — POST (public)
- `/api/auth/login` — POST (public)
- `/api/auth/verify` — POST (public)
- `/api/auth/password` — POST (public)
- `/api/auth/forgot-password` — POST (public)
- `/api/auth/resend-code` — POST (public)
- `/api/auth/logout` — POST (auth required)
- `/api/users/` — POST create, GET list (auth required)
- `/api/users/import` — POST CSV import (auth required)
- `/api/users/:id` — GET get, PUT update (auth required)
- `/api/users/:id/role` — GET get_role, PUT update_role (auth required)

### Handlers
- `handlers/auth.rs` — 7 handlers; AuthError enum maps to HTTP status codes; login sets HttpOnly cookies
- `handlers/user.rs` — 5 handlers; ListUsersQuery pagination (page default=1, page_size default=10)
- `handlers/role.rs` — 2 handlers; RoleApiError (InvalidUserId→400, InvalidRole→400, UserNotFound→404, Repository→500)
- `handlers/monitoring.rs` — system_health via Extension<SystemMonitor>

### Middleware
- `middleware/auth.rs` — JWT auth: checks Authorization Bearer header then access_token cookie; inserts Claims into extensions
  - AuthMiddlewareError: MissingToken, InvalidTokenFormat, InvalidToken, InvalidTokenType (all 401)
  - Claims FromRequestParts extractor

### Responses
- `responses/mod.rs` — ApiResponse<T> { success, data?, error? }; 7 concrete wrappers for OpenAPI schema

### OpenAPI/Swagger
- ApiDoc struct in routes/mod.rs with utoipa
- Security scheme: Bearer JWT ("jwt_token")
- Tags: auth, health, users, roles
- Available at `/swagger-ui/` (feature flag `swagger`)

---

## Infrastructure Layer (src/infrastructure/)

### Database
- `database/connection.rs` — create_pool(config, url), run_migrations(url) (spawn_blocking)
- `database/schema.rs` — auto-generated Diesel schema (users, refresh_tokens)
- `database/transaction.rs` — transaction helpers
- `database/models/user.rs` — UserModel (Queryable/Insertable/AsChangeset); touch() updates updated_at
- `database/models/auth.rs` — RefreshTokenModel (Queryable/Insertable); is_valid(), revoke()
- `database/models/common.rs` — Timestamped, SoftDeletable, HasUuid traits
- `database/repositories/user.rs` — UserRepositoryImpl: model_to_entity/entity_to_model conversion; upsert via ON CONFLICT
- `database/repositories/auth.rs` — AuthRepositoryImpl: user + refresh token operations; creates inactive users by default

### Email
- `email/lettre_service.rs` — LettreEmailService: SMTP via SMTP_HOST/USER/PASS/FROM env vars; TLS for non-localhost
- `email/noop_service.rs` — NoopEmailService: logs only (dev/test)
- `email/templates.rs` — Askama templates: WelcomeTemplate, ConfirmationTemplate, ForgotPasswordTemplate

### Cache
- `cache/mod.rs` — placeholder ("To be implemented when needed")

### External APIs
- `external_apis/mod.rs` — placeholder ("To be implemented when needed")

### Monitoring
- `monitoring.rs` — SystemMonitor (sysinfo): cpu_usage, total/used_memory, uptime → SystemMetrics

---

## Shared Layer (src/shared/)
- `utils/jwt.rs` — JwtManager: HS256, Claims {sub, exp, iat, jti, token_type, iss, aud}; create_access/refresh_token, verify_token
- `utils/password.rs` — PasswordManager: Argon2 hash/verify (static methods); PasswordError
- `utils/mod.rs` — now() → DateTime<Utc>, is_valid_email()
- `errors/mod.rs` — AppError: Database→500, NotFound→404, Validation→400, Unauthorized→401, Forbidden→403, Internal→500, Config→500
- `telemetry/mod.rs` — init_telemetry(): tracing-subscriber with EnvFilter (RUST_LOG default "info,axum_backend=debug")

---

## Config Layer (src/config/)
- **AppConfig**: database_url (required), server_host/port, jwt_secret/expiry/issuer/audience, confirm_code_expiry, rust_log, db_config
  - jwt_secret default: "dev-secret-change-in-production" (NOTE: CLAUDE.md says no fallback — mismatch)
- **DatabaseConfig**: max_connections=10, min_connections=2, connect_timeout=30s, idle_timeout=600s, max_lifetime=1800s
  - Creates deadpool Pool<AsyncPgConnection> with Tokio1 runtime
