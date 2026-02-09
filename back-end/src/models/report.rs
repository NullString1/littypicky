use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, ToSchema)]
#[sqlx(type_name = "report_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ReportStatus {
    Pending,
    Claimed,
    Cleared,
    Verified,
}

#[derive(Debug, Clone, FromRow, ToSchema)]
pub struct LitterReport {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub description: Option<String>,
    pub photo_before: Option<String>,
    pub status: ReportStatus,
    pub claimed_by: Option<Uuid>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub cleared_by: Option<Uuid>,
    pub cleared_at: Option<DateTime<Utc>>,
    pub photo_after: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReportResponse {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub description: Option<String>,
    pub photo_before: Option<String>,
    pub status: ReportStatus,
    pub claimed_by: Option<Uuid>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub cleared_by: Option<Uuid>,
    pub cleared_at: Option<DateTime<Utc>>,
    pub photo_after: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<LitterReport> for ReportResponse {
    fn from(report: LitterReport) -> Self {
        ReportResponse {
            id: report.id,
            reporter_id: report.reporter_id,
            latitude: report.latitude,
            longitude: report.longitude,
            description: report.description,
            // Return S3 URL directly (or None if not set)
            photo_before: report.photo_before,
            status: report.status,
            claimed_by: report.claimed_by,
            claimed_at: report.claimed_at,
            cleared_by: report.cleared_by,
            cleared_at: report.cleared_at,
            // Return S3 URL directly (or None if not set)
            photo_after: report.photo_after,
            created_at: report.created_at,
            updated_at: report.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateReportRequest {
    #[schema(example = 51.5074)]
    pub latitude: f64,
    #[schema(example = -0.1278)]
    pub longitude: f64,
    #[schema(example = "Plastic bottles near the park entrance")]
    pub description: Option<String>,
    #[schema(example = "data:image/jpeg;base64,...")]
    pub photo_base64: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ClearReportRequest {
    #[schema(example = "data:image/jpeg;base64,...")]
    pub photo_base64: String,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct NearbyReportsQuery {
    #[param(example = 51.5074)]
    pub latitude: f64,
    #[param(example = -0.1278)]
    pub longitude: f64,
    #[param(example = 5.0, minimum = 0.1, maximum = 100.0)]
    pub radius_km: Option<f64>,
}
