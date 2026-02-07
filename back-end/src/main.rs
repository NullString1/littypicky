mod config;
mod db;
mod error;
mod models;
mod auth;
mod services;
mod templates;
mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "back_end=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = config::Config::from_env()?;
    tracing::info!("Configuration loaded");

    // Create database pool
    let pool = db::create_pool(&config).await?;
    tracing::info!("Database pool created");

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Migrations completed");

    // Initialize services
    let jwt_service = auth::JwtService::new(config.jwt.clone());
    let email_service = services::EmailService::new(config.email.clone())?;
    let image_service = Arc::new(services::ImageService::new(config.image.clone()));
    let auth_service = Arc::new(services::AuthService::new(
        pool.clone(),
        jwt_service.clone(),
        email_service,
        config.clone(),
    ));

    tracing::info!("Services initialized");

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        // Health check
        .route("/", get(|| async { "LittyPicky API v0.1.0" }))
        .route("/health", get(health_check))
        
        // Auth routes (public)
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/verify-email", post(handlers::verify_email))
        .route("/api/auth/resend-verification", post(handlers::resend_verification))
        .route("/api/auth/forgot-password", post(handlers::forgot_password))
        .route("/api/auth/reset-password", post(handlers::reset_password))
        .route("/api/auth/refresh", post(handlers::refresh_token))
        .route("/api/auth/logout", post(handlers::logout))
        
        // User routes (authenticated)
        .route("/api/users/me", get(handlers::get_current_user))
        
        // Add state
        .with_state(auth_service.clone())
        .layer(cors);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("Server starting on {}", addr);
    tracing::info!("API endpoints:");
    tracing::info!("  POST /api/auth/register");
    tracing::info!("  POST /api/auth/login");
    tracing::info!("  POST /api/auth/verify-email");
    tracing::info!("  POST /api/auth/resend-verification");
    tracing::info!("  POST /api/auth/forgot-password");
    tracing::info!("  POST /api/auth/reset-password");
    tracing::info!("  POST /api/auth/refresh");
    tracing::info!("  POST /api/auth/logout");
    tracing::info!("  GET  /api/users/me");
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
