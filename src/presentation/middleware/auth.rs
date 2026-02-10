use crate::shared::utils::jwt::{Claims, JwtManager};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthState {
    pub jwt_manager: Arc<JwtManager>,
}

pub async fn auth_middleware(
    State(state): State<AuthState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AuthMiddlewareError> {
    let jwt_manager = &state.jwt_manager;
    let (mut parts, body) = req.into_parts();

    // 1. Try Authorization header
    let token = if let Some(auth_header) = parts.headers.get(header::AUTHORIZATION) {
        auth_header
            .to_str()
            .ok()
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|t| t.to_string())
    } else {
        None
    };

    // 2. If no header, try Cookie
    let token = if let Some(t) = token {
        Some(t)
    } else {
        parts.headers.get(header::COOKIE).and_then(|c| c.to_str().ok()).and_then(|c| {
            c.split(';')
                .find_map(|s| s.trim().strip_prefix("access_token=").map(|token| token.to_string()))
        })
    };

    let token = token.ok_or(AuthMiddlewareError::MissingToken)?;

    // Verify token
    let claims = jwt_manager
        .verify_token(&token)
        .map_err(|e| AuthMiddlewareError::InvalidToken(e.to_string()))?;

    // Check if it's an access token
    if claims.token_type != "access" {
        return Err(AuthMiddlewareError::InvalidTokenType);
    }

    // Insert claims into request extensions for handlers to use
    parts.extensions.insert(claims);

    let req = Request::from_parts(parts, body);
    Ok(next.run(req).await)
}

#[derive(Debug)]
pub enum AuthMiddlewareError {
    MissingToken,
    InvalidTokenFormat,
    InvalidToken(String),
    InvalidTokenType,
}

impl IntoResponse for AuthMiddlewareError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthMiddlewareError::MissingToken => {
                (StatusCode::UNAUTHORIZED, "Missing authorization token")
            },
            AuthMiddlewareError::InvalidTokenFormat => {
                (StatusCode::UNAUTHORIZED, "Invalid token format. Expected: Bearer <token>")
            },
            AuthMiddlewareError::InvalidToken(_) => {
                (StatusCode::UNAUTHORIZED, "Invalid or expired token")
            },
            AuthMiddlewareError::InvalidTokenType => {
                (StatusCode::UNAUTHORIZED, "Invalid token type. Expected access token")
            },
        };

        let body = Json(serde_json::json!({
            "success": false,
            "error": message,
        }));

        (status, body).into_response()
    }
}

// Extractor for Claims from request extensions
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized: No claims found".to_string()))
    }
}
