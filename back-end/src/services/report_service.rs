use crate::error::AppError;
use crate::models::report::{CreateReportRequest, LitterReport, ReportStatus};
use crate::services::image_service::ImageService;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ReportService {
    pool: PgPool,
    image_service: ImageService,
}

impl ReportService {
    pub fn new(pool: PgPool, image_service: ImageService) -> Self {
        Self {
            pool,
            image_service,
        }
    }

    /// Create a new litter report
    pub async fn create_report(
        &self,
        user_id: Uuid,
        request: CreateReportRequest,
    ) -> Result<LitterReport, AppError> {
        // Check if user's email is verified
        let user = sqlx::query!(
            "SELECT email_verified FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if !user.email_verified {
            return Err(AppError::Forbidden(
                "Email must be verified to create reports".to_string(),
            ));
        }

        // Process the image
        let processed_photo = self
            .image_service
            .process_image(&request.photo_base64)?;

        // Create the report with PostGIS geometry
        let report = sqlx::query_as!(
            LitterReport,
            r#"
            INSERT INTO litter_reports (
                reporter_id, latitude, longitude, location, description,
                photo_before, status, city, country
            )
            VALUES (
                $1, $2, $3,
                ST_SetSRID(ST_MakePoint($3, $2), 4326),
                $4, $5, $6, $7, $8
            )
            RETURNING
                id, reporter_id, latitude, longitude, description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, city, country, created_at, updated_at
            "#,
            user_id,
            request.latitude,
            request.longitude,
            request.description,
            processed_photo,
            ReportStatus::Pending as ReportStatus,
            request.city,
            request.country
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(report)
    }

    /// Get reports near a location using PostGIS
    pub async fn get_nearby_reports(
        &self,
        latitude: f64,
        longitude: f64,
        radius_km: f64,
    ) -> Result<Vec<LitterReport>, AppError> {
        // Convert km to meters for PostGIS
        let radius_meters = radius_km * 1000.0;

        let reports = sqlx::query_as!(
            LitterReport,
            r#"
            SELECT
                id, reporter_id, latitude, longitude, description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, city, country, created_at, updated_at
            FROM litter_reports
            WHERE ST_DWithin(
                location,
                ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography,
                $3
            )
            AND status IN ('pending', 'claimed')
            ORDER BY created_at DESC
            LIMIT 100
            "#,
            longitude,
            latitude,
            radius_meters
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(reports)
    }

    /// Get a single report by ID
    pub async fn get_report_by_id(&self, report_id: Uuid) -> Result<LitterReport, AppError> {
        let report = sqlx::query_as!(
            LitterReport,
            r#"
            SELECT
                id, reporter_id, latitude, longitude, description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, city, country, created_at, updated_at
            FROM litter_reports
            WHERE id = $1
            "#,
            report_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Report not found".to_string()))?;

        Ok(report)
    }

    /// Claim a report for cleanup
    pub async fn claim_report(
        &self,
        report_id: Uuid,
        user_id: Uuid,
    ) -> Result<LitterReport, AppError> {
        // Check current status
        let current_report = self.get_report_by_id(report_id).await?;

        if current_report.status != ReportStatus::Pending {
            return Err(AppError::BadRequest(
                "Report is not available for claiming".to_string(),
            ));
        }

        if current_report.reporter_id == user_id {
            return Err(AppError::BadRequest(
                "Cannot claim your own report".to_string(),
            ));
        }

        // Update the report
        let report = sqlx::query_as!(
            LitterReport,
            r#"
            UPDATE litter_reports
            SET status = $1,
                claimed_by = $2,
                claimed_at = $3
            WHERE id = $4
            RETURNING
                id, reporter_id, latitude, longitude, description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, city, country, created_at, updated_at
            "#,
            ReportStatus::Claimed as ReportStatus,
            user_id,
            Utc::now(),
            report_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(report)
    }

    /// Mark a report as cleared with after photo
    pub async fn clear_report(
        &self,
        report_id: Uuid,
        user_id: Uuid,
        photo_base64: String,
    ) -> Result<LitterReport, AppError> {
        // Check current status
        let current_report = self.get_report_by_id(report_id).await?;

        if current_report.status != ReportStatus::Claimed {
            return Err(AppError::BadRequest(
                "Report must be claimed before clearing".to_string(),
            ));
        }

        if current_report.claimed_by != Some(user_id) {
            return Err(AppError::Forbidden(
                "Only the user who claimed this report can clear it".to_string(),
            ));
        }

        // Process the after photo
        let processed_photo = self.image_service.process_image(&photo_base64)?;

        // Update the report
        let report = sqlx::query_as!(
            LitterReport,
            r#"
            UPDATE litter_reports
            SET status = $1,
                cleared_by = $2,
                cleared_at = $3,
                photo_after = $4
            WHERE id = $5
            RETURNING
                id, reporter_id, latitude, longitude, description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, city, country, created_at, updated_at
            "#,
            ReportStatus::Cleared as ReportStatus,
            user_id,
            Utc::now(),
            processed_photo,
            report_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(report)
    }

    /// Get all reports by a user (as reporter)
    pub async fn get_user_reports(&self, user_id: Uuid) -> Result<Vec<LitterReport>, AppError> {
        let reports = sqlx::query_as!(
            LitterReport,
            r#"
            SELECT
                id, reporter_id, latitude, longitude, description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, city, country, created_at, updated_at
            FROM litter_reports
            WHERE reporter_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(reports)
    }

    /// Get all reports cleared by a user
    pub async fn get_user_cleared_reports(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<LitterReport>, AppError> {
        let reports = sqlx::query_as!(
            LitterReport,
            r#"
            SELECT
                id, reporter_id, latitude, longitude, description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, city, country, created_at, updated_at
            FROM litter_reports
            WHERE cleared_by = $1
            ORDER BY cleared_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(reports)
    }
}
