use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// DATABASE MODELS
// ============================================================================

#[derive(Debug, Clone, FromRow, ToSchema)]
pub struct FeedPost {
    pub id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub like_count: i32,
    pub comment_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, ToSchema)]
pub struct FeedPostImage {
    pub id: Uuid,
    pub post_id: Uuid,
    pub image_url: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct FeedComment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, ToSchema)]
pub struct FeedPostLike {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// API RESPONSE MODELS
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct FeedPostResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    #[schema(example = "John Doe")]
    pub author_name: String,
    pub author_avatar: Option<String>,
    pub content: String,
    pub images: Vec<String>,
    pub like_count: i32,
    pub comment_count: i32,
    pub comments: Vec<FeedCommentResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FeedCommentResponse {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Option<Uuid>,
    #[schema(example = "Jane Smith")]
    pub author_name: Option<String>,
    pub author_avatar: Option<String>,
    pub content: String,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// REQUEST DTOs
// ============================================================================

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateFeedPostRequest {
    #[validate(length(min = 1, max = 500))]
    #[schema(example = "Just cleaned up the local park!")]
    pub content: String,
    #[validate(length(max = 10))]
    pub images: Vec<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateFeedPostRequest {
    #[validate(length(min = 1, max = 500))]
    #[schema(example = "Updated: Just cleaned up the local park!")]
    pub content: String,
    #[validate(length(max = 10))]
    pub images: Vec<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateFeedCommentRequest {
    #[validate(length(min = 1, max = 250))]
    #[schema(example = "Great work! Thanks for cleaning up!")]
    pub content: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateFeedCommentRequest {
    #[validate(length(min = 1, max = 250))]
    #[schema(example = "Updated: Great work! Thanks for cleaning up!")]
    pub content: String,
}

// ============================================================================
// QUERY PARAMETERS
// ============================================================================

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct FeedQueryParams {
    #[schema(example = 0)]
    pub offset: Option<i32>,
    #[schema(example = 20)]
    pub limit: Option<i32>,
}

impl FeedQueryParams {
    pub fn offset(&self) -> i32 {
        self.offset.unwrap_or(0).max(0)
    }

    pub fn limit(&self) -> i32 {
        let limit = self.limit.unwrap_or(20);
        limit.clamp(1, 100) // Prevent extremely large requests
    }
}
