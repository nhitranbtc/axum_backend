use crate::{
    application::{
        dto::{CreatePostDto, ListPostsQueryDto, PostResponseDto, UpdatePostDto},
        use_cases::post::{
            CreatePostUseCase, DeletePostUseCase, GetPostUseCase, ListPostsUseCase,
            UpdatePostUseCase,
        },
    },
    domain::repositories::{post::PostRepository, user_repository::UserRepository},
    infrastructure::cache::CacheRepository,
    presentation::responses::ApiResponse,
    shared::{utils::jwt::Claims, AppError},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// Create a new post
#[utoipa::path(
    post,
    path = "/api/posts",
    request_body = CreatePostDto,
    responses(
        (status = 201, description = "Post created successfully", body = PostResponseWrapper),
        (status = 400, description = "Invalid input", body = ErrorResponseWrapper),
        (status = 401, description = "Unauthorized", body = ErrorResponseWrapper)
    ),
    tag = "posts",
    security(
        ("jwt_token" = [])
    )
)]
pub async fn create_post<R: PostRepository>(
    State(use_case): State<Arc<CreatePostUseCase<R>>>,
    claims: Claims,
    Json(payload): Json<CreatePostDto>,
) -> Result<impl IntoResponse, AppError> {
    let author_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token subject".to_string()))?;

    let post = use_case.execute(author_id, payload).await?;
    let response = PostResponseDto::from(post);

    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

/// Get post by ID
#[utoipa::path(
    get,
    path = "/api/posts/{id}",
    responses(
        (status = 200, description = "Post found", body = PostResponseWrapper),
        (status = 404, description = "Post not found", body = ErrorResponseWrapper)
    ),
    params(
        ("id" = String, Path, description = "Post ID")
    ),
    tag = "posts",
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get_post<R: PostRepository, C: CacheRepository + ?Sized>(
    State(use_case): State<Arc<GetPostUseCase<R, C>>>,
    Path(post_id): Path<String>,
) -> Result<Json<ApiResponse<PostResponseDto>>, AppError> {
    let post = use_case.execute(&post_id).await?;
    Ok(Json(ApiResponse::success(PostResponseDto::from(post))))
}

/// List posts
#[utoipa::path(
    get,
    path = "/api/posts",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default 1)"),
        ("page_size" = Option<i64>, Query, description = "Page size (default 10, max 100)"),
        ("status" = Option<String>, Query, description = "Filter by status")
    ),
    responses(
        (status = 200, description = "Posts listed successfully", body = PostListResponseWrapper),
        (status = 400, description = "Invalid query", body = ErrorResponseWrapper)
    ),
    tag = "posts",
    security(
        ("jwt_token" = [])
    )
)]
pub async fn list_posts<R: PostRepository>(
    State(use_case): State<Arc<ListPostsUseCase<R>>>,
    Query(query): Query<ListPostsQueryDto>,
) -> Result<Json<ApiResponse<crate::application::dto::PostListResponseDto>>, AppError> {
    let result = use_case.execute(query.page, query.page_size, query.status.as_deref()).await?;
    Ok(Json(ApiResponse::success(result)))
}

/// Update post by ID
#[utoipa::path(
    put,
    path = "/api/posts/{id}",
    request_body = UpdatePostDto,
    responses(
        (status = 200, description = "Post updated successfully", body = PostResponseWrapper),
        (status = 403, description = "Forbidden", body = ErrorResponseWrapper),
        (status = 404, description = "Post not found", body = ErrorResponseWrapper)
    ),
    params(
        ("id" = String, Path, description = "Post ID")
    ),
    tag = "posts",
    security(
        ("jwt_token" = [])
    )
)]
pub async fn update_post<PR: PostRepository, UR: UserRepository, C: CacheRepository + ?Sized>(
    State(use_case): State<Arc<UpdatePostUseCase<PR, UR, C>>>,
    claims: Claims,
    Path(post_id): Path<String>,
    Json(payload): Json<UpdatePostDto>,
) -> Result<Json<ApiResponse<PostResponseDto>>, AppError> {
    let actor_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token subject".to_string()))?;
    let post = use_case.execute(actor_id, &post_id, payload).await?;
    Ok(Json(ApiResponse::success(PostResponseDto::from(post))))
}

/// Soft delete post by ID
#[utoipa::path(
    delete,
    path = "/api/posts/{id}",
    responses(
        (status = 200, description = "Post deleted successfully", body = StringResponseWrapper),
        (status = 403, description = "Forbidden", body = ErrorResponseWrapper),
        (status = 404, description = "Post not found", body = ErrorResponseWrapper)
    ),
    params(
        ("id" = String, Path, description = "Post ID")
    ),
    tag = "posts",
    security(
        ("jwt_token" = [])
    )
)]
pub async fn delete_post<PR: PostRepository, UR: UserRepository, C: CacheRepository + ?Sized>(
    State(use_case): State<Arc<DeletePostUseCase<PR, UR, C>>>,
    claims: Claims,
    Path(post_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let actor_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token subject".to_string()))?;
    use_case.execute(actor_id, &post_id).await?;
    Ok(Json(ApiResponse::success("Post deleted successfully".to_string())))
}
