use crate::{
    application::{
        dto::auth::{
            AuthResponse, ForgotPasswordRequest, LoginRequest, LogoutRequest, RegisterRequest,
            RegisterResponse, SetPasswordRequest, VerifyEmailRequest,
        },
        use_cases::{
            ForgotPasswordUseCase, LoginUseCase, LogoutUseCase, RegisterUseCase,
            SetPasswordUseCase, VerifyEmailUseCase,
        },
    },
    domain::repositories::AuthRepository,
    infrastructure::cache::CacheRepository,
    presentation::responses::ApiResponse,
    shared::utils::jwt::Claims,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use std::sync::Arc;
use time::Duration;
use validator::Validate;

#[derive(Debug)]
pub enum AuthError {
    ValidationError(String),
    RegisterError(String),
    LoginError(String),
    LogoutError(String),
    Unauthorized(String),
    VerifyEmailError(String),
    SetPasswordError(String),
    ForgotPasswordError(String),
    ResendCodeError(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AuthError::RegisterError(msg) => (StatusCode::BAD_REQUEST, msg),
            AuthError::LoginError(msg) => (StatusCode::UNAUTHORIZED, msg),
            AuthError::LogoutError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AuthError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AuthError::VerifyEmailError(msg) => (StatusCode::BAD_REQUEST, msg),
            AuthError::SetPasswordError(msg) => (StatusCode::BAD_REQUEST, msg),
            AuthError::ForgotPasswordError(msg) => (StatusCode::BAD_REQUEST, msg),
            AuthError::ResendCodeError(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = Json(serde_json::json!({
            "success": false,
            "error": message,
        }));

        (status, body).into_response()
    }
}

/// Register a new user
#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponseWrapper),
        (status = 400, description = "Validation error or registration failed", body = ErrorResponseWrapper)
    ),
    tag = "auth"
)]
pub async fn register<R: AuthRepository, C: CacheRepository + ?Sized>(
    State(use_case): State<Arc<RegisterUseCase<R, C>>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<ApiResponse<RegisterResponse>>), AuthError> {
    // Validate input
    payload.validate().map_err(|e| AuthError::ValidationError(e.to_string()))?;

    // Execute use case
    let response = use_case
        .execute(payload.email, payload.name)
        .await
        .map_err(|e| AuthError::RegisterError(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

/// Login user
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "User logged in successfully", body = AuthResponseWrapper),
        (status = 401, description = "Invalid credentials", body = ErrorResponseWrapper)
    ),
    tag = "auth"
)]
pub async fn login<R: AuthRepository>(
    State(use_case): State<Arc<LoginUseCase<R>>>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<ApiResponse<AuthResponse>>), AuthError> {
    // Validate input
    payload.validate().map_err(|e| AuthError::ValidationError(e.to_string()))?;

    // Execute use case
    let response = use_case
        .execute(payload.email, payload.password, payload.code)
        .await
        .map_err(|e| AuthError::LoginError(e.to_string()))?;

    // Set HttpOnly cookies
    let access_cookie = Cookie::build(("access_token", response.access_token.clone()))
        .http_only(true)
        .path("/")
        .same_site(SameSite::Lax)
        .secure(false) // Set to true in production
        .max_age(Duration::seconds(response.expires_in))
        .build();

    let refresh_cookie = Cookie::build(("refresh_token", response.refresh_token.clone()))
        .http_only(true)
        .path("/")
        .same_site(SameSite::Lax)
        .secure(false) // Set to true in production
        .max_age(Duration::days(7))
        .build();

    let jar = jar.add(access_cookie).add(refresh_cookie);
    Ok((jar, Json(ApiResponse::success(response))))
}

/// Logout user (revoke refresh token)
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Logged out successfully", body = StringResponseWrapper),
        (status = 401, description = "Unauthorized", body = ErrorResponseWrapper),
        (status = 500, description = "Logout failed", body = ErrorResponseWrapper)
    ),
    tag = "auth",
    security(
        ("jwt_token" = [])
    )
)]
pub async fn logout<R: AuthRepository>(
    State(use_case): State<Arc<LogoutUseCase<R>>>,
    jar: CookieJar,
    claims: Claims,
    Json(payload): Json<LogoutRequest>,
) -> Result<(CookieJar, Json<ApiResponse<String>>), AuthError> {
    let user_id = claims
        .sub
        .parse()
        .map_err(|_| AuthError::Unauthorized("Invalid user ID".to_string()))?;

    // Determine refresh token from payload or cookie
    let refresh_token = if let Some(rt) = &payload.refresh_token {
        Some(rt.clone())
    } else {
        jar.get("refresh_token").map(|c| c.value().to_string())
    };

    if payload.logout_all {
        // Logout from all devices
        use_case
            .execute_all(user_id)
            .await
            .map_err(|e| AuthError::LogoutError(e.to_string()))?;
    } else if let Some(token) = refresh_token {
        // Logout from current device
        use_case
            .execute(&token)
            .await
            .map_err(|e| AuthError::LogoutError(e.to_string()))?;
    } else {
        return Err(AuthError::ValidationError(
            "Either refresh_token or logout_all must be provided".to_string(),
        ));
    }

    // Clear cookies by setting expired cookies
    let access_cookie = Cookie::build(("access_token", ""))
        .http_only(true)
        .path("/")
        .max_age(Duration::seconds(-1))
        .build();

    let refresh_cookie = Cookie::build(("refresh_token", ""))
        .http_only(true)
        .path("/")
        .max_age(Duration::seconds(-1))
        .build();

    let jar = jar.add(access_cookie).add(refresh_cookie);

    Ok((jar, Json(ApiResponse::success("Logged out successfully".to_string()))))
}

/// Verify email
#[utoipa::path(
    post,
    path = "/api/auth/verify",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "Email verified successfully", body = StringResponseWrapper),
        (status = 400, description = "Verification failed", body = ErrorResponseWrapper)
    ),
    tag = "auth"
)]
pub async fn verify_email<R: AuthRepository>(
    State(use_case): State<Arc<VerifyEmailUseCase<R>>>,
    Json(payload): Json<VerifyEmailRequest>,
) -> Result<Json<ApiResponse<String>>, AuthError> {
    // Validate input
    payload.validate().map_err(|e| AuthError::ValidationError(e.to_string()))?;

    // Execute use case
    let message = use_case
        .execute(payload.email, payload.code)
        .await
        .map_err(|e| AuthError::VerifyEmailError(e.to_string()))?;

    Ok(Json(ApiResponse::success(message)))
}

/// Set password
#[utoipa::path(
    post,
    path = "/api/auth/password",
    request_body = SetPasswordRequest,
    responses(
        (status = 200, description = "Password set successfully", body = StringResponseWrapper),
        (status = 400, description = "Failed to set password", body = ErrorResponseWrapper)
    ),
    tag = "auth"
)]
pub async fn set_password<R: AuthRepository>(
    State(use_case): State<Arc<SetPasswordUseCase<R>>>,
    Json(payload): Json<SetPasswordRequest>,
) -> Result<Json<ApiResponse<String>>, AuthError> {
    // Validate input
    payload.validate().map_err(|e| AuthError::ValidationError(e.to_string()))?;

    // Execute use case
    let message = use_case
        .execute(payload.email, payload.code, payload.password)
        .await
        .map_err(|e| AuthError::SetPasswordError(e.to_string()))?;

    Ok(Json(ApiResponse::success(message)))
}

/// Forgot password
#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Confirmation code sent", body = StringResponseWrapper),
        (status = 400, description = "Invalid email or user not found", body = ErrorResponseWrapper)
    ),
    tag = "auth"
)]
pub async fn forgot_password<R: AuthRepository>(
    State(use_case): State<Arc<ForgotPasswordUseCase<R>>>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Json<ApiResponse<String>>, AuthError> {
    // Validate input
    payload.validate().map_err(|e| AuthError::ValidationError(e.to_string()))?;

    // Execute use case
    let message = use_case
        .execute(payload.email)
        .await
        .map_err(|e| AuthError::ForgotPasswordError(e.to_string()))?;

    Ok(Json(ApiResponse::success(message)))
}

/// Resend Confirmation Code
#[utoipa::path(
    post,
    path = "/api/auth/resend-code",
    request_body = ResendConfirmCodeRequest,
    responses(
        (status = 200, description = "Confirmation code resent", body = StringResponseWrapper),
        (status = 400, description = "Invalid email or already verified", body = ErrorResponseWrapper)
    ),
    tag = "auth"
)]
pub async fn resend_code<R: AuthRepository>(
    State(use_case): State<Arc<crate::application::use_cases::ResendConfirmCodeUseCase<R>>>,
    Json(payload): Json<crate::application::dto::auth::ResendConfirmCodeRequest>,
) -> Result<Json<ApiResponse<String>>, AuthError> {
    // Validate input
    payload.validate().map_err(|e| AuthError::ValidationError(e.to_string()))?;

    // Execute use case
    let message = use_case
        .execute(payload.email)
        .await
        .map_err(|e| AuthError::ResendCodeError(e.to_string()))?;

    Ok(Json(ApiResponse::success(message)))
}
