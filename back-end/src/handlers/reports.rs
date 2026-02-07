use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::models::report::{ClearReportRequest, CreateReportRequest, ReportResponse, NearbyReportsQuery};
use crate::services::report_service::ReportService;
use crate::services::scoring_service::ScoringService;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ReportHandlerState {
    pub report_service: ReportService,
    pub scoring_service: ScoringService,
}

/// Create a new litter report
/// POST /api/reports
#[utoipa::path(
    post,
    path = "/api/reports",
    tag = "Reports",
    request_body = CreateReportRequest,
    responses(
        (status = 201, description = "Report created successfully", body = ReportResponse),
        (status = 400, description = "Invalid input or image"),
        (status = 403, description = "Email verification required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_report(
    State(state): State<Arc<ReportHandlerState>>,
    auth_user: AuthUser,
    Json(request): Json<CreateReportRequest>,
) -> Result<impl IntoResponse, AppError> {
    let report = state
        .report_service
        .create_report(auth_user.id, request)
        .await?;

    let response: ReportResponse = report.into();
    Ok((StatusCode::CREATED, Json(response)))
}

/// Get nearby reports
/// GET /api/reports/nearby?latitude=X&longitude=Y&radius_km=Z
#[utoipa::path(
    get,
    path = "/api/reports/nearby",
    tag = "Reports",
    params(
        NearbyReportsQuery
    ),
    responses(
        (status = 200, description = "Returns reports within radius", body = Vec<ReportResponse>),
        (status = 400, description = "Invalid coordinates")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_nearby_reports(
    State(state): State<Arc<ReportHandlerState>>,
    _auth_user: AuthUser,
    Query(query): Query<NearbyReportsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Default to 5km radius if not specified
    let radius = query.radius_km.unwrap_or(5.0);

    let reports = state
        .report_service
        .get_nearby_reports(query.latitude, query.longitude, radius)
        .await?;

    let responses: Vec<ReportResponse> = reports.into_iter().map(|r| r.into()).collect();
    Ok(Json(responses))
}

/// Get a single report by ID
/// GET /api/reports/:id
#[utoipa::path(
    get,
    path = "/api/reports/{id}",
    tag = "Reports",
    params(
        ("id" = Uuid, Path, description = "Report ID")
    ),
    responses(
        (status = 200, description = "Returns report details", body = ReportResponse),
        (status = 404, description = "Report not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_report(
    State(state): State<Arc<ReportHandlerState>>,
    _auth_user: AuthUser,
    Path(report_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let report = state.report_service.get_report_by_id(report_id).await?;
    let response: ReportResponse = report.into();
    Ok(Json(response))
}

/// Claim a report for cleanup
/// POST /api/reports/:id/claim
#[utoipa::path(
    post,
    path = "/api/reports/{id}/claim",
    tag = "Reports",
    params(
        ("id" = Uuid, Path, description = "Report ID")
    ),
    responses(
        (status = 200, description = "Report claimed successfully", body = ReportResponse),
        (status = 404, description = "Report not found"),
        (status = 400, description = "Report already claimed or not in pending status")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn claim_report(
    State(state): State<Arc<ReportHandlerState>>,
    auth_user: AuthUser,
    Path(report_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let report = state.report_service.claim_report(report_id, auth_user.id).await?;
    let response: ReportResponse = report.into();
    Ok(Json(response))
}

/// Clear a report with after photo
/// POST /api/reports/:id/clear
#[utoipa::path(
    post,
    path = "/api/reports/{id}/clear",
    tag = "Reports",
    request_body = ClearReportRequest,
    params(
        ("id" = Uuid, Path, description = "Report ID")
    ),
    responses(
        (status = 200, description = "Report cleared successfully. Points awarded.", body = ReportResponse),
        (status = 404, description = "Report not found"),
        (status = 400, description = "Report not claimed by you or invalid status")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn clear_report(
    State(state): State<Arc<ReportHandlerState>>,
    auth_user: AuthUser,
    Path(report_id): Path<Uuid>,
    Json(request): Json<ClearReportRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Clear the report
    let report = state
        .report_service
        .clear_report(report_id, auth_user.id, request.photo_base64)
        .await?;

    // Award points to the user
    state
        .scoring_service
        .award_clear_points(auth_user.id, report_id, report.latitude, report.longitude)
        .await?;

    let response: ReportResponse = report.into();
    Ok(Json(response))
}

/// Get all reports created by the current user
/// GET /api/reports/my-reports
#[utoipa::path(
    get,
    path = "/api/reports/my-reports",
    tag = "Reports",
    responses(
        (status = 200, description = "Returns user's reports", body = Vec<ReportResponse>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_my_reports(
    State(state): State<Arc<ReportHandlerState>>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let reports = state.report_service.get_user_reports(auth_user.id).await?;
    let responses: Vec<ReportResponse> = reports.into_iter().map(|r| r.into()).collect();
    Ok(Json(responses))
}

/// Get all reports cleared by the current user
/// GET /api/reports/my-clears
#[utoipa::path(
    get,
    path = "/api/reports/my-clears",
    tag = "Reports",
    responses(
        (status = 200, description = "Returns user's cleared reports", body = Vec<ReportResponse>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_my_cleared_reports(
    State(state): State<Arc<ReportHandlerState>>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let reports = state
        .report_service
        .get_user_cleared_reports(auth_user.id)
        .await?;
    let responses: Vec<ReportResponse> = reports.into_iter().map(|r| r.into()).collect();
    Ok(Json(responses))
}
