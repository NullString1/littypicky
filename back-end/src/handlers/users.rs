use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::models::user::{User, UserResponse, UserRole, UpdateUserRequest};
use axum::{extract::State, response::IntoResponse, Json};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(FromRow, Serialize, ToSchema)]
pub struct UserScoreRecord {
    pub total_points: i32,
    pub total_reports: i32,
    pub total_clears: i32,
    pub total_verifications: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub last_clear_date: Option<NaiveDate>,
}

#[derive(Clone)]
pub struct UserHandlerState {
    pub pool: PgPool,
}

/// Get current authenticated user's profile
/// GET /api/users/me
#[utoipa::path(
    get,
    path = "/api/users/me",
    tag = "Users",
    responses(
        (status = 200, description = "Returns user profile", body = UserResponse),
        (status = 404, description = "User not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

/// Update current user's profile
/// PATCH /api/users/me
#[utoipa::path(
    patch,
    path = "/api/users/me",
    tag = "Users",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = UserResponse),
        (status = 400, description = "Invalid parameters")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_current_user(
    State(state): State<Arc<UserHandlerState>>,
    auth_user: AuthUser,
    Json(update): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Build dynamic query based on what fields are being updated
    let mut query = String::from("UPDATE users SET updated_at = NOW()");
    let mut param_count = 1;
    
    if update.full_name.is_some() {
        param_count += 1;
        query.push_str(&format!(", full_name = ${}", param_count));
    }
    if update.city.is_some() {
        param_count += 1;
        query.push_str(&format!(", city = ${}", param_count));
    }
    if update.country.is_some() {
        param_count += 1;
        query.push_str(&format!(", country = ${}", param_count));
    }
    if update.search_radius_km.is_some() {
        param_count += 1;
        query.push_str(&format!(", search_radius_km = ${}", param_count));
    }
    
    query.push_str(" WHERE id = $1 RETURNING id, email, password_hash, full_name, city, country, search_radius_km, role, is_active, email_verified, email_verified_at, oauth_provider, oauth_subject, created_at, updated_at");
    
    // Build the query dynamically
    let mut query_builder = sqlx::query_as::<_, User>(&query).bind(auth_user.id);
    
    if let Some(name) = update.full_name {
        query_builder = query_builder.bind(name);
    }
    if let Some(city) = update.city {
        query_builder = query_builder.bind(city);
    }
    if let Some(country) = update.country {
        query_builder = query_builder.bind(country);
    }
    if let Some(radius) = update.search_radius_km {
        if radius < 1 || radius > 100 {
            return Err(AppError::BadRequest("Search radius must be between 1 and 100 km".to_string()));
        }
        query_builder = query_builder.bind(radius);
    }
    
    let user = query_builder
        .fetch_one(&state.pool)
        .await?;

    let response: UserResponse = user.into();
    Ok(Json(response))
}

/// Get user's score and statistics
/// GET /api/users/me/score
#[utoipa::path(
    get,
    path = "/api/users/me/score",
    tag = "Users",
    responses(
        (status = 200, description = "Returns user statistics and score", body = UserScoreRecord),
        (status = 404, description = "Score not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_current_user_score(
    State(state): State<Arc<UserHandlerState>>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let score = sqlx::query_as::<_, UserScoreRecord>(
        r#"
        SELECT total_points, total_reports, total_clears, total_verifications,
               current_streak, longest_streak, last_cleared_date as last_clear_date
        FROM user_scores
        WHERE user_id = $1
        "#
    )
    .bind(auth_user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Score not found".to_string()))?;

    Ok(Json(score))
}
