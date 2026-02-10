use crate::{
    application::use_cases::{
        ForgotPasswordUseCase, LoginUseCase, LogoutUseCase, RegisterUseCase, SetPasswordUseCase,
        VerifyEmailUseCase,
    },
    domain::repositories::AuthRepository,
    presentation::handlers::auth,
};
use axum::{middleware, routing::post, Router};
use std::sync::Arc;

use crate::presentation::middleware::auth::{auth_middleware, AuthState};

pub fn create_auth_routes<R: AuthRepository + 'static>(
    register_uc: Arc<RegisterUseCase<R>>,
    login_uc: Arc<LoginUseCase<R>>,
    logout_uc: Arc<LogoutUseCase<R>>,
    verify_uc: Arc<VerifyEmailUseCase<R>>,
    set_password_uc: Arc<SetPasswordUseCase<R>>,
    forgot_password_uc: Arc<ForgotPasswordUseCase<R>>,
    resend_code_uc: Arc<crate::application::use_cases::ResendConfirmCodeUseCase<R>>,
    jwt_manager: Arc<crate::shared::utils::jwt::JwtManager>,
) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/register", post(auth::register::<R>))
        .with_state(register_uc)
        .route("/login", post(auth::login::<R>))
        .with_state(login_uc)
        .route("/verify", post(auth::verify_email::<R>))
        .with_state(verify_uc)
        .route("/password", post(auth::set_password::<R>))
        .with_state(set_password_uc)
        .route("/forgot-password", post(auth::forgot_password::<R>))
        .with_state(forgot_password_uc)
        .route("/resend-code", post(auth::resend_code::<R>))
        .with_state(resend_code_uc);

    // Create auth state for middleware
    let auth_state = AuthState { jwt_manager };

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/logout", post(auth::logout::<R>))
        .with_state(logout_uc.clone())
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware));

    // Combine routes
    Router::new().merge(public_routes).merge(protected_routes)
}
