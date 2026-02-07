// Integration tests for verification endpoints

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

mod helpers;
use helpers::{create_test_app, get_test_pool};

/// Helper to create a verified user and get auth token
async fn create_verified_user_and_login(app: &axum::Router, email: &str) -> String {
    // Register user
    let response = app
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

    assert_eq!(response.status(), StatusCode::CREATED);

    // Get database pool and mark user as verified
    let pool = get_test_pool().await;
    sqlx::query("UPDATE users SET email_verified = true, email_verified_at = NOW() WHERE email = $1")
        .bind(email)
        .execute(&pool)
        .await
        .expect("Failed to verify user");

    // Now login
    let response = app
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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth_response: Value = serde_json::from_slice(&body).unwrap();
    auth_response["access_token"].as_str().unwrap().to_string()
}

/// Helper to create a report and return the report ID
async fn create_test_report(app: &axum::Router, token: &str) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/reports")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(
                    json!({
                        "latitude": 51.5074,
                        "longitude": -0.1278,
                        "description": "Test litter",
                        "photo_base64": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
                        "city": "London",
                        "country": "UK"
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
    let report: Value = serde_json::from_slice(&body).unwrap();
    report["id"].as_str().unwrap().to_string()
}

/// Helper to create 5 cleared reports for a user to enable verification
async fn enable_verification_for_user(app: &axum::Router, verifier_token: &str, verifier_email: &str) {
    // Create 5 different reporters and have them create reports
    // Use verifier email as unique prefix to avoid conflicts between tests
    let prefix = verifier_email.split('@').next().unwrap();
    for i in 0..5 {
        let reporter_email = format!("{}_dummy_reporter_{}@example.com", prefix, i);
        let reporter_token = create_verified_user_and_login(app, &reporter_email).await;
        let report_id = create_test_report(app, &reporter_token).await;
        
        // Verifier claims and clears each report
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/reports/{}/claim", report_id))
                    .header("authorization", format!("Bearer {}", verifier_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/reports/{}/clear", report_id))
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", verifier_token))
                    .body(Body::from(
                        json!({
                            "photo_base64": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn test_cannot_verify_without_clearing_enough_reports() {
    let app = create_test_app().await;
    
    // Create reporter and report
    let reporter_token = create_verified_user_and_login(&app, "reporter@example.com").await;
    let report_id = create_test_report(&app, &reporter_token).await;
    
    // Create claimer and clear the report
    let claimer_token = create_verified_user_and_login(&app, "claimer@example.com").await;
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/claim", report_id))
                .header("authorization", format!("Bearer {}", claimer_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/clear", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", claimer_token))
                .body(Body::from(
                    json!({
                        "photo_base64": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Create verifier with no clears and try to verify
    let verifier_token = create_verified_user_and_login(&app, "verifier@example.com").await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/verify", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", verifier_token))
                .body(Body::from(
                    json!({
                        "is_verified": true,
                        "comment": "Looks good"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();
    assert!(error["error"].as_str().unwrap().contains("need to clear at least 5 reports"));
}

#[tokio::test]
async fn test_cannot_verify_own_report() {
    let app = create_test_app().await;
    
    // Create user who will be reporter and verifier
    let user_token = create_verified_user_and_login(&app, "user@example.com").await;
    enable_verification_for_user(&app, &user_token, "user@example.com").await;
    
    // User creates a report
    let report_id = create_test_report(&app, &user_token).await;
    
    // Someone else claims and clears it
    let claimer_token = create_verified_user_and_login(&app, "claimer@example.com").await;
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/claim", report_id))
                .header("authorization", format!("Bearer {}", claimer_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/clear", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", claimer_token))
                .body(Body::from(
                    json!({
                        "photo_base64": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Original reporter tries to verify
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/verify", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", user_token))
                .body(Body::from(
                    json!({
                        "is_verified": true,
                        "comment": "Looks good"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();
    assert!(error["error"].as_str().unwrap().contains("cannot verify your own report"));
}

#[tokio::test]
async fn test_cannot_verify_report_you_cleared() {
    let app = create_test_app().await;
    
    // Create user who will clear and try to verify
    let user_token = create_verified_user_and_login(&app, "user2@example.com").await;
    enable_verification_for_user(&app, &user_token, "user@example.com").await;
    
    // Someone else creates a report
    let reporter_token = create_verified_user_and_login(&app, "reporter2@example.com").await;
    let report_id = create_test_report(&app, &reporter_token).await;
    
    // User claims and clears it
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/claim", report_id))
                .header("authorization", format!("Bearer {}", user_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/clear", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", user_token))
                .body(Body::from(
                    json!({
                        "photo_base64": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // User tries to verify the report they cleared
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/verify", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", user_token))
                .body(Body::from(
                    json!({
                        "is_verified": true,
                        "comment": "Looks good"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();
    assert!(error["error"].as_str().unwrap().contains("cannot verify a report you cleared"));
}

#[tokio::test]
async fn test_verify_report_success() {
    let app = create_test_app().await;
    
    // Create reporter and report
    let reporter_token = create_verified_user_and_login(&app, "reporter3@example.com").await;
    let report_id = create_test_report(&app, &reporter_token).await;
    
    // Create claimer and clear the report
    let claimer_token = create_verified_user_and_login(&app, "claimer3@example.com").await;
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/claim", report_id))
                .header("authorization", format!("Bearer {}", claimer_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/clear", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", claimer_token))
                .body(Body::from(
                    json!({
                        "photo_base64": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Create verifier with enough clears and verify
    let verifier_token = create_verified_user_and_login(&app, "verifier3@example.com").await;
    enable_verification_for_user(&app, &verifier_token, "verifier3@example.com").await;
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/verify", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", verifier_token))
                .body(Body::from(
                    json!({
                        "is_verified": true,
                        "comment": "Looks good"
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
    let verification: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(verification["is_verified"].as_bool().unwrap(), true);
    assert_eq!(verification["comment"].as_str().unwrap(), "Looks good");
}

#[tokio::test]
async fn test_cannot_verify_same_report_twice() {
    let app = create_test_app().await;
    
    // Create reporter and report
    let reporter_token = create_verified_user_and_login(&app, "reporter4@example.com").await;
    let report_id = create_test_report(&app, &reporter_token).await;
    
    // Create claimer and clear the report
    let claimer_token = create_verified_user_and_login(&app, "claimer4@example.com").await;
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/claim", report_id))
                .header("authorization", format!("Bearer {}", claimer_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/clear", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", claimer_token))
                .body(Body::from(
                    json!({
                        "photo_base64": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Create verifier with enough clears
    let verifier_token = create_verified_user_and_login(&app, "verifier4@example.com").await;
    enable_verification_for_user(&app, &verifier_token, "verifier4@example.com").await;
    
    // First verification
    let response1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/verify", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", verifier_token))
                .body(Body::from(
                    json!({
                        "is_verified": true,
                        "comment": "Looks good"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response1.status(), StatusCode::CREATED);
    
    // Try to verify again
    let response2 = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/verify", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", verifier_token))
                .body(Body::from(
                    json!({
                        "is_verified": true,
                        "comment": "Still looks good"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::BAD_REQUEST);
    
    let body = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();
    let error: Value = serde_json::from_slice(&body).unwrap();
    assert!(error["error"].as_str().unwrap().contains("already verified"));
}

#[tokio::test]
async fn test_report_becomes_verified_after_enough_verifications() {
    let app = create_test_app().await;
    
    // Create reporter and report
    let reporter_token = create_verified_user_and_login(&app, "reporter5@example.com").await;
    let report_id = create_test_report(&app, &reporter_token).await;
    
    // Create claimer and clear the report
    let claimer_token = create_verified_user_and_login(&app, "claimer5@example.com").await;
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/claim", report_id))
                .header("authorization", format!("Bearer {}", claimer_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/clear", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", claimer_token))
                .body(Body::from(
                    json!({
                        "photo_base64": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Create 3 verifiers and have them verify (MIN_VERIFICATIONS_NEEDED=3)
    for i in 1..=3 {
        let verifier_email = format!("verifier5_{}@example.com", i);
        let verifier_token = create_verified_user_and_login(&app, &verifier_email).await;
        enable_verification_for_user(&app, &verifier_token, &verifier_email).await;
        
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/reports/{}/verify", report_id))
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", verifier_token))
                    .body(Body::from(
                        json!({
                            "is_verified": true,
                            "comment": format!("Verification {}", i)
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
    }
    
    // Check the report is now verified
    let check_token = create_verified_user_and_login(&app, "checker@example.com").await;
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/reports/{}", report_id))
                .header("authorization", format!("Bearer {}", check_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let report: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(report["status"].as_str().unwrap(), "Verified");
}

#[tokio::test]
async fn test_get_report_verifications() {
    let app = create_test_app().await;
    
    // Create reporter and report
    let reporter_token = create_verified_user_and_login(&app, "reporter6@example.com").await;
    let report_id = create_test_report(&app, &reporter_token).await;
    
    // Create claimer and clear the report
    let claimer_token = create_verified_user_and_login(&app, "claimer6@example.com").await;
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/claim", report_id))
                .header("authorization", format!("Bearer {}", claimer_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/clear", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", claimer_token))
                .body(Body::from(
                    json!({
                        "photo_base64": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Create 2 verifiers
    let verifier1_token = create_verified_user_and_login(&app, "verifier6_1@example.com").await;
    enable_verification_for_user(&app, &verifier1_token, "verifier6_1@example.com").await;
    
    let verifier2_token = create_verified_user_and_login(&app, "verifier6_2@example.com").await;
    enable_verification_for_user(&app, &verifier2_token, "verifier6_2@example.com").await;
    
    // Add verifications
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/verify", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", verifier1_token))
                .body(Body::from(
                    json!({
                        "is_verified": true,
                        "comment": "Good job"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/reports/{}/verify", report_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", verifier2_token))
                .body(Body::from(
                    json!({
                        "is_verified": false,
                        "comment": "Doesn't look clean"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Get verifications
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/reports/{}/verifications", report_id))
                .header("authorization", format!("Bearer {}", reporter_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let verifications: Value = serde_json::from_slice(&body).unwrap();
    assert!(verifications.is_array());
    assert_eq!(verifications.as_array().unwrap().len(), 2);
}
