use crate::{
    config::JwtConfig,
    error::{AppError, Result},
    models::UserRole,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Clone)]
pub struct JwtService {
    config: JwtConfig,
}

impl JwtService {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    pub fn create_access_token(
        &self,
        user_id: Uuid,
        email: &str,
        role: &UserRole,
    ) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.access_expiry);

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            role: match role {
                UserRole::Admin => "admin".to_string(),
                UserRole::User => "user".to_string(),
            },
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.secret.as_bytes()),
        )
        .map_err(|e| AppError::Auth(format!("Failed to create token: {}", e)))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| AppError::Auth(format!("Invalid token: {}", e)))
    }
}
