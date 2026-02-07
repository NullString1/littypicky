use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub full_name: String,
    pub city: String,
    pub country: String,
    pub search_radius_km: i32,
    pub role: UserRole,
    pub is_active: bool,
    pub email_verified: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub oauth_provider: Option<String>,
    pub oauth_subject: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "SecurePassword123", min_length = 8)]
    pub password: String,
    #[schema(example = "John Doe")]
    pub full_name: String,
    #[schema(example = "London")]
    pub city: String,
    #[schema(example = "UK")]
    pub country: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "SecurePassword123")]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub city: String,
    pub country: String,
    pub search_radius_km: i32,
    pub role: UserRole,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            city: user.city,
            country: user.country,
            search_radius_km: user.search_radius_km,
            role: user.role,
            email_verified: user.email_verified,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    #[schema(example = "Jane Doe")]
    pub full_name: Option<String>,
    #[schema(example = "Manchester")]
    pub city: Option<String>,
    #[schema(example = "UK")]
    pub country: Option<String>,
    #[schema(example = 10, minimum = 1, maximum = 100)]
    pub search_radius_km: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthTokens {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
    pub user: UserResponse,
}
