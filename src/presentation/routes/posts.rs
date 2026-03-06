use crate::presentation::middleware::auth::{auth_middleware, AuthState};
use crate::{
    application::use_cases::post::{
        CreatePostUseCase, DeletePostUseCase, GetPostUseCase, ListPostsUseCase, UpdatePostUseCase,
    },
    infrastructure::cache::CacheRepository,
    infrastructure::database::{DbPool, PostRepositoryImpl, UserRepositoryImpl},
    presentation::handlers::post::{create_post, delete_post, get_post, list_posts, update_post},
    shared::utils::jwt::JwtManager,
};
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;

/// Create post-related routes
pub fn post_routes(
    pool: DbPool,
    jwt_manager: Arc<JwtManager>,
    cache_repository: Arc<dyn CacheRepository>,
) -> Router {
    let post_repo = Arc::new(PostRepositoryImpl::new(pool.clone()));
    let user_repo = Arc::new(UserRepositoryImpl::new(pool));

    let create_post_uc = Arc::new(CreatePostUseCase::new(post_repo.clone()));
    let get_post_uc = Arc::new(GetPostUseCase::new(post_repo.clone(), cache_repository.clone()));
    let list_posts_uc = Arc::new(ListPostsUseCase::new(post_repo.clone()));
    let update_post_uc = Arc::new(UpdatePostUseCase::new(
        post_repo.clone(),
        user_repo.clone(),
        cache_repository.clone(),
    ));
    let delete_post_uc = Arc::new(DeletePostUseCase::new(post_repo, user_repo, cache_repository));

    let auth_state = AuthState { jwt_manager };

    Router::new()
        .route("/", post(create_post).with_state(create_post_uc))
        .route("/", get(list_posts).with_state(list_posts_uc))
        .route("/:id", get(get_post).with_state(get_post_uc))
        .route("/:id", put(update_post).with_state(update_post_uc))
        .route("/:id", axum::routing::delete(delete_post).with_state(delete_post_uc))
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware))
}
