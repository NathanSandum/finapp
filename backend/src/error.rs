use crate::jsend::{JSend, JSendResponse};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Serialize)]
struct Message {
    message: String,
}

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Forbidden,
    Unauthorized,
    InvalidCredentials,
    Conflict(String),
    Validation(Vec<FieldError>),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "not found: {msg}"),
            AppError::Forbidden => write!(f, "forbidden"),
            AppError::Unauthorized => write!(f, "unauthorized"),
            AppError::InvalidCredentials => write!(f, "invalid credentials"),
            AppError::Conflict(msg) => write!(f, "conflict: {msg}"),
            AppError::Validation(_) => write!(f, "validation failed"),
            AppError::Internal(msg) => write!(f, "internal: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<DieselError> for AppError {
    fn from(err: DieselError) -> Self {
        match err {
            DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                AppError::Conflict("unique constraint violated".into())
            }
            DieselError::NotFound => AppError::NotFound("record not found".into()),
            other => AppError::Internal(format!("diesel: {other}")),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound(msg) => fail(StatusCode::NOT_FOUND, Message { message: msg }),
            AppError::Forbidden => fail(
                StatusCode::FORBIDDEN,
                Message {
                    message: "Insufficient permissions".into(),
                },
            ),
            AppError::Unauthorized => fail(
                StatusCode::UNAUTHORIZED,
                Message {
                    message: "Authentication required".into(),
                },
            ),
            AppError::InvalidCredentials => fail(
                StatusCode::UNAUTHORIZED,
                Message {
                    message: "Invalid email or password".into(),
                },
            ),
            AppError::Conflict(msg) => fail(StatusCode::CONFLICT, Message { message: msg }),
            AppError::Validation(fields) => {
                let mut data = Map::new();
                for FieldError { field, message } in fields {
                    data.insert(field, Value::String(message));
                }
                fail(StatusCode::UNPROCESSABLE_ENTITY, Value::Object(data))
            }
            AppError::Internal(_) => JSendResponse::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                JSend::error("an unexpected error occurred"),
            )
            .into_response(),
        }
    }
}

fn fail<D: Serialize>(status: StatusCode, data: D) -> Response {
    JSendResponse::new(status, JSend::fail(data)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;

    async fn body_json(err: AppError) -> serde_json::Value {
        let resp = err.into_response();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    async fn status(err: AppError) -> u16 {
        err.into_response().status().as_u16()
    }

    #[tokio::test]
    async fn not_found_is_404() {
        assert_eq!(status(AppError::NotFound("thing".into())).await, 404);
    }

    #[tokio::test]
    async fn forbidden_is_403() {
        assert_eq!(status(AppError::Forbidden).await, 403);
    }

    #[tokio::test]
    async fn unauthorized_is_401() {
        assert_eq!(status(AppError::Unauthorized).await, 401);
    }

    #[tokio::test]
    async fn conflict_is_409() {
        assert_eq!(status(AppError::Conflict("duplicate".into())).await, 409);
    }

    #[tokio::test]
    async fn internal_is_500() {
        assert_eq!(status(AppError::Internal("test".into())).await, 500);
    }

    #[tokio::test]
    async fn client_error_is_jsend_fail() {
        let body = body_json(AppError::NotFound("workspace".into())).await;
        assert_eq!(body["status"], "fail");
        assert_eq!(body["data"]["message"], "workspace");
    }

    #[tokio::test]
    async fn server_error_is_jsend_error() {
        let body = body_json(AppError::Internal("secret".into())).await;
        assert_eq!(body["status"], "error");
        assert!(body["message"].is_string());
    }

    #[tokio::test]
    async fn internal_does_not_leak_details() {
        let body = body_json(AppError::Internal("secret database password".into())).await;
        let msg = body["message"].as_str().unwrap();
        assert!(!msg.contains("secret"), "internal error leaks details");
        assert!(!msg.contains("database"), "internal error leaks details");
    }

    #[tokio::test]
    async fn validation_is_422_with_field_map() {
        let err = AppError::Validation(vec![
            FieldError {
                field: "email".into(),
                message: "is required".into(),
            },
            FieldError {
                field: "password".into(),
                message: "too short".into(),
            },
        ]);
        let resp = err.into_response();
        assert_eq!(resp.status().as_u16(), 422);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["status"], "fail");
        assert_eq!(body["data"]["email"], "is required");
        assert_eq!(body["data"]["password"], "too short");
    }
}
