mod config;
mod db;
mod error;
mod models;
mod auth;
mod services;
mod templates;
mod handlers;
mod rate_limit;

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
    let image_service = services::ImageService::new(config.image.clone());
    let report_service = services::ReportService::new(pool.clone(), image_service);
    let scoring_service = services::ScoringService::new(pool.clone(), config.scoring.clone());
    
    let auth_service = Arc::new(services::AuthService::new(
        pool.clone(),
        jwt_service.clone(),
        email_service,
        config.clone(),
    ));

    // Handler states
    let user_state = Arc::new(handlers::UserHandlerState {
        pool: pool.clone(),
    });

    let report_state = Arc::new(handlers::ReportHandlerState {
        report_service: report_service.clone(),
        scoring_service: scoring_service.clone(),
    });

    let verification_state = Arc::new(handlers::VerificationHandlerState {
        pool: pool.clone(),
        report_service: report_service.clone(),
        scoring_service: scoring_service.clone(),
        scoring_config: config.scoring.clone(),
    });

    let leaderboard_state = Arc::new(handlers::LeaderboardHandlerState {
        pool: pool.clone(),
    });

    tracing::info!("Services initialized");

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Add rate limiting
    let rate_limiter = rate_limit::get_rate_limiter_layer();

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
        .with_state(auth_service.clone())
        
        // User routes (authenticated)
        .route("/api/users/me", get(handlers::get_current_user))
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_service.clone(),
            auth::middleware::require_auth,
        ))
        .with_state(user_state)
        
        // Report routes (authenticated)
        .route("/api/reports", post(handlers::create_report))
        .route("/api/reports/nearby", get(handlers::get_nearby_reports))
        .route("/api/reports/my-reports", get(handlers::get_my_reports))
        .route("/api/reports/my-clears", get(handlers::get_my_cleared_reports))
        .route("/api/reports/:id", get(handlers::get_report))
        .route("/api/reports/:id/claim", post(handlers::claim_report))
        .route("/api/reports/:id/clear", post(handlers::clear_report))
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_service.clone(),
            auth::middleware::require_auth,
        ))
        .with_state(report_state)
        
        // Verification routes (authenticated)
        .route("/api/reports/:id/verify", post(handlers::verify_report))
        .route("/api/reports/:id/verifications", get(handlers::get_report_verifications))
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_service.clone(),
            auth::middleware::require_auth,
        ))
        .with_state(verification_state)
        
        // Leaderboard routes (authenticated)
        .route("/api/leaderboards", get(handlers::get_global_leaderboard))
        .route("/api/leaderboards/city/:city", get(handlers::get_city_leaderboard))
        .route("/api/leaderboards/country/:country", get(handlers::get_country_leaderboard))
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_service.clone(),
            auth::middleware::require_auth,
        ))
        .with_state(leaderboard_state)
        
        .layer(rate_limiter)
        .layer(cors);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("Server starting on {}", addr);
    tracing::info!("API endpoints:");
    tracing::info!("  Authentication (public):");
    tracing::info!("    POST /api/auth/register");
    tracing::info!("    POST /api/auth/login");
    tracing::info!("    POST /api/auth/verify-email");
    tracing::info!("    POST /api/auth/resend-verification");
    tracing::info!("    POST /api/auth/forgot-password");
    tracing::info!("    POST /api/auth/reset-password");
    tracing::info!("    POST /api/auth/refresh");
    tracing::info!("    POST /api/auth/logout");
    tracing::info!("  User (authenticated):");
    tracing::info!("    GET  /api/users/me");
    tracing::info!("  Reports (authenticated):");
    tracing::info!("    POST /api/reports");
    tracing::info!("    GET  /api/reports/nearby?latitude=X&longitude=Y&radius_km=Z");
    tracing::info!("    GET  /api/reports/my-reports");
    tracing::info!("    GET  /api/reports/my-clears");
    tracing::info!("    GET  /api/reports/:id");
    tracing::info!("    POST /api/reports/:id/claim");
    tracing::info!("    POST /api/reports/:id/clear");
    tracing::info!("  Verifications (authenticated):");
    tracing::info!("    POST /api/reports/:id/verify");
    tracing::info!("    GET  /api/reports/:id/verifications");
    tracing::info!("  Leaderboards (authenticated):");
    tracing::info!("    GET  /api/leaderboards?period=weekly|monthly|all_time");
    tracing::info!("    GET  /api/leaderboards/city/:city?period=...");
    tracing::info!("    GET  /api/leaderboards/country/:country?period=...");
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
