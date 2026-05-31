use std::sync::Arc;

use serde_json::Value;
use tokio::net::TcpListener;

/// Helper: start the app on a random port and return the base URL.
/// Returns `None` when AUTH_URL or DATA_API_URL are not set (CI / no .env).
async fn spawn_app() -> Option<String> {
    let config = Arc::new(rust_neon_template::config::Config::from_env().ok()?);
    let app = rust_neon_template::routes(config);

    let listener = TcpListener::bind("127.0.0.1:0").await.ok()?;
    let addr = listener.local_addr().ok()?;
    let base = format!("http://{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    Some(base)
}

/// Call at the start of each test to skip when env vars are missing.
fn skip_if_no_env() -> bool {
    std::env::var("AUTH_URL").is_err() || std::env::var("DATA_API_URL").is_err()
}

/// Helper: make a request and return the JSON value.
async fn get(url: &str) -> (u16, Value) {
    let resp = reqwest::get(url).await.unwrap();
    let status = resp.status().as_u16();
    let body: Value = resp.json().await.unwrap_or(Value::Null);
    (status, body)
}

/// Helper: make a POST request and return the JSON value.
async fn post(url: &str, body: &Value) -> (u16, Value) {
    let client = reqwest::Client::new();
    let resp = client.post(url).json(body).send().await.unwrap();
    let status = resp.status().as_u16();
    let body: Value = resp.json().await.unwrap_or(Value::Null);
    (status, body)
}

/// Helper: make a PATCH request with auth header.
async fn patch(url: &str, token: &str, body: &Value) -> (u16, Value) {
    let client = reqwest::Client::new();
    let resp = client
        .patch(url)
        .header("Authorization", format!("Bearer {}", token))
        .json(body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let body: Value = resp.json().await.unwrap_or(Value::Null);
    (status, body)
}

/// Helper: make a DELETE request with auth header.
async fn delete(url: &str, token: &str) -> (u16, Value) {
    let client = reqwest::Client::new();
    let resp = client
        .delete(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let body: Value = resp.json().await.unwrap_or(Value::Null);
    (status, body)
}

/// Helper: sign in and return the JWT token.
async fn get_token(base: &str, email: &str, password: &str) -> String {
    let (status, body) = post(
        &format!("{}/api/v1/auth/sign-in", base),
        &serde_json::json!({ "email": email, "password": password }),
    )
    .await;
    assert_eq!(status, 200, "sign-in failed: {:?}", body);
    body["data"]["token"].as_str().unwrap().to_string()
}

// ── Tests ──

#[tokio::test]
async fn test_health() {
    if skip_if_no_env() {
        eprintln!("skipped: AUTH_URL / DATA_API_URL not set");
        return;
    }
    let base = spawn_app().await.unwrap();
    let (status, body) = get(&format!("{}/health", base)).await;

    assert_eq!(status, 200);
    assert_eq!(body["data"]["status"], "ok");
    assert!(!body["data"]["checks"]["auth"].as_str().unwrap().is_empty());
    assert!(
        !body["data"]["checks"]["data_api"]
            .as_str()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn test_auth_no_auth_header() {
    if skip_if_no_env() {
        eprintln!("skipped: AUTH_URL / DATA_API_URL not set");
        return;
    }
    let base = spawn_app().await.unwrap();
    let (status, body) = get(&format!("{}/api/v1/notes", base)).await;

    assert_eq!(status, 401);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn test_auth_wrong_password() {
    if skip_if_no_env() {
        eprintln!("skipped: AUTH_URL / DATA_API_URL not set");
        return;
    }
    let base = spawn_app().await.unwrap();
    let (status, body) = post(
        &format!("{}/api/v1/auth/sign-in", base),
        &serde_json::json!({ "email": "nonexistent@test.com", "password": "wrong" }),
    )
    .await;

    assert_eq!(status, 401);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn test_notes_crud_flow() {
    if skip_if_no_env() {
        eprintln!("skipped: AUTH_URL / DATA_API_URL not set");
        return;
    }
    let base = spawn_app().await.unwrap();

    // Sign in with the test user.
    let token = get_token(&base, "kylepeterkoine4@gmail.com", "super_secret").await;

    // ── List notes ──
    let client = reqwest::Client::new();
    let resp = client
        .get(&format!("{}/api/v1/notes", base))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let notes: Value = resp.json().await.unwrap();
    assert!(notes["data"].is_array());

    // ── Create a note ──
    let (status, _body) = post(
        &format!("{}/api/v1/notes", base),
        &serde_json::json!({ "title": "Test Note", "content": "Created by integration test" }),
    )
    .await;
    // Without auth token → should fail with 401
    assert_eq!(status, 401);

    // Create WITH auth token.
    let client = reqwest::Client::new();
    let resp = client
        .post(&format!("{}/api/v1/notes", base))
        .header("Authorization", format!("Bearer {}", token))
        .json(
            &serde_json::json!({ "title": "Test Note", "content": "Created by integration test" }),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 201);
    let created: Value = resp.json().await.unwrap();
    assert_eq!(created["data"]["title"], "Test Note");
    let note_id = created["data"]["id"].as_i64().unwrap();

    // ── Get the created note ──
    let (status, _body) = get(&format!("{}/api/v1/notes/{}", base, note_id)).await;
    assert_eq!(status, 401, "no token should fail");

    let client = reqwest::Client::new();
    let resp = client
        .get(&format!("{}/api/v1/notes/{}", base, note_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let note: Value = resp.json().await.unwrap();
    assert_eq!(note["data"]["id"], note_id);

    // ── Update the note ──
    let (status, body) = patch(
        &format!("{}/api/v1/notes/{}", base, note_id),
        &token,
        &serde_json::json!({ "title": "Updated Title", "content": "Updated content" }),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(body["data"][0]["title"], "Updated Title");

    // ── Get non-existent note ──
    let (status, _body) = get(&format!("{}/api/v1/notes/999999", base)).await;
    assert_eq!(status, 401, "no token should fail");

    let client = reqwest::Client::new();
    let resp = client
        .get(&format!("{}/api/v1/notes/999999", base))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);
    let err: Value = resp.json().await.unwrap();
    assert_eq!(err["error"]["code"], "NOT_FOUND");

    // ── Update non-existent note ──
    let (status, body) = patch(
        &format!("{}/api/v1/notes/999999", base),
        &token,
        &serde_json::json!({ "title": "Nope", "content": "Nope" }),
    )
    .await;
    assert_eq!(status, 404);
    assert_eq!(body["error"]["code"], "NOT_FOUND");

    // ── Delete non-existent note ──
    let (status, body) = delete(&format!("{}/api/v1/notes/999999", base), &token).await;
    assert_eq!(status, 404);
    assert_eq!(body["error"]["code"], "NOT_FOUND");

    // ── Delete the note ──
    let (status, body) = delete(&format!("{}/api/v1/notes/{}", base, note_id), &token).await;
    assert_eq!(status, 200);
    assert_eq!(body["data"]["message"], "deleted");

    // ── Verify deletion ──
    let client = reqwest::Client::new();
    let resp = client
        .get(&format!("{}/api/v1/notes/{}", base, note_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);

    // ── Sign out ──
    let (status, body) = post(
        &format!("{}/api/v1/auth/sign-out", base),
        &serde_json::json!({}),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(body["data"]["message"], "signed out");

    // ── Get /me with valid token should return user info ──
    // Get a fresh token for the me test
    let token2 = get_token(&base, "kylepeterkoine4@gmail.com", "super_secret").await;
    let client = reqwest::Client::new();
    let resp = client
        .get(&format!("{}/api/v1/auth/me", base))
        .header("Authorization", format!("Bearer {}", token2))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let me: Value = resp.json().await.unwrap();
    assert_eq!(me["data"]["email"], "kylepeterkoine4@gmail.com");
    assert!(me["data"]["sub"].is_string());
    assert!(me["data"]["name"].is_string());
    assert_eq!(me["data"]["role"], "authenticated");

    // ── Get /me without token should return 401 ──
    let (status, body) = get(&format!("{}/api/v1/auth/me", base)).await;
    assert_eq!(status, 401);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}
