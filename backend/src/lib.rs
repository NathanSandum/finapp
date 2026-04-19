pub mod auth;
pub mod domain;
pub mod error;
pub mod permissions;
pub mod routes;
pub mod schema;
pub mod test_helpers;

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

pub fn build_router(pool: DbPool) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/logout", post(routes::auth::logout))
        .route("/workspaces", post(routes::workspaces::create))
        .route("/workspaces", get(routes::workspaces::list))
        .route("/workspaces/{workspace}", get(routes::workspaces::get))
        .route(
            "/workspaces/{workspace}/members/{user}",
            delete(routes::workspaces::remove_member),
        )
        // Categories
        .route(
            "/workspaces/{workspace}/categories",
            get(routes::categories::list).post(routes::categories::create),
        )
        .route(
            "/workspaces/{workspace}/categories/{category}",
            delete(routes::categories::delete),
        )
        // Accounts
        .route(
            "/workspaces/{workspace}/accounts",
            get(routes::accounts::list).post(routes::accounts::create),
        )
        .route(
            "/workspaces/{workspace}/accounts/{account}",
            delete(routes::accounts::delete),
        )
        .route(
            "/workspaces/{workspace}/accounts/{account}/balance",
            get(routes::accounts::balance),
        )
        // Import
        .route(
            "/workspaces/{workspace}/import",
            post(routes::import::import),
        )
        // Transactions
        .route(
            "/workspaces/{workspace}/transactions",
            post(routes::transactions::create),
        )
        .route(
            "/workspaces/{workspace}/accounts/{account}/transactions",
            get(routes::transactions::list),
        )
        .route(
            "/workspaces/{workspace}/transactions/{transaction}",
            delete(routes::transactions::delete),
        )
        .layer(Extension(pool))
        .layer(CorsLayer::permissive())
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
