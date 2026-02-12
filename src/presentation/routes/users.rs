use crate::presentation::middleware::auth::{auth_middleware, AuthState};
use crate::{
    application::use_cases::{
        CreateUserUseCase, GetUserRoleUseCase, GetUserUseCase, ImportUsersUseCase,
        ListUsersUseCase, UpdateUserRoleUseCase, UpdateUserUseCase,
    },
    infrastructure::cache::CacheRepository,
    infrastructure::database::repositories::{AuthRepositoryImpl, UserRepositoryImpl},
    infrastructure::database::DbPool,
    presentation::{
        handlers::role::{get_user_role, update_user_role},
        handlers::user::{create_user, get_user, import_users, list_users, update_user},
    },
    shared::utils::jwt::JwtManager,
};
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;

/// Create user-related routes
pub fn user_routes(
    pool: DbPool,
    auth_repo: Arc<AuthRepositoryImpl>,
    jwt_manager: Arc<JwtManager>,
    cache_repository: Arc<dyn CacheRepository>,
) -> Router {
    // Create repository
    let user_repo = Arc::new(UserRepositoryImpl::new(pool));

    // Create use cases
    let create_user_uc =
        Arc::new(CreateUserUseCase::new(user_repo.clone(), cache_repository.clone()));
    let get_user_uc = Arc::new(GetUserUseCase::new(user_repo.clone(), cache_repository.clone()));
    let list_users_uc =
        Arc::new(ListUsersUseCase::new(user_repo.clone(), cache_repository.clone()));
    let update_user_uc =
        Arc::new(UpdateUserUseCase::new(user_repo.clone(), cache_repository.clone()));
    let delete_user_uc = Arc::new(crate::application::use_cases::DeleteUserUseCase::new(
        user_repo.clone(),
        cache_repository.clone(),
    ));
    let import_users_uc = Arc::new(ImportUsersUseCase::new(auth_repo.clone()));

    // Role management use cases
    let get_role_uc = Arc::new(GetUserRoleUseCase::new(user_repo.clone()));
    let update_role_uc = Arc::new(UpdateUserRoleUseCase::new(user_repo.clone()));

    // Create auth state for middleware
    let auth_state = AuthState { jwt_manager };

    Router::new()
        .route("/", post(create_user).with_state(create_user_uc))
        .route("/", get(list_users).with_state(list_users_uc))
        .route("/import", post(import_users).with_state(import_users_uc))
        .route("/:id", get(get_user).with_state(get_user_uc))
        .route("/:id", put(update_user).with_state(update_user_uc))
        .route(
            "/:id",
            axum::routing::delete(crate::presentation::handlers::user::delete_user)
                .with_state(delete_user_uc),
        )
        // Role management endpoints
        .route("/:id/role", get(get_user_role).with_state(get_role_uc))
        .route("/:id/role", put(update_user_role).with_state(update_role_uc))
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware))
}
