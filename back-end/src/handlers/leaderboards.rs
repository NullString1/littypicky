use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::models::score::LeaderboardEntry;
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct LeaderboardHandlerState {
    pub pool: PgPool,
}

#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    pub period: Option<String>, // "weekly", "monthly", "all_time"
}

/// Get global leaderboard
/// GET /api/leaderboards?period=weekly
pub async fn get_global_leaderboard(
    State(state): State<Arc<LeaderboardHandlerState>>,
    _auth_user: AuthUser,
    Query(query): Query<LeaderboardQuery>,
) -> Result<impl IntoResponse, AppError> {
    let leaderboard = get_leaderboard(&state.pool, None, None, query.period).await?;
    Ok(Json(leaderboard))
}

/// Get leaderboard by city
/// GET /api/leaderboards/city/:city?period=weekly
pub async fn get_city_leaderboard(
    State(state): State<Arc<LeaderboardHandlerState>>,
    _auth_user: AuthUser,
    Path(city): Path<String>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<impl IntoResponse, AppError> {
    let leaderboard = get_leaderboard(&state.pool, Some(city), None, query.period).await?;
    Ok(Json(leaderboard))
}

/// Get leaderboard by country
/// GET /api/leaderboards/country/:country?period=weekly
pub async fn get_country_leaderboard(
    State(state): State<Arc<LeaderboardHandlerState>>,
    _auth_user: AuthUser,
    Path(country): Path<String>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<impl IntoResponse, AppError> {
    let leaderboard = get_leaderboard(&state.pool, None, Some(country), query.period).await?;
    Ok(Json(leaderboard))
}

/// Internal helper to build leaderboard query
async fn get_leaderboard(
    pool: &PgPool,
    city: Option<String>,
    country: Option<String>,
    period: Option<String>,
) -> Result<Vec<LeaderboardEntry>, AppError> {
    // Calculate time filter based on period
    let time_filter = match period.as_deref() {
        Some("weekly") => Some(Utc::now() - Duration::weeks(1)),
        Some("monthly") => Some(Utc::now() - Duration::days(30)),
        Some("all_time") | None => None,
        _ => {
            return Err(AppError::BadRequest(
                "Invalid period. Use 'weekly', 'monthly', or 'all_time'".to_string(),
            ))
        }
    };

    // Build the query dynamically based on filters
    let leaderboard = if let Some(time) = time_filter {
        // Time-based leaderboard (recent activity) - don't need user_scores for time-based
        if let Some(c) = city {
            // City + time filter
            sqlx::query_as!(
                LeaderboardEntry,
                r#"
                SELECT 
                    u.id as user_id,
                    u.full_name,
                    u.city,
                    u.country,
                    COALESCE(SUM(CASE 
                        WHEN lr.cleared_at > $1 THEN 10 
                        ELSE 0 
                    END), 0)::int as "total_points!",
                    COUNT(CASE WHEN lr.cleared_at > $1 THEN 1 END)::int as "reports_cleared!",
                    0 as "current_streak!",
                    ROW_NUMBER() OVER (ORDER BY SUM(CASE WHEN lr.cleared_at > $1 THEN 10 ELSE 0 END) DESC) as "rank!"
                FROM users u
                LEFT JOIN litter_reports lr ON u.id = lr.cleared_by
                WHERE u.city = $2
                GROUP BY u.id, u.full_name, u.city, u.country
                HAVING COUNT(CASE WHEN lr.cleared_at > $1 THEN 1 END) > 0
                ORDER BY SUM(CASE WHEN lr.cleared_at > $1 THEN 10 ELSE 0 END) DESC
                LIMIT 20
                "#,
                time,
                c
            )
            .fetch_all(pool)
            .await?
        } else if let Some(co) = country {
            // Country + time filter
            sqlx::query_as!(
                LeaderboardEntry,
                r#"
                SELECT 
                    u.id as user_id,
                    u.full_name,
                    u.city,
                    u.country,
                    COALESCE(SUM(CASE 
                        WHEN lr.cleared_at > $1 THEN 10 
                        ELSE 0 
                    END), 0)::int as "total_points!",
                    COUNT(CASE WHEN lr.cleared_at > $1 THEN 1 END)::int as "reports_cleared!",
                    0 as "current_streak!",
                    ROW_NUMBER() OVER (ORDER BY SUM(CASE WHEN lr.cleared_at > $1 THEN 10 ELSE 0 END) DESC) as "rank!"
                FROM users u
                LEFT JOIN litter_reports lr ON u.id = lr.cleared_by
                WHERE u.country = $2
                GROUP BY u.id, u.full_name, u.city, u.country
                HAVING COUNT(CASE WHEN lr.cleared_at > $1 THEN 1 END) > 0
                ORDER BY SUM(CASE WHEN lr.cleared_at > $1 THEN 10 ELSE 0 END) DESC
                LIMIT 20
                "#,
                time,
                co
            )
            .fetch_all(pool)
            .await?
        } else {
            // Global + time filter
            sqlx::query_as!(
                LeaderboardEntry,
                r#"
                SELECT 
                    u.id as user_id,
                    u.full_name,
                    u.city,
                    u.country,
                    COALESCE(SUM(CASE 
                        WHEN lr.cleared_at > $1 THEN 10 
                        ELSE 0 
                    END), 0)::int as "total_points!",
                    COUNT(CASE WHEN lr.cleared_at > $1 THEN 1 END)::int as "reports_cleared!",
                    0 as "current_streak!",
                    ROW_NUMBER() OVER (ORDER BY SUM(CASE WHEN lr.cleared_at > $1 THEN 10 ELSE 0 END) DESC) as "rank!"
                FROM users u
                LEFT JOIN litter_reports lr ON u.id = lr.cleared_by
                GROUP BY u.id, u.full_name, u.city, u.country
                HAVING COUNT(CASE WHEN lr.cleared_at > $1 THEN 1 END) > 0
                ORDER BY SUM(CASE WHEN lr.cleared_at > $1 THEN 10 ELSE 0 END) DESC
                LIMIT 20
                "#,
                time
            )
            .fetch_all(pool)
            .await?
        }
    } else {
        // All-time leaderboard (use user_scores table)
        if let Some(c) = city {
            // City filter
            sqlx::query_as!(
                LeaderboardEntry,
                r#"
                SELECT 
                    u.id as user_id,
                    u.full_name,
                    u.city,
                    u.country,
                    us.total_points,
                    us.reports_cleared,
                    us.current_streak,
                    ROW_NUMBER() OVER (ORDER BY us.total_points DESC) as "rank!"
                FROM users u
                INNER JOIN user_scores us ON u.id = us.user_id
                WHERE u.city = $1 AND us.reports_cleared > 0
                ORDER BY us.total_points DESC
                LIMIT 20
                "#,
                c
            )
            .fetch_all(pool)
            .await?
        } else if let Some(co) = country {
            // Country filter
            sqlx::query_as!(
                LeaderboardEntry,
                r#"
                SELECT 
                    u.id as user_id,
                    u.full_name,
                    u.city,
                    u.country,
                    us.total_points,
                    us.reports_cleared,
                    us.current_streak,
                    ROW_NUMBER() OVER (ORDER BY us.total_points DESC) as "rank!"
                FROM users u
                INNER JOIN user_scores us ON u.id = us.user_id
                WHERE u.country = $1 AND us.reports_cleared > 0
                ORDER BY us.total_points DESC
                LIMIT 20
                "#,
                co
            )
            .fetch_all(pool)
            .await?
        } else {
            // Global
            sqlx::query_as!(
                LeaderboardEntry,
                r#"
                SELECT 
                    u.id as user_id,
                    u.full_name,
                    u.city,
                    u.country,
                    us.total_points,
                    us.reports_cleared,
                    us.current_streak,
                    ROW_NUMBER() OVER (ORDER BY us.total_points DESC) as "rank!"
                FROM users u
                INNER JOIN user_scores us ON u.id = us.user_id
                WHERE us.reports_cleared > 0
                ORDER BY us.total_points DESC
                LIMIT 20
                "#
            )
            .fetch_all(pool)
            .await?
        }
    };

    Ok(leaderboard)
}
