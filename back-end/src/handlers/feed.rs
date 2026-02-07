use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::models::feed::{
    CreateFeedCommentRequest, CreateFeedPostRequest, FeedQueryParams, UpdateFeedCommentRequest,
    UpdateFeedPostRequest,
};
use crate::services::feed_service::FeedService;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct FeedHandlerState {
    pub feed_service: FeedService,
}

// ============================================================================
// POST HANDLERS
// ============================================================================

/// Create a new feed post with optional images
/// POST /api/feed
#[utoipa::path(
    post,
    path = "/api/feed",
    tag = "Feed",
    request_body = CreateFeedPostRequest,
    responses(
        (status = 201, description = "Post created successfully", body = crate::models::feed::FeedPostResponse),
        (status = 400, description = "Invalid input (content or images)"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_post(
    State(state): State<Arc<FeedHandlerState>>,
    auth_user: AuthUser,
    Json(request): Json<CreateFeedPostRequest>,
) -> Result<impl IntoResponse, AppError> {
    let post = state.feed_service.create_post(auth_user.id, request).await?;
    Ok((StatusCode::CREATED, Json(post)))
}

/// Get paginated feed posts (infinite scroll)
/// GET /api/feed?offset=0&limit=20
#[utoipa::path(
    get,
    path = "/api/feed",
    tag = "Feed",
    params(
        FeedQueryParams
    ),
    responses(
        (status = 200, description = "Returns paginated posts", body = Vec<crate::models::feed::FeedPostResponse>),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_feed(
    State(state): State<Arc<FeedHandlerState>>,
    _auth_user: AuthUser,
    Query(params): Query<FeedQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    let posts = state
        .feed_service
        .get_feed(params.offset(), params.limit())
        .await?;
    Ok(Json(posts))
}

/// Get a single feed post by ID
/// GET /api/feed/:id
#[utoipa::path(
    get,
    path = "/api/feed/{id}",
    tag = "Feed",
    params(
        ("id" = Uuid, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Returns the post", body = crate::models::feed::FeedPostResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_post(
    State(state): State<Arc<FeedHandlerState>>,
    _auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let post = state.feed_service.get_post(id).await?;
    Ok(Json(post))
}

/// Update a feed post (owner only)
/// PATCH /api/feed/:id
#[utoipa::path(
    patch,
    path = "/api/feed/{id}",
    tag = "Feed",
    request_body = UpdateFeedPostRequest,
    params(
        ("id" = Uuid, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Post updated successfully", body = crate::models::feed::FeedPostResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the post owner"),
        (status = 404, description = "Post not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_post(
    State(state): State<Arc<FeedHandlerState>>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateFeedPostRequest>,
) -> Result<impl IntoResponse, AppError> {
    let post = state
        .feed_service
        .update_post(id, auth_user.id, request)
        .await?;
    Ok(Json(post))
}

/// Delete a feed post (owner only)
/// DELETE /api/feed/:id
#[utoipa::path(
    delete,
    path = "/api/feed/{id}",
    tag = "Feed",
    params(
        ("id" = Uuid, Path, description = "Post ID")
    ),
    responses(
        (status = 204, description = "Post deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the post owner"),
        (status = 404, description = "Post not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_post(
    State(state): State<Arc<FeedHandlerState>>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    state.feed_service.delete_post(id, auth_user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// COMMENT HANDLERS
// ============================================================================

/// Create a comment on a post
/// POST /api/feed/:post_id/comments
#[utoipa::path(
    post,
    path = "/api/feed/{post_id}/comments",
    tag = "Feed Comments",
    request_body = CreateFeedCommentRequest,
    params(
        ("post_id" = Uuid, Path, description = "Post ID")
    ),
    responses(
        (status = 201, description = "Comment created successfully", body = crate::models::feed::FeedComment),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_comment(
    State(state): State<Arc<FeedHandlerState>>,
    auth_user: AuthUser,
    Path(post_id): Path<Uuid>,
    Json(request): Json<CreateFeedCommentRequest>,
) -> Result<impl IntoResponse, AppError> {
    let comment = state
        .feed_service
        .create_comment(post_id, auth_user.id, request)
        .await?;
    Ok((StatusCode::CREATED, Json(comment)))
}

/// Get all comments on a post
/// GET /api/feed/:post_id/comments
#[utoipa::path(
    get,
    path = "/api/feed/{post_id}/comments",
    tag = "Feed Comments",
    params(
        ("post_id" = Uuid, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Returns comments", body = Vec<crate::models::feed::FeedCommentResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_comments(
    State(state): State<Arc<FeedHandlerState>>,
    _auth_user: AuthUser,
    Path(post_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let comments = state.feed_service.get_comments(post_id).await?;
    Ok(Json(comments))
}

/// Update a comment (owner only)
/// PATCH /api/feed/comments/:comment_id
#[utoipa::path(
    patch,
    path = "/api/feed/comments/{comment_id}",
    tag = "Feed Comments",
    request_body = UpdateFeedCommentRequest,
    params(
        ("comment_id" = Uuid, Path, description = "Comment ID")
    ),
    responses(
        (status = 200, description = "Comment updated successfully", body = crate::models::feed::FeedComment),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the comment owner"),
        (status = 404, description = "Comment not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_comment(
    State(state): State<Arc<FeedHandlerState>>,
    auth_user: AuthUser,
    Path(comment_id): Path<Uuid>,
    Json(request): Json<UpdateFeedCommentRequest>,
) -> Result<impl IntoResponse, AppError> {
    let comment = state
        .feed_service
        .update_comment(comment_id, auth_user.id, request)
        .await?;
    Ok(Json(comment))
}

/// Delete a comment (owner only, soft-delete)
/// DELETE /api/feed/comments/:comment_id
#[utoipa::path(
    delete,
    path = "/api/feed/comments/{comment_id}",
    tag = "Feed Comments",
    params(
        ("comment_id" = Uuid, Path, description = "Comment ID")
    ),
    responses(
        (status = 204, description = "Comment deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not the comment owner"),
        (status = 404, description = "Comment not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_comment(
    State(state): State<Arc<FeedHandlerState>>,
    auth_user: AuthUser,
    Path(comment_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    state
        .feed_service
        .delete_comment(comment_id, auth_user.id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// LIKE HANDLERS
// ============================================================================

/// Like a post
/// POST /api/feed/:post_id/like
#[utoipa::path(
    post,
    path = "/api/feed/{post_id}/like",
    tag = "Feed Likes",
    params(
        ("post_id" = Uuid, Path, description = "Post ID")
    ),
    responses(
        (status = 201, description = "Post liked successfully (or already liked)"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn like_post(
    State(state): State<Arc<FeedHandlerState>>,
    auth_user: AuthUser,
    Path(post_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    state.feed_service.like_post(post_id, auth_user.id).await?;
    Ok(StatusCode::CREATED)
}

/// Unlike a post
/// DELETE /api/feed/:post_id/like
#[utoipa::path(
    delete,
    path = "/api/feed/{post_id}/like",
    tag = "Feed Likes",
    params(
        ("post_id" = Uuid, Path, description = "Post ID")
    ),
    responses(
        (status = 204, description = "Post unliked successfully (or wasn't liked)"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Post not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn unlike_post(
    State(state): State<Arc<FeedHandlerState>>,
    auth_user: AuthUser,
    Path(post_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    state.feed_service.unlike_post(post_id, auth_user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
