use crate::config::OAuthConfig;
use crate::error::AppError;
use anyhow::anyhow;
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    reqwest::async_http_client,
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    RedirectUrl, Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};

const GOOGLE_ISSUER_URL: &str = "https://accounts.google.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub email_verified: bool,
    pub oauth_subject: String, // Google's "sub" claim
}

pub struct OAuthService {
    client: CoreClient,
}

impl OAuthService {
    /// Create a new OAuth service
    pub async fn new(config: OAuthConfig) -> Result<Self, AppError> {
        // Discover Google's OpenID Connect configuration
        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(GOOGLE_ISSUER_URL.to_string())
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid issuer URL: {}", e)))?,
            async_http_client,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to discover provider metadata: {}", e)))?;

        // Create the OAuth2 client
        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(config.google_client_id),
            Some(ClientSecret::new(config.google_client_secret)),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.google_redirect_uri)
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid redirect URI: {}", e)))?,
        );

        Ok(Self { client })
    }

    /// Generate the authorization URL to redirect the user to Google
    pub fn get_authorization_url(&self) -> (String, CsrfToken, Nonce) {
        let (auth_url, csrf_token, nonce) = self
            .client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .url();

        (auth_url.to_string(), csrf_token, nonce)
    }

    /// Exchange authorization code for user information
    pub async fn exchange_code(
        &self,
        code: String,
        nonce: Nonce,
    ) -> Result<OAuthUserInfo, AppError> {
        // Exchange the authorization code for an access token
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(|e| {
                AppError::Auth(format!("Failed to exchange authorization code: {}", e))
            })?;

        // Extract the ID token
        let id_token = token_response
            .extra_fields()
            .id_token()
            .ok_or_else(|| AppError::Auth("No ID token in response".to_string()))?;

        // Verify the ID token
        let claims = id_token
            .claims(&self.client.id_token_verifier(), &nonce)
            .map_err(|e| AppError::Auth(format!("Failed to verify ID token: {}", e)))?;

        // Extract user information from claims
        let email = claims
            .email()
            .ok_or_else(|| AppError::Auth("No email in ID token".to_string()))?
            .as_str()
            .to_string();

        let email_verified = claims.email_verified().unwrap_or(false);

        let name = claims.name().and_then(|n| {
            n.get(None) // Get name in default locale
                .map(|name| name.as_str().to_string())
        });

        let picture = claims.picture().and_then(|p| {
            p.get(None) // Get picture URL in default locale
                .map(|url| url.as_str().to_string())
        });

        let oauth_subject = claims.subject().as_str().to_string();

        Ok(OAuthUserInfo {
            email,
            name,
            picture,
            email_verified,
            oauth_subject,
        })
    }
}
