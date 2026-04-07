---
name: api
description: API endpoint inventory with routes, auth requirements, and handler mappings
type: project
---

# API Endpoint Inventory

## Public Endpoints (no auth)
| Method | Path | Handler | Use Case |
|--------|------|---------|----------|
| GET | /health | health_check | Health check |
| POST | /api/auth/register | auth::register | RegisterUseCase |
| POST | /api/auth/login | auth::login | LoginUseCase |
| POST | /api/auth/verify | auth::verify_email | VerifyEmailUseCase |
| POST | /api/auth/password | auth::set_password | SetPasswordUseCase |
| POST | /api/auth/forgot-password | auth::forgot_password | ForgotPasswordUseCase |
| POST | /api/auth/resend-code | auth::resend_code | ResendCodeUseCase |

## Authenticated Endpoints (JWT required)
| Method | Path | Handler | Use Case |
|--------|------|---------|----------|
| POST | /api/auth/logout | auth::logout | LogoutUseCase |
| POST | /api/users/ | user::create_user | CreateUserUseCase |
| GET | /api/users/ | user::list_users | ListUsersUseCase |
| POST | /api/users/import | user::import_users | ImportUsersUseCase |
| GET | /api/users/:id | user::get_user | GetUserUseCase |
| PUT | /api/users/:id | user::update_user | UpdateUserUseCase |
| GET | /api/users/:id/role | role::get_user_role | GetUserRoleUseCase |
| PUT | /api/users/:id/role | role::update_user_role | UpdateUserRoleUseCase |

## Internal/Monitoring
| Method | Path | Handler | Notes |
|--------|------|---------|-------|
| GET | /metrics | inline closure | Prometheus metrics (axum-prometheus) |
| GET | /system-health | monitoring::system_health | System info (sysinfo) |

## Auth Flow
1. Register → creates inactive user with confirmation code → sends email
2. Verify email → activates user
3. Set password → stores Argon2 hash
4. Login → returns JWT access + refresh tokens
5. Refresh → exchange refresh token for new access token
6. Logout → revokes refresh token

## Swagger
- Available at `/swagger-ui/` when `swagger` feature is enabled (default)
- Uses utoipa 4 + utoipa-swagger-ui 7

## Database Schema
### users
- id (UUID PK), email (unique), name, password_hash, role (varchar 20)
- is_active, email_verified, confirmation_code, confirmation_code_expires_at
- last_login, created_at, updated_at
- Indexes: idx_users_email (unique), idx_users_role, idx_users_is_active

### refresh_tokens
- id (UUID PK), user_id (FK → users ON DELETE CASCADE), token_hash (unique)
- expires_at, created_at, revoked_at
- Indexes: idx_refresh_tokens_user_id
