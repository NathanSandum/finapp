mod auth;
pub mod domain;
mod error;
pub mod jsend;
mod permissions;
mod routes;
mod schema;
pub mod test_helpers;

use axum::{
    extract::Extension,
    routing::{delete, get, post, put},
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
        .route(
            "/workspaces/{workspace}",
            get(routes::workspaces::get).put(routes::workspaces::update),
        )
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
            put(routes::accounts::update).delete(routes::accounts::delete),
        )
        .route(
            "/workspaces/{workspace}/accounts/{account}/balance",
            get(routes::accounts::balance),
        )
        // Entries
        .route(
            "/workspaces/{workspace}/entries",
            get(routes::entries::list).post(routes::entries::create),
        )
        .route(
            "/workspaces/{workspace}/entries/merge",
            post(routes::entries::merge),
        )
        .route(
            "/workspaces/{workspace}/entries/{entry}",
            get(routes::entries::get)
                .put(routes::entries::update)
                .delete(routes::entries::delete),
        )
        // Reports (legacy query-param style)
        .route(
            "/workspaces/{workspace}/reports/balance-sheet",
            get(routes::reports::balance_sheet),
        )
        .route(
            "/workspaces/{workspace}/reports/income-statement",
            get(routes::reports::income_statement),
        )
        // Period-based reports
        .route(
            "/workspaces/{workspace}/{year}/{period}/income-statement",
            get(routes::period_reports::income_statement),
        )
        .route(
            "/workspaces/{workspace}/{year}/{period}/balance-sheet",
            get(routes::period_reports::balance_sheet),
        )
        .route(
            "/workspaces/{workspace}/{year}/{period}/entries",
            get(routes::period_reports::entries),
        )
        // Baselines
        .route(
            "/workspaces/{workspace}/baselines",
            get(routes::baselines::list).post(routes::baselines::create),
        )
        .route(
            "/workspaces/{workspace}/baselines/{baseline}",
            delete(routes::baselines::delete),
        )
        .route(
            "/workspaces/{workspace}/baselines/{baseline}/variance",
            get(routes::baselines::variance),
        )
        .layer(Extension(pool))
        .layer(CorsLayer::permissive())
}
