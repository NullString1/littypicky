use crate::error::AppError;
use crate::services::report_service::ReportService;
use crate::services::s3_service::S3Service;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ImageHandlerState {
    pub report_service: ReportService,
    pub s3_service: S3Service,
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
        (status = 200, description = "Returns image", content_type = "image/webp"),
        (status = 404, description = "Report or image not found")
    )
)]
pub async fn get_report_before_photo(
    State(state): State<Arc<ImageHandlerState>>,
    Path(report_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let report = state.report_service.get_report_by_id(report_id).await?;
    
    // Get photo URL
    let photo_url = report.photo_before
        .ok_or_else(|| AppError::NotFound("No before photo found".to_string()))?;
    
    // Extract S3 key from URL
    let key = state.s3_service.extract_key_from_url(report.photo_before.as_ref().ok_or_else(|| AppError::NotFound("Before photo not found".into()))?)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Invalid S3 URL")))?;
    
    // Get image data from S3
    let image_data = state.s3_service.get_image(&key).await?;
    
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/webp"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
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
        (status = 200, description = "Returns image", content_type = "image/webp"),
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
    
    // Extract S3 key from URL
    let key = state.s3_service.extract_key_from_url(&photo_after)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Invalid S3 URL")))?;
    
    // Get image data from S3
    let image_data = state.s3_service.get_image(&key).await?;
    
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/webp"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        image_data,
    ))
}
