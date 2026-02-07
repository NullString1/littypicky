// Integration tests for authentication endpoints

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

// Test helper to create test app
mod helpers;
use helpers::create_test_app;

#[tokio::test]
async fn test_user_registration() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "test@example.com",
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

    let status = response.status();
    if status != StatusCode::CREATED {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        eprintln!("Response status: {}, body: {}", status, body_str);
    }
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_duplicate_registration() {
    let app = create_test_app().await;

    let user_data = json!({
        "email": "duplicate@example.com",
        "password": "password123",
        "full_name": "Test User",
        "city": "London",
        "country": "UK"
    })
    .to_string();

    // First registration should succeed
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(user_data.clone()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Second registration with same email should fail
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(user_data))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_login_with_invalid_credentials() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "nonexistent@example.com",
                        "password": "wrongpassword"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_registration_validation() {
    let app = create_test_app().await;

    // Test with invalid email
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "not-an-email",
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

    let status = response.status();
    if status != StatusCode::BAD_REQUEST {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        eprintln!("Invalid email test - Response status: {}, body: {}", status, body_str);
    }
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Test with short password
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "test@example.com",
                        "password": "123",
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
