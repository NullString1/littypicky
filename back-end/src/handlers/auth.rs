use crate::{
    error::Result,
    models::*,
    services::AuthService,
};
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email)]
    #[schema(example = "user@example.com")]
    pub email: String,
    #[validate(length(min = 8))]
    #[schema(example = "SecurePassword123", min_length = 8)]
    pub password: String,
    #[validate(length(min = 1))]
    #[schema(example = "John Doe")]
    pub full_name: String,
    #[validate(length(min = 1))]
    #[schema(example = "London")]
    pub city: String,
    #[validate(length(min = 1))]
    #[schema(example = "UK")]
    pub country: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
    #[schema(example = "Operation successful")]
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "Authentication",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully. Verification email sent.", body = MessageResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Email already registered")
    )
)]
pub async fn register(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<MessageResponse>)> {
    tracing::info!("Registering user: {}", req.email);
    
    // Validate the request
    if let Err(e) = req.validate() {
        tracing::warn!("Validation failed for {}: {}", req.email, e);
        return Err(crate::error::AppError::BadRequest(format!("Validation error: {}", e)));
    }
    
    let message = match auth_service
        .register_user(&req.email, &req.password, &req.full_name, &req.city, &req.country)
        .await {
            Ok(msg) => msg,
            Err(e) => {
                tracing::error!("Registration failed for {}: {:?}", req.email, e);
                return Err(e);
            }
        };

    tracing::info!("User registered successfully: {}", req.email);

    Ok((
        StatusCode::CREATED,
        Json(MessageResponse { message }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthTokens),
        (status = 401, description = "Invalid credentials"),
        (status = 403, description = "Email not verified")
    )
)]
pub async fn login(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthTokens>> {
    let tokens = auth_service.login_user(&req.email, &req.password).await?;
    Ok(Json(tokens))
}

#[utoipa::path(
    post,
    path = "/api/auth/verify-email",
    tag = "Authentication",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "Email verified successfully", body = AuthTokens),
        (status = 400, description = "Invalid or expired token")
    )
)]
pub async fn verify_email(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<VerifyEmailRequest>,
) -> Result<Json<AuthTokens>> {
    let tokens = auth_service.verify_email(&req.token).await?;
    Ok(Json(tokens))
}

#[utoipa::path(
    post,
    path = "/api/auth/resend-verification",
    tag = "Authentication",
    request_body = ResendVerificationRequest,
    responses(
        (status = 200, description = "Verification email sent", body = MessageResponse),
        (status = 400, description = "Email already verified"),
        (status = 404, description = "User not found")
    )
)]
pub async fn resend_verification(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<ResendVerificationRequest>,
) -> Result<Json<MessageResponse>> {
    let message = auth_service.resend_verification(&req.email).await?;
    Ok(Json(MessageResponse { message }))
}

#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
    tag = "Authentication",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Password reset email sent (if email exists)", body = MessageResponse)
    )
)]
pub async fn forgot_password(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<Json<MessageResponse>> {
    let message = auth_service.forgot_password(&req.email).await?;
    Ok(Json(MessageResponse { message }))
}

#[utoipa::path(
    post,
    path = "/api/auth/reset-password",
    tag = "Authentication",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successful", body = MessageResponse),
        (status = 400, description = "Invalid or expired token")
    )
)]
pub async fn reset_password(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<MessageResponse>> {
    let message = auth_service.reset_password(&req.token, &req.new_password).await?;
    Ok(Json(MessageResponse { message }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshTokenResponse {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "Authentication",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = RefreshTokenResponse),
        (status = 401, description = "Invalid or expired refresh token")
    )
)]
pub async fn refresh_token(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>> {
    let access_token = auth_service.refresh_access_token(&req.refresh_token).await?;
    Ok(Json(RefreshTokenResponse { access_token }))
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "Authentication",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Logged out successfully", body = MessageResponse)
    )
)]
pub async fn logout(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<MessageResponse>> {
    let message = auth_service.logout(&req.refresh_token).await?;
    Ok(Json(MessageResponse { message }))
}
