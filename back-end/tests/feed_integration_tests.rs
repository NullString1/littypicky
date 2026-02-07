// Integration tests for feed feature

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

mod helpers;
use helpers::{create_test_app, get_test_pool, cleanup_test_data};

// Helper to create a test user and get auth token
async fn create_user_and_get_token(app: &mut axum::Router, email: &str) -> (Uuid, String) {
    // Register user
    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": email,
                        "password": "password123",
                        "full_name": "Test User",
                        "city": "London",
                        "country": "UK"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Login user
    let login_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": email,
                        "password": "password123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    let json: Value = serde_json::from_str(&body_str).unwrap();
    
    let token = json["access_token"].as_str().unwrap().to_string();
    let user_id = json["user"]["id"].as_str().unwrap();
    let user_id = Uuid::parse_str(user_id).unwrap();

    (user_id, token)
}

// ============================================================================
// POST TESTS
// ============================================================================

#[tokio::test]
async fn test_create_post_success() {
    let mut app = create_test_app().await;
    let (user_id, token) = create_user_and_get_token(&mut app, "user1@test.com").await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/feed")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(
                    json!({
                        "content": "Test post content",
                        "images": []
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    
    assert_eq!(json["content"].as_str().unwrap(), "Test post content");
    assert_eq!(json["like_count"].as_i64().unwrap(), 0);
    assert_eq!(json["comment_count"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn test_create_post_empty_content() {
    let mut app = create_test_app().await;
    let (_, token) = create_user_and_get_token(&mut app, "user2@test.com").await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/feed")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(
                    json!({
                        "content": "",
                        "images": []
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_post_content_too_long() {
    let mut app = create_test_app().await;
    let (_, token) = create_user_and_get_token(&mut app, "user3@test.com").await;

    let long_content = "x".repeat(501); // Exceeds 500 char limit

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/feed")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(
                    json!({
                        "content": long_content,
                        "images": []
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_post_too_many_images() {
    let mut app = create_test_app().await;
    let (_, token) = create_user_and_get_token(&mut app, "user4@test.com").await;

    let mut images = vec![];
    for _ in 0..11 {
        images.push("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==");
    }

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/feed")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(
                    json!({
                        "content": "Test post",
                        "images": images
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// GET TESTS
// ============================================================================

#[tokio::test]
async fn test_get_feed_pagination() {
    let mut app = create_test_app().await;
    let (_, token) = create_user_and_get_token(&mut app, "user5@test.com").await;

    // Create 5 posts
    for i in 0..5 {
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/feed")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::from(
                        json!({
                            "content": format!("Post {}", i),
                            "images": []
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    // Test default limit (20)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/feed?offset=0&limit=20")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let posts: Vec<Value> = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    assert_eq!(posts.len(), 5);

    // Test with limit=2
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/feed?offset=0&limit=2")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let posts: Vec<Value> = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    assert_eq!(posts.len(), 2);

    // Test with offset=2, limit=2
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/feed?offset=2&limit=2")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let posts: Vec<Value> = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    assert_eq!(posts.len(), 2);
}

#[tokio::test]
async fn test_get_single_post() {
    let mut app = create_test_app().await;
    let (_, token) = create_user_and_get_token(&mut app, "user6@test.com").await;

    // Create a post
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/feed")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(
                    json!({
                        "content": "Test post",
                        "images": []
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    let post_id = json["id"].as_str().unwrap();

    // Get the post
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/feed/{}", post_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    assert_eq!(json["content"].as_str().unwrap(), "Test post");
}

#[tokio::test]
async fn test_get_nonexistent_post() {
    let mut app = create_test_app().await;
    let (_, token) = create_user_and_get_token(&mut app, "user7@test.com").await;

    let fake_id = Uuid::new_v4();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/feed/{}", fake_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// LIKE TESTS
// ============================================================================

#[tokio::test]
async fn test_like_post_idempotent() {
    let mut app = create_test_app().await;
    let (_, token1) = create_user_and_get_token(&mut app, "user8@test.com").await;
    let (_, token2) = create_user_and_get_token(&mut app, "user9@test.com").await;

    // Create a post by user1
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/feed")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token1))
                .body(Body::from(
                    json!({
                        "content": "Test post for liking",
                        "images": []
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    let post_id = json["id"].as_str().unwrap();
    assert_eq!(json["like_count"].as_i64().unwrap(), 0);

    // User2 likes the post
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/feed/{}/like", post_id))
                .header("authorization", format!("Bearer {}", token2))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check like count increased
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/feed/{}", post_id))
                .header("authorization", format!("Bearer {}", token1))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    assert_eq!(json["like_count"].as_i64().unwrap(), 1);

    // User2 likes again (should be idempotent - no duplicate)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/feed/{}/like", post_id))
                .header("authorization", format!("Bearer {}", token2))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Like count should still be 1
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/feed/{}", post_id))
                .header("authorization", format!("Bearer {}", token1))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    assert_eq!(json["like_count"].as_i64().unwrap(), 1);
}

#[tokio::test]
async fn test_unlike_post() {
    let mut app = create_test_app().await;
    let (_, token1) = create_user_and_get_token(&mut app, "user10@test.com").await;
    let (_, token2) = create_user_and_get_token(&mut app, "user11@test.com").await;

    // Create a post
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/feed")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token1))
                .body(Body::from(
                    json!({
                        "content": "Test post for unliking",
                        "images": []
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    let post_id = json["id"].as_str().unwrap();

    // Like the post
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/feed/{}/like", post_id))
                .header("authorization", format!("Bearer {}", token2))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Unlike the post
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/feed/{}/like", post_id))
                .header("authorization", format!("Bearer {}", token2))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify like count is back to 0
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/feed/{}", post_id))
                .header("authorization", format!("Bearer {}", token1))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    assert_eq!(json["like_count"].as_i64().unwrap(), 0);
}

// ============================================================================
// COMMENT TESTS
// ============================================================================

#[tokio::test]
async fn test_create_comment_success() {
    let mut app = create_test_app().await;
    let (_, token1) = create_user_and_get_token(&mut app, "user12@test.com").await;
    let (_, token2) = create_user_and_get_token(&mut app, "user13@test.com").await;

    // Create a post
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/feed")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token1))
                .body(Body::from(
                    json!({
                        "content": "Test post for comments",
                        "images": []
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    let post_id = json["id"].as_str().unwrap();

    // Add a comment
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/feed/{}/comments", post_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token2))
                .body(Body::from(
                    json!({
                        "content": "Great post!"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    assert_eq!(json["content"].as_str().unwrap(), "Great post!");
    assert!(!json["is_deleted"].as_bool().unwrap());

    // Verify comment_count increased
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/feed/{}", post_id))
                .header("authorization", format!("Bearer {}", token1))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    assert_eq!(json["comment_count"].as_i64().unwrap(), 1);
}

#[tokio::test]
async fn test_delete_comment_soft_delete() {
    let mut app = create_test_app().await;
    let (_, token1) = create_user_and_get_token(&mut app, "user14@test.com").await;
    let (_, token2) = create_user_and_get_token(&mut app, "user15@test.com").await;

    // Create a post
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/feed")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token1))
                .body(Body::from(
                    json!({
                        "content": "Test post",
                        "images": []
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    let post_id = json["id"].as_str().unwrap();

    // Add a comment
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/feed/{}/comments", post_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token2))
                .body(Body::from(
                    json!({
                        "content": "This will be deleted"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    let comment_id = json["id"].as_str().unwrap();

    // Delete the comment
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/feed/comments/{}", comment_id))
                .header("authorization", format!("Bearer {}", token2))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Get comments and verify soft delete
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/feed/{}/comments", post_id))
                .header("authorization", format!("Bearer {}", token1))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let comments: Vec<Value> = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    
    assert_eq!(comments.len(), 1);
    assert!(comments[0]["is_deleted"].as_bool().unwrap());
}

// ============================================================================
// DELETE TESTS
// ============================================================================

#[tokio::test]
async fn test_delete_post_ownership() {
    let mut app = create_test_app().await;
    let (_, token1) = create_user_and_get_token(&mut app, "user16@test.com").await;
    let (_, token2) = create_user_and_get_token(&mut app, "user17@test.com").await;

    // User1 creates a post
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/feed")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token1))
                .body(Body::from(
                    json!({
                        "content": "Test post",
                        "images": []
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&body)).unwrap();
    let post_id = json["id"].as_str().unwrap();

    // User2 tries to delete (should fail)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/feed/{}", post_id))
                .header("authorization", format!("Bearer {}", token2))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // User1 deletes successfully
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/feed/{}", post_id))
                .header("authorization", format!("Bearer {}", token1))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_unauthorized_without_token() {
    let mut app = create_test_app().await;
    let (_, token) = create_user_and_get_token(&mut app, "user18@test.com").await;

    // Create a post (succeeds with token)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/feed")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(
                    json!({
                        "content": "Test",
                        "images": []
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Try to get feed without token (should fail)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/feed?offset=0&limit=20")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
