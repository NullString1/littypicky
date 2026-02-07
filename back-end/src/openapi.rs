use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "LittyPicky API",
        version = "0.1.0",
        description = "A gamified litter-picking community app with geospatial features",
        contact(
            name = "LittyPicky Team",
            email = "team@littypicky.com"
        ),
        license(
            name = "MIT",
        )
    ),
    paths(
        // Auth endpoints
        crate::handlers::auth::register,
        crate::handlers::auth::login,
        crate::handlers::auth::verify_email,
        crate::handlers::auth::resend_verification,
        crate::handlers::auth::forgot_password,
        crate::handlers::auth::reset_password,
        crate::handlers::auth::refresh_token,
        crate::handlers::auth::logout,
        
        // OAuth endpoints
        crate::handlers::oauth::google_login,
        crate::handlers::oauth::google_callback,
        
        // User endpoints
        crate::handlers::users::get_current_user,
        crate::handlers::users::update_current_user,
        crate::handlers::users::get_current_user_score,
        
        // Report endpoints
        crate::handlers::reports::create_report,
        crate::handlers::reports::get_nearby_reports,
        crate::handlers::reports::get_my_reports,
        crate::handlers::reports::get_my_cleared_reports,
        crate::handlers::reports::get_report,
        crate::handlers::reports::claim_report,
        crate::handlers::reports::clear_report,
        
        // Verification endpoints
        crate::handlers::verifications::verify_report,
        crate::handlers::verifications::get_report_verifications,
        
        // Leaderboard endpoints
        crate::handlers::leaderboards::get_global_leaderboard,
        crate::handlers::leaderboards::get_city_leaderboard,
        crate::handlers::leaderboards::get_country_leaderboard,
        
        // Admin endpoints
        crate::handlers::admin::list_users,
        crate::handlers::admin::get_user_by_id,
        crate::handlers::admin::toggle_user_ban,
        crate::handlers::admin::list_all_reports,
        crate::handlers::admin::delete_report,
    ),
    components(
        schemas(
            // Auth models
            crate::handlers::auth::RegisterRequest,
            crate::handlers::auth::MessageResponse,
            crate::handlers::auth::RefreshTokenRequest,
            crate::handlers::auth::RefreshTokenResponse,
            crate::models::user::LoginRequest,
            crate::models::user::AuthTokens,
            crate::models::user::UserResponse,
            crate::models::user::UpdateUserRequest,
            crate::models::user::User,
            crate::models::user::UserRole,
            crate::models::email_token::VerifyEmailRequest,
            crate::models::email_token::ResendVerificationRequest,
            crate::models::email_token::ForgotPasswordRequest,
            crate::models::email_token::ResetPasswordRequest,
            
            // Report models
            crate::models::report::CreateReportRequest,
            crate::models::report::ClearReportRequest,
            crate::models::report::LitterReport,
            crate::models::report::ReportResponse,
            crate::models::report::ReportStatus,
            
            // Verification models
            crate::models::verification::VerifyReportRequest,
            crate::models::verification::ReportVerification,
            
            // Score models
            crate::models::score::UserScore,
            
            // Admin models
            crate::handlers::admin::BanUserRequest,
            crate::handlers::admin::AdminReportView,
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication and registration"),
        (name = "OAuth", description = "OAuth authentication with Google"),
        (name = "Users", description = "User profile management"),
        (name = "Reports", description = "Litter report management"),
        (name = "Verifications", description = "Report verification"),
        (name = "Leaderboards", description = "User rankings and leaderboards"),
        (name = "Admin", description = "Administrative endpoints (admin role required)"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Add JWT Bearer authentication to OpenAPI
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}
