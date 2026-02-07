use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct ReportVerification {
    pub id: Uuid,
    pub report_id: Uuid,
    pub verifier_id: Uuid,
    pub is_verified: bool,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateVerificationRequest {
    #[schema(example = true)]
    pub is_verified: bool,
    #[schema(example = "Good job!")]
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
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
