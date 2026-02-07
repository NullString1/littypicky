// User profile handlers - to be implemented

use crate::{auth::AuthUser, error::Result, models::UserResponse};
use axum::{Extension, Json};

pub async fn get_current_user(
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<UserResponse>> {
    // TODO: Fetch full user from database
    Ok(Json(UserResponse {
        id: auth_user.id,
        email: auth_user.email,
        full_name: "TODO".to_string(),
        city: "TODO".to_string(),
        country: "TODO".to_string(),
        search_radius_km: 5,
        role: auth_user.role,
        email_verified: true,
        created_at: chrono::Utc::now(),
    }))
}
