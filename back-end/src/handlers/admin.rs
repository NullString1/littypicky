use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::models::user::{User, UserResponse};
use crate::models::ReportStatus;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone)]
pub struct AdminHandlerState {
    pub pool: PgPool,
}

#[derive(Deserialize, ToSchema)]
pub struct ListUsersQuery {
    #[schema(example = 1)]
    pub page: Option<i64>,
    #[schema(example = 20)]
    pub limit: Option<i64>,
}

#[derive(Serialize, FromRow, ToSchema)]
pub struct AdminReportView {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub description: Option<String>,
    pub status: ReportStatus,
    pub claimed_by: Option<Uuid>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub cleared_by: Option<Uuid>,
    pub cleared_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub reporter_name: String,
    pub reporter_email: String,
}

/// Get all users (paginated)
/// GET /api/admin/users?page=1&limit=20
#[utoipa::path(
    get,
    path = "/api/admin/users",
    tag = "Admin",
    responses(
        (status = 200, description = "Returns list of users", body = Vec<UserResponse>),
        (status = 403, description = "Admin access required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_users(
    State(state): State<Arc<AdminHandlerState>>,
    _auth_user: AuthUser, // Verified by require_admin middleware
) -> Result<impl IntoResponse, AppError> {
    let users = sqlx::query_as::<_, User>(
        r"
        SELECT * FROM users
        ORDER BY created_at DESC
        LIMIT 100
        ",
    )
    .fetch_all(&state.pool)
    .await?;

    let user_responses: Vec<UserResponse> =
        users.into_iter().map(std::convert::Into::into).collect();

    Ok(Json(user_responses))
}

/// Get user by ID
/// GET /api/admin/users/:id
#[utoipa::path(
    get,
    path = "/api/admin/users/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Returns user details", body = UserResponse),
        (status = 404, description = "User not found"),
        (status = 403, description = "Admin access required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_by_id(
    State(state): State<Arc<AdminHandlerState>>,
    Path(user_id): Path<Uuid>,
    _auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(UserResponse::from(user)))
}

/// Ban/unban a user
/// PUT /api/admin/users/:id/ban
#[derive(Deserialize, ToSchema)]
pub struct BanUserRequest {
    #[schema(example = false)]
    pub is_active: bool,
}

#[utoipa::path(
    put,
    path = "/api/admin/users/{id}/ban",
    tag = "Admin",
    request_body = BanUserRequest,
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User ban status updated", body = UserResponse),
        (status = 404, description = "User not found"),
        (status = 403, description = "Admin access required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn toggle_user_ban(
    State(state): State<Arc<AdminHandlerState>>,
    Path(user_id): Path<Uuid>,
    _auth_user: AuthUser,
    Json(payload): Json<BanUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET is_active = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
    )
    .bind(payload.is_active)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(serde_json::json!({
        "message": if payload.is_active { "User unbanned" } else { "User banned" },
        "user": UserResponse::from(user)
    })))
}

/// Get all reports (not just nearby)
/// GET /api/admin/reports
#[utoipa::path(
    get,
    path = "/api/admin/reports",
    tag = "Admin",
    responses(
        (status = 200, description = "Returns all reports", body = Vec<AdminReportView>),
        (status = 403, description = "Admin access required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_all_reports(
    State(state): State<Arc<AdminHandlerState>>,
    _auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let reports = sqlx::query_as::<_, AdminReportView>(
        r"
        SELECT 
            lr.id,
            lr.reporter_id,
            ST_Y(lr.location)::double precision as latitude,
            ST_X(lr.location)::double precision as longitude,
            lr.description,
            lr.status,
            lr.claimed_by,
            lr.claimed_at,
            lr.cleared_by,
            lr.cleared_at,
            lr.created_at,
            u.full_name as reporter_name,
            u.email as reporter_email
        FROM litter_reports lr
        JOIN users u ON lr.reporter_id = u.id
        ORDER BY lr.created_at DESC
        LIMIT 100
        ",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(reports))
}

/// Delete a report (for spam/inappropriate content)
/// DELETE /api/admin/reports/:id
#[utoipa::path(
    delete,
    path = "/api/admin/reports/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "Report ID")
    ),
    responses(
        (status = 200, description = "Report deleted"),
        (status = 404, description = "Report not found"),
        (status = 403, description = "Admin access required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_report(
    State(state): State<Arc<AdminHandlerState>>,
    Path(report_id): Path<Uuid>,
    _auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query!("DELETE FROM litter_reports WHERE id = $1", report_id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Report not found".to_string()));
    }

    Ok(Json(serde_json::json!({
        "message": "Report deleted successfully"
    })))
}
