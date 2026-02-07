use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "report_status", rename_all = "lowercase")]
pub enum ReportStatus {
    Pending,
    Claimed,
    Cleared,
    Verified,
}

#[derive(Debug, Clone, FromRow)]
pub struct LitterReport {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub description: Option<String>,
    pub photo_before: String,
    pub status: ReportStatus,
    pub claimed_by: Option<Uuid>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub cleared_by: Option<Uuid>,
    pub cleared_at: Option<DateTime<Utc>>,
    pub photo_after: Option<String>,
    pub city: String,
    pub country: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub description: Option<String>,
    pub photo_before: String,
    pub status: ReportStatus,
    pub claimed_by: Option<Uuid>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub cleared_by: Option<Uuid>,
    pub cleared_at: Option<DateTime<Utc>>,
    pub photo_after: Option<String>,
    pub city: String,
    pub country: String,
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
            photo_before: report.photo_before,
            status: report.status,
            claimed_by: report.claimed_by,
            claimed_at: report.claimed_at,
            cleared_by: report.cleared_by,
            cleared_at: report.cleared_at,
            photo_after: report.photo_after,
            city: report.city,
            country: report.country,
            created_at: report.created_at,
            updated_at: report.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateReportRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub description: Option<String>,
    pub photo_base64: String,
    pub city: String,
    pub country: String,
}

#[derive(Debug, Deserialize)]
pub struct ClearReportRequest {
    pub photo_base64: String,
}

#[derive(Debug, Deserialize)]
pub struct NearbyReportsQuery {
    pub latitude: f64,
    pub longitude: f64,
    pub radius_km: Option<f64>,
}
