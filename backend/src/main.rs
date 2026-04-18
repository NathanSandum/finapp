mod auth;
mod domain;
mod error;
mod schema;
mod test_helpers;

use axum::{
    extract::Extension,
    routing::{get, post},
    Json, Router,
};
use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

use crate::domain::session::{NewSession, Session};
use crate::domain::user::{NewUser, User};
use crate::error::AppError;

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[tokio::main]
async fn main() {
    let manager = ConnectionManager::<SqliteConnection>::new("finapp.db");
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("failed to create db pool");

    let app = Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
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

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    display_name: String,
    password: String,
}

async fn register(
    Extension(pool): Extension<DbPool>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let user = NewUser::new(&body.email, &body.display_name, &body.password)?
        .insert(&mut conn)?;

    let session = NewSession::create(&user, &mut conn)?;

    Ok(Json(json!({
        "user": {
            "id": user.id,
            "email": user.email,
            "display_name": user.display_name,
        },
        "token": session.token,
        "expires_at": session.expires_at.to_string(),
    })))
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

async fn login(
    Extension(pool): Extension<DbPool>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<Value>, AppError> {
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    let user = User::from_email(&body.email, &mut conn)?
        .ok_or(AppError::Unauthorized)?;

    if !user.verify_password(&body.password) {
        return Err(AppError::Unauthorized);
    }

    let session = NewSession::create(&user, &mut conn)?;

    Ok(Json(json!({
        "user": {
            "id": user.id,
            "email": user.email,
            "display_name": user.display_name,
        },
        "token": session.token,
        "expires_at": session.expires_at.to_string(),
    })))
}

async fn logout(
    Extension(pool): Extension<DbPool>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, AppError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    Session::invalidate(token, &mut conn)?;

    Ok(Json(json!({ "status": "ok" })))
}
