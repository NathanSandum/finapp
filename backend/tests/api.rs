use axum::{
    body::Body,
    http::{header, HeaderMap, Method, Request, StatusCode},
    Router,
};
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::collections::HashMap;
use tower::ServiceExt;

use backend::config::Mode;
use backend::database::MIGRATIONS;
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
    backend::build_router(test_pool(), Mode::Dev)
}

#[derive(Default, Clone, Debug)]
struct Session {
    cookies: HashMap<String, String>,
}

impl Session {
    fn cookie_header(&self) -> Option<String> {
        if self.cookies.is_empty() {
            return None;
        }
        let mut parts: Vec<String> = self
            .cookies
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect();
        parts.sort();
        Some(parts.join("; "))
    }

    fn capture(&mut self, headers: &HeaderMap) {
        for value in headers.get_all(header::SET_COOKIE).iter() {
            let Ok(raw) = value.to_str() else { continue };
            let pair = raw.split(';').next().unwrap_or("");
            let Some((name, val)) = pair.split_once('=') else {
                continue;
            };
            self.cookies
                .insert(name.trim().to_string(), val.trim().to_string());
        }
    }
}

async fn send(
    app: &Router,
    method: Method,
    uri: &str,
    session: &mut Session,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);

    if let Some(cookie) = session.cookie_header() {
        builder = builder.header(header::COOKIE, cookie);
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
    session.capture(resp.headers());
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let envelope: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    let body = match envelope.get("status").and_then(Value::as_str) {
        Some("success") | Some("fail") => envelope.get("data").cloned().unwrap_or(Value::Null),
        _ => envelope,
    };
    (status, body)
}

async fn get(app: &Router, uri: &str, session: &mut Session) -> (StatusCode, Value) {
    send(app, Method::GET, uri, session, None).await
}

async fn post(
    app: &Router,
    uri: &str,
    session: &mut Session,
    body: Value,
) -> (StatusCode, Value) {
    send(app, Method::POST, uri, session, Some(body)).await
}

async fn put(
    app: &Router,
    uri: &str,
    session: &mut Session,
    body: Value,
) -> (StatusCode, Value) {
    send(app, Method::PUT, uri, session, Some(body)).await
}

async fn delete(app: &Router, uri: &str, session: &mut Session) -> (StatusCode, Value) {
    send(app, Method::DELETE, uri, session, None).await
}

/// Register a user and return (session, user_id).
async fn register(app: &Router, email: &str) -> (Session, String) {
    let mut session = Session::default();
    let (status, body) = post(
        app,
        "/auth/register",
        &mut session,
        json!({
            "email": email,
            "display_name": "Test User",
            "password": "password123",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "register failed: {body}");
    let user_id = body["user"]["id"].as_str().unwrap().to_string();
    (session, user_id)
}

/// Register + create a workspace, return (session, user_id, workspace_id).
async fn setup_workspace(app: &Router) -> (Session, String, String) {
    let (mut session, user_id) = register(app, "alice@example.com").await;

    let (status, body) = post(
        app,
        "/workspaces",
        &mut session,
        json!({ "name": "My Finances" }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "create workspace failed: {body}"
    );
    let workspace = body["workspace"].as_str().unwrap().to_string();

    (session, user_id, workspace)
}

// ── Auth tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn health_check() {
    let app = app();
    let mut session = Session::default();
    let (status, body) = send(&app, Method::GET, "/health", &mut session, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["service"], "finapp");
}

#[tokio::test]
async fn register_and_login() {
    let app = app();

    let (session, _) = register(&app, "alice@example.com").await;
    assert!(session.cookie_header().is_some(), "register sets session cookie");

    let mut login_session = Session::default();
    let (status, body) = post(
        &app,
        "/auth/login",
        &mut login_session,
        json!({ "email": "alice@example.com", "password": "password123" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user"]["email"], "alice@example.com");
    assert!(login_session.cookie_header().is_some(), "login sets session cookie");
}

#[tokio::test]
async fn login_wrong_password() {
    let app = app();
    register(&app, "alice@example.com").await;

    let mut session = Session::default();
    let (status, body) = post(
        &app,
        "/auth/login",
        &mut session,
        json!({ "email": "alice@example.com", "password": "wrong" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["message"], "Invalid email or password");
}

#[tokio::test]
async fn logout_invalidates_session() {
    let app = app();
    let (mut session, _) = register(&app, "alice@example.com").await;

    let (status, _) = post(&app, "/auth/logout", &mut session, json!({})).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = get(&app, "/workspaces", &mut session).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unauthenticated_request_is_rejected() {
    let app = app();
    let mut session = Session::default();
    let (status, _) = send(&app, Method::GET, "/workspaces", &mut session, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Workspace tests ──────────────────────────────────────────────────────────

#[tokio::test]
async fn create_and_list_workspaces() {
    let app = app();
    let (mut session, _, _) = setup_workspace(&app).await;

    let (status, body) = get(&app, "/workspaces", &mut session).await;
    assert_eq!(status, StatusCode::OK);
    let workspaces = body["workspaces"].as_array().unwrap();
    assert_eq!(workspaces.len(), 1);
    assert_eq!(workspaces[0]["name"], "My Finances");
}

#[tokio::test]
async fn get_workspace_returns_members() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (status, body) = get(&app, &format!("/workspaces/{workspace_id}"), &mut session).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "My Finances");
    assert_eq!(body["your_role"], "owner");
    assert!(!body["members"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn non_member_cannot_access_workspace() {
    let app = app();
    let (_, _, workspace_id) = setup_workspace(&app).await;

    let (mut outsider, _) = register(&app, "outsider@example.com").await;
    let (status, _) = get(
        &app,
        &format!("/workspaces/{workspace_id}"),
        &mut outsider,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── Account tests ────────────────────────────────────────────────────────────

#[tokio::test]
async fn workspace_has_default_accounts() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let accounts = body["accounts"].as_array().unwrap();
    // 5 roots + 4 asset + 1 liability + 1 revenue + 7 expense + 1 equity = 19
    assert_eq!(accounts.len(), 19);

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
async fn accounts_response_includes_role() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut session,
    )
    .await;
    let accounts = body["accounts"].as_array().unwrap();

    let assets_root = accounts
        .iter()
        .find(|a| a["name"] == "Assets" && a["parent_id"].is_null())
        .unwrap();
    assert_eq!(assets_root["role"], "assets_root");

    let chequing = accounts.iter().find(|a| a["name"] == "Chequing").unwrap();
    assert_eq!(chequing["role"], "cash");

    let savings = accounts.iter().find(|a| a["name"] == "Savings").unwrap();
    assert!(savings["role"].is_null());
}

#[tokio::test]
async fn anchors_endpoint_returns_all_seven() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/anchors"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(body["assets"]["role"], "assets_root");
    assert_eq!(body["liabilities"]["role"], "liabilities_root");
    assert_eq!(body["equity"]["role"], "equity_root");
    assert_eq!(body["revenue"]["role"], "revenue_root");
    assert_eq!(body["expenses"]["role"], "expenses_root");
    assert_eq!(body["cash"]["role"], "cash");
    assert_eq!(body["cash"]["name"], "Chequing");
    assert_eq!(body["opening_equity"]["role"], "opening_equity");
    assert_eq!(body["opening_equity"]["name"], "Opening Balances");
}

#[tokio::test]
async fn create_child_account_via_api() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut session,
    )
    .await;
    let accounts = body["accounts"].as_array().unwrap();
    let assets = accounts
        .iter()
        .find(|a| a["name"] == "Assets" && a["parent_id"].is_null())
        .unwrap();
    let assets_id = assets["id"].as_str().unwrap().to_string();

    let (status, body) = post(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut session,
        json!({ "name": "Brokerage", "parent_id": assets_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "Brokerage");
    assert_eq!(body["account_type"], "asset");
    assert_eq!(body["parent_id"], assets_id);
}

#[tokio::test]
async fn create_delete_create_user_account_via_api() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut session,
    )
    .await;
    let accounts = body["accounts"].as_array().unwrap();
    let expenses = accounts
        .iter()
        .find(|a| a["name"] == "Expenses" && a["parent_id"].is_null())
        .unwrap();
    let expenses_id = expenses["id"].as_str().unwrap().to_string();

    let (status, body) = post(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut session,
        json!({ "name": "test", "parent_id": expenses_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "first create: {body}");
    let first_id = body["id"].as_str().unwrap().to_string();

    let (status, _) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{first_id}"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = post(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut session,
        json!({ "name": "test", "parent_id": expenses_id }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "recreate after delete must not conflict: {body}"
    );
    assert_eq!(body["id"], first_id, "should restore same id");
}

#[tokio::test]
async fn create_root_account_via_api_rejected() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (status, body) = post(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut session,
        json!({ "name": "Custom Root" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn delete_leaf_account() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut session,
    )
    .await;
    let accounts = body["accounts"].as_array().unwrap();
    let savings = accounts.iter().find(|a| a["name"] == "Savings").unwrap();
    let savings_id = savings["id"].as_str().unwrap().to_string();

    let (status, _) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{savings_id}"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn delete_anchor_account_rejected() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut session,
    )
    .await;
    let accounts = body["accounts"].as_array().unwrap();
    let chequing = accounts.iter().find(|a| a["name"] == "Chequing").unwrap();
    let chequing_id = chequing["id"].as_str().unwrap().to_string();

    let (status, _) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{chequing_id}"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn delete_account_with_children_rejected() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut session,
    )
    .await;
    let accounts = body["accounts"].as_array().unwrap();
    let assets = accounts
        .iter()
        .find(|a| a["name"] == "Assets" && a["parent_id"].is_null())
        .unwrap();
    let assets_id = assets["id"].as_str().unwrap().to_string();

    let (status, _) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{assets_id}"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn account_balance_zero_when_no_postings() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut session,
    )
    .await;
    let accounts = body["accounts"].as_array().unwrap();
    let chequing = accounts.iter().find(|a| a["name"] == "Chequing").unwrap();
    let chequing_id = chequing["id"].as_str().unwrap().to_string();

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{chequing_id}/balance"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["balance"], 0);
}

#[tokio::test]
async fn account_balance_reflects_postings() {
    let pool = test_pool();
    let app = backend::build_router(pool.clone(), Mode::Dev);
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut session,
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

    {
        use backend::domain::account::AccountRef;
        use backend::domain::entry::{NewEntry, PostingInput};
        use backend::domain::workspace::WorkspaceRef;

        let mut conn = pool.get().unwrap();
        let workspace_ref: WorkspaceRef = serde_json::from_value(json!(workspace_id)).unwrap();
        let chequing_ref: AccountRef = serde_json::from_value(json!(chequing_id)).unwrap();
        let food_ref: AccountRef = serde_json::from_value(json!(food_id)).unwrap();

        NewEntry::new(
            workspace_ref,
            "2026-01-15".parse().unwrap(),
            "Lunch",
            vec![
                PostingInput::new(chequing_ref, -1200),
                PostingInput::new(food_ref, 1200),
            ],
        )
        .insert(&mut conn)
        .unwrap();
    }

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{chequing_id}/balance"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["balance"], -1200);

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{food_id}/balance"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["balance"], 1200);
}

#[tokio::test]
async fn cross_workspace_account_access_denied() {
    let app = app();
    let (_, _, workspace_id) = setup_workspace(&app).await;

    let (mut bob, _) = register(&app, "bob@example.com").await;

    let (status, _) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut bob,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── Entry tests ─────────────────────────────────────────────────────────────

async fn find_account_id(
    app: &Router,
    workspace_id: &str,
    session: &mut Session,
    name: &str,
) -> String {
    let (_, body) = get(
        app,
        &format!("/workspaces/{workspace_id}/accounts"),
        session,
    )
    .await;
    let accounts = body["accounts"].as_array().unwrap();
    accounts
        .iter()
        .find(|a| a["name"] == name)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn create_entry(
    app: &Router,
    workspace_id: &str,
    session: &mut Session,
    from_account: &str,
    to_account: &str,
    amount: i32,
    date: &str,
    description: &str,
) -> (StatusCode, Value) {
    post(
        app,
        &format!("/workspaces/{workspace_id}/entries"),
        session,
        json!({
            "date": date,
            "description": description,
            "postings": [
                { "account_id": from_account, "amount": -amount },
                { "account_id": to_account, "amount": amount },
            ]
        }),
    )
    .await
}

#[tokio::test]
async fn create_entry_succeeds() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    let (status, body) = create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &food,
        1200,
        "2026-01-15",
        "Lunch",
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "create entry failed: {body}");
    assert_eq!(body["date"], "2026-01-15");
    assert_eq!(body["description"], "Lunch");
    let postings = body["postings"].as_array().unwrap();
    assert_eq!(postings.len(), 2);
}

#[tokio::test]
async fn create_entry_non_zero_sum_rejected() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    let (status, _) = post(
        &app,
        &format!("/workspaces/{workspace_id}/entries"),
        &mut session,
        json!({
            "date": "2026-01-15",
            "description": "Bad",
            "postings": [
                { "account_id": chequing, "amount": -1200 },
                { "account_id": food, "amount": 1000 },
            ]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn list_entries_returns_all_with_postings() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;
    let rent = find_account_id(&app, &workspace_id, &mut session, "Rent").await;

    create_entry(&app, &workspace_id, &mut session, &chequing, &food, 1200, "2026-01-15", "Lunch")
        .await;
    create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &rent,
        150000,
        "2026-01-16",
        "Rent",
    )
    .await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/entries"),
        &mut session,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let entries = body["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    for entry in entries {
        assert!(!entry["postings"].as_array().unwrap().is_empty());
    }
}

#[tokio::test]
async fn get_single_entry() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    let (_, created) = create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &food,
        1200,
        "2026-01-15",
        "Lunch",
    )
    .await;
    let entry_id = created["id"].as_str().unwrap().to_string();

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/entries/{entry_id}"),
        &mut session,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["description"], "Lunch");
    assert_eq!(body["postings"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn delete_entry_soft_deletes() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    let (_, created) = create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &food,
        1200,
        "2026-01-15",
        "Lunch",
    )
    .await;
    let entry_id = created["id"].as_str().unwrap().to_string();

    let (status, _) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/entries/{entry_id}"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/entries"),
        &mut session,
    )
    .await;
    assert!(body["entries"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn cross_workspace_entry_denied() {
    let app = app();
    let (_, _, workspace_id) = setup_workspace(&app).await;
    let (mut outsider, _) = register(&app, "outsider@example.com").await;

    let (status, _) = get(
        &app,
        &format!("/workspaces/{workspace_id}/entries"),
        &mut outsider,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── Report tests ────────────────────────────────────────────────────────────

#[tokio::test]
async fn balance_sheet_empty_workspace() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/reports/balance-sheet?as_of=2026-12-31"),
        &mut session,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["total_assets"], 0);
    assert_eq!(body["total_liabilities"], 0);
    assert_eq!(body["net_income"], 0);
}

#[tokio::test]
async fn balance_sheet_with_entries() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let employment = find_account_id(&app, &workspace_id, &mut session, "Employment").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    create_entry(
        &app,
        &workspace_id,
        &mut session,
        &employment,
        &chequing,
        500000,
        "2026-01-01",
        "Salary",
    )
    .await;
    create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &food,
        1200,
        "2026-01-05",
        "Lunch",
    )
    .await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/reports/balance-sheet?as_of=2026-01-31"),
        &mut session,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_assets"], 498800);
    assert_eq!(body["net_income"], 498800);
    assert_eq!(body["total_assets"], body["total_liabilities_and_equity"]);
}

#[tokio::test]
async fn income_statement_for_period() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let employment = find_account_id(&app, &workspace_id, &mut session, "Employment").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    create_entry(
        &app,
        &workspace_id,
        &mut session,
        &employment,
        &chequing,
        500000,
        "2026-01-01",
        "Salary",
    )
    .await;
    create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &food,
        1200,
        "2026-01-05",
        "Lunch",
    )
    .await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/reports/income-statement?start_date=2026-01-01&end_date=2026-01-31"),
        &mut session,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_revenue"], 500000);
    assert_eq!(body["total_expenses"], 1200);
    assert_eq!(body["net_income"], 498800);
}

#[tokio::test]
async fn balance_sheet_missing_date_rejected() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (status, _) = get(
        &app,
        &format!("/workspaces/{workspace_id}/reports/balance-sheet"),
        &mut session,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn income_statement_missing_dates_rejected() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (status, _) = get(
        &app,
        &format!("/workspaces/{workspace_id}/reports/income-statement"),
        &mut session,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

// ── Baseline tests ──────────────────────────────────────────────────────────

#[tokio::test]
async fn create_baseline_succeeds() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    let (status, body) = post(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        &mut session,
        json!({
            "account_id": food,
            "period_start": "2026-01-01",
            "amount": 50000,
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "create baseline failed: {body}");
    assert_eq!(body["amount"], 50000);
    assert_eq!(body["period_start"], "2026-01-01");
}

#[tokio::test]
async fn create_duplicate_baseline_rejected() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    post(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        &mut session,
        json!({ "account_id": food, "period_start": "2026-01-01", "amount": 50000 }),
    )
    .await;

    let (status, _) = post(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        &mut session,
        json!({ "account_id": food, "period_start": "2026-01-01", "amount": 60000 }),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn list_baselines() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;
    let rent = find_account_id(&app, &workspace_id, &mut session, "Rent").await;

    post(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        &mut session,
        json!({ "account_id": food, "period_start": "2026-01-01", "amount": 50000 }),
    )
    .await;
    post(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        &mut session,
        json!({ "account_id": rent, "period_start": "2026-01-01", "amount": 150000 }),
    )
    .await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        &mut session,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["baselines"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn delete_baseline() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    let (_, created) = post(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        &mut session,
        json!({ "account_id": food, "period_start": "2026-01-01", "amount": 50000 }),
    )
    .await;
    let baseline_id = created["id"].as_str().unwrap().to_string();

    let (status, _) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/baselines/{baseline_id}"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        &mut session,
    )
    .await;
    assert!(body["baselines"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn baseline_variance_endpoint() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    let (_, created) = post(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        &mut session,
        json!({ "account_id": food, "period_start": "2026-01-01", "amount": 50000 }),
    )
    .await;
    let baseline_id = created["id"].as_str().unwrap().to_string();

    create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &food,
        7500,
        "2026-01-15",
        "Groceries",
    )
    .await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/baselines/{baseline_id}/variance"),
        &mut session,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["variance"], -42500);
    assert_eq!(body["amount"], 50000);
    assert_eq!(body["actual"], 7500);
}

// ── Auth edge cases ─────────────────────────────────────────────────────────

#[tokio::test]
async fn duplicate_registration_rejected() {
    let app = app();
    register(&app, "alice@example.com").await;

    let mut session = Session::default();
    let (status, _) = post(
        &app,
        "/auth/register",
        &mut session,
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
    let (mut session, user_id, workspace_id) = setup_workspace(&app).await;

    let (status, body) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/members/{user_id}"),
        &mut session,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "last owner removal should fail: {body}"
    );
}

// ── Period-based report tests ──────────────────────────────────────────────

#[tokio::test]
async fn period_income_statement_jan() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let employment = find_account_id(&app, &workspace_id, &mut session, "Employment").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    create_entry(
        &app,
        &workspace_id,
        &mut session,
        &employment,
        &chequing,
        500000,
        "2026-01-01",
        "Salary",
    )
    .await;
    create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &food,
        1200,
        "2026-01-05",
        "Lunch",
    )
    .await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/2026/Jan/income-statement"),
        &mut session,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_revenue"], 500000);
    assert_eq!(body["total_expenses"], 1200);
    assert_eq!(body["net_income"], 498800);
}

#[tokio::test]
async fn period_balance_sheet_q1() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let employment = find_account_id(&app, &workspace_id, &mut session, "Employment").await;

    create_entry(
        &app,
        &workspace_id,
        &mut session,
        &employment,
        &chequing,
        500000,
        "2026-01-15",
        "Salary",
    )
    .await;
    create_entry(
        &app,
        &workspace_id,
        &mut session,
        &employment,
        &chequing,
        300000,
        "2026-03-15",
        "Salary",
    )
    .await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/2026/Q1/balance-sheet"),
        &mut session,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_assets"], 800000);
}

#[tokio::test]
async fn period_entries_filtered_by_date() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let employment = find_account_id(&app, &workspace_id, &mut session, "Employment").await;

    create_entry(
        &app,
        &workspace_id,
        &mut session,
        &employment,
        &chequing,
        500000,
        "2026-01-15",
        "Jan Salary",
    )
    .await;
    create_entry(
        &app,
        &workspace_id,
        &mut session,
        &employment,
        &chequing,
        300000,
        "2026-02-15",
        "Feb Salary",
    )
    .await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/2026/Jan/entries"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["entries"].as_array().unwrap().len(), 1);

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/2026/Q1/entries"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["entries"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn period_invalid_period_rejected() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (status, _) = get(
        &app,
        &format!("/workspaces/{workspace_id}/2026/Foo/income-statement"),
        &mut session,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

// ── Update entry tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn update_entry_date() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    let (_, created) = create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &food,
        1200,
        "2026-01-15",
        "Lunch",
    )
    .await;
    let entry_id = created["id"].as_str().unwrap().to_string();

    let (status, body) = put(
        &app,
        &format!("/workspaces/{workspace_id}/entries/{entry_id}"),
        &mut session,
        json!({ "date": "2026-03-01" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "update entry failed: {body}");
    assert_eq!(body["date"], "2026-03-01");
    assert_eq!(body["description"], "Lunch");
    assert_eq!(body["postings"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn update_entry_description() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    let (_, created) = create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &food,
        1200,
        "2026-01-15",
        "Lunch",
    )
    .await;
    let entry_id = created["id"].as_str().unwrap().to_string();

    let (status, body) = put(
        &app,
        &format!("/workspaces/{workspace_id}/entries/{entry_id}"),
        &mut session,
        json!({ "description": "Dinner" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "update entry failed: {body}");
    assert_eq!(body["description"], "Dinner");
    assert_eq!(body["date"], "2026-01-15");
    assert_eq!(body["postings"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn update_entry_postings() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;
    let rent = find_account_id(&app, &workspace_id, &mut session, "Rent").await;

    let (_, created) = create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &food,
        1200,
        "2026-01-15",
        "Lunch",
    )
    .await;
    let entry_id = created["id"].as_str().unwrap().to_string();

    let (status, body) = put(
        &app,
        &format!("/workspaces/{workspace_id}/entries/{entry_id}"),
        &mut session,
        json!({
            "postings": [
                { "account_id": chequing, "amount": -5000 },
                { "account_id": rent, "amount": 5000 },
            ]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "update entry failed: {body}");
    let postings = body["postings"].as_array().unwrap();
    assert_eq!(postings.len(), 2);

    let account_ids: Vec<&str> = postings
        .iter()
        .map(|p| p["account_id"].as_str().unwrap())
        .collect();
    assert!(account_ids.contains(&rent.as_str()));
    assert!(!account_ids.contains(&food.as_str()));
}

#[tokio::test]
async fn update_entry_requires_editor() {
    let pool = test_pool();
    let app = backend::build_router(pool.clone(), Mode::Dev);
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    let (_, created) = create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &food,
        1200,
        "2026-01-15",
        "Lunch",
    )
    .await;
    let entry_id = created["id"].as_str().unwrap().to_string();

    let (mut viewer, viewer_id) = register(&app, "viewer@example.com").await;
    {
        use diesel::prelude::*;
        let mut connection = pool.get().unwrap();
        diesel::sql_query(format!(
            "INSERT INTO workspace_members (workspace_id, user_id, role, joined_at) VALUES ('{}', '{}', 'viewer', datetime('now'))",
            workspace_id, viewer_id
        ))
        .execute(&mut connection)
        .unwrap();
    }

    let (status, _) = put(
        &app,
        &format!("/workspaces/{workspace_id}/entries/{entry_id}"),
        &mut viewer,
        json!({ "description": "Hacked" }),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn update_entry_not_found() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let fake_id = "00000000-0000-0000-0000-000000000000";
    let (status, _) = put(
        &app,
        &format!("/workspaces/{workspace_id}/entries/{fake_id}"),
        &mut session,
        json!({ "description": "Ghost" }),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Update account tests ───────────────────────────────────────────────────

#[tokio::test]
async fn update_account_rename() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing_id = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;

    let (status, body) = put(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{chequing_id}"),
        &mut session,
        json!({ "name": "Main Chequing" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "rename failed: {body}");
    assert_eq!(body["name"], "Main Chequing");
    assert_eq!(body["id"], chequing_id.as_str());
    assert_eq!(body["account_type"], "asset");
}

#[tokio::test]
async fn update_account_reparent() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let food_id = find_account_id(&app, &workspace_id, &mut session, "Food").await;
    let expenses_id = find_account_id(&app, &workspace_id, &mut session, "Expenses").await;

    let (_, created) = post(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &mut session,
        json!({ "name": "Categories", "parent_id": expenses_id }),
    )
    .await;
    let categories_id = created["id"].as_str().unwrap().to_string();

    let (status, body) = put(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{food_id}"),
        &mut session,
        json!({ "parent_id": categories_id }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "reparent failed: {body}");
    assert_eq!(body["parent_id"], categories_id);
    assert_eq!(body["id"], food_id.as_str());
}

#[tokio::test]
async fn update_account_rename_requires_editor() {
    let pool = test_pool();
    let app = backend::build_router(pool.clone(), Mode::Dev);
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing_id = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;

    let (mut viewer, viewer_id) = register(&app, "viewer@example.com").await;
    {
        use diesel::prelude::*;
        let mut connection = pool.get().unwrap();
        diesel::sql_query(format!(
            "INSERT INTO workspace_members (workspace_id, user_id, role, joined_at) VALUES ('{}', '{}', 'viewer', datetime('now'))",
            workspace_id, viewer_id
        ))
        .execute(&mut connection)
        .unwrap();
    }

    let (status, _) = put(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{chequing_id}"),
        &mut viewer,
        json!({ "name": "Hacked" }),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── Merge entries tests ────────────────────────────────────────────────────

#[tokio::test]
async fn merge_entries_succeeds() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;
    let savings = find_account_id(&app, &workspace_id, &mut session, "Savings").await;

    let (_, entry_one) = create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &food,
        1200,
        "2026-01-15",
        "Lunch",
    )
    .await;
    let (_, entry_two) = create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &savings,
        5000,
        "2026-01-16",
        "Savings",
    )
    .await;

    let entry_one_id = entry_one["id"].as_str().unwrap().to_string();
    let entry_two_id = entry_two["id"].as_str().unwrap().to_string();

    let (status, body) = post(
        &app,
        &format!("/workspaces/{workspace_id}/entries/merge"),
        &mut session,
        json!({
            "source_entry_ids": [entry_one_id, entry_two_id],
            "date": "2026-01-15",
            "description": "Merged transfer",
            "postings": [
                { "account_id": chequing, "amount": -6200 },
                { "account_id": savings, "amount": 6200 },
            ]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "merge failed: {body}");
    assert_eq!(body["description"], "Merged transfer");
    assert_eq!(body["postings"].as_array().unwrap().len(), 2);

    let (_, list_body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/entries"),
        &mut session,
    )
    .await;
    let entries = list_body["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["description"], "Merged transfer");
}

#[tokio::test]
async fn merge_entries_non_zero_sum_rejected() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    let (_, entry_one) = create_entry(
        &app,
        &workspace_id,
        &mut session,
        &chequing,
        &food,
        1200,
        "2026-01-15",
        "Lunch",
    )
    .await;
    let entry_one_id = entry_one["id"].as_str().unwrap().to_string();

    let (status, _) = post(
        &app,
        &format!("/workspaces/{workspace_id}/entries/merge"),
        &mut session,
        json!({
            "source_entry_ids": [entry_one_id],
            "date": "2026-01-15",
            "description": "Bad merge",
            "postings": [
                { "account_id": chequing, "amount": -1200 },
                { "account_id": food, "amount": 1000 },
            ]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

// ── Recurring rule tests ────────────────────────────────────────────────────

#[tokio::test]
async fn create_recurring_rule_succeeds() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let rent = find_account_id(&app, &workspace_id, &mut session, "Rent").await;

    // Future dtstart so insert-time backfill is a no-op and assertions stay stable.
    let (status, body) = post(
        &app,
        &format!("/workspaces/{workspace_id}/recurring"),
        &mut session,
        json!({
            "description": "Monthly rent",
            "schedule": "monthly-first",
            "dtstart": "2099-01-01",
            "templates": [
                { "account_id": chequing, "amount": -150000 },
                { "account_id": rent, "amount": 150000 },
            ]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "create recurring failed: {body}");
    assert_eq!(body["description"], "Monthly rent");
    assert_eq!(body["dtstart"], "2099-01-01");
    assert_eq!(body["schedule"], "monthly-first");
    assert_eq!(body["rrule"], "FREQ=MONTHLY;BYMONTHDAY=1");
    assert_eq!(body["next_pending"], "2099-01-01");
    assert_eq!(body["templates"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn create_recurring_rejects_unknown_schedule() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let rent = find_account_id(&app, &workspace_id, &mut session, "Rent").await;

    let (status, _) = post(
        &app,
        &format!("/workspaces/{workspace_id}/recurring"),
        &mut session,
        json!({
            "description": "Bad",
            "schedule": "every-blue-moon",
            "dtstart": "2099-01-01",
            "templates": [
                { "account_id": chequing, "amount": -150000 },
                { "account_id": rent, "amount": 150000 },
            ]
        }),
    )
    .await;

    // axum returns 400 when serde rejects the body before the handler runs.
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "expected 400 or 422 for unknown schedule, got {status}"
    );
}

#[tokio::test]
async fn create_recurring_rejects_non_zero_sum_templates() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let rent = find_account_id(&app, &workspace_id, &mut session, "Rent").await;

    let (status, _) = post(
        &app,
        &format!("/workspaces/{workspace_id}/recurring"),
        &mut session,
        json!({
            "description": "Bad",
            "schedule": "monthly-first",
            "dtstart": "2099-01-01",
            "templates": [
                { "account_id": chequing, "amount": -150000 },
                { "account_id": rent, "amount": 100000 },
            ]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn list_recurring_rules() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let rent = find_account_id(&app, &workspace_id, &mut session, "Rent").await;
    let food = find_account_id(&app, &workspace_id, &mut session, "Food").await;

    post(
        &app,
        &format!("/workspaces/{workspace_id}/recurring"),
        &mut session,
        json!({
            "description": "Rent",
            "schedule": "monthly-first",
            "dtstart": "2099-03-01",
            "templates": [
                { "account_id": chequing, "amount": -150000 },
                { "account_id": rent, "amount": 150000 },
            ]
        }),
    )
    .await;

    post(
        &app,
        &format!("/workspaces/{workspace_id}/recurring"),
        &mut session,
        json!({
            "description": "Groceries",
            "schedule": "weekly",
            "dtstart": "2099-01-05",
            "templates": [
                { "account_id": chequing, "amount": -5000 },
                { "account_id": food, "amount": 5000 },
            ]
        }),
    )
    .await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/recurring"),
        &mut session,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let rules = body["recurring_rules"].as_array().unwrap();
    assert_eq!(rules.len(), 2);
    // Ordered by dtstart ascending
    assert_eq!(rules[0]["description"], "Groceries");
    assert_eq!(rules[1]["description"], "Rent");
}

#[tokio::test]
async fn get_recurring_rule_returns_templates_and_next_pending() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let rent = find_account_id(&app, &workspace_id, &mut session, "Rent").await;

    let (_, created) = post(
        &app,
        &format!("/workspaces/{workspace_id}/recurring"),
        &mut session,
        json!({
            "description": "Rent",
            "schedule": "monthly-first",
            "dtstart": "2099-01-01",
            "templates": [
                { "account_id": chequing, "amount": -150000 },
                { "account_id": rent, "amount": 150000 },
            ]
        }),
    )
    .await;
    let rule_id = created["id"].as_str().unwrap().to_string();

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/recurring/{rule_id}"),
        &mut session,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["schedule"], "monthly-first");
    assert_eq!(body["next_pending"], "2099-01-01");
    assert_eq!(body["templates"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn delete_recurring_rule_excludes_from_list() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let rent = find_account_id(&app, &workspace_id, &mut session, "Rent").await;

    let (_, created) = post(
        &app,
        &format!("/workspaces/{workspace_id}/recurring"),
        &mut session,
        json!({
            "description": "Rent",
            "schedule": "monthly-first",
            "dtstart": "2099-01-01",
            "templates": [
                { "account_id": chequing, "amount": -150000 },
                { "account_id": rent, "amount": 150000 },
            ]
        }),
    )
    .await;
    let rule_id = created["id"].as_str().unwrap().to_string();

    let (status, _) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/recurring/{rule_id}"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/recurring"),
        &mut session,
    )
    .await;
    assert!(body["recurring_rules"].as_array().unwrap().is_empty());

    // GET after delete returns 404
    let (status, _) = get(
        &app,
        &format!("/workspaces/{workspace_id}/recurring/{rule_id}"),
        &mut session,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_run_is_idempotent() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &mut session, "Chequing").await;
    let rent = find_account_id(&app, &workspace_id, &mut session, "Rent").await;

    // Past dtstart → insert backfills, so the materialize endpoint should post 0 more.
    post(
        &app,
        &format!("/workspaces/{workspace_id}/recurring"),
        &mut session,
        json!({
            "description": "Old rent",
            "schedule": "monthly-first",
            "dtstart": "2020-01-01",
            "templates": [
                { "account_id": chequing, "amount": -150000 },
                { "account_id": rent, "amount": 150000 },
            ]
        }),
    )
    .await;

    let (status, body) = post(
        &app,
        &format!("/workspaces/{workspace_id}/recurring/runs"),
        &mut session,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create run failed: {body}");
    assert_eq!(body["posted"], 0, "create-time backfill should leave nothing due");
    assert!(body["at"].is_string(), "response should carry timestamp");

    // Entries from backfill should be present on the workspace.
    let (_, entries_body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/entries"),
        &mut session,
    )
    .await;
    let entries = entries_body["entries"].as_array().unwrap();
    let materialized: Vec<_> = entries
        .iter()
        .filter(|e| e["source_rule"].as_str().is_some() && e["description"] == "Old rent")
        .collect();
    assert!(materialized.len() >= 12, "expected backfilled monthly entries");
}

#[tokio::test]
async fn non_member_cannot_create_recurring_rule() {
    let app = app();
    let (_, _, workspace_id) = setup_workspace(&app).await;
    let (mut outsider, _) = register(&app, "outsider@example.com").await;

    let (status, _) = post(
        &app,
        &format!("/workspaces/{workspace_id}/recurring"),
        &mut outsider,
        json!({
            "description": "Sneaky",
            "schedule": "monthly-first",
            "dtstart": "2099-01-01",
            "templates": []
        }),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── Workspace update tests ──────────────────────────────────────────────────

#[tokio::test]
async fn update_workspace_name() {
    let app = app();
    let (mut session, _, workspace_id) = setup_workspace(&app).await;

    let (status, body) = put(
        &app,
        &format!("/workspaces/{workspace_id}"),
        &mut session,
        json!({ "name": "Renamed Finances" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "update workspace failed: {body}");
    assert_eq!(body["name"], "Renamed Finances");
}

