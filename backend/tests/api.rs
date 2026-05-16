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
    let envelope: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    // Unwrap JSend envelope: return `data` for success/fail, or the full envelope
    // for `error` (no data field) or non-JSend bodies.
    let body = match envelope.get("status").and_then(Value::as_str) {
        Some("success") | Some("fail") => envelope
            .get("data")
            .cloned()
            .unwrap_or(Value::Null),
        _ => envelope,
    };
    (status, body)
}

async fn get(app: &Router, uri: &str, token: &str) -> (StatusCode, Value) {
    send(app, Method::GET, uri, Some(token), None).await
}

async fn post(app: &Router, uri: &str, token: Option<&str>, body: Value) -> (StatusCode, Value) {
    send(app, Method::POST, uri, token, Some(body)).await
}

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
    assert_eq!(body["service"], "finapp");
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
    // 5 roots + 4 asset + 1 liability + 1 revenue + 7 expense + 1 equity = 19
    assert_eq!(accounts.len(), 19);

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
async fn accounts_response_includes_role() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &token,
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
    let (token, _, workspace_id) = setup_workspace(&app).await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/anchors"),
        &token,
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
async fn create_root_account_via_api_rejected() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    // Roots are seeded; no public path to create them.
    let (status, body) = post(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        Some(&token),
        json!({ "name": "Custom Root" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[tokio::test]
async fn delete_leaf_account() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    // Find a non-anchor leaf account (Chequing is an anchor and cannot be deleted).
    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        &token,
    )
    .await;
    let accounts = body["accounts"].as_array().unwrap();
    let savings = accounts
        .iter()
        .find(|a| a["name"] == "Savings")
        .unwrap();
    let savings_id = savings["id"].as_str().unwrap();

    let (status, _) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{savings_id}"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn delete_anchor_account_rejected() {
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

    let (status, _) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{chequing_id}"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
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
    assert_eq!(body["balance"], 0);
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

    // Verify balance via API
    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{chequing_id}/balance"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["balance"], -1200);

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{food_id}/balance"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["balance"], 1200);
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

// ── Entry tests ─────────────────────────────────────────────────────────────

/// Helper: find account ID by name from accounts list
async fn find_account_id(app: &Router, workspace_id: &str, token: &str, name: &str) -> String {
    let (_, body) = get(
        app,
        &format!("/workspaces/{workspace_id}/accounts"),
        token,
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

/// Helper: create a simple 2-posting entry via API
async fn create_entry(
    app: &Router,
    workspace_id: &str,
    token: &str,
    from_account: &str,
    to_account: &str,
    amount: i32,
    date: &str,
    description: &str,
) -> (StatusCode, Value) {
    post(
        app,
        &format!("/workspaces/{workspace_id}/entries"),
        Some(token),
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
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    let (status, body) = create_entry(
        &app, &workspace_id, &token, &chequing, &food, 1200, "2026-01-15", "Lunch",
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
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    let (status, _) = post(
        &app,
        &format!("/workspaces/{workspace_id}/entries"),
        Some(&token),
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
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;
    let rent = find_account_id(&app, &workspace_id, &token, "Rent").await;

    create_entry(&app, &workspace_id, &token, &chequing, &food, 1200, "2026-01-15", "Lunch").await;
    create_entry(&app, &workspace_id, &token, &chequing, &rent, 150000, "2026-01-16", "Rent").await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/entries"),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let entries = body["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    // Each entry has postings
    for entry in entries {
        assert!(!entry["postings"].as_array().unwrap().is_empty());
    }
}

#[tokio::test]
async fn get_single_entry() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    let (_, created) = create_entry(
        &app, &workspace_id, &token, &chequing, &food, 1200, "2026-01-15", "Lunch",
    )
    .await;
    let entry_id = created["id"].as_str().unwrap();

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/entries/{entry_id}"),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["description"], "Lunch");
    assert_eq!(body["postings"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn delete_entry_soft_deletes() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    let (_, created) = create_entry(
        &app, &workspace_id, &token, &chequing, &food, 1200, "2026-01-15", "Lunch",
    )
    .await;
    let entry_id = created["id"].as_str().unwrap();

    let (status, _) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/entries/{entry_id}"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // List should exclude deleted entry
    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/entries"),
        &token,
    )
    .await;
    assert!(body["entries"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn cross_workspace_entry_denied() {
    let app = app();
    let (_, _, workspace_id) = setup_workspace(&app).await;
    let (outsider_token, _) = register(&app, "outsider@example.com").await;

    let (status, _) = get(
        &app,
        &format!("/workspaces/{workspace_id}/entries"),
        &outsider_token,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── Report tests ────────────────────────────────────────────────────────────

#[tokio::test]
async fn balance_sheet_empty_workspace() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/reports/balance-sheet?as_of=2026-12-31"),
        &token,
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
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let employment = find_account_id(&app, &workspace_id, &token, "Employment").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    create_entry(&app, &workspace_id, &token, &employment, &chequing, 500000, "2026-01-01", "Salary").await;
    create_entry(&app, &workspace_id, &token, &chequing, &food, 1200, "2026-01-05", "Lunch").await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/reports/balance-sheet?as_of=2026-01-31"),
        &token,
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
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let employment = find_account_id(&app, &workspace_id, &token, "Employment").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    create_entry(&app, &workspace_id, &token, &employment, &chequing, 500000, "2026-01-01", "Salary").await;
    create_entry(&app, &workspace_id, &token, &chequing, &food, 1200, "2026-01-05", "Lunch").await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/reports/income-statement?start_date=2026-01-01&end_date=2026-01-31"),
        &token,
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
    let (token, _, workspace_id) = setup_workspace(&app).await;

    let (status, _) = get(
        &app,
        &format!("/workspaces/{workspace_id}/reports/balance-sheet"),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn income_statement_missing_dates_rejected() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    let (status, _) = get(
        &app,
        &format!("/workspaces/{workspace_id}/reports/income-statement"),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

// ── Baseline tests ──────────────────────────────────────────────────────────

#[tokio::test]
async fn create_baseline_succeeds() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    let (status, body) = post(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        Some(&token),
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
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    post(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        Some(&token),
        json!({ "account_id": food, "period_start": "2026-01-01", "amount": 50000 }),
    )
    .await;

    let (status, _) = post(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        Some(&token),
        json!({ "account_id": food, "period_start": "2026-01-01", "amount": 60000 }),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn list_baselines() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;
    let rent = find_account_id(&app, &workspace_id, &token, "Rent").await;

    post(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        Some(&token),
        json!({ "account_id": food, "period_start": "2026-01-01", "amount": 50000 }),
    )
    .await;
    post(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        Some(&token),
        json!({ "account_id": rent, "period_start": "2026-01-01", "amount": 150000 }),
    )
    .await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["baselines"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn delete_baseline() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    let (_, created) = post(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        Some(&token),
        json!({ "account_id": food, "period_start": "2026-01-01", "amount": 50000 }),
    )
    .await;
    let baseline_id = created["id"].as_str().unwrap();

    let (status, _) = delete(
        &app,
        &format!("/workspaces/{workspace_id}/baselines/{baseline_id}"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        &token,
    )
    .await;
    assert!(body["baselines"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn baseline_variance_endpoint() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    // Create baseline: $500 food budget for January
    let (_, created) = post(
        &app,
        &format!("/workspaces/{workspace_id}/baselines"),
        Some(&token),
        json!({ "account_id": food, "period_start": "2026-01-01", "amount": 50000 }),
    )
    .await;
    let baseline_id = created["id"].as_str().unwrap();

    // Spend $75 on food
    create_entry(&app, &workspace_id, &token, &chequing, &food, 7500, "2026-01-15", "Groceries").await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/baselines/{baseline_id}/variance"),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    // Variance = actual (7500) - budget (50000) = -42500 (under budget)
    assert_eq!(body["variance"], -42500);
    assert_eq!(body["amount"], 50000);
    assert_eq!(body["actual"], 7500);
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

// ── Period-based report tests ──────────────────────────────────────────────

#[tokio::test]
async fn period_income_statement_jan() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let employment = find_account_id(&app, &workspace_id, &token, "Employment").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    create_entry(&app, &workspace_id, &token, &employment, &chequing, 500000, "2026-01-01", "Salary").await;
    create_entry(&app, &workspace_id, &token, &chequing, &food, 1200, "2026-01-05", "Lunch").await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/2026/Jan/income-statement"),
        &token,
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
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let employment = find_account_id(&app, &workspace_id, &token, "Employment").await;

    create_entry(&app, &workspace_id, &token, &employment, &chequing, 500000, "2026-01-15", "Salary").await;
    create_entry(&app, &workspace_id, &token, &employment, &chequing, 300000, "2026-03-15", "Salary").await;

    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/2026/Q1/balance-sheet"),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    // Q1 balance sheet = as of Mar 31, both entries included
    assert_eq!(body["total_assets"], 800000);
}

#[tokio::test]
async fn period_entries_filtered_by_date() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let employment = find_account_id(&app, &workspace_id, &token, "Employment").await;

    create_entry(&app, &workspace_id, &token, &employment, &chequing, 500000, "2026-01-15", "Jan Salary").await;
    create_entry(&app, &workspace_id, &token, &employment, &chequing, 300000, "2026-02-15", "Feb Salary").await;

    // Jan only
    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/2026/Jan/entries"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["entries"].as_array().unwrap().len(), 1);

    // Q1 gets both
    let (status, body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/2026/Q1/entries"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["entries"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn period_invalid_period_rejected() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    let (status, _) = get(
        &app,
        &format!("/workspaces/{workspace_id}/2026/Foo/income-statement"),
        &token,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

// ── Update entry tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn update_entry_date() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    let (_, created) = create_entry(
        &app, &workspace_id, &token, &chequing, &food, 1200, "2026-01-15", "Lunch",
    )
    .await;
    let entry_id = created["id"].as_str().unwrap();

    let (status, body) = put(
        &app,
        &format!("/workspaces/{workspace_id}/entries/{entry_id}"),
        &token,
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
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    let (_, created) = create_entry(
        &app, &workspace_id, &token, &chequing, &food, 1200, "2026-01-15", "Lunch",
    )
    .await;
    let entry_id = created["id"].as_str().unwrap();

    let (status, body) = put(
        &app,
        &format!("/workspaces/{workspace_id}/entries/{entry_id}"),
        &token,
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
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;
    let rent = find_account_id(&app, &workspace_id, &token, "Rent").await;

    let (_, created) = create_entry(
        &app, &workspace_id, &token, &chequing, &food, 1200, "2026-01-15", "Lunch",
    )
    .await;
    let entry_id = created["id"].as_str().unwrap();

    let (status, body) = put(
        &app,
        &format!("/workspaces/{workspace_id}/entries/{entry_id}"),
        &token,
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

    // Verify old postings replaced: should reference rent, not food
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
    let app = backend::build_router(pool.clone());
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    let (_, created) = create_entry(
        &app, &workspace_id, &token, &chequing, &food, 1200, "2026-01-15", "Lunch",
    )
    .await;
    let entry_id = created["id"].as_str().unwrap();

    // Register a second user and add them as a viewer directly via DB
    let (viewer_token, _viewer_id) = register(&app, "viewer@example.com").await;
    {
        use diesel::prelude::*;
        let mut connection = pool.get().unwrap();
        let viewer_user_id: String = serde_json::from_value::<Value>(json!(_viewer_id))
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        diesel::sql_query(format!(
            "INSERT INTO workspace_members (workspace_id, user_id, role, joined_at) VALUES ('{}', '{}', 'viewer', datetime('now'))",
            workspace_id, viewer_user_id
        ))
        .execute(&mut connection)
        .unwrap();
    }

    let (status, _) = put(
        &app,
        &format!("/workspaces/{workspace_id}/entries/{entry_id}"),
        &viewer_token,
        json!({ "description": "Hacked" }),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn update_entry_not_found() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    let fake_id = "00000000-0000-0000-0000-000000000000";
    let (status, _) = put(
        &app,
        &format!("/workspaces/{workspace_id}/entries/{fake_id}"),
        &token,
        json!({ "description": "Ghost" }),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Update account tests ───────────────────────────────────────────────────

#[tokio::test]
async fn update_account_rename() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing_id = find_account_id(&app, &workspace_id, &token, "Chequing").await;

    let (status, body) = put(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{chequing_id}"),
        &token,
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
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let food_id = find_account_id(&app, &workspace_id, &token, "Food").await;
    let expenses_id = find_account_id(&app, &workspace_id, &token, "Expenses").await;

    // Create a sub-category under Expenses
    let (_, created) = post(
        &app,
        &format!("/workspaces/{workspace_id}/accounts"),
        Some(&token),
        json!({ "name": "Categories", "parent_id": expenses_id }),
    )
    .await;
    let categories_id = created["id"].as_str().unwrap();

    let (status, body) = put(
        &app,
        &format!("/workspaces/{workspace_id}/accounts/{food_id}"),
        &token,
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
    let app = backend::build_router(pool.clone());
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing_id = find_account_id(&app, &workspace_id, &token, "Chequing").await;

    // Register a viewer
    let (viewer_token, viewer_id) = register(&app, "viewer@example.com").await;
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
        &viewer_token,
        json!({ "name": "Hacked" }),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── Merge entries tests ────────────────────────────────────────────────────

#[tokio::test]
async fn merge_entries_succeeds() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;
    let savings = find_account_id(&app, &workspace_id, &token, "Savings").await;

    let (_, entry_one) = create_entry(
        &app, &workspace_id, &token, &chequing, &food, 1200, "2026-01-15", "Lunch",
    )
    .await;
    let (_, entry_two) = create_entry(
        &app, &workspace_id, &token, &chequing, &savings, 5000, "2026-01-16", "Savings",
    )
    .await;

    let entry_one_id = entry_one["id"].as_str().unwrap();
    let entry_two_id = entry_two["id"].as_str().unwrap();

    let (status, body) = post(
        &app,
        &format!("/workspaces/{workspace_id}/entries/merge"),
        Some(&token),
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

    // Verify originals are gone from list
    let (_, list_body) = get(
        &app,
        &format!("/workspaces/{workspace_id}/entries"),
        &token,
    )
    .await;
    let entries = list_body["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["description"], "Merged transfer");
}

#[tokio::test]
async fn merge_entries_non_zero_sum_rejected() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;
    let chequing = find_account_id(&app, &workspace_id, &token, "Chequing").await;
    let food = find_account_id(&app, &workspace_id, &token, "Food").await;

    let (_, entry_one) = create_entry(
        &app, &workspace_id, &token, &chequing, &food, 1200, "2026-01-15", "Lunch",
    )
    .await;
    let entry_one_id = entry_one["id"].as_str().unwrap();

    let (status, _) = post(
        &app,
        &format!("/workspaces/{workspace_id}/entries/merge"),
        Some(&token),
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

// ── Workspace update tests ──────────────────────────────────────────────────

#[tokio::test]
async fn update_workspace_name() {
    let app = app();
    let (token, _, workspace_id) = setup_workspace(&app).await;

    let (status, body) = put(
        &app,
        &format!("/workspaces/{workspace_id}"),
        &token,
        json!({ "name": "Renamed Finances" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "update workspace failed: {body}");
    assert_eq!(body["name"], "Renamed Finances");
}
