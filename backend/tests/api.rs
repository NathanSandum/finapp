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
    let (token, user_id) = register(app, "alice@example.com").await;

    let (status, body) = post(
        app,
        "/workspaces",
        Some(&token),
        json!({ "name": "My Finances" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create workspace failed: {body}");
    let ws_id = body["id"].as_str().unwrap().to_string();

    (token, user_id, ws_id)
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
    let (token, _, ws_id) = setup_workspace(&app).await;

    let (status, body) = get(&app, &format!("/workspaces/{ws_id}"), &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "My Finances");
    assert_eq!(body["your_role"], "owner");
    assert!(!body["members"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn non_member_cannot_access_workspace() {
    let app = app();
    let (_, _, ws_id) = setup_workspace(&app).await;

    let (outsider_token, _) = register(&app, "outsider@example.com").await;
    let (status, _) = get(&app, &format!("/workspaces/{ws_id}"), &outsider_token).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── Category tests ───────────────────────────────────────────────────────────

#[tokio::test]
async fn category_crud() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;

    // Create
    let (status, body) = post(
        &app,
        &format!("/workspaces/{ws_id}/categories"),
        Some(&token),
        json!({ "name": "Groceries", "description": "Food" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let cat_id = body["id"].as_str().unwrap().to_string();
    assert_eq!(body["name"], "Groceries");

    // List
    let (status, body) = get(&app, &format!("/workspaces/{ws_id}/categories"), &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["categories"].as_array().unwrap().len(), 1);

    // Delete
    let (status, _) = delete(
        &app,
        &format!("/workspaces/{ws_id}/categories/{cat_id}"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // List should be empty after soft delete
    let (status, body) = get(&app, &format!("/workspaces/{ws_id}/categories"), &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["categories"].as_array().unwrap().len(), 0);
}

// ── Account tests ────────────────────────────────────────────────────────────

#[tokio::test]
async fn account_crud_and_balance() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;

    // Create account
    let (status, body) = post(
        &app,
        &format!("/workspaces/{ws_id}/accounts"),
        Some(&token),
        json!({ "name": "Chequing", "account_type": "chequing" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let acct_id = body["id"].as_str().unwrap().to_string();
    assert_eq!(body["currency"], "CAD");

    // List
    let (status, body) = get(&app, &format!("/workspaces/{ws_id}/accounts"), &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["accounts"].as_array().unwrap().len(), 1);

    // Balance (should be 0)
    let (status, body) = get(
        &app,
        &format!("/workspaces/{ws_id}/accounts/{acct_id}/balance"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["cleared_balance"], 0);

    // Delete
    let (status, _) = delete(
        &app,
        &format!("/workspaces/{ws_id}/accounts/{acct_id}"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = get(&app, &format!("/workspaces/{ws_id}/accounts"), &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["accounts"].as_array().unwrap().len(), 0);
}

// ── Transaction tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn transaction_lifecycle() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;

    // Create account + category
    let (_, acct_body) = post(
        &app,
        &format!("/workspaces/{ws_id}/accounts"),
        Some(&token),
        json!({ "name": "Chequing", "account_type": "chequing" }),
    )
    .await;
    let acct_id = acct_body["id"].as_str().unwrap();

    let (_, cat_body) = post(
        &app,
        &format!("/workspaces/{ws_id}/categories"),
        Some(&token),
        json!({ "name": "Groceries" }),
    )
    .await;
    let cat_id = cat_body["id"].as_str().unwrap();

    // Create transaction
    let (status, txn_body) = post(
        &app,
        &format!("/workspaces/{ws_id}/transactions"),
        Some(&token),
        json!({
            "account": acct_id,
            "date": "2026-04-15",
            "description": "Weekly groceries",
            "amount": -5000,
            "cleared": true,
            "line_items": [
                { "category_id": cat_id, "amount": -5000 }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let txn_id = txn_body["id"].as_str().unwrap().to_string();
    assert_eq!(txn_body["amount"], -5000);
    assert_eq!(txn_body["line_items"].as_array().unwrap().len(), 1);

    // List transactions for account
    let (status, body) = get(
        &app,
        &format!("/workspaces/{ws_id}/accounts/{acct_id}/transactions"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["transactions"].as_array().unwrap().len(), 1);

    // Balance should reflect cleared transaction
    let (_, balance_body) = get(
        &app,
        &format!("/workspaces/{ws_id}/accounts/{acct_id}/balance"),
        &token,
    )
    .await;
    assert_eq!(balance_body["cleared_balance"], -5000);

    // Delete transaction
    let (status, _) = delete(
        &app,
        &format!("/workspaces/{ws_id}/transactions/{txn_id}"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Balance should be back to 0
    let (_, balance_body) = get(
        &app,
        &format!("/workspaces/{ws_id}/accounts/{acct_id}/balance"),
        &token,
    )
    .await;
    assert_eq!(balance_body["cleared_balance"], 0);
}

#[tokio::test]
async fn transaction_validation_errors() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;

    let (_, acct_body) = post(
        &app,
        &format!("/workspaces/{ws_id}/accounts"),
        Some(&token),
        json!({ "name": "Chequing", "account_type": "chequing" }),
    )
    .await;
    let acct_id = acct_body["id"].as_str().unwrap();

    let (_, cat_body) = post(
        &app,
        &format!("/workspaces/{ws_id}/categories"),
        Some(&token),
        json!({ "name": "Groceries" }),
    )
    .await;
    let cat_id = cat_body["id"].as_str().unwrap();

    // Mismatched line items sum
    let (status, body) = post(
        &app,
        &format!("/workspaces/{ws_id}/transactions"),
        Some(&token),
        json!({
            "account": acct_id,
            "date": "2026-04-15",
            "amount": -5000,
            "line_items": [
                { "category_id": cat_id, "amount": -3000 }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["code"], "validation_error");

    // Empty line items
    let (status, _) = post(
        &app,
        &format!("/workspaces/{ws_id}/transactions"),
        Some(&token),
        json!({
            "account": acct_id,
            "date": "2026-04-15",
            "amount": -5000,
            "line_items": []
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn cross_workspace_access_denied() {
    let app = app();
    let (_, _, ws_id) = setup_workspace(&app).await;

    let (token_b, _) = register(&app, "bob@example.com").await;

    // Bob can't list Alice's categories
    let (status, _) = get(
        &app,
        &format!("/workspaces/{ws_id}/categories"),
        &token_b,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Bob can't list Alice's accounts
    let (status, _) = get(
        &app,
        &format!("/workspaces/{ws_id}/accounts"),
        &token_b,
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
    // Can't test multi-owner without an invite API — skip for now.
    // But we CAN test that the last owner is rejected.
    let app = app();
    let (token, user_id, ws_id) = setup_workspace(&app).await;

    let (status, body) = delete(
        &app,
        &format!("/workspaces/{ws_id}/members/{user_id}"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "last owner removal should fail: {body}");
}

// ── Balance: pending vs cleared ─────────────────────────────────────────────

#[tokio::test]
async fn balance_only_includes_cleared() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;

    let (_, acct_body) = post(
        &app,
        &format!("/workspaces/{ws_id}/accounts"),
        Some(&token),
        json!({ "name": "Chequing", "account_type": "chequing" }),
    )
    .await;
    let acct_id = acct_body["id"].as_str().unwrap();

    let (_, cat_body) = post(
        &app,
        &format!("/workspaces/{ws_id}/categories"),
        Some(&token),
        json!({ "name": "Income" }),
    )
    .await;
    let cat_id = cat_body["id"].as_str().unwrap();

    // Cleared transaction
    post(
        &app,
        &format!("/workspaces/{ws_id}/transactions"),
        Some(&token),
        json!({
            "account": acct_id,
            "date": "2026-04-10",
            "amount": 100000,
            "cleared": true,
            "line_items": [{ "category_id": cat_id, "amount": 100000 }]
        }),
    )
    .await;

    // Pending transaction — should NOT affect balance
    post(
        &app,
        &format!("/workspaces/{ws_id}/transactions"),
        Some(&token),
        json!({
            "account": acct_id,
            "date": "2026-04-12",
            "amount": -5000,
            "line_items": [{ "category_id": cat_id, "amount": -5000 }]
        }),
    )
    .await;

    let (_, body) = get(
        &app,
        &format!("/workspaces/{ws_id}/accounts/{acct_id}/balance"),
        &token,
    )
    .await;
    assert_eq!(body["cleared_balance"], 100000);
}

// ── Transfer transaction ────────────────────────────────────────────────────

#[tokio::test]
async fn transfer_transaction_records_transfer_account() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;

    let (_, chq_body) = post(
        &app,
        &format!("/workspaces/{ws_id}/accounts"),
        Some(&token),
        json!({ "name": "Chequing", "account_type": "chequing" }),
    )
    .await;
    let chq_id = chq_body["id"].as_str().unwrap();

    let (_, sav_body) = post(
        &app,
        &format!("/workspaces/{ws_id}/accounts"),
        Some(&token),
        json!({ "name": "Savings", "account_type": "savings" }),
    )
    .await;
    let sav_id = sav_body["id"].as_str().unwrap();

    let (_, cat_body) = post(
        &app,
        &format!("/workspaces/{ws_id}/categories"),
        Some(&token),
        json!({ "name": "Transfer" }),
    )
    .await;
    let cat_id = cat_body["id"].as_str().unwrap();

    let (status, txn_body) = post(
        &app,
        &format!("/workspaces/{ws_id}/transactions"),
        Some(&token),
        json!({
            "account": chq_id,
            "transfer_account": sav_id,
            "date": "2026-04-15",
            "amount": -10000,
            "line_items": [{ "category_id": cat_id, "amount": -10000 }]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(txn_body["transfer_account"], sav_id);
}

// ── Baseline tests ─────────────────────────────────────────────────────────

/// Helper: create a category and return its id.
async fn create_category(app: &Router, ws_id: &str, token: &str, name: &str) -> String {
    let (_, body) = post(
        app,
        &format!("/workspaces/{ws_id}/categories"),
        Some(token),
        json!({ "name": name }),
    )
    .await;
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn create_and_list_baselines() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;
    let cat_id = create_category(&app, &ws_id, &token, "Groceries").await;

    // Create baseline
    let (status, body) = post(
        &app,
        &format!("/workspaces/{ws_id}/baselines"),
        Some(&token),
        json!({
            "category": cat_id,
            "period_type": "monthly",
            "period_start": "2026-04-01",
            "amount": -50000,
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["amount"], -50000);
    assert_eq!(body["period_type"], "monthly");
    assert_eq!(body["category"], cat_id);

    // List baselines for this period
    let (status, body) = get(
        &app,
        &format!("/workspaces/{ws_id}/baselines?period_start=2026-04-01"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let baselines = body["baselines"].as_array().unwrap();
    assert_eq!(baselines.len(), 1);
    assert_eq!(baselines[0]["amount"], -50000);
}

#[tokio::test]
async fn update_baseline_amount() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;
    let cat_id = create_category(&app, &ws_id, &token, "Groceries").await;

    let (_, body) = post(
        &app,
        &format!("/workspaces/{ws_id}/baselines"),
        Some(&token),
        json!({
            "category": cat_id,
            "period_type": "monthly",
            "period_start": "2026-04-01",
            "amount": -50000,
        }),
    )
    .await;
    let baseline_id = body["id"].as_str().unwrap();

    // Update
    let (status, body) = put(
        &app,
        &format!("/workspaces/{ws_id}/baselines/{baseline_id}"),
        &token,
        json!({ "amount": -60000 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["amount"], -60000);

    // Verify via list
    let (_, body) = get(
        &app,
        &format!("/workspaces/{ws_id}/baselines?period_start=2026-04-01"),
        &token,
    )
    .await;
    assert_eq!(body["baselines"][0]["amount"], -60000);
}

#[tokio::test]
async fn duplicate_baseline_rejected() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;
    let cat_id = create_category(&app, &ws_id, &token, "Groceries").await;

    post(
        &app,
        &format!("/workspaces/{ws_id}/baselines"),
        Some(&token),
        json!({
            "category": cat_id,
            "period_type": "monthly",
            "period_start": "2026-04-01",
            "amount": -50000,
        }),
    )
    .await;

    let (status, _) = post(
        &app,
        &format!("/workspaces/{ws_id}/baselines"),
        Some(&token),
        json!({
            "category": cat_id,
            "period_type": "monthly",
            "period_start": "2026-04-01",
            "amount": -60000,
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn baseline_summary_with_actuals() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;
    let cat_id = create_category(&app, &ws_id, &token, "Groceries").await;

    // Create account for transactions
    let (_, acct_body) = post(
        &app,
        &format!("/workspaces/{ws_id}/accounts"),
        Some(&token),
        json!({ "name": "Chequing", "account_type": "chequing" }),
    )
    .await;
    let acct_id = acct_body["id"].as_str().unwrap();

    // Set baseline
    post(
        &app,
        &format!("/workspaces/{ws_id}/baselines"),
        Some(&token),
        json!({
            "category": cat_id,
            "period_type": "monthly",
            "period_start": "2026-04-01",
            "amount": -50000,
        }),
    )
    .await;

    // Create cleared transaction
    post(
        &app,
        &format!("/workspaces/{ws_id}/transactions"),
        Some(&token),
        json!({
            "account": acct_id,
            "date": "2026-04-10",
            "amount": -45000,
            "cleared": true,
            "line_items": [{ "category_id": cat_id, "amount": -45000 }]
        }),
    )
    .await;

    // Get summary
    let (status, body) = get(
        &app,
        &format!("/workspaces/{ws_id}/baselines/summary?period_start=2026-04-01&period_end=2026-04-30"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let items = body["summary"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["actual"], -45000);
    assert_eq!(items[0]["baseline"], -50000);
    assert_eq!(items[0]["variance"], 5000); // spent less than baseline
}

#[tokio::test]
async fn non_member_cannot_access_baselines() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;
    let cat_id = create_category(&app, &ws_id, &token, "Groceries").await;

    // Create a baseline as owner
    post(
        &app,
        &format!("/workspaces/{ws_id}/baselines"),
        Some(&token),
        json!({
            "category": cat_id,
            "period_type": "monthly",
            "period_start": "2026-04-01",
            "amount": -50000,
        }),
    )
    .await;

    // Outsider can't list
    let (outsider_token, _) = register(&app, "outsider@example.com").await;
    let (status, _) = get(
        &app,
        &format!("/workspaces/{ws_id}/baselines?period_start=2026-04-01"),
        &outsider_token,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
// ── Import tests ───────────────────────────────────────────────────────────

#[tokio::test]
async fn import_creates_accounts_categories_and_transactions() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;

    let (status, body) = post(
        &app,
        &format!("/workspaces/{ws_id}/import"),
        Some(&token),
        json!({
            "transactions": [
                {
                    "account": "Chequing",
                    "account_type": "chequing",
                    "date": "2026-01-02",
                    "description": "Household Expenses",
                    "amount": -12318,
                    "category": "Household Expenses",
                    "cleared": true
                },
                {
                    "account": "Chequing",
                    "account_type": "chequing",
                    "date": "2026-01-02",
                    "description": "Cafes",
                    "amount": -901,
                    "category": "Cafes",
                    "cleared": true
                },
                {
                    "account": "Chequing",
                    "account_type": "chequing",
                    "date": "2026-01-05",
                    "description": "Benefits and Tax Credits",
                    "amount": 13325,
                    "category": "Benefits and Tax Credits",
                    "cleared": true
                }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "import failed: {body}");
    assert_eq!(body["accounts_created"], 1);
    assert_eq!(body["categories_created"], 3);
    assert_eq!(body["transactions_created"], 3);

    // Verify accounts exist
    let (_, acct_body) = get(&app, &format!("/workspaces/{ws_id}/accounts"), &token).await;
    let accounts = acct_body["accounts"].as_array().unwrap();
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0]["name"], "Chequing");

    // Verify categories exist
    let (_, cat_body) = get(&app, &format!("/workspaces/{ws_id}/categories"), &token).await;
    assert_eq!(cat_body["categories"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn import_reuses_existing_accounts_and_categories() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;

    // Pre-create an account and category
    post(
        &app,
        &format!("/workspaces/{ws_id}/accounts"),
        Some(&token),
        json!({ "name": "Chequing", "account_type": "chequing" }),
    )
    .await;
    post(
        &app,
        &format!("/workspaces/{ws_id}/categories"),
        Some(&token),
        json!({ "name": "Cafes" }),
    )
    .await;

    let (status, body) = post(
        &app,
        &format!("/workspaces/{ws_id}/import"),
        Some(&token),
        json!({
            "transactions": [
                {
                    "account": "Chequing",
                    "account_type": "chequing",
                    "date": "2026-01-06",
                    "description": "Cafes",
                    "amount": -1506,
                    "category": "Cafes",
                    "cleared": true
                }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "import failed: {body}");
    assert_eq!(body["accounts_created"], 0);
    assert_eq!(body["categories_created"], 0);
    assert_eq!(body["transactions_created"], 1);

    // Should still only have 1 account, 1 category
    let (_, acct_body) = get(&app, &format!("/workspaces/{ws_id}/accounts"), &token).await;
    assert_eq!(acct_body["accounts"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn import_multiple_accounts() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;

    let (status, body) = post(
        &app,
        &format!("/workspaces/{ws_id}/import"),
        Some(&token),
        json!({
            "transactions": [
                {
                    "account": "Chequing",
                    "account_type": "chequing",
                    "date": "2026-01-02",
                    "description": "Groceries",
                    "amount": -5000,
                    "category": "Household Expenses",
                    "cleared": true
                },
                {
                    "account": "Credit Card",
                    "account_type": "credit",
                    "date": "2026-01-03",
                    "description": "Recreation",
                    "amount": -2559,
                    "category": "Recreation",
                    "cleared": true
                }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["accounts_created"], 2);
    assert_eq!(body["transactions_created"], 2);
}

#[tokio::test]
async fn import_empty_transactions_succeeds() {
    let app = app();
    let (token, _, ws_id) = setup_workspace(&app).await;

    let (status, body) = post(
        &app,
        &format!("/workspaces/{ws_id}/import"),
        Some(&token),
        json!({ "transactions": [] }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["transactions_created"], 0);
}

#[tokio::test]
async fn import_non_member_denied() {
    let app = app();
    let (_, _, ws_id) = setup_workspace(&app).await;
    let (outsider_token, _) = register(&app, "outsider@example.com").await;

    let (status, _) = post(
        &app,
        &format!("/workspaces/{ws_id}/import"),
        Some(&outsider_token),
        json!({
            "transactions": [{
                "account": "Chequing",
                "account_type": "chequing",
                "date": "2026-01-02",
                "amount": -1000,
                "category": "Test",
                "cleared": true
            }]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
