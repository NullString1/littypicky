use crate::{
    auth::{generate_token, JwtService},
    config::Config,
    error::{AppError, Result},
    models::{User, UserRole, AuthTokens, UserResponse},
    services::EmailService,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Duration, Utc};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

#[derive(FromRow)]
struct TokenRecord {
    user_id: Uuid,
    expires_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct PasswordResetRecord {
    user_id: Uuid,
    expires_at: DateTime<Utc>,
    used: bool,
}

pub struct AuthService {
    pool: PgPool,
    jwt_service: JwtService,
    email_service: EmailService,
    config: Config,
}

impl AuthService {
    pub fn new(pool: PgPool, jwt_service: JwtService, email_service: EmailService, config: Config) -> Self {
        Self {
            pool,
            jwt_service,
            email_service,
            config,
        }
    }

    pub async fn register_user(
        &self,
        email: &str,
        password: &str,
        full_name: &str,
        city: &str,
        country: &str,
    ) -> Result<String> {
        // Check if user already exists
        let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(&self.pool)
            .await?;

        if existing > 0 {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        // Hash password
        let password_hash = self.hash_password(password)?;

        // Create user
        let user_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO users (email, password_hash, full_name, city, country, email_verified) 
             VALUES ($1, $2, $3, $4, $5, false) 
             RETURNING id"
        )
        .bind(email)
        .bind(password_hash)
        .bind(full_name)
        .bind(city)
        .bind(country)
        .fetch_one(&self.pool)
        .await?;

        // Initialize user score
        sqlx::query(
            "INSERT INTO user_scores (user_id) VALUES ($1)"
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // Generate verification token
        let token = generate_token();
        let expires_at = Utc::now() + Duration::hours(self.config.email.verification_expiry_hours);

        sqlx::query(
            "INSERT INTO email_verification_tokens (user_id, token, expires_at) 
             VALUES ($1, $2, $3)"
        )
        .bind(user_id)
        .bind(&token)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        // Send verification email
        self.email_service
            .send_verification_email(email, full_name, &token)
            .await?;

        Ok("Registration successful. Please check your email to verify your account.".to_string())
    }

    pub async fn login_user(&self, email: &str, password: &str) -> Result<AuthTokens> {
        // Get user
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND is_active = true"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::Auth("Invalid credentials".to_string()))?;

        // Check if email is verified
        if !user.email_verified {
            return Err(AppError::Forbidden(
                "Please verify your email address before logging in".to_string()
            ));
        }

        // Check if user has a password (OAuth users don't) and verify it
        match &user.password_hash {
            Some(hash) => self.verify_password(password, hash)?,
            None => return Err(AppError::Auth("Please use OAuth to login".to_string())),
        };

        // Generate tokens
        self.create_auth_tokens(user).await
    }

    pub async fn verify_email(&self, token: &str) -> Result<AuthTokens> {
        // Find and validate token
        let verification = sqlx::query_as::<_, TokenRecord>(
            "SELECT user_id, expires_at FROM email_verification_tokens 
             WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::BadRequest("Invalid or expired verification token".to_string()))?;

        if verification.expires_at < Utc::now() {
            return Err(AppError::BadRequest("Verification token has expired".to_string()));
        }

        // Update user
        sqlx::query(
            "UPDATE users SET email_verified = true, email_verified_at = NOW() 
             WHERE id = $1"
        )
        .bind(verification.user_id)
        .execute(&self.pool)
        .await?;

        // Delete verification token
        sqlx::query("DELETE FROM email_verification_tokens WHERE token = $1")
            .bind(token)
            .execute(&self.pool)
            .await?;

        // Get user and generate tokens
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(verification.user_id)
            .fetch_one(&self.pool)
            .await?;

        self.create_auth_tokens(user).await
    }

    pub async fn resend_verification(&self, email: &str) -> Result<String> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if user.email_verified {
            return Err(AppError::BadRequest("Email already verified".to_string()));
        }

        // Delete old tokens
        sqlx::query("DELETE FROM email_verification_tokens WHERE user_id = $1")
            .bind(user.id)
            .execute(&self.pool)
            .await?;

        // Generate new token
        let token = generate_token();
        let expires_at = Utc::now() + Duration::hours(self.config.email.verification_expiry_hours);

        sqlx::query(
            "INSERT INTO email_verification_tokens (user_id, token, expires_at) 
             VALUES ($1, $2, $3)"
        )
        .bind(user.id)
        .bind(&token)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        // Send email
        self.email_service
            .send_verification_email(&user.email, &user.full_name, &token)
            .await?;

        Ok("Verification email sent".to_string())
    }

    pub async fn forgot_password(&self, email: &str) -> Result<String> {
        // Always return success to prevent email enumeration
        let user = match sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await? {
            Some(u) => u,
            None => return Ok("If the email exists, a password reset link has been sent".to_string()),
        };

        // Don't send reset for OAuth users
        if user.password_hash.is_none() {
            return Ok("If the email exists, a password reset link has been sent".to_string());
        }

        // Delete old tokens
        sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1")
            .bind(user.id)
            .execute(&self.pool)
            .await?;

        // Generate token
        let token = generate_token();
        let expires_at = Utc::now() + Duration::hours(self.config.email.password_reset_expiry_hours);

        sqlx::query(
            "INSERT INTO password_reset_tokens (user_id, token, expires_at) 
             VALUES ($1, $2, $3)"
        )
        .bind(user.id)
        .bind(&token)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        // Send email
        self.email_service
            .send_password_reset_email(&user.email, &user.full_name, &token)
            .await?;

        Ok("If the email exists, a password reset link has been sent".to_string())
    }

    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<String> {
        // Find and validate token
        let reset = sqlx::query_as::<_, PasswordResetRecord>(
            "SELECT user_id, expires_at, used FROM password_reset_tokens 
             WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::BadRequest("Invalid or expired reset token".to_string()))?;

        if reset.expires_at < Utc::now() {
            return Err(AppError::BadRequest("Reset token has expired".to_string()));
        }

        if reset.used {
            return Err(AppError::BadRequest("Reset token already used".to_string()));
        }

        // Hash new password
        let password_hash = self.hash_password(new_password)?;

        // Update password
        sqlx::query(
            "UPDATE users SET password_hash = $1 WHERE id = $2"
        )
        .bind(password_hash)
        .bind(reset.user_id)
        .execute(&self.pool)
        .await?;

        // Mark token as used
        sqlx::query("UPDATE password_reset_tokens SET used = true WHERE token = $1")
            .bind(token)
            .execute(&self.pool)
            .await?;

        // Invalidate all refresh tokens for security
        sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1")
            .bind(reset.user_id)
            .execute(&self.pool)
            .await?;

        // Get user and send confirmation
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(reset.user_id)
            .fetch_one(&self.pool)
            .await?;

        self.email_service
            .send_password_reset_confirmation(&user.email, &user.full_name)
            .await?;

        Ok("Password successfully reset".to_string())
    }

    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<String> {
        // Verify the refresh token exists and is valid
        let token_record = sqlx::query_as::<_, TokenRecord>(
            "SELECT user_id, expires_at FROM refresh_tokens WHERE token_hash = $1"
        )
        .bind(refresh_token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::Auth("Invalid refresh token".to_string()))?;

        if token_record.expires_at < Utc::now() {
            // Clean up expired token
            sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = $1")
                .bind(refresh_token)
                .execute(&self.pool)
                .await?;
            return Err(AppError::Auth("Refresh token expired".to_string()));
        }

        // Get user
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = true")
            .bind(token_record.user_id)
            .fetch_one(&self.pool)
            .await?;

        // Generate new access token
        let access_token = self.jwt_service.create_access_token(user.id, &user.email, &user.role)?;

        Ok(access_token)
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<String> {
        sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = $1")
            .bind(refresh_token)
            .execute(&self.pool)
            .await?;

        Ok("Logged out successfully".to_string())
    }

    // Helper methods

    async fn create_auth_tokens(&self, user: User) -> Result<AuthTokens> {
        let access_token = self.jwt_service.create_access_token(user.id, &user.email, &user.role)?;
        
        let refresh_token = generate_token();
        let expires_at = Utc::now() + Duration::seconds(self.config.jwt.refresh_expiry);

        sqlx::query(
            "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)"
        )
        .bind(user.id)
        .bind(&refresh_token)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
            user: user.into(),
        })
    }

    fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to hash password: {}", e)))
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<()> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid password hash: {}", e)))?;
        
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Auth("Invalid credentials".to_string()))
    }
}
