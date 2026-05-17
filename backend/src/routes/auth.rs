use axum::{extract::Extension, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth_backend::{AuthBackend, Credentials};
use crate::domain::user::{NewUser, User};
use crate::error::AppError;
use crate::jsend::JSend;
use crate::DbPool;

type AuthSession = axum_login::AuthSession<AuthBackend>;

#[derive(Deserialize)]
pub struct RegisterRequest {
    email: String,
    display_name: String,
    password: String,
}

pub async fn register(
    Extension(db): Extension<DbPool>,
    mut auth_session: AuthSession,
    Json(body): Json<RegisterRequest>,
) -> Result<JSend<Value>, AppError> {
    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;
    let user = NewUser::new(&body.email, &body.display_name, &body.password)?.insert(&mut conn)?;
    drop(conn);

    login_session(&mut auth_session, &user).await?;
    Ok(JSend::success(user_payload(&user)))
}

pub async fn login(
    mut auth_session: AuthSession,
    Json(creds): Json<Credentials>,
) -> Result<JSend<Value>, AppError> {
    let user = match auth_session.authenticate(creds).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(AppError::InvalidCredentials),
        Err(axum_login::Error::Backend(err)) => return Err(err),
        Err(axum_login::Error::Session(err)) => {
            return Err(AppError::Internal(format!("session: {err}")))
        }
    };
    login_session(&mut auth_session, &user).await?;
    Ok(JSend::success(user_payload(&user)))
}

pub async fn logout(mut auth_session: AuthSession) -> Result<JSend<Value>, AppError> {
    auth_session
        .logout()
        .await
        .map_err(|err| AppError::Internal(format!("logout: {err:?}")))?;
    Ok(JSend::success(Value::Null))
}

pub async fn me(auth_session: AuthSession) -> Result<JSend<Value>, AppError> {
    let user = auth_session.user.as_ref().ok_or(AppError::Unauthorized)?;
    Ok(JSend::success(user_payload(user)))
}

async fn login_session(session: &mut AuthSession, user: &User) -> Result<(), AppError> {
    session
        .login(user)
        .await
        .map_err(|err| AppError::Internal(format!("login: {err:?}")))
}

fn user_payload(user: &User) -> Value {
    json!({
        "user": {
            "id": user.id,
            "email": user.email,
            "display_name": user.display_name,
        }
    })
}
