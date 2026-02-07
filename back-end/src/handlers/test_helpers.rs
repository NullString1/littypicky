use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sqlx::PgPool;
use crate::{
    error::AppError,
    services::AuthService,
};

#[derive(Clone)]
pub struct TestHelperState {
    pub pool: PgPool,
    pub auth_service: Arc<AuthService>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TestHelperResponse {
    pub success: bool,
    pub message: String,
}

/// Verify an email address for testing purposes
/// This bypasses the normal email verification flow
/// 
/// **WARNING: This endpoint should ONLY be enabled in test/development environments**
#[utoipa::path(
    post,
    path = "/api/test/verify-email/{email}",
    tag = "test-helpers",
    params(
        ("email" = String, Path, description = "Email address to verify")
    ),
    responses(
        (status = 200, description = "Email verified successfully", body = TestHelperResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn verify_email_for_testing(
    State(state): State<Arc<TestHelperState>>,
    Path(email): Path<String>,
) -> Result<Json<TestHelperResponse>, AppError> {
    let mut tx = state.pool.begin().await?;

    // Update the user's email_verified field to true
    let result = sqlx::query!(
        r#"
        UPDATE users 
        SET email_verified = true, email_verified_at = NOW()
        WHERE email = $1
        RETURNING id
        "#,
        email
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(user) = result {
        // Delete any existing email verification tokens for this user
        sqlx::query!(
            "DELETE FROM email_verification_tokens WHERE user_id = $1",
            user.id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(Json(TestHelperResponse {
            success: true,
            message: format!("Email {} verified successfully", email),
        }))
    } else {
        Err(AppError::NotFound(format!("User with email {} not found", email)))
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CleanupRequest {
    pub email: String,
}

/// Clean up all test data for a specific user
/// Deletes the user and all associated data (reports, verifications, etc.)
/// 
/// **WARNING: This endpoint should ONLY be enabled in test/development environments**
#[utoipa::path(
    delete,
    path = "/api/test/cleanup",
    tag = "test-helpers",
    request_body = CleanupRequest,
    responses(
        (status = 200, description = "Test data cleaned up successfully", body = TestHelperResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn cleanup_test_data(
    State(state): State<Arc<TestHelperState>>,
    Json(payload): Json<CleanupRequest>,
) -> Result<Json<TestHelperResponse>, AppError> {
    let mut tx = state.pool.begin().await?;

    // Get user ID
    let user = sqlx::query!(
        "SELECT id FROM users WHERE email = $1",
        payload.email
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(user) = user {
        let user_id = user.id;

        // Delete verifications by this user
        sqlx::query!("DELETE FROM report_verifications WHERE verifier_id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        // Delete verifications for reports created by this user
        sqlx::query!(
            "DELETE FROM report_verifications WHERE report_id IN (SELECT id FROM litter_reports WHERE reporter_id = $1 OR cleared_by = $1)",
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // Delete reports created or cleared by this user
        sqlx::query!(
            "DELETE FROM litter_reports WHERE reporter_id = $1 OR cleared_by = $1",
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // Delete email verification tokens
        sqlx::query!("DELETE FROM email_verification_tokens WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        // Delete password reset tokens
        sqlx::query!("DELETE FROM password_reset_tokens WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        // Delete refresh tokens
        sqlx::query!("DELETE FROM refresh_tokens WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        // Delete the user
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(Json(TestHelperResponse {
            success: true,
            message: format!("Successfully cleaned up data for user {}", payload.email),
        }))
    } else {
        // User doesn't exist, but that's okay for cleanup
        Ok(Json(TestHelperResponse {
            success: true,
            message: format!("No data found for user {}", payload.email),
        }))
    }
}

/// Get the current test environment status
#[utoipa::path(
    get,
    path = "/api/test/status",
    tag = "test-helpers",
    responses(
        (status = 200, description = "Test helpers are enabled"),
    )
)]
pub async fn test_status() -> Json<TestHelperResponse> {
    Json(TestHelperResponse {
        success: true,
        message: "Test helpers are enabled".to_string(),
    })
}
