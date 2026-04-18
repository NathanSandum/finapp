use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Forbidden,
    Unauthorized,
    Conflict(String),
    Validation(Vec<FieldError>),
    Internal,
}

#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<FieldError>>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                ErrorBody { code: "not_found", message: msg, fields: None },
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                ErrorBody { code: "forbidden", message: "insufficient permissions".into(), fields: None },
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                ErrorBody { code: "unauthorized", message: "authentication required".into(), fields: None },
            ),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT,
                ErrorBody { code: "conflict", message: msg, fields: None },
            ),
            AppError::Validation(fields) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorBody { code: "validation_error", message: "validation failed".into(), fields: Some(fields) },
            ),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorBody { code: "internal_error", message: "an unexpected error occurred".into(), fields: None },
            ),
        };
        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;

    async fn body_json(err: AppError) -> serde_json::Value {
        let resp = err.into_response();
        let bytes = resp
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
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
        assert_eq!(status(AppError::Internal).await, 500);
    }

    #[tokio::test]
    async fn response_has_code_and_message() {
        let body = body_json(AppError::NotFound("workspace".into())).await;
        assert!(body.get("code").is_some(), "missing 'code' field");
        assert!(body.get("message").is_some(), "missing 'message' field");
        assert!(body.get("fields").is_none(), "unexpected 'fields' on non-validation error");
    }

    #[tokio::test]
    async fn internal_does_not_leak_details() {
        let body = body_json(AppError::Internal).await;
        let msg = body["message"].as_str().unwrap();
        assert!(!msg.contains("panic"), "internal error leaks details");
        assert!(!msg.contains("thread"), "internal error leaks details");
    }

    #[tokio::test]
    async fn validation_is_422_with_fields() {
        let err = AppError::Validation(vec![
            FieldError { field: "email".into(), message: "is required".into() },
            FieldError { field: "password".into(), message: "too short".into() },
        ]);
        let resp = err.into_response();
        assert_eq!(resp.status().as_u16(), 422);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["code"], "validation_error");
        let fields = body["fields"].as_array().unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0]["field"], "email");
        assert_eq!(fields[1]["field"], "password");
    }
}
