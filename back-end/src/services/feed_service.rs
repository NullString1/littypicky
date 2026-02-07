use crate::error::AppError;
use crate::models::feed::{
    CreateFeedCommentRequest, CreateFeedPostRequest, FeedComment, FeedCommentResponse,
    FeedPost, FeedPostResponse, UpdateFeedCommentRequest, UpdateFeedPostRequest,
};
use crate::models::user::User;
use crate::services::image_service::ImageService;
use crate::services::s3_service::S3Service;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct FeedService {
    pool: PgPool,
    image_service: ImageService,
    s3_service: S3Service,
}

impl FeedService {
    #[must_use]
    pub fn new(
        pool: PgPool,
        image_service: ImageService,
        s3_service: S3Service,
    ) -> Self {
        Self {
            pool,
            image_service,
            s3_service,
        }
    }

    // ========================================================================
    // POST OPERATIONS
    // ========================================================================

    /// Create a new feed post with multiple images
    pub async fn create_post(
        &self,
        user_id: Uuid,
        request: CreateFeedPostRequest,
    ) -> Result<FeedPostResponse, AppError> {
        // Validate content
        if request.content.trim().is_empty() || request.content.len() > 500 {
            return Err(AppError::BadRequest(
                "Content must be between 1 and 500 characters".to_string(),
            ));
        }

        if request.images.len() > 10 {
            return Err(AppError::BadRequest(
                "Maximum 10 images per post".to_string(),
            ));
        }

        // Begin transaction for atomic operation
        let mut tx = self.pool.begin().await?;

        // Insert the post
        let post = sqlx::query_as!(
            FeedPost,
            r#"
            INSERT INTO feed_posts (user_id, content, like_count, comment_count)
            VALUES ($1, $2, 0, 0)
            RETURNING id, user_id, content, like_count, comment_count, created_at, updated_at
            "#,
            user_id,
            request.content.trim()
        )
        .fetch_one(&mut *tx)
        .await?;

        // Process and upload images if any
        let mut image_urls = Vec::new();
        for (position, image_base64) in request.images.iter().enumerate() {
            // Process image (compress to WebP, etc.)
            let processed_image = self.image_service.process_image(image_base64.clone()).await?;

            // Upload to S3
            let image_url = self
                .s3_service
                .upload_image(processed_image, "feed/posts")
                .await?;

            image_urls.push(image_url.clone());

            // Insert image record
            sqlx::query!(
                r#"
                INSERT INTO feed_post_images (post_id, image_url, position)
                VALUES ($1, $2, $3)
                "#,
                post.id,
                image_url,
                position as i32
            )
            .execute(&mut *tx)
            .await?;
        }

        // Commit transaction
        tx.commit().await?;

        // Fetch user info for response
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, password_hash, full_name, city, country,
                   search_radius_km, role as "role: crate::models::user::UserRole",
                   is_active, email_verified, email_verified_at, oauth_provider, oauth_subject,
                   created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(FeedPostResponse {
            id: post.id,
            user_id: post.user_id,
            author_name: user.full_name,
            author_avatar: None,
            content: post.content,
            images: image_urls,
            like_count: post.like_count,
            comment_count: post.comment_count,
            comments: Vec::new(),
            created_at: post.created_at,
            updated_at: post.updated_at,
        })
    }

    /// Get paginated feed posts
    pub async fn get_feed(&self, offset: i32, limit: i32) -> Result<Vec<FeedPostResponse>, AppError> {
        let limit = limit.clamp(1, 100);
        let offset = offset.max(0);

        // Fetch posts with user info
        let posts = sqlx::query!(
            r#"
            SELECT 
                fp.id, fp.user_id, fp.content, fp.like_count, fp.comment_count,
                fp.created_at, fp.updated_at,
                u.full_name
            FROM feed_posts fp
            JOIN users u ON fp.user_id = u.id
            ORDER BY fp.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let mut responses = Vec::new();
        for post in posts {
            // Fetch images for this post
            let images: Vec<String> = sqlx::query!(
                "SELECT image_url FROM feed_post_images WHERE post_id = $1 ORDER BY position",
                post.id
            )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|img| img.image_url)
            .collect();

            // Fetch comments for this post
            let comments = self.get_comments_for_post(post.id).await?;

            responses.push(FeedPostResponse {
                id: post.id,
                user_id: post.user_id,
                author_name: post.full_name,
                author_avatar: None,
                content: post.content,
                images,
                like_count: post.like_count,
                comment_count: post.comment_count,
                comments,
                created_at: post.created_at,
                updated_at: post.updated_at,
            });
        }

        Ok(responses)
    }

    /// Get a single post by ID
    pub async fn get_post(&self, post_id: Uuid) -> Result<FeedPostResponse, AppError> {
        let post = sqlx::query!(
            r#"
            SELECT 
                fp.id, fp.user_id, fp.content, fp.like_count, fp.comment_count,
                fp.created_at, fp.updated_at,
                u.full_name
            FROM feed_posts fp
            JOIN users u ON fp.user_id = u.id
            WHERE fp.id = $1
            "#,
            post_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

        // Fetch images
        let images: Vec<String> = sqlx::query!(
            "SELECT image_url FROM feed_post_images WHERE post_id = $1 ORDER BY position",
            post_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|img| img.image_url)
        .collect();

        // Fetch comments
        let comments = self.get_comments_for_post(post_id).await?;

        Ok(FeedPostResponse {
            id: post.id,
            user_id: post.user_id,
            author_name: post.full_name,
            author_avatar: None,
            content: post.content,
            images,
            like_count: post.like_count,
            comment_count: post.comment_count,
            comments,
            created_at: post.created_at,
            updated_at: post.updated_at,
        })
    }

    /// Update a post (ownership required)
    pub async fn update_post(
        &self,
        post_id: Uuid,
        user_id: Uuid,
        request: UpdateFeedPostRequest,
    ) -> Result<FeedPostResponse, AppError> {
        // Verify ownership
        let post = sqlx::query!("SELECT user_id FROM feed_posts WHERE id = $1", post_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

        if post.user_id != user_id {
            return Err(AppError::Forbidden(
                "You can only edit your own posts".to_string(),
            ));
        }

        if request.content.trim().is_empty() || request.content.len() > 500 {
            return Err(AppError::BadRequest(
                "Content must be between 1 and 500 characters".to_string(),
            ));
        }

        if request.images.len() > 10 {
            return Err(AppError::BadRequest(
                "Maximum 10 images per post".to_string(),
            ));
        }

        // Begin transaction
        let mut tx = self.pool.begin().await?;

        // Update post content and timestamp
        sqlx::query!(
            "UPDATE feed_posts SET content = $1, updated_at = NOW() WHERE id = $2",
            request.content.trim(),
            post_id
        )
        .execute(&mut *tx)
        .await?;

        // Delete old images
        sqlx::query!("DELETE FROM feed_post_images WHERE post_id = $1", post_id)
            .execute(&mut *tx)
            .await?;

        // Upload new images
        let mut image_urls = Vec::new();
        for (position, image_base64) in request.images.iter().enumerate() {
            let processed_image = self.image_service.process_image(image_base64.clone()).await?;
            let image_url = self
                .s3_service
                .upload_image(processed_image, "feed/posts")
                .await?;

            image_urls.push(image_url.clone());

            sqlx::query!(
                "INSERT INTO feed_post_images (post_id, image_url, position) VALUES ($1, $2, $3)",
                post_id,
                image_url,
                position as i32
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // Fetch updated post
        self.get_post(post_id).await
    }

    /// Delete a post (ownership or admin required)
    pub async fn delete_post(&self, post_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        // Verify ownership
        let post = sqlx::query!("SELECT user_id FROM feed_posts WHERE id = $1", post_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

        if post.user_id != user_id {
            return Err(AppError::Forbidden(
                "You can only delete your own posts".to_string(),
            ));
        }

        // Delete post (cascade will handle images, comments, likes)
        sqlx::query!("DELETE FROM feed_posts WHERE id = $1", post_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ========================================================================
    // COMMENT OPERATIONS
    // ========================================================================

    /// Create a comment on a post
    pub async fn create_comment(
        &self,
        post_id: Uuid,
        user_id: Uuid,
        request: CreateFeedCommentRequest,
    ) -> Result<FeedComment, AppError> {
        // Verify post exists
        let _post = sqlx::query!("SELECT id FROM feed_posts WHERE id = $1", post_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

        if request.content.trim().is_empty() || request.content.len() > 250 {
            return Err(AppError::BadRequest(
                "Comment must be between 1 and 250 characters".to_string(),
            ));
        }

        // Begin transaction for atomic increment
        let mut tx = self.pool.begin().await?;

        // Create comment
        let comment = sqlx::query_as!(
            FeedComment,
            r#"
            INSERT INTO feed_comments (post_id, user_id, content, is_deleted)
            VALUES ($1, $2, $3, false)
            RETURNING id, post_id, user_id, content, is_deleted, created_at, updated_at
            "#,
            post_id,
            user_id,
            request.content.trim()
        )
        .fetch_one(&mut *tx)
        .await?;

        // Increment post comment count
        sqlx::query!(
            "UPDATE feed_posts SET comment_count = comment_count + 1 WHERE id = $1",
            post_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(comment)
    }

    /// Get comments for a post (internal helper)
    async fn get_comments_for_post(&self, post_id: Uuid) -> Result<Vec<FeedCommentResponse>, AppError> {
        let comments = sqlx::query!(
            r#"
            SELECT fc.id, fc.post_id, fc.user_id, fc.content, fc.is_deleted,
                   fc.created_at, fc.updated_at, u.full_name
            FROM feed_comments fc
            LEFT JOIN users u ON fc.user_id = u.id
            WHERE fc.post_id = $1
            ORDER BY fc.created_at ASC
            "#,
            post_id
        )
        .fetch_all(&self.pool)
        .await?;

        let responses = comments
            .into_iter()
            .map(|c| FeedCommentResponse {
                id: c.id,
                post_id: c.post_id,
                user_id: if c.is_deleted { None } else { Some(c.user_id) },
                author_name: if c.is_deleted { None } else { Some(c.full_name) },
                author_avatar: None,
                content: if c.is_deleted {
                    "[deleted]".to_string()
                } else {
                    c.content
                },
                is_deleted: c.is_deleted,
                created_at: c.created_at,
                updated_at: c.updated_at,
            })
            .collect();

        Ok(responses)
    }

    /// Get comments for a post (public API method)
    pub async fn get_comments(&self, post_id: Uuid) -> Result<Vec<FeedCommentResponse>, AppError> {
        // Verify post exists
        let _post = sqlx::query!("SELECT id FROM feed_posts WHERE id = $1", post_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

        self.get_comments_for_post(post_id).await
    }

    /// Update a comment (ownership required)
    pub async fn update_comment(
        &self,
        comment_id: Uuid,
        user_id: Uuid,
        request: UpdateFeedCommentRequest,
    ) -> Result<FeedComment, AppError> {
        // Verify ownership
        let comment = sqlx::query!("SELECT user_id FROM feed_comments WHERE id = $1", comment_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        if comment.user_id != user_id {
            return Err(AppError::Forbidden(
                "You can only edit your own comments".to_string(),
            ));
        }

        if request.content.trim().is_empty() || request.content.len() > 250 {
            return Err(AppError::BadRequest(
                "Comment must be between 1 and 250 characters".to_string(),
            ));
        }

        let updated = sqlx::query_as!(
            FeedComment,
            r#"
            UPDATE feed_comments
            SET content = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, post_id, user_id, content, is_deleted, created_at, updated_at
            "#,
            request.content.trim(),
            comment_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    /// Delete a comment (soft-delete, ownership required)
    pub async fn delete_comment(&self, comment_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        // Verify ownership
        let comment = sqlx::query!(
            "SELECT user_id, post_id FROM feed_comments WHERE id = $1",
            comment_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        if comment.user_id != user_id {
            return Err(AppError::Forbidden(
                "You can only delete your own comments".to_string(),
            ));
        }

        // Begin transaction for atomic soft-delete + decrement
        let mut tx = self.pool.begin().await?;

        // Soft delete comment
        sqlx::query!(
            "UPDATE feed_comments SET is_deleted = true, updated_at = NOW() WHERE id = $1",
            comment_id
        )
        .execute(&mut *tx)
        .await?;

        // Decrement post comment count
        sqlx::query!(
            "UPDATE feed_posts SET comment_count = comment_count - 1 WHERE id = $1",
            comment.post_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    // ========================================================================
    // LIKE OPERATIONS
    // ========================================================================

    /// Like a post (idempotent)
    pub async fn like_post(&self, post_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
        // Verify post exists
        let _post = sqlx::query!("SELECT id FROM feed_posts WHERE id = $1", post_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

        // Begin transaction
        let mut tx = self.pool.begin().await?;

        // Check if already liked
        let existing = sqlx::query!(
            "SELECT id FROM feed_post_likes WHERE post_id = $1 AND user_id = $2",
            post_id,
            user_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        if existing.is_some() {
            // Already liked, return false (no new like)
            return Ok(false);
        }

        // Insert like
        sqlx::query!(
            "INSERT INTO feed_post_likes (post_id, user_id) VALUES ($1, $2)",
            post_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // Increment post like count
        sqlx::query!(
            "UPDATE feed_posts SET like_count = like_count + 1 WHERE id = $1",
            post_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(true)
    }

    /// Unlike a post
    pub async fn unlike_post(&self, post_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        // Verify post exists
        let _post = sqlx::query!("SELECT id FROM feed_posts WHERE id = $1", post_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

        // Begin transaction
        let mut tx = self.pool.begin().await?;

        // Delete like
        sqlx::query!(
            "DELETE FROM feed_post_likes WHERE post_id = $1 AND user_id = $2",
            post_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // Decrement post like count (only if like existed)
        sqlx::query!(
            r#"
            UPDATE feed_posts
            SET like_count = GREATEST(like_count - 1, 0)
            WHERE id = $1
            "#,
            post_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Check if user has liked a post
    pub async fn has_user_liked(&self, post_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
        let like = sqlx::query!(
            "SELECT id FROM feed_post_likes WHERE post_id = $1 AND user_id = $2",
            post_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(like.is_some())
    }
}
