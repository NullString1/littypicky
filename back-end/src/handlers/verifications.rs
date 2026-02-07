use crate::auth::middleware::AuthUser;
use crate::config::ScoringConfig;
use crate::error::AppError;
use crate::models::report::ReportStatus;
use crate::models::verification::{CreateVerificationRequest, ReportVerification, VerificationResponse};
use crate::services::report_service::ReportService;
use crate::services::scoring_service::ScoringService;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use sqlx::PgPool;

#[derive(Clone)]
pub struct VerificationHandlerState {
    pub pool: PgPool,
    pub report_service: ReportService,
    pub scoring_service: ScoringService,
    pub scoring_config: ScoringConfig,
}

/// Verify a cleared report
/// POST /api/reports/:id/verify
pub async fn verify_report(
    State(state): State<Arc<VerificationHandlerState>>,
    auth_user: AuthUser,
    Path(report_id): Path<Uuid>,
    Json(request): Json<CreateVerificationRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check if user can verify reports (has cleared enough)
    let can_verify = state.scoring_service.can_verify_reports(auth_user.id).await?;
    if !can_verify {
        return Err(AppError::Forbidden(format!(
            "You need to clear at least {} reports before you can verify others",
            state.scoring_config.min_clears_to_verify
        )));
    }

    // Get the report
    let report = state.report_service.get_report_by_id(report_id).await?;

    // Check report status
    if report.status != ReportStatus::Cleared {
        return Err(AppError::BadRequest(
            "Report must be cleared before it can be verified".to_string(),
        ));
    }

    // Check user is not the reporter or clearer
    if report.reporter_id == auth_user.id {
        return Err(AppError::BadRequest(
            "You cannot verify your own report".to_string(),
        ));
    }

    if report.cleared_by == Some(auth_user.id) {
        return Err(AppError::BadRequest(
            "You cannot verify a report you cleared".to_string(),
        ));
    }

    // Check if user has already verified this report
    let existing = sqlx::query!(
        "SELECT id FROM report_verifications WHERE report_id = $1 AND verifier_id = $2",
        report_id,
        auth_user.id
    )
    .fetch_optional(&state.pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "You have already verified this report".to_string(),
        ));
    }

    // Create the verification
    let verification = sqlx::query_as!(
        ReportVerification,
        r#"
        INSERT INTO report_verifications (report_id, verifier_id, is_verified, comment)
        VALUES ($1, $2, $3, $4)
        RETURNING id, report_id, verifier_id, is_verified, comment, created_at
        "#,
        report_id,
        auth_user.id,
        request.is_verified,
        request.comment
    )
    .fetch_one(&state.pool)
    .await?;

    // Award points to the verifier
    state.scoring_service.award_verification_points(auth_user.id).await?;

    // Check if we have enough positive verifications to mark report as verified
    if request.is_verified {
        let positive_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM report_verifications WHERE report_id = $1 AND is_verified = true",
            report_id
        )
        .fetch_one(&state.pool)
        .await?
        .unwrap_or(0);

        if positive_count >= state.scoring_config.min_verifications_needed as i64 {
            // Update report to verified status
            sqlx::query!(
                r#"UPDATE litter_reports SET status = $1 WHERE id = $2"#,
                ReportStatus::Verified as ReportStatus,
                report_id
            )
            .execute(&state.pool)
            .await?;

            // Award bonus points to the clearer
            if let Some(clearer_id) = report.cleared_by {
                state
                    .scoring_service
                    .award_verified_report_bonus(clearer_id)
                    .await?;
            }
        }
    }

    let response: VerificationResponse = verification.into();
    Ok((StatusCode::CREATED, Json(response)))
}

/// Get all verifications for a report
/// GET /api/reports/:id/verifications
pub async fn get_report_verifications(
    State(state): State<Arc<VerificationHandlerState>>,
    _auth_user: AuthUser,
    Path(report_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Verify report exists
    state.report_service.get_report_by_id(report_id).await?;

    let verifications = sqlx::query_as!(
        ReportVerification,
        r#"
        SELECT id, report_id, verifier_id, is_verified, comment, created_at
        FROM report_verifications
        WHERE report_id = $1
        ORDER BY created_at DESC
        "#,
        report_id
    )
    .fetch_all(&state.pool)
    .await?;

    let responses: Vec<VerificationResponse> = verifications.into_iter().map(|v| v.into()).collect();
    Ok(Json(responses))
}
