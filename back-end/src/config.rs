use serde::Deserialize;
use std::env;
use std::fs;

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
    pub tls: Option<TlsConfig>,
    pub enable_test_helpers: bool,
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
    pub report_points: i32,
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

#[derive(Debug, Clone, Deserialize)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        fn read_env_file_value(key: &str) -> Option<String> {
            let file_key = format!("{key}_FILE");
            if let Ok(path) = env::var(file_key) {
                if let Ok(contents) = fs::read_to_string(path) {
                    return Some(contents.trim().to_string());
                }
            }
            env::var(key).ok()
        }

        fn require_env(key: &str) -> Result<String, anyhow::Error> {
            read_env_file_value(key)
                .ok_or_else(|| anyhow::anyhow!("Missing env var {key}"))
        }

        fn env_or_default(key: &str, default: &str) -> Result<String, anyhow::Error> {
            Ok(read_env_file_value(key).unwrap_or_else(|| default.to_string()))
        }

        Ok(Config {
            server: ServerConfig {
                host: env_or_default("HOST", "0.0.0.0")?,
                port: env_or_default("PORT", "8080")?.parse()?,
            },
            database: DatabaseConfig {
                url: require_env("DATABASE_URL")?,
            },
            jwt: JwtConfig {
                secret: require_env("JWT_SECRET")?,
                access_expiry: env_or_default("JWT_ACCESS_EXPIRY", "900")?.parse()?,
                refresh_expiry: env_or_default("JWT_REFRESH_EXPIRY", "2592000")?.parse()?,
            },
            oauth: OAuthConfig {
                google_client_id: require_env("GOOGLE_CLIENT_ID")?,
                google_client_secret: require_env("GOOGLE_CLIENT_SECRET")?,
                google_redirect_uri: require_env("GOOGLE_REDIRECT_URI")?,
            },
            email: EmailConfig {
                smtp_host: require_env("SMTP_HOST")?,
                smtp_port: require_env("SMTP_PORT")?.parse()?,
                smtp_username: require_env("SMTP_USERNAME")?,
                smtp_password: require_env("SMTP_PASSWORD")?,
                smtp_from_email: require_env("SMTP_FROM_EMAIL")?,
                smtp_from_name: require_env("SMTP_FROM_NAME")?,
                verification_expiry_hours: env_or_default("EMAIL_VERIFICATION_EXPIRY_HOURS", "24")?
                    .parse()?,
                password_reset_expiry_hours: env_or_default("PASSWORD_RESET_EXPIRY_HOURS", "1")?
                    .parse()?,
                frontend_url: require_env("FRONTEND_URL")?,
            },
            rate_limit: RateLimitConfig {
                auth_per_min: env_or_default("RATE_LIMIT_AUTH_PER_MIN", "5")?.parse()?,
                reports_per_hour: env_or_default("RATE_LIMIT_REPORTS_PER_HOUR", "10")?.parse()?,
                verifications_per_hour: env_or_default("RATE_LIMIT_VERIFICATIONS_PER_HOUR", "20")?.parse()?,
                general_per_min: env_or_default("RATE_LIMIT_GENERAL_PER_MIN", "100")?.parse()?,
                email_verification_per_hour: env_or_default("RATE_LIMIT_EMAIL_VERIFICATION_PER_HOUR", "3")?.parse()?,
                password_reset_per_hour: env_or_default("RATE_LIMIT_PASSWORD_RESET_PER_HOUR", "3")?.parse()?,
            },
            image: ImageConfig {
                max_size_mb: env_or_default("MAX_PHOTO_SIZE_MB", "5")?.parse()?,
                webp_quality: env_or_default("WEBP_QUALITY", "80")?.parse()?,
                max_width: env_or_default("MAX_IMAGE_WIDTH", "1920")?.parse()?,
                max_height: env_or_default("MAX_IMAGE_HEIGHT", "1920")?.parse()?,
            },
            scoring: ScoringConfig {
                min_clears_to_verify: env_or_default("MIN_CLEARS_TO_VERIFY", "5")?.parse()?,
                min_verifications_needed: env_or_default("MIN_VERIFICATIONS_NEEDED", "3")?.parse()?,
                report_points: env_or_default("REPORT_POINTS", "10")?.parse()?,
                base_points_per_clear: env_or_default("BASE_POINTS_PER_CLEAR", "10")?.parse()?,
                streak_bonus_points: env_or_default("STREAK_BONUS_POINTS", "5")?.parse()?,
                first_in_area_bonus: env_or_default("FIRST_IN_AREA_BONUS", "20")?.parse()?,
                verification_bonus: env_or_default("VERIFICATION_BONUS", "2")?.parse()?,
                verified_report_bonus: env_or_default("VERIFIED_REPORT_BONUS", "10")?.parse()?,
            },
            s3: S3Config {
                endpoint: env_or_default("S3_ENDPOINT", "http://127.0.0.1:9000")?,
                region: env_or_default("S3_REGION", "us-east-1")?,
                bucket: env_or_default("S3_BUCKET", "littypicky-images")?,
                access_key: env_or_default("S3_ACCESS_KEY", "minioadmin")?,
                secret_key: env_or_default("S3_SECRET_KEY", "minioadmin123")?,
                public_url: env_or_default("S3_PUBLIC_URL", "http://127.0.0.1:9000/littypicky-images")?,
            },
            tls: match (
                read_env_file_value("TLS_CERT_PATH"),
                read_env_file_value("TLS_KEY_PATH"),
            ) {
                (Some(cert_path), Some(key_path)) => Some(TlsConfig { cert_path, key_path }),
                _ => None,
            },
            enable_test_helpers: env_or_default("ENABLE_TEST_HELPERS", "false")?
                .parse()
                .unwrap_or(false),
        })
    }
}
