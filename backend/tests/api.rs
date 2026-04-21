use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    Router,
};
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use backend::test_helpers::MIGRATIONS;
use backend::DbPool;

// ── Test helpers ─────────────────────────────────────────────────────────────

fn test_pool() -> DbPool {
    let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
    let pool = r2d2::Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("failed to create test pool");

    {
        use diesel_migrations::MigrationHarness;
        let mut conn = pool.get().expect("failed to get connection");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("failed to run migrations");
    }

    pool
}

fn app() -> Router {
    backend::build_router(test_pool())
}

async fn send(
    app: &Router,
    method: Method,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);

    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }

    let req = if let Some(b) = body {
        builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(&b).unwrap()))
            .unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    };

    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, json)
}

async fn get(app: &Router, uri: &str, token: &str) -> (StatusCode, Value) {
    send(app, Method::GET, uri, Some(token), None).await
}

async fn post(app: &Router, uri: &str, token: Option<&str>, body: Value) -> (StatusCode, Value) {
    send(app, Method::POST, uri, token, Some(body)).await
}

#[allow(dead_code)]
async fn put(app: &Router, uri: &str, token: &str, body: Value) -> (StatusCode, Value) {
    send(app, Method::PUT, uri, Some(token), Some(body)).await
}

async fn delete(app: &Router, uri: &str, token: &str) -> (StatusCode, Value) {
    send(app, Method::DELETE, uri, Some(token), None).await
}

/// Register a user and return (token, user_id).
async fn register(app: &Router, email: &str) -> (String, String) {
    let (status, body) = post(
        app,
        "/auth/register",
        None,
        json!({
            "email": email,
            "display_name": "Test User",
            "password": "password123",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "register failed: {body}");
    let token = body["token"].as_str().unwrap().to_string();
    let user_id = body["user"]["id"].as_str().unwrap().to_string();
    (token, user_id)
}

/// Register + create a workspace, return (token, user_id, workspace_id).
async fn setup_workspace(app: &Router) -> (String, String, String) {
    let (token, user) = register(app, "alice@example.com").await;

    let (status, body) = post(
        app,
        "/workspaces",
        Some(&token),
        json!({ "name": "My Finances" }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "create workspace failed: {body}"
    );
    let workspace = body["workspace"].as_str().unwrap().to_string();

    (token, user, workspace)
}

// ── Auth tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn health_check() {
    let app = app();
    let (status, body) = send(&app, Method::GET, "/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn register_and_login() {
    let app = app();

    let (token, _) = register(&app, "alice@example.com").await;
    assert!(!token.is_empty());

    // Login with same credentials
    let (status, body) = post(
        &app,
        "/auth/login",
        None,
        json!({ "email": "alice@example.com", "password": "password123" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["token"].as_str().is_some());
}

#[tokio::test]
async fn login_wrong_password() {
    let app = app();
    register(&app, "alice@example.com").await;

    let (status, _) = post(
        &app,
        "/auth/login",
        None,
        json!({ "email": "alice@example.com", "password": "wrong" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_invalidates_token() {
    let app = app();
    let (token, _) = register(&app, "alice@example.com").await;

    let (status, _) = post(&app, "/auth/logout", Some(&token), json!({})).await;
    assert_eq!(status, StatusCode::OK);

    // Token should no longer work
    let (status, _) = get(&app, "/workspaces", &token).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unauthenticated_request_is_rejected() {
    let app = app();
    let (status, _) = send(&app, Method::GET, "/workspaces", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Workspace tests ──────────────────────────────────────────────────────────

#[tokio::test]
async fn create_and_list_workspaces() {
    let app = app();
    let (token, _, _) = setup_workspace(&app).await;

    let (status, body) = get(&app, "/workspaces", &token).await;
    assert_eq!(status, StatusCode::OK);
    let workspaces = body["workspaces"].as_array().unwrap();
    assert_eq!(workspaces.len(), 1);
    assert_eq!(workspaces[0]["name"], "My Finances");
}

#[tokio::test]
async fn get_workspace_returns_members() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    let (status, body) = get(&app, &format!("/workspaces/{workspace_id}"), &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "My Finances");
    assert_eq!(body["your_role"], "owner");
    assert!(!body["members"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn non_member_cannot_access_workspace() {
    let app = app();
    let (_, _, workspace_id) = setup_workspace(&app).await;

    let (outsider_token, _) = register(&app, "outsider@example.com").await;
    let (status, _) = get(
        &app,
        &format!("/workspaces/{workspace_id}"),
        &outsider_token,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── Account tests ────────────────────────────────────────────────────────────

#[tokio::test]
async fn workspace_has_default_accounts() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let accounts = body["accounts"].as_array().unwrap();
    // 5 roots + 4 asset children + 1 liability child + 1 revenue child + 7 expense children = 18
    assert_eq!(accounts.len(), 18);

    // Check root accounts exist
    let root_names: Vec<&str> = accounts
        .iter()
        .filter(|a| a["parent_id"].is_null())
        .map(|a| a["name"].as_str().unwrap())
        .collect();
    assert!(root_names.contains(&"Assets"));
    assert!(root_names.contains(&"Liabilities"));
    assert!(root_names.contains(&"Equity"));
    assert!(root_names.contains(&"Revenue"));
    assert!(root_names.contains(&"Expenses"));
}

#[tokio::test]
async fn create_child_account_via_api() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    // Find the Assets root account
    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &token,
    )
    .await;
    let accounts = body["accounts"].as_array().unwrap();
    let assets = accounts
        .iter()
        .find(|a| a["name"] == "Assets" && a["parent_id"].is_null())
        .unwrap();
    let assets_id = assets["id"].as_str().unwrap();

    // Create child under Assets
    let (status, body) = post(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        Some(&token),
        json!({ "name": "Brokerage", "parent_id": assets_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "Brokerage");
    assert_eq!(body["account_type"], "asset");
    assert_eq!(body["parent_id"], assets_id);
}

#[tokio::test]
async fn create_root_account_via_api() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    // Creating root account requires account_type
    let (status, body) = post(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        Some(&token),
        json!({ "name": "Custom Root", "account_type": "asset" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["account_type"], "asset");
    assert!(body["parent_id"].is_null());
}

#[tokio::test]
async fn create_root_without_type_rejected() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    let (status, body) = post(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        Some(&token),
        json!({ "name": "No Type" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn delete_leaf_account() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    // Find a leaf account (e.g. Chequing under Assets)
    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &token,
    )
    .await;
    let accounts = body["accounts"].as_array().unwrap();
    let chequing = accounts
        .iter()
        .find(|a| a["name"] == "Chequing")
        .unwrap();
    let chequing_id = chequing["id"].as_str().unwrap();

    let (status, _) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{chequing_id}"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn delete_account_with_children_rejected() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    // Find Assets root (has children)
    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &token,
    )
    .await;
    let accounts = body["accounts"].as_array().unwrap();
    let assets = accounts
        .iter()
        .find(|a| a["name"] == "Assets" && a["parent_id"].is_null())
        .unwrap();
    let assets_id = assets["id"].as_str().unwrap();

    let (status, _) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{assets_id}"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn account_balance_zero_when_no_postings() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &token,
    )
    .await;
    let accounts = body["accounts"].as_array().unwrap();
    let chequing = accounts
        .iter()
        .find(|a| a["name"] == "Chequing")
        .unwrap();
    let chequing_id = chequing["id"].as_str().unwrap();

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{chequing_id}/balance"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["cleared_balance"], 0);
}

#[tokio::test]
async fn account_balance_reflects_postings() {
    let pool = test_pool();
    let app = backend::build_router(pool.clone());
    let (token, _, workspace_id) = setup_workspace(&app).await;

    // Get account IDs via API
    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &token,
    )
    .await;
    let accounts = body["accounts"].as_array().unwrap();
    let chequing_id = accounts
        .iter()
        .find(|a| a["name"] == "Chequing")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let food_id = accounts
        .iter()
        .find(|a| a["name"] == "Food")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create entry via domain model (no route yet)
    {
        use backend::domain::account::AccountRef;
        use backend::domain::entry::{NewEntry, PostingInput};
        use backend::domain::workspace::WorkspaceRef;

        let mut conn = pool.get().unwrap();
        let workspace_ref: WorkspaceRef =
            serde_json::from_value(json!(workspace_id)).unwrap();
        let chequing_ref: AccountRef =
            serde_json::from_value(json!(chequing_id)).unwrap();
        let food_ref: AccountRef =
            serde_json::from_value(json!(food_id)).unwrap();

        NewEntry::new(
            workspace_ref,
            "2026-01-15",
            "Lunch",
            vec![
                PostingInput::new(chequing_ref, -1200),
                PostingInput::new(food_ref, 1200),
            ],
        )
        .cleared(true)
        .insert(&mut conn)
        .unwrap();
    }

    // Verify balance via API
    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{chequing_id}/balance"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["cleared_balance"], -1200);

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{food_id}/balance"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["cleared_balance"], 1200);
}

#[tokio::test]
async fn cross_workspace_account_access_denied() {
    let app = app();
    let (_, _, workspace_id) = setup_workspace(&app).await;

    let (bob_token, _) = register(&app, "bob@example.com").await;

    let (status, _) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &bob_token,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── Auth edge cases ─────────────────────────────────────────────────────────

#[tokio::test]
async fn duplicate_registration_rejected() {
    let app = app();
    register(&app, "alice@example.com").await;

    let (status, _) = post(
        &app,
        "/auth/register",
        None,
        json!({
            "email": "alice@example.com",
            "display_name": "Alice Again",
            "password": "password123",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

// ── Member removal ──────────────────────────────────────────────────────────

#[tokio::test]
async fn owner_can_leave_if_not_last_owner() {
    let app = app();
    let (token, user_id, workspace_id) = setup_workspace(&app).await;

    let (status, body) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/members/{user_id}"),
        &token,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "last owner removal should fail: {body}"
    );
}
