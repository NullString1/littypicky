use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
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

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub city: String,
    pub country: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub search_radius_km: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}
