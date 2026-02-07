use crate::{
    auth::JwtService,
    error::{AppError, Result},
    models::UserRole,
};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
}

pub async fn require_auth(
    State(jwt_service): State<JwtService>,
    mut req: Request,
    next: Next,
) -> Result<Response> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    let claims = jwt_service.verify_token(token)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("Invalid user ID in token".to_string()))?;

    let role = match claims.role.as_str() {
        "admin" => UserRole::Admin,
        "user" => UserRole::User,
        _ => return Err(AppError::Auth("Invalid role in token".to_string())),
    };

    let auth_user = AuthUser {
        id: user_id,
        email: claims.email,
        role,
    };

    req.extensions_mut().insert(auth_user);

    Ok(next.run(req).await)
}

pub async fn require_admin(
    req: Request,
    next: Next,
) -> Result<Response> {
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(AppError::Unauthorized)?
        .clone();

    match auth_user.role {
        UserRole::Admin => Ok(next.run(req).await),
        _ => Err(AppError::Forbidden("Admin access required".to_string())),
    }
}
