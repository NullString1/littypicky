use crate::error::AppError;
use crate::services::{AuthService, OAuthService};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use openidconnect::Nonce;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::{IntoParams, ToSchema};

/// Shared state for OAuth handlers
#[derive(Clone)]
pub struct OAuthHandlerState {
    pub oauth_service: Arc<OAuthService>,
    pub auth_service: Arc<AuthService>,
    pub frontend_url: String,
    pub redirect_url: String,
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
    Ok(Redirect::to(&auth_url.to_string()))
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

    let html = format!(
        r#"<!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8">
        <title>Redirecting...</title>
    </head>
    <body>
        <p>Logging in...</p>
        <script>
            localStorage.setItem('token', '{}');
            localStorage.setItem('refreshToken', '{}');
            localStorage.setItem('user', '{}');
            window.location.href = '/app/feed';
        </script>
    </body>
    </html>"#,
        auth_tokens.access_token, auth_tokens.refresh_token, serde_json::to_string(&auth_tokens.user).unwrap()
    );

    return Ok(axum::response::Html(html).into_response());
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

    let redirect_url = format!(
        "{}#access_token={}&refresh_token={}",
        state.redirect_url, auth_tokens.access_token, auth_tokens.refresh_token
    );

    Ok(Redirect::to(&redirect_url))
}
