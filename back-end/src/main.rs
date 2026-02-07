mod config;
mod db;
mod error;
mod models;
mod auth;
mod services;
mod templates;
mod handlers;
mod rate_limit;
mod openapi;

use axum::{
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use openapi::ApiDoc;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
    let oauth_service = Arc::new(services::OAuthService::new(config.oauth.clone()).await?);
    
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

    let oauth_state = Arc::new(handlers::OAuthHandlerState {
        oauth_service: oauth_service.clone(),
        auth_service: auth_service.clone(),
        session_store: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    });

    let admin_state = Arc::new(handlers::AdminHandlerState {
        pool: pool.clone(),
    });

    tracing::info!("Services initialized");

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create per-endpoint rate limiters
    let auth_rate_limiter = rate_limit::create_rate_limiter(config.rate_limit.auth_per_min);
    let reports_rate_limiter = rate_limit::create_rate_limiter_per_hour(config.rate_limit.reports_per_hour);
    let verifications_rate_limiter = rate_limit::create_rate_limiter_per_hour(config.rate_limit.verifications_per_hour);
    let general_rate_limiter = rate_limit::create_rate_limiter(config.rate_limit.general_per_min);
    let email_verification_limiter = rate_limit::create_rate_limiter_per_hour(config.rate_limit.email_verification_per_hour);
    let password_reset_limiter = rate_limit::create_rate_limiter_per_hour(config.rate_limit.password_reset_per_hour);

    // Build router
    let app = Router::new()
        // Health check
        .route("/", get(|| async { "LittyPicky API v0.1.0" }))
        .route("/health", get(health_check))
        
        // OpenAPI/Swagger documentation
        .route("/api/openapi.json", get(openapi_json))
        .merge(SwaggerUi::new("/swagger-ui").url("/api/openapi.json", ApiDoc::openapi()))
        
        // Auth routes (public) with different rate limits
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/verify-email", post(handlers::verify_email))
        .route("/api/auth/refresh", post(handlers::refresh_token))
        .route("/api/auth/logout", post(handlers::logout))
        .with_state(auth_service.clone())
        .layer(auth_rate_limiter.clone())
        
        .route("/api/auth/resend-verification", post(handlers::resend_verification))
        .with_state(auth_service.clone())
        .layer(email_verification_limiter.clone())
        
        .route("/api/auth/forgot-password", post(handlers::forgot_password))
        .route("/api/auth/reset-password", post(handlers::reset_password))
        .with_state(auth_service.clone())
        .layer(password_reset_limiter.clone())
        
        // OAuth routes (public)
        .route("/api/auth/google", get(handlers::google_login))
        .route("/api/auth/google/callback", get(handlers::google_callback))
        .with_state(oauth_state)
        .layer(auth_rate_limiter.clone())
        
        // User routes (authenticated)
        .route("/api/users/me", get(handlers::get_current_user))
        .route("/api/users/me", patch(handlers::update_current_user))
        .route("/api/users/me/score", get(handlers::get_current_user_score))
        .with_state(user_state)
        .layer(general_rate_limiter.clone())
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_service.clone(),
            auth::middleware::require_auth,
        ))
        
        // Report routes (authenticated)
        .route("/api/reports", post(handlers::create_report))
        .route("/api/reports/nearby", get(handlers::get_nearby_reports))
        .route("/api/reports/my-reports", get(handlers::get_my_reports))
        .route("/api/reports/my-clears", get(handlers::get_my_cleared_reports))
        .route("/api/reports/:id", get(handlers::get_report))
        .route("/api/reports/:id/claim", post(handlers::claim_report))
        .route("/api/reports/:id/clear", post(handlers::clear_report))
        .with_state(report_state)
        .layer(reports_rate_limiter.clone())
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_service.clone(),
            auth::middleware::require_auth,
        ))
        
        // Verification routes (authenticated)
        .route("/api/reports/:id/verify", post(handlers::verify_report))
        .route("/api/reports/:id/verifications", get(handlers::get_report_verifications))
        .with_state(verification_state)
        .layer(verifications_rate_limiter.clone())
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_service.clone(),
            auth::middleware::require_auth,
        ))
        
        // Leaderboard routes (authenticated)
        .route("/api/leaderboards", get(handlers::get_global_leaderboard))
        .route("/api/leaderboards/city/:city", get(handlers::get_city_leaderboard))
        .route("/api/leaderboards/country/:country", get(handlers::get_country_leaderboard))
        .with_state(leaderboard_state)
        .layer(general_rate_limiter.clone())
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_service.clone(),
            auth::middleware::require_auth,
        ))
        
        // Admin routes (authenticated + admin role required)
        .route("/api/admin/users", get(handlers::list_users))
        .route("/api/admin/users/:id", get(handlers::get_user_by_id))
        .route("/api/admin/users/:id/ban", put(handlers::toggle_user_ban))
        .route("/api/admin/reports", get(handlers::list_all_reports))
        .route("/api/admin/reports/:id", delete(handlers::delete_report))
        .with_state(admin_state)
        .layer(general_rate_limiter.clone())
        .route_layer(axum::middleware::from_fn(auth::middleware::require_admin))
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_service.clone(),
            auth::middleware::require_auth,
        ))
        
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
    tracing::info!("  Admin (authenticated, admin role required):");
    tracing::info!("    GET    /api/admin/users");
    tracing::info!("    GET    /api/admin/users/:id");
    tracing::info!("    PUT    /api/admin/users/:id/ban");
    tracing::info!("    GET    /api/admin/reports");
    tracing::info!("    DELETE /api/admin/reports/:id");
    tracing::info!("  Documentation:");
    tracing::info!("    GET  /api/openapi.json - OpenAPI 3.0 specification");
    tracing::info!("    GET  /swagger-ui - Interactive API documentation");
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

/// Returns the OpenAPI JSON specification
async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
