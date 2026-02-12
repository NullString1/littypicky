use crate::error::AppError;
use crate::models::report::{CreateReportRequest, LitterReport, ReportStatus};
use crate::services::image_service::ImageService;
use crate::services::s3_service::S3Service;
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct NominatimAddress {
    road: Option<String>,
    amenity: Option<String>,
    shop: Option<String>,
    building: Option<String>,
    house_number: Option<String>,
    suburb: Option<String>,
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NominatimResponse {
    address: Option<NominatimAddress>,
    display_name: Option<String>,
}

#[derive(Clone)]
pub struct ReportService {
    pool: PgPool,
    image_service: ImageService,
    s3_service: S3Service,
}

impl ReportService {
    #[must_use]
    pub fn new(pool: PgPool, image_service: ImageService, s3_service: S3Service) -> Self {
        Self {
            pool,
            image_service,
            s3_service,
        }
    }

    async fn get_address_from_coords(&self, lat: f64, lon: f64) -> Option<String> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://nominatim.openstreetmap.org/reverse?format=json&lat={}&lon={}&zoom=18&addressdetails=1",
            lat, lon
        );

        match client
            .get(&url)
            .header("User-Agent", "LittyPicky/1.0")
            .send()
            .await
        {
            Ok(resp) => match resp.json::<NominatimResponse>().await {
                Ok(data) => {
                    if let Some(addr) = data.address {
                        // Prioritize specific POI names if close (Nominatim handles distance logic for us somewhat by returning the specific object)
                        // We want "Tesco, Example Street" or "52 Example Street" or "Example Street"

                        let street = addr
                            .road
                            .or(addr.suburb)
                            .or(addr.village)
                            .or(addr.town)
                            .or(addr.city);

                        // Check for POI/Building
                        let poi = addr.amenity.or(addr.shop).or(addr.building);

                        match (poi, addr.house_number, street) {
                            (Some(p), Some(s), _) if p.eq_ignore_ascii_case(&s) => Some(p), // Avoid duplication
                            (Some(p), _, Some(s)) => Some(format!("{}, {}", p, s)),
                            (Some(p), _, None) => Some(p),
                            (None, Some(n), Some(s)) => Some(format!("{} {}", n, s)),
                            (None, None, Some(s)) => Some(s),
                            _ => data.display_name, // Fallback to full display name if nothing clean is found
                        }
                    } else {
                        None
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse Nominatim response: {}", e);
                    None
                }
            },
            Err(e) => {
                eprintln!("Failed to fetch address: {}", e);
                None
            }
        }
    }

    /// Create a new litter report
    pub async fn create_report(
        &self,
        user_id: Uuid,
        request: CreateReportRequest,
    ) -> Result<LitterReport, AppError> {
        // Check if user's email is verified
        let user = sqlx::query!("SELECT email_verified FROM users WHERE id = $1", user_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if !user.email_verified {
            return Err(AppError::Forbidden(
                "Email must be verified to create reports".to_string(),
            ));
        }

        // Process the image (async to avoid blocking)
        let processed_image = self
            .image_service
            .process_image(request.photo_base64)
            .await?;

        // Upload to S3
        let photo_url = self
            .s3_service
            .upload_image(processed_image, "reports/before")
            .await?;

        // Get address from coordinates
        let address = self
            .get_address_from_coords(request.latitude, request.longitude)
            .await;

        // Create the report with PostGIS geometry
        let report = sqlx::query_as!(
            LitterReport,
            r#"
            INSERT INTO litter_reports (
                reporter_id, location, description,
                photo_before, status, address
            )
            VALUES (
                $1,
                ST_SetSRID(ST_MakePoint($3, $2), 4326),
                $4, $5, $6, $7
            )
            RETURNING
                id, reporter_id,
                ST_Y(location)::double precision as "latitude!",
                ST_X(location)::double precision as "longitude!",
                description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, created_at, updated_at, address
            "#,
            user_id,
            request.latitude,
            request.longitude,
            request.description,
            photo_url,
            ReportStatus::Pending as ReportStatus,
            address
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(report)
    }

    /// Get reports near a location using `PostGIS`
    pub async fn get_nearby_reports(
        &self,
        latitude: f64,
        longitude: f64,
        radius_km: f64,
    ) -> Result<Vec<LitterReport>, AppError> {
        let radius_meters = radius_km * 1000.0;

        let reports = sqlx::query_as!(
            LitterReport,
            r#"
            SELECT
                id, reporter_id,
                ST_Y(location)::double precision as "latitude!",
                ST_X(location)::double precision as "longitude!",
                description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, created_at, updated_at, address
            FROM litter_reports
            WHERE ST_DWithin(
                location::geography,
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

    /// Get reports that need verification near a location
    pub async fn get_verification_queue(
        &self,
        latitude: f64,
        longitude: f64,
        radius_km: f64,
        user_id: Uuid,
    ) -> Result<Vec<LitterReport>, AppError> {
        let radius_meters = radius_km * 1000.0;

        let reports = sqlx::query_as!(
            LitterReport,
            r#"
            SELECT
                id, reporter_id,
                ST_Y(location)::double precision as "latitude!",
                ST_X(location)::double precision as "longitude!",
                description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, created_at, updated_at, address
            FROM litter_reports
            WHERE ST_DWithin(
                location::geography,
                ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography,
                $3
            )
            AND status = 'cleared'
            AND (cleared_by IS NULL OR cleared_by != $4)
            AND id NOT IN (
                SELECT report_id FROM report_verifications WHERE verifier_id = $4
            )
            ORDER BY cleared_at DESC
            LIMIT 50
            "#,
            longitude,
            latitude,
            radius_meters,
            user_id
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
                id, reporter_id,
                ST_Y(location)::double precision as "latitude!",
                ST_X(location)::double precision as "longitude!",
                description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, created_at, updated_at, address
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
                id, reporter_id,
                ST_Y(location)::double precision as "latitude!",
                ST_X(location)::double precision as "longitude!",
                description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, created_at, updated_at, address
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

        // Process the after photo (async to avoid blocking)
        let processed_image = self.image_service.process_image(photo_base64).await?;

        // Upload to S3
        let photo_url = self
            .s3_service
            .upload_image(processed_image, "reports/after")
            .await?;

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
                id, reporter_id,
                ST_Y(location)::double precision as "latitude!",
                ST_X(location)::double precision as "longitude!",
                description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, created_at, updated_at, address
            "#,
            ReportStatus::Cleared as ReportStatus,
            user_id,
            chrono::Utc::now(),
            photo_url,
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
                id, reporter_id,
                ST_Y(location)::double precision as "latitude!",
                ST_X(location)::double precision as "longitude!",
                description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, created_at, updated_at, address
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
                id, reporter_id,
                ST_Y(location)::double precision as "latitude!",
                ST_X(location)::double precision as "longitude!",
                description,
                photo_before, status as "status: ReportStatus",
                claimed_by, claimed_at, cleared_by, cleared_at,
                photo_after, created_at, updated_at, address
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
