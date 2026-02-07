use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::models::user::{User, UserResponse, UserRole};
use axum::{extract::State, response::IntoResponse, Json};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserHandlerState {
    pub pool: PgPool,
}

/// Get current authenticated user's profile
/// GET /api/users/me
pub async fn get_current_user(
    State(state): State<Arc<UserHandlerState>>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, full_name, city, country,
               search_radius_km, role as "role: UserRole", is_active,
               email_verified, email_verified_at, oauth_provider,
               oauth_subject, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        auth_user.id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let response: UserResponse = user.into();
    Ok(Json(response))
}
