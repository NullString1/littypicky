use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ReportVerification {
    pub id: Uuid,
    pub report_id: Uuid,
    pub verifier_id: Uuid,
    pub is_verified: bool,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVerificationRequest {
    pub is_verified: bool,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VerificationResponse {
    pub id: Uuid,
    pub report_id: Uuid,
    pub verifier_id: Uuid,
    pub is_verified: bool,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<ReportVerification> for VerificationResponse {
    fn from(verification: ReportVerification) -> Self {
        VerificationResponse {
            id: verification.id,
            report_id: verification.report_id,
            verifier_id: verification.verifier_id,
            is_verified: verification.is_verified,
            comment: verification.comment,
            created_at: verification.created_at,
        }
    }
}
