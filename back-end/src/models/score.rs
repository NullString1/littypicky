use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct UserScore {
    pub id: Uuid,
    pub user_id: Uuid,
    pub total_points: i32,
    pub reports_cleared: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub last_cleared_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ScoreResponse {
    pub user_id: Uuid,
    pub total_points: i32,
    pub reports_cleared: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
}

impl From<UserScore> for ScoreResponse {
    fn from(score: UserScore) -> Self {
        ScoreResponse {
            user_id: score.user_id,
            total_points: score.total_points,
            reports_cleared: score.reports_cleared,
            current_streak: score.current_streak,
            longest_streak: score.longest_streak,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LeaderboardEntry {
    pub user_id: Uuid,
    pub full_name: String,
    pub city: String,
    pub country: String,
    pub total_points: i32,
    pub reports_cleared: i32,
    pub current_streak: i32,
    pub rank: i64,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct LeaderboardQuery {
    #[param(example = "weekly")]
    pub period: Option<String>, // "weekly", "monthly", "all_time"
}
