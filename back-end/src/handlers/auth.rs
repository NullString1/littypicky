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

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub city: String,
    pub country: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

pub async fn register(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<MessageResponse>)> {
    let message = auth_service
        .register_user(&req.email, &req.password, &req.full_name, &req.city, &req.country)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(MessageResponse { message }),
    ))
}

pub async fn login(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthTokens>> {
    let tokens = auth_service.login_user(&req.email, &req.password).await?;
    Ok(Json(tokens))
}

pub async fn verify_email(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<VerifyEmailRequest>,
) -> Result<Json<AuthTokens>> {
    let tokens = auth_service.verify_email(&req.token).await?;
    Ok(Json(tokens))
}

pub async fn resend_verification(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<ResendVerificationRequest>,
) -> Result<Json<MessageResponse>> {
    let message = auth_service.resend_verification(&req.email).await?;
    Ok(Json(MessageResponse { message }))
}

pub async fn forgot_password(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<Json<MessageResponse>> {
    let message = auth_service.forgot_password(&req.email).await?;
    Ok(Json(MessageResponse { message }))
}

pub async fn reset_password(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<MessageResponse>> {
    let message = auth_service.reset_password(&req.token, &req.new_password).await?;
    Ok(Json(MessageResponse { message }))
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
}

pub async fn refresh_token(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>> {
    let access_token = auth_service.refresh_access_token(&req.refresh_token).await?;
    Ok(Json(RefreshTokenResponse { access_token }))
}

pub async fn logout(
    State(auth_service): State<Arc<AuthService>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<MessageResponse>> {
    let message = auth_service.logout(&req.refresh_token).await?;
    Ok(Json(MessageResponse { message }))
}
