use axum::{extract::Extension, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::domain::session::{NewSession, Session, SessionToken};
use crate::domain::user::{NewUser, User};
use crate::error::AppError;
use crate::jsend::JSend;
use crate::DbPool;

#[derive(Deserialize)]
pub struct RegisterRequest {
    email: String,
    display_name: String,
    password: String,
}

pub async fn register(
    Extension(db): Extension<DbPool>,
    Json(body): Json<RegisterRequest>,
) -> Result<JSend<Value>, AppError> {
    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    let user = NewUser::new(&body.email, &body.display_name, &body.password)?.insert(&mut conn)?;
    let session = NewSession::create(&user, &mut conn)?;

    Ok(JSend::success(session_payload(&user, &session)))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

pub async fn login(
    Extension(db): Extension<DbPool>,
    Json(body): Json<LoginRequest>,
) -> Result<JSend<Value>, AppError> {
    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    let user = User::from_email(&body.email, &mut conn)?.ok_or(AppError::Unauthorized)?;
    if !user.verify_password(&body.password) {
        return Err(AppError::Unauthorized);
    }

    let session = NewSession::create(&user, &mut conn)?;

    Ok(JSend::success(session_payload(&user, &session)))
}

pub async fn logout(
    Extension(db): Extension<DbPool>,
    headers: axum::http::HeaderMap,
) -> Result<JSend<Value>, AppError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    Session::invalidate(token, &mut conn)?;

    Ok(JSend::success(Value::Null))
}

fn session_payload(user: &User, session: &SessionToken) -> Value {
    json!({
        "user": {
            "id": user.id,
            "email": user.email,
            "display_name": user.display_name,
        },
        "token": session.token,
        "expires_at": session.expires_at.to_string(),
    })
}
