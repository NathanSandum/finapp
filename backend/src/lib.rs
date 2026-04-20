mod auth;
pub mod domain;
mod error;
mod permissions;
mod routes;
mod schema;
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

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

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
        // TODO: Wave 2 — transaction routes (postings)
        // TODO: Wave 3 — baseline routes
        // TODO: Wave 4 — report routes, import
        .layer(Extension(pool))
        .layer(CorsLayer::permissive())
}
