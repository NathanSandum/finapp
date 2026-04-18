mod auth;
mod domain;
mod error;
mod permissions;
mod routes;
mod schema;
mod test_helpers;

use axum::{
    extract::Extension,
    routing::{delete, get, post},
    Json, Router,
};
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[tokio::main]
async fn main() {
    let manager = ConnectionManager::<SqliteConnection>::new("finapp.db");
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("failed to create db pool");

    let app = Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/logout", post(routes::auth::logout))
        .route("/workspaces", post(routes::workspaces::create))
        .route("/workspaces", get(routes::workspaces::list))
        .route("/workspaces/{workspace_id}", get(routes::workspaces::get))
        .route(
            "/workspaces/{workspace_id}/members/{user_id}",
            delete(routes::workspaces::remove_member),
        )
        // Categories
        .route(
            "/workspaces/{workspace_id}/categories",
            get(routes::categories::list).post(routes::categories::create),
        )
        .route(
            "/workspaces/{workspace_id}/categories/{category_id}",
            delete(routes::categories::delete),
        )
        // Accounts
        .route(
            "/workspaces/{workspace_id}/accounts",
            get(routes::accounts::list).post(routes::accounts::create),
        )
        .route(
            "/workspaces/{workspace_id}/accounts/{account_id}",
            delete(routes::accounts::delete),
        )
        .route(
            "/workspaces/{workspace_id}/accounts/{account_id}/balance",
            get(routes::accounts::balance),
        )
        // Transactions
        .route(
            "/workspaces/{workspace_id}/transactions",
            post(routes::transactions::create),
        )
        .route(
            "/workspaces/{workspace_id}/accounts/{account_id}/transactions",
            get(routes::transactions::list),
        )
        .route(
            "/workspaces/{workspace_id}/transactions/{transaction_id}",
            delete(routes::transactions::delete),
        )
        .layer(Extension(pool))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Backend running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
