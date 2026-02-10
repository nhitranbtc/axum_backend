use crate::{
    application::{
        dto::{RoleResponse, UpdateRoleRequest},
        use_cases::user::role_management::{
            GetRoleError, GetUserRoleUseCase, UpdateRoleError, UpdateUserRoleUseCase,
        },
    },
    domain::repositories::user_repository::UserRepository,
    presentation::responses::ApiResponse,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

/// Get user role by ID
#[utoipa::path(
    get,
    path = "/api/users/{id}/role",
    tag = "roles",
    params(
        ("id" = String, Path, description = "User ID (UUID)")
    ),
    responses(
        (status = 200, description = "User role retrieved successfully", body = RoleResponseWrapper),
        (status = 400, description = "Invalid user ID", body = ErrorResponseWrapper),
        (status = 404, description = "User not found", body = ErrorResponseWrapper),
        (status = 401, description = "Unauthorized", body = ErrorResponseWrapper)
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get_user_role<R: UserRepository + 'static>(
    State(use_case): State<Arc<GetUserRoleUseCase<R>>>,
    Path(user_id): Path<String>,
) -> Result<Json<ApiResponse<RoleResponse>>, RoleApiError> {
    let role_response = use_case.execute(&user_id).await?;
    Ok(Json(ApiResponse::success(role_response)))
}

/// Update user role by ID
#[utoipa::path(
    put,
    path = "/api/users/{id}/role",
    tag = "roles",
    params(
        ("id" = String, Path, description = "User ID (UUID)")
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "User role updated successfully", body = RoleResponseWrapper),
        (status = 400, description = "Invalid user ID or role", body = ErrorResponseWrapper),
        (status = 404, description = "User not found", body = ErrorResponseWrapper),
        (status = 401, description = "Unauthorized", body = ErrorResponseWrapper)
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn update_user_role<R: UserRepository + 'static>(
    State(use_case): State<Arc<UpdateUserRoleUseCase<R>>>,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<Json<ApiResponse<RoleResponse>>, RoleApiError> {
    let role_response = use_case.execute(&user_id, &payload.role).await?;
    Ok(Json(ApiResponse::success(role_response)))
}

/// Error type for role API handlers
#[derive(Debug)]
pub enum RoleApiError {
    InvalidUserId,
    InvalidRole(String),
    UserNotFound,
    Repository(String),
}

impl From<GetRoleError> for RoleApiError {
    fn from(err: GetRoleError) -> Self {
        match err {
            GetRoleError::InvalidUserId => RoleApiError::InvalidUserId,
            GetRoleError::UserNotFound => RoleApiError::UserNotFound,
            GetRoleError::Repository(msg) => RoleApiError::Repository(msg),
        }
    }
}

impl From<UpdateRoleError> for RoleApiError {
    fn from(err: UpdateRoleError) -> Self {
        match err {
            UpdateRoleError::InvalidUserId => RoleApiError::InvalidUserId,
            UpdateRoleError::InvalidRole(role) => RoleApiError::InvalidRole(role),
            UpdateRoleError::UserNotFound => RoleApiError::UserNotFound,
            UpdateRoleError::Repository(msg) => RoleApiError::Repository(msg),
        }
    }
}

impl IntoResponse for RoleApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            RoleApiError::InvalidUserId => (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()),
            RoleApiError::InvalidRole(role) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid role: '{}'. Must be 'admin', 'editor', or 'viewer'", role),
            ),
            RoleApiError::UserNotFound => (StatusCode::NOT_FOUND, "User not found".to_string()),
            RoleApiError::Repository(msg) => {
                tracing::error!("Repository error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            },
        };

        (status, Json(ApiResponse::<()>::error(message))).into_response()
    }
}
