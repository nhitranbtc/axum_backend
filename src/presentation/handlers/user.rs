use crate::{
    application::{
        dto::{CreateUserDto, UpdateUserDto, UserResponseDto},
        use_cases::{
            CreateUserUseCase, GetUserUseCase, ImportUsersUseCase, ListUsersUseCase,
            UpdateUserUseCase,
        },
    },
    domain::repositories::{user_repository::UserRepository, AuthRepository},
    presentation::responses::ApiResponse,
    shared::AppError,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

// ... (keep existing code)

/// Import users from CSV
#[utoipa::path(
    post,
    path = "/api/users/import",
    responses(
        (status = 200, description = "Users imported successfully", body = StringResponseWrapper),
        (status = 500, description = "Internal server error", body = ErrorResponseWrapper)
    ),
    tag = "users",
    security(
        ("jwt_token" = [])
    )
)]
pub async fn import_users<R: AuthRepository>(
    State(use_case): State<Arc<ImportUsersUseCase<R>>>,
) -> Result<impl IntoResponse, AppError> {
    let csv_path = "import/users.csv";
    let csv_data = std::fs::read(csv_path)
        .map_err(|e| AppError::Config(format!("Failed to read CSV file: {}", e)))?;

    let count = use_case
        .execute(&csv_data)
        .await
        .map_err(|e| AppError::Validation(e.to_string()))?;

    Ok(Json(ApiResponse::success(format!("Successfully imported {} users", count))))
}

/// Query parameters for listing users
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct ListUsersQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/api/users",
    request_body = CreateUserDto,
    responses(
        (status = 201, description = "User created successfully", body = UserResponseWrapper),
        (status = 400, description = "Invalid input", body = ErrorResponseWrapper)
    ),
    tag = "users",
    security(
        ("jwt_token" = [])
    )
)]
pub async fn create_user<R: UserRepository>(
    State(use_case): State<Arc<CreateUserUseCase<R>>>,
    Json(payload): Json<CreateUserDto>,
) -> Result<impl IntoResponse, AppError> {
    let user = use_case.execute(payload).await?;
    let response = UserResponseDto::from(user);

    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

/// Get user by ID
#[utoipa::path(
    get,
    path = "/api/users/{id}",
    responses(
        (status = 200, description = "User found", body = UserResponseWrapper),
        (status = 404, description = "User not found", body = ErrorResponseWrapper)
    ),
    params(
        ("id" = String, Path, description = "User ID")
    ),
    tag = "users",
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get_user<R: UserRepository>(
    State(use_case): State<Arc<GetUserUseCase<R>>>,
    Path(user_id): Path<String>,
) -> Result<Json<ApiResponse<UserResponseDto>>, AppError> {
    let user = use_case.execute(&user_id).await?;
    let response = UserResponseDto::from(user);

    Ok(Json(ApiResponse::success(response)))
}

/// List users with pagination
#[utoipa::path(
    get,
    path = "/api/users",
    params(
        ListUsersQuery
    ),
    responses(
        (status = 200, description = "Users list", body = UserListResponseWrapper)
    ),
    tag = "users",
    security(
        ("jwt_token" = [])
    )
)]
pub async fn list_users<R: UserRepository>(
    State(use_case): State<Arc<ListUsersUseCase<R>>>,
    Query(params): Query<ListUsersQuery>,
) -> Result<Json<ApiResponse<Vec<UserResponseDto>>>, AppError> {
    let users = use_case.execute(params.page, params.page_size).await?;
    let response: Vec<UserResponseDto> = users.iter().map(UserResponseDto::from).collect();

    Ok(Json(ApiResponse::success(response)))
}

/// Update user
#[utoipa::path(
    put,
    path = "/api/users/{id}",
    request_body = UpdateUserDto,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponseWrapper),
        (status = 404, description = "User not found", body = ErrorResponseWrapper)
    ),
    params(
        ("id" = String, Path, description = "User ID")
    ),
    tag = "users",
    security(
        ("jwt_token" = [])
    )
)]
pub async fn update_user<R: UserRepository>(
    State(use_case): State<Arc<UpdateUserUseCase<R>>>,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateUserDto>,
) -> Result<Json<ApiResponse<UserResponseDto>>, AppError> {
    let user = use_case.execute(&user_id, payload).await?;
    let response = UserResponseDto::from(user);

    Ok(Json(ApiResponse::success(response)))
}
