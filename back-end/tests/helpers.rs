// Test helpers for integration tests

use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;

// Re-export modules for tests
use back_end::{auth, config, db, handlers, services};

pub async fn create_test_app() -> Router {
    // Load test environment variables
    dotenvy::from_filename(".env.test").ok();

    // Load test configuration
    let config = config::Config::from_env().expect("Failed to load config");

    // Create test database pool
    let pool = db::create_pool(&config)
        .await
        .expect("Failed to create pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Clean up test data before each test
    cleanup_test_data(&pool).await;

    build_test_router(config, pool).await
}

/// Helper to get a database pool for test helpers
#[allow(dead_code)]
pub async fn get_test_pool() -> sqlx::PgPool {
    dotenvy::from_filename(".env.test").ok();
    let config = config::Config::from_env().expect("Failed to load config");
    db::create_pool(&config)
        .await
        .expect("Failed to create pool")
}

async fn build_test_router(config: config::Config, pool: sqlx::PgPool) -> Router {
    // Initialize S3 service for tests
    let s3_service = services::S3Service::new(config.s3.clone())
        .await
        .expect("Failed to create S3 service");
    s3_service
        .initialize()
        .await
        .expect("Failed to initialize S3 bucket");

    // Initialize services
    let jwt_service = auth::JwtService::new(config.jwt.clone());
    // Use real email service with MailHog for tests
    let email_service =
        services::EmailService::new(config.email.clone()).expect("Failed to create email service");
    let image_service = services::ImageService::new(config.image.clone());
    let report_service = services::ReportService::new(pool.clone(), image_service.clone(), s3_service.clone());
    let feed_service = services::FeedService::new(pool.clone(), image_service, s3_service.clone());
    let scoring_service = services::ScoringService::new(pool.clone(), config.scoring.clone());

    let auth_service = Arc::new(services::AuthService::new(
        pool.clone(),
        jwt_service.clone(),
        email_service,
        config.clone(),
    ));

    let user_state = Arc::new(handlers::UserHandlerState { pool: pool.clone() });

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

    let leaderboard_state = Arc::new(handlers::LeaderboardHandlerState { pool: pool.clone() });

    let feed_state = Arc::new(handlers::FeedHandlerState {
        feed_service: feed_service.clone(),
    });

    // Build router - using nested routers to properly separate auth states
    use axum::routing::{delete, get, patch, post};

    // Auth routes (no auth middleware)
    let auth_router = Router::new()
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/verify-email", post(handlers::verify_email))
        .route(
            "/api/auth/resend-verification",
            post(handlers::resend_verification),
        )
        .route("/api/auth/forgot-password", post(handlers::forgot_password))
        .route("/api/auth/reset-password", post(handlers::reset_password))
        .route("/api/auth/refresh", post(handlers::refresh_token))
        .route("/api/auth/logout", post(handlers::logout))
        .with_state(auth_service.clone());

    // User routes (with auth middleware)
    let user_router = Router::new()
        .route("/api/users/me", get(handlers::get_current_user))
        .with_state(user_state)
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_service.clone(),
            auth::middleware::require_auth,
        ));

    // Report routes (with auth middleware)
    let report_router = Router::new()
        .route("/api/reports", post(handlers::create_report))
        .route("/api/reports/nearby", get(handlers::get_nearby_reports))
        .route("/api/reports/my-reports", get(handlers::get_my_reports))
        .route(
            "/api/reports/my-clears",
            get(handlers::get_my_cleared_reports),
        )
        .route("/api/reports/:id", get(handlers::get_report))
        .route("/api/reports/:id/claim", post(handlers::claim_report))
        .route("/api/reports/:id/clear", post(handlers::clear_report))
        .with_state(report_state)
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_service.clone(),
            auth::middleware::require_auth,
        ));

    // Verification routes (with auth middleware)
    let verification_router = Router::new()
        .route("/api/reports/:id/verify", post(handlers::verify_report))
        .route(
            "/api/reports/:id/verifications",
            get(handlers::get_report_verifications),
        )
        .with_state(verification_state)
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_service.clone(),
            auth::middleware::require_auth,
        ));

    // Leaderboard routes (with auth middleware)
    let leaderboard_router = Router::new()
        .route("/api/leaderboards", get(handlers::get_global_leaderboard))
        .route(
            "/api/leaderboards/city/:city",
            get(handlers::get_city_leaderboard),
        )
        .route(
            "/api/leaderboards/country/:country",
            get(handlers::get_country_leaderboard),
        )
        .with_state(leaderboard_state)
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_service.clone(),
            auth::middleware::require_auth,
        ));

    // Feed routes (with auth middleware)
    let feed_router = Router::new()
        .route("/api/feed", post(handlers::create_post))
        .route("/api/feed", get(handlers::get_feed))
        .route("/api/feed/:id", get(handlers::get_post))
        .route("/api/feed/:id", patch(handlers::update_post))
        .route("/api/feed/:id", delete(handlers::delete_post))
        .route("/api/feed/:post_id/comments", post(handlers::create_comment))
        .route("/api/feed/:post_id/comments", get(handlers::get_comments))
        .route(
            "/api/feed/comments/:comment_id",
            patch(handlers::update_comment),
        )
        .route(
            "/api/feed/comments/:comment_id",
            delete(handlers::delete_comment),
        )
        .route("/api/feed/:post_id/like", post(handlers::like_post))
        .route("/api/feed/:post_id/like", delete(handlers::unlike_post))
        .with_state(feed_state)
        .route_layer(axum::middleware::from_fn_with_state(
            jwt_service.clone(),
            auth::middleware::require_auth,
        ));

    // Combine all routers
    Router::new()
        .route("/", get(|| async { "LittyPicky API v0.1.0" }))
        .route("/health", get(health_check))
        .merge(auth_router)
        .merge(user_router)
        .merge(report_router)
        .merge(verification_router)
        .merge(leaderboard_router)
        .merge(feed_router)
}

async fn health_check() -> &'static str {
    "OK"
}

// Helper to clean up test data between tests
pub async fn cleanup_test_data(pool: &PgPool) {
    // Delete in correct order to respect foreign key constraints
    sqlx::query!("DELETE FROM report_verifications")
        .execute(pool)
        .await
        .expect("Failed to clean report_verifications");

    sqlx::query!("DELETE FROM user_scores")
        .execute(pool)
        .await
        .expect("Failed to clean user_scores");

    sqlx::query!("DELETE FROM litter_reports")
        .execute(pool)
        .await
        .expect("Failed to clean litter_reports");

    sqlx::query!("DELETE FROM feed_post_likes")
        .execute(pool)
        .await
        .expect("Failed to clean feed_post_likes");

    sqlx::query!("DELETE FROM feed_comments")
        .execute(pool)
        .await
        .expect("Failed to clean feed_comments");

    sqlx::query!("DELETE FROM feed_post_images")
        .execute(pool)
        .await
        .expect("Failed to clean feed_post_images");

    sqlx::query!("DELETE FROM feed_posts")
        .execute(pool)
        .await
        .expect("Failed to clean feed_posts");

    sqlx::query!("DELETE FROM refresh_tokens")
        .execute(pool)
        .await
        .expect("Failed to clean refresh_tokens");

    sqlx::query!("DELETE FROM email_verification_tokens")
        .execute(pool)
        .await
        .expect("Failed to clean email_verification_tokens");

    sqlx::query!("DELETE FROM password_reset_tokens")
        .execute(pool)
        .await
        .expect("Failed to clean password_reset_tokens");

    sqlx::query!("DELETE FROM users")
        .execute(pool)
        .await
        .expect("Failed to clean users");
}
