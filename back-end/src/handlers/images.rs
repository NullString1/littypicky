use crate::error::AppError;
use crate::services::report_service::ReportService;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use base64::{engine::general_purpose, Engine as _};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ImageHandlerState {
    pub report_service: ReportService,
}

/// Get report before photo
/// GET /api/images/reports/:id/before
#[utoipa::path(
    get,
    path = "/api/images/reports/{id}/before",
    tag = "Images",
    params(
        ("id" = Uuid, Path, description = "Report ID")
    ),
    responses(
        (status = 200, description = "Returns image", content_type = "image/jpeg"),
        (status = 404, description = "Report or image not found")
    )
)]
pub async fn get_report_before_photo(
    State(state): State<Arc<ImageHandlerState>>,
    Path(report_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let report = state.report_service.get_report_by_id(report_id).await?;
    
    // Extract base64 data from data URL (e.g., "data:image/jpeg;base64,...")
    let base64_data = if report.photo_before.starts_with("data:") {
        report.photo_before
            .split_once(",")
            .map(|(_, data)| data)
            .unwrap_or(&report.photo_before)
    } else {
        &report.photo_before
    };
    
    // Decode base64
    let image_data = general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|_| AppError::BadRequest("Invalid image data".into()))?;
    
    // Detect content type from data URL
    let content_type = if report.photo_before.starts_with("data:image/jpeg") {
        "image/jpeg"
    } else if report.photo_before.starts_with("data:image/png") {
        "image/png"
    } else if report.photo_before.starts_with("data:image/webp") {
        "image/webp"
    } else {
        "image/jpeg" // default
    };
    
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type), (header::CACHE_CONTROL, "public, max-age=86400")],
        image_data,
    ))
}

/// Get report after photo
/// GET /api/images/reports/:id/after
#[utoipa::path(
    get,
    path = "/api/images/reports/{id}/after",
    tag = "Images",
    params(
        ("id" = Uuid, Path, description = "Report ID")
    ),
    responses(
        (status = 200, description = "Returns image", content_type = "image/jpeg"),
        (status = 404, description = "Report or image not found")
    )
)]
pub async fn get_report_after_photo(
    State(state): State<Arc<ImageHandlerState>>,
    Path(report_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let report = state.report_service.get_report_by_id(report_id).await?;
    
    let photo_after = report.photo_after
        .ok_or_else(|| AppError::NotFound("After photo not found".into()))?;
    
    // Extract base64 data from data URL
    let base64_data = if photo_after.starts_with("data:") {
        photo_after
            .split_once(",")
            .map(|(_, data)| data)
            .unwrap_or(&photo_after)
    } else {
        &photo_after
    };
    
    // Decode base64
    let image_data = general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|_| AppError::BadRequest("Invalid image data".into()))?;
    
    // Detect content type
    let content_type = if photo_after.starts_with("data:image/jpeg") {
        "image/jpeg"
    } else if photo_after.starts_with("data:image/png") {
        "image/png"
    } else if photo_after.starts_with("data:image/webp") {
        "image/webp"
    } else {
        "image/jpeg"
    };
    
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type), (header::CACHE_CONTROL, "public, max-age=86400")],
        image_data,
    ))
}
