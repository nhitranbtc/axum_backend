use crate::application::dto::{
    auth::{AuthResponse, RegisterResponse},
    user::UserResponseDto,
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;

/// Standard API response wrapper
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// Documentation-only concrete response schemas to fix generic resolution issues
#[derive(ToSchema)]
pub struct AuthResponseWrapper {
    pub success: bool,
    pub data: Option<AuthResponse>,
    pub error: Option<String>,
}

#[derive(ToSchema)]
pub struct RegisterResponseWrapper {
    pub success: bool,
    pub data: Option<RegisterResponse>,
    pub error: Option<String>,
}

#[derive(ToSchema)]
pub struct UserResponseWrapper {
    pub success: bool,
    pub data: Option<UserResponseDto>,
    pub error: Option<String>,
}

#[derive(ToSchema)]
pub struct UserListResponseWrapper {
    pub success: bool,
    pub data: Option<Vec<UserResponseDto>>,
    pub error: Option<String>,
}

#[derive(ToSchema)]
pub struct StringResponseWrapper {
    pub success: bool,
    pub data: Option<String>,
    pub error: Option<String>,
}

#[derive(ToSchema)]
pub struct RoleResponseWrapper {
    pub success: bool,
    pub data: Option<crate::application::dto::RoleResponse>,
    pub error: Option<String>,
}

#[derive(ToSchema)]
pub struct ErrorResponseWrapper {
    pub success: bool,
    pub data: Option<()>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    pub fn error(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse { success: false, data: None, error: Some(message.into()) }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = if self.success { StatusCode::OK } else { StatusCode::BAD_REQUEST };

        (status, Json(self)).into_response()
    }
}
