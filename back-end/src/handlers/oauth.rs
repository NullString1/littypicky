use crate::error::AppError;
use crate::models::AuthTokens;
use crate::services::{AuthService, OAuthService};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use openidconnect::{CsrfToken, Nonce};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};

/// Shared state for OAuth handlers
#[derive(Clone)]
pub struct OAuthHandlerState {
    pub oauth_service: Arc<OAuthService>,
    pub auth_service: Arc<AuthService>,
    /// Store CSRF tokens and nonces temporarily (in production, use Redis or database)
    pub session_store: Arc<RwLock<HashMap<String, String>>>,
}

/// Query parameters for OAuth callback
#[derive(Debug, Deserialize, IntoParams)]
pub struct OAuthCallback {
    code: String,
    state: String,
}

/// Response for OAuth login
#[derive(Serialize, ToSchema)]
pub struct OAuthLoginResponse {
    #[schema(example = "https://accounts.google.com/o/oauth2/v2/auth?...")]
    pub auth_url: String,
}

/// Initiate Google OAuth login
/// GET /api/auth/google
#[utoipa::path(
    get,
    path = "/api/auth/google",
    tag = "OAuth",
    responses(
        (status = 200, description = "Returns Google OAuth authorization URL", body = OAuthLoginResponse)
    )
)]
pub async fn google_login(
    State(state): State<Arc<OAuthHandlerState>>,
) -> Result<impl IntoResponse, AppError> {
    let (auth_url, csrf_token, nonce) = state.oauth_service.get_authorization_url();

    // Store the nonce associated with the CSRF token
    // In production, this should use Redis or a database with TTL
    let mut session_store = state.session_store.write().await;
    session_store.insert(csrf_token.secret().clone(), nonce.secret().clone());

    // Return the authorization URL for the client to redirect to
    Ok(Json(OAuthLoginResponse { auth_url }))
}

/// Handle Google OAuth callback
/// GET /api/auth/google/callback
#[utoipa::path(
    get,
    path = "/api/auth/google/callback",
    tag = "OAuth",
    params(
        OAuthCallback
    ),
    responses(
        (status = 200, description = "OAuth login successful", body = AuthTokens),
        (status = 401, description = "Invalid or expired session"),
        (status = 500, description = "OAuth exchange failed")
    )
)]
pub async fn google_callback(
    State(state): State<Arc<OAuthHandlerState>>,
    Query(params): Query<OAuthCallback>,
) -> Result<impl IntoResponse, AppError> {
    // Retrieve the nonce for this CSRF token
    let nonce_secret = {
        let mut session_store = state.session_store.write().await;
        session_store
            .remove(&params.state)
            .ok_or_else(|| AppError::Auth("Invalid or expired session".to_string()))?
    };

    let nonce = Nonce::new(nonce_secret);

    // Exchange the authorization code for user info
    let oauth_info = state
        .oauth_service
        .exchange_code(params.code, nonce)
        .await?;

    // Login or create user
    let auth_tokens = state.auth_service.oauth_login(oauth_info).await?;

    // In a web app, you might:
    // 1. Set HTTP-only cookies with the tokens
    // 2. Redirect to a frontend route with a success parameter
    // 3. Have the frontend retrieve the tokens from a secure endpoint
    
    // For this API, we'll return JSON
    // In production, consider redirecting to your frontend with tokens in a secure way
    Ok((StatusCode::OK, Json(auth_tokens)))
}

/// Alternative: Redirect-based callback for web apps
/// This version redirects to the frontend with tokens in URL fragment (client-side only)
pub async fn google_callback_redirect(
    State(state): State<Arc<OAuthHandlerState>>,
    Query(params): Query<OAuthCallback>,
) -> Result<Redirect, AppError> {
    // Retrieve the nonce for this CSRF token
    let nonce_secret = {
        let mut session_store = state.session_store.write().await;
        session_store
            .remove(&params.state)
            .ok_or_else(|| AppError::Auth("Invalid or expired session".to_string()))?
    };

    let nonce = Nonce::new(nonce_secret);

    // Exchange the authorization code for user info
    let oauth_info = state
        .oauth_service
        .exchange_code(params.code, nonce)
        .await?;

    // Login or create user
    let auth_tokens = state.auth_service.oauth_login(oauth_info).await?;

    // Redirect to frontend with tokens in URL fragment (only accessible client-side)
    // Change this URL to your frontend URL
    let redirect_url = format!(
        "http://localhost:3000/auth/callback#access_token={}&refresh_token={}",
        auth_tokens.access_token,
        auth_tokens.refresh_token
    );

    Ok(Redirect::to(&redirect_url))
}
