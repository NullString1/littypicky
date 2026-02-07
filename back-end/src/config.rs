use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub oauth: OAuthConfig,
    pub email: EmailConfig,
    pub rate_limit: RateLimitConfig,
    pub image: ImageConfig,
    pub scoring: ScoringConfig,
    pub s3: S3Config,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub access_expiry: i64,
    pub refresh_expiry: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthConfig {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub verification_expiry_hours: i64,
    pub password_reset_expiry_hours: i64,
    pub frontend_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub auth_per_min: u32,
    pub reports_per_hour: u32,
    pub verifications_per_hour: u32,
    pub general_per_min: u32,
    pub email_verification_per_hour: u32,
    pub password_reset_per_hour: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageConfig {
    pub max_size_mb: usize,
    pub webp_quality: f32,
    pub max_width: u32,
    pub max_height: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScoringConfig {
    pub min_clears_to_verify: i32,
    pub min_verifications_needed: i32,
    pub base_points_per_clear: i32,
    pub streak_bonus_points: i32,
    pub first_in_area_bonus: i32,
    pub verification_bonus: i32,
    pub verified_report_bonus: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub public_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        Ok(Config {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()?,
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")?,
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET")?,
                access_expiry: env::var("JWT_ACCESS_EXPIRY")
                    .unwrap_or_else(|_| "900".to_string())
                    .parse()?,
                refresh_expiry: env::var("JWT_REFRESH_EXPIRY")
                    .unwrap_or_else(|_| "2592000".to_string())
                    .parse()?,
            },
            oauth: OAuthConfig {
                google_client_id: env::var("GOOGLE_CLIENT_ID")?,
                google_client_secret: env::var("GOOGLE_CLIENT_SECRET")?,
                google_redirect_uri: env::var("GOOGLE_REDIRECT_URI")?,
            },
            email: EmailConfig {
                smtp_host: env::var("SMTP_HOST")?,
                smtp_port: env::var("SMTP_PORT")?.parse()?,
                smtp_username: env::var("SMTP_USERNAME")?,
                smtp_password: env::var("SMTP_PASSWORD")?,
                smtp_from_email: env::var("SMTP_FROM_EMAIL")?,
                smtp_from_name: env::var("SMTP_FROM_NAME")?,
                verification_expiry_hours: env::var("EMAIL_VERIFICATION_EXPIRY_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()?,
                password_reset_expiry_hours: env::var("PASSWORD_RESET_EXPIRY_HOURS")
                    .unwrap_or_else(|_| "1".to_string())
                    .parse()?,
                frontend_url: env::var("FRONTEND_URL")?,
            },
            rate_limit: RateLimitConfig {
                auth_per_min: env::var("RATE_LIMIT_AUTH_PER_MIN")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
                reports_per_hour: env::var("RATE_LIMIT_REPORTS_PER_HOUR")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                verifications_per_hour: env::var("RATE_LIMIT_VERIFICATIONS_PER_HOUR")
                    .unwrap_or_else(|_| "20".to_string())
                    .parse()?,
                general_per_min: env::var("RATE_LIMIT_GENERAL_PER_MIN")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()?,
                email_verification_per_hour: env::var("RATE_LIMIT_EMAIL_VERIFICATION_PER_HOUR")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()?,
                password_reset_per_hour: env::var("RATE_LIMIT_PASSWORD_RESET_PER_HOUR")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()?,
            },
            image: ImageConfig {
                max_size_mb: env::var("MAX_PHOTO_SIZE_MB")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
                webp_quality: env::var("WEBP_QUALITY")
                    .unwrap_or_else(|_| "80".to_string())
                    .parse()?,
                max_width: env::var("MAX_IMAGE_WIDTH")
                    .unwrap_or_else(|_| "1920".to_string())
                    .parse()?,
                max_height: env::var("MAX_IMAGE_HEIGHT")
                    .unwrap_or_else(|_| "1920".to_string())
                    .parse()?,
            },
            scoring: ScoringConfig {
                min_clears_to_verify: env::var("MIN_CLEARS_TO_VERIFY")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
                min_verifications_needed: env::var("MIN_VERIFICATIONS_NEEDED")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()?,
                base_points_per_clear: env::var("BASE_POINTS_PER_CLEAR")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                streak_bonus_points: env::var("STREAK_BONUS_POINTS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
                first_in_area_bonus: env::var("FIRST_IN_AREA_BONUS")
                    .unwrap_or_else(|_| "20".to_string())
                    .parse()?,
                verification_bonus: env::var("VERIFICATION_BONUS")
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()?,
                verified_report_bonus: env::var("VERIFIED_REPORT_BONUS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
            },
            s3: S3Config {
                endpoint: env::var("S3_ENDPOINT")
                    .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string()),
                region: env::var("S3_REGION")
                    .unwrap_or_else(|_| "us-east-1".to_string()),
                bucket: env::var("S3_BUCKET")
                    .unwrap_or_else(|_| "littypicky-images".to_string()),
                access_key: env::var("S3_ACCESS_KEY")
                    .unwrap_or_else(|_| "minioadmin".to_string()),
                secret_key: env::var("S3_SECRET_KEY")
                    .unwrap_or_else(|_| "minioadmin123".to_string()),
                public_url: env::var("S3_PUBLIC_URL")
                    .unwrap_or_else(|_| "http://127.0.0.1:9000/littypicky-images".to_string()),
            },
        })
    }
}
