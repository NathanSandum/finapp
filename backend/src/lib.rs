mod auth;
pub mod auth_backend;
pub mod config;
pub mod database;
pub mod domain;
mod error;
pub mod jsend;
mod permissions;
mod routes;
mod schema;
pub mod seed;
pub mod session_store;
pub mod test_helpers;

use axum::{
    extract::Extension,
    routing::{delete, get, post, put},
    Router,
};
use axum_login::tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer};
use axum_login::AuthManagerLayerBuilder;
use diesel::r2d2::{self, ConnectionManager};
use diesel::{RunQueryDsl, SqliteConnection};
use serde_json::{json, Value};
use time::Duration;
use tower::ServiceBuilder;
use tower_http::request_id::MakeRequestUuid;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tower_http::ServiceBuilderExt;

use crate::auth_backend::AuthBackend;
use crate::config::Mode;
use crate::jsend::JSend;
use crate::session_store::DbSessionStore;

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

async fn health(Extension(db): Extension<DbPool>) -> JSend<Value> {
    let db_status = match db.get() {
        Ok(mut conn) => match diesel::sql_query("SELECT 1").execute(&mut conn) {
            Ok(_) => "ok",
            Err(_) => "error",
        },
        Err(_) => "error",
    };
    JSend::success(json!({
        "service": "finapp",
        "version": env!("CARGO_PKG_VERSION"),
        "db": db_status,
    }))
}

pub fn build_router(db: DbPool, mode: Mode) -> Router {
    let session_store = DbSessionStore::new(db.clone());
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(mode == Mode::Prod)
        .with_same_site(SameSite::Strict)
        .with_expiry(Expiry::OnInactivity(Duration::weeks(1)));
    let auth_backend = AuthBackend::new(db.clone());
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer).build();

    let router = Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/logout", post(routes::auth::logout))
        .route("/auth/me", get(routes::auth::me))
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
        .route(
            "/workspaces/{workspace}/anchors",
            get(routes::accounts::anchors),
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
        // Recurring rules
        .route(
            "/workspaces/{workspace}/recurring",
            get(routes::recurring::list).post(routes::recurring::create),
        )
        .route(
            "/workspaces/{workspace}/recurring/{rule}",
            get(routes::recurring::get).delete(routes::recurring::delete),
        )
        .route(
            "/workspaces/{workspace}/recurring/runs",
            post(routes::recurring::create_run),
        );

    let observability = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true)),
        )
        .propagate_x_request_id();

    router
        .layer(auth_layer)
        .layer(Extension(db))
        .layer(observability)
}
