use crate::config::ScoringConfig;
use crate::error::AppError;
use crate::models::score::UserScore;
use chrono::{Duration, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ScoringService {
    pool: PgPool,
    config: ScoringConfig,
}

impl ScoringService {
    pub fn new(pool: PgPool, config: ScoringConfig) -> Self {
        Self { pool, config }
    }

    /// Calculate and award points when a user clears a report
    pub async fn award_clear_points(
        &self,
        user_id: Uuid,
        report_id: Uuid,
        latitude: f64,
        longitude: f64,
    ) -> Result<UserScore, AppError> {
        // Get or create user score
        let user_score = self.get_or_create_user_score(user_id).await?;

        // Calculate base points
        let mut points = self.config.base_points_per_clear;

        // Calculate streak bonus
        let today = Utc::now().date_naive();
        let (new_streak, is_streak_continued) = self.calculate_streak(&user_score, today);
        let streak_bonus = new_streak * self.config.streak_bonus_points;
        points += streak_bonus;

        // Check if this is the first clear in the area (1km radius, last 24 hours)
        let is_first_in_area = self
            .is_first_clear_in_area(latitude, longitude, user_id)
            .await?;
        if is_first_in_area {
            points += self.config.first_in_area_bonus;
        }

        // Update user score
        let new_total_points = user_score.total_points + points;
        let new_reports_cleared = user_score.reports_cleared + 1;
        let new_longest_streak = new_streak.max(user_score.longest_streak);

        let updated_score = sqlx::query_as!(
            UserScore,
            r#"
            UPDATE user_scores
            SET total_points = $1,
                reports_cleared = $2,
                current_streak = $3,
                longest_streak = $4,
                last_cleared_date = $5
            WHERE user_id = $6
            RETURNING id, user_id, total_points, reports_cleared,
                      current_streak, longest_streak, last_cleared_date,
                      created_at, updated_at
            "#,
            new_total_points,
            new_reports_cleared,
            new_streak,
            new_longest_streak,
            today,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_score)
    }

    /// Award points to a user who verified a report
    pub async fn award_verification_points(&self, user_id: Uuid) -> Result<UserScore, AppError> {
        let user_score = self.get_or_create_user_score(user_id).await?;
        let new_total = user_score.total_points + self.config.verification_bonus;

        let updated_score = sqlx::query_as!(
            UserScore,
            r#"
            UPDATE user_scores
            SET total_points = $1
            WHERE user_id = $2
            RETURNING id, user_id, total_points, reports_cleared,
                      current_streak, longest_streak, last_cleared_date,
                      created_at, updated_at
            "#,
            new_total,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_score)
    }

    /// Award bonus points when a report gets verified (to the clearer)
    pub async fn award_verified_report_bonus(
        &self,
        clearer_id: Uuid,
    ) -> Result<UserScore, AppError> {
        let user_score = self.get_or_create_user_score(clearer_id).await?;
        let new_total = user_score.total_points + self.config.verified_report_bonus;

        let updated_score = sqlx::query_as!(
            UserScore,
            r#"
            UPDATE user_scores
            SET total_points = $1
            WHERE user_id = $2
            RETURNING id, user_id, total_points, reports_cleared,
                      current_streak, longest_streak, last_cleared_date,
                      created_at, updated_at
            "#,
            new_total,
            clearer_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_score)
    }

    /// Get or create a user's score record
    async fn get_or_create_user_score(&self, user_id: Uuid) -> Result<UserScore, AppError> {
        // Try to get existing score
        if let Some(score) = sqlx::query_as!(
            UserScore,
            r#"
            SELECT id, user_id, total_points, reports_cleared,
                   current_streak, longest_streak, last_cleared_date,
                   created_at, updated_at
            FROM user_scores
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
        {
            return Ok(score);
        }

        // Create new score record
        let new_score = sqlx::query_as!(
            UserScore,
            r#"
            INSERT INTO user_scores (user_id, total_points, reports_cleared, current_streak, longest_streak)
            VALUES ($1, 0, 0, 0, 0)
            RETURNING id, user_id, total_points, reports_cleared,
                      current_streak, longest_streak, last_cleared_date,
                      created_at, updated_at
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(new_score)
    }

    /// Calculate the new streak based on last cleared date
    fn calculate_streak(&self, user_score: &UserScore, today: NaiveDate) -> (i32, bool) {
        if let Some(last_date) = user_score.last_cleared_date {
            let days_diff = (today - last_date).num_days();

            match days_diff {
                0 => {
                    // Same day - keep current streak
                    (user_score.current_streak, false)
                }
                1 => {
                    // Consecutive day - increment streak
                    (user_score.current_streak + 1, true)
                }
                _ => {
                    // Streak broken - start new streak
                    (1, false)
                }
            }
        } else {
            // First clear ever - start streak at 1
            (1, true)
        }
    }

    /// Check if this is the first clear in the area (1km, 24 hours)
    async fn is_first_clear_in_area(
        &self,
        latitude: f64,
        longitude: f64,
        user_id: Uuid,
    ) -> Result<bool, AppError> {
        let radius_meters = 1000.0; // 1km
        let time_threshold = Utc::now() - Duration::hours(24);

        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM litter_reports
            WHERE cleared_by IS NOT NULL
              AND cleared_at > $1
              AND cleared_by != $2
              AND ST_DWithin(
                  location,
                  ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography,
                  $5
              )
            "#,
            time_threshold,
            user_id,
            longitude,
            latitude,
            radius_meters
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.unwrap_or(0) == 0)
    }

    /// Get user score by user ID
    pub async fn get_user_score(&self, user_id: Uuid) -> Result<UserScore, AppError> {
        self.get_or_create_user_score(user_id).await
    }

    /// Check if user can verify reports (has cleared enough reports)
    pub async fn can_verify_reports(&self, user_id: Uuid) -> Result<bool, AppError> {
        let score = self.get_or_create_user_score(user_id).await?;
        Ok(score.reports_cleared >= self.config.min_clears_to_verify)
    }
}
