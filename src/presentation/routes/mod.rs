pub mod auth;
pub mod health;
pub mod users;

pub use auth::create_auth_routes;
pub use health::health_routes;
pub use users::user_routes;

use crate::infrastructure::SystemMonitor;
use crate::{
    application::{
        dto::auth::{
            AuthResponse, ForgotPasswordRequest, LoginRequest, LogoutRequest, RefreshTokenRequest,
            RegisterRequest, ResendConfirmCodeRequest, SetPasswordRequest, UserInfo,
            VerifyEmailRequest,
        },
        use_cases::{
            ForgotPasswordUseCase, LoginUseCase, LogoutUseCase, RegisterUseCase,
            SetPasswordUseCase, VerifyEmailUseCase,
        },
    },
    infrastructure::database::{repositories::AuthRepositoryImpl, DbPool},
    presentation::responses::{
        AuthResponseWrapper, ErrorResponseWrapper, StringResponseWrapper, UserListResponseWrapper,
        UserResponseWrapper,
    },
    shared::utils::jwt::JwtManager,
};
use axum::Router;
use axum::{routing::get, Extension};
use axum_prometheus::{metrics_exporter_prometheus::PrometheusHandle, PrometheusMetricLayer};
use std::sync::Arc;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::presentation::routes::health::health_check,
        crate::presentation::handlers::auth::register,
        crate::presentation::handlers::auth::login,
        crate::presentation::handlers::auth::logout,
        crate::presentation::handlers::auth::verify_email,
        crate::presentation::handlers::auth::set_password,
        crate::presentation::handlers::auth::forgot_password,
        crate::presentation::handlers::auth::resend_code,
        crate::presentation::handlers::user::create_user,
        crate::presentation::handlers::user::get_user,
        crate::presentation::handlers::user::list_users,
        crate::presentation::handlers::user::update_user,
        crate::presentation::handlers::user::import_users,
        crate::presentation::handlers::role::get_user_role,
        crate::presentation::handlers::role::update_user_role,
    ),
    components(
        schemas(
            RegisterRequest,
            LoginRequest,
            LogoutRequest,
            ForgotPasswordRequest,
            ResendConfirmCodeRequest,
            RefreshTokenRequest,
            VerifyEmailRequest,
            SetPasswordRequest,
            AuthResponse,
            UserInfo,
            crate::application::dto::user::CreateUserDto,
            crate::application::dto::user::UpdateUserDto,
            crate::application::dto::user::UserResponseDto,
            crate::application::dto::role_dto::UpdateRoleRequest,
            crate::application::dto::role_dto::RoleResponse,
            crate::application::dto::role_dto::RolePermissions,
            crate::presentation::handlers::user::ListUsersQuery,
            AuthResponseWrapper,
            StringResponseWrapper,
            ErrorResponseWrapper,
            UserResponseWrapper,
            UserListResponseWrapper,
            crate::presentation::responses::RoleResponseWrapper,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "health", description = "System health endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "roles", description = "Role management endpoints")
    ),
    info(
        title = "Axum Backend API",
        version = "0.1.0",
        description = "A robust backend API built with Axum following DDD principles."
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt_token",
                SecurityScheme::Http(
                    HttpBuilder::new().scheme(HttpAuthScheme::Bearer).bearer_format("JWT").build(),
                ),
            )
        }
    }
}

/// Create the main application router
pub fn create_router(
    pool: DbPool,
    jwt_secret: String,
    jwt_access_expiry: i64,
    jwt_refresh_expiry: i64,
    jwt_issuer: String,
    jwt_audience: String,
    confirm_code_expiry: i64,
    prometheus_layer: PrometheusMetricLayer<'static>,
    metric_handle: PrometheusHandle,
    email_service: Arc<dyn crate::application::services::email::EmailService>,
) -> Router {
    // Create repositories
    let auth_repo = Arc::new(AuthRepositoryImpl::new(pool.clone()));

    // Create JWT manager
    let jwt_manager = Arc::new(JwtManager::new(
        jwt_secret,
        jwt_access_expiry,
        jwt_refresh_expiry,
        jwt_issuer,
        jwt_audience,
    ));

    // Create use cases
    let register_uc = Arc::new(RegisterUseCase::new(
        auth_repo.clone(),
        email_service.clone(),
        confirm_code_expiry,
    ));
    let login_uc = Arc::new(LoginUseCase::new(auth_repo.clone(), jwt_manager.clone()));
    let logout_uc = Arc::new(LogoutUseCase::new(auth_repo.clone()));
    let verify_uc = Arc::new(VerifyEmailUseCase::new(auth_repo.clone()));
    let set_password_uc = Arc::new(SetPasswordUseCase::new(auth_repo.clone()));
    let forgot_password_uc = Arc::new(ForgotPasswordUseCase::new(
        auth_repo.clone(),
        email_service.clone(),
        confirm_code_expiry,
    ));

    // Monitoring Setup
    let system_monitor = Arc::new(SystemMonitor::new());

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(health_routes())
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .route(
            "/api/admin/system",
            get(crate::presentation::handlers::monitoring::system_health),
        )
        .nest(
            "/api/auth",
            create_auth_routes(
                register_uc,
                login_uc,
                logout_uc,
                verify_uc,
                set_password_uc,
                forgot_password_uc,
                Arc::new(crate::application::use_cases::ResendConfirmCodeUseCase::new(
                    auth_repo.clone(),
                    email_service.clone(),
                    confirm_code_expiry,
                )),
                jwt_manager.clone(),
            ),
        )
        .nest("/api/users", user_routes(pool, auth_repo, jwt_manager))
        .layer(prometheus_layer)
        .layer(Extension(system_monitor))
}
