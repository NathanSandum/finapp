use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::Value;

/// JSend response envelope. https://github.com/omniti-labs/jsend
///
/// - `Success` — request succeeded; `data` carries the payload (may be `null`).
/// - `Fail` — request rejected due to client input (validation, missing field, etc.);
///   `data` describes which fields failed.
/// - `Error` — server-side failure; `message` is required, `code` and `data` optional.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum JSend<T: Serialize = Value> {
    Success {
        data: T,
    },
    Fail {
        data: Value,
    },
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<u16>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<Value>,
    },
}

impl<T: Serialize> JSend<T> {
    pub fn success(data: T) -> Self {
        JSend::Success { data }
    }
}

impl JSend {
    pub fn fail(data: Value) -> Self {
        JSend::Fail { data }
    }

    pub fn error(message: impl Into<String>) -> Self {
        JSend::Error {
            message: message.into(),
            code: None,
            data: None,
        }
    }

    pub fn with_code(mut self, code_value: u16) -> Self {
        if let JSend::Error { code, .. } = &mut self {
            *code = Some(code_value);
        }
        self
    }

    pub fn with_data(mut self, data_value: Value) -> Self {
        if let JSend::Error { data, .. } = &mut self {
            *data = Some(data_value);
        }
        self
    }
}

/// Pair an HTTP status with a JSend body. Use when wrapping in a route handler that
/// must return a non-200 status (e.g. 201 Created with a `Success` body, 422 with a
/// `Fail` body, 500 with an `Error` body).
pub struct JSendResponse<T: Serialize = Value> {
    pub status: StatusCode,
    pub body: JSend<T>,
}

impl<T: Serialize> JSendResponse<T> {
    pub fn new(status: StatusCode, body: JSend<T>) -> Self {
        Self { status, body }
    }
}

impl<T: Serialize> IntoResponse for JSendResponse<T> {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

/// Default `IntoResponse` for a bare `JSend`: 200 for success, 400 for fail, 500 for error.
/// Use `JSendResponse` when a different status code is needed.
impl<T: Serialize> IntoResponse for JSend<T> {
    fn into_response(self) -> Response {
        let status = match &self {
            JSend::Success { .. } => StatusCode::OK,
            JSend::Fail { .. } => StatusCode::BAD_REQUEST,
            JSend::Error { code, .. } => code
                .and_then(|c| StatusCode::from_u16(c).ok())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        };
        (status, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use serde::Serialize;
    use serde_json::json;

    async fn body_json<R: IntoResponse>(resp: R) -> serde_json::Value {
        let bytes = resp
            .into_response()
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[derive(Serialize)]
    struct User {
        id: u32,
        name: &'static str,
    }

    #[test]
    fn success_serializes_status_and_data() {
        let envelope = JSend::success(json!({ "id": 1 }));
        let value = serde_json::to_value(envelope).unwrap();
        assert_eq!(value, json!({ "status": "success", "data": { "id": 1 } }));
    }

    #[test]
    fn success_with_typed_payload() {
        let envelope = JSend::success(User { id: 7, name: "x" });
        let value = serde_json::to_value(envelope).unwrap();
        assert_eq!(
            value,
            json!({ "status": "success", "data": { "id": 7, "name": "x" } })
        );
    }

    #[test]
    fn success_data_can_be_null() {
        let envelope: JSend = JSend::success(Value::Null);
        let value = serde_json::to_value(envelope).unwrap();
        assert_eq!(value, json!({ "status": "success", "data": null }));
    }

    #[test]
    fn fail_serializes_status_and_data() {
        let envelope = JSend::fail(json!({ "email": "is required" }));
        let value = serde_json::to_value(envelope).unwrap();
        assert_eq!(
            value,
            json!({ "status": "fail", "data": { "email": "is required" } })
        );
    }

    #[test]
    fn error_serializes_minimal_shape() {
        let envelope = JSend::error("database unreachable");
        let value = serde_json::to_value(envelope).unwrap();
        assert_eq!(
            value,
            json!({ "status": "error", "message": "database unreachable" })
        );
    }

    #[test]
    fn error_with_code_and_data() {
        let envelope = JSend::error("db timeout")
            .with_code(503)
            .with_data(json!({ "retry_after": 5 }));
        let value = serde_json::to_value(envelope).unwrap();
        assert_eq!(
            value,
            json!({
                "status": "error",
                "message": "db timeout",
                "code": 503,
                "data": { "retry_after": 5 }
            })
        );
    }

    #[test]
    fn error_omits_optional_fields_when_unset() {
        let envelope = JSend::error("boom");
        let value = serde_json::to_value(envelope).unwrap();
        let obj = value.as_object().unwrap();
        assert!(!obj.contains_key("code"));
        assert!(!obj.contains_key("data"));
    }

    #[tokio::test]
    async fn into_response_success_is_200() {
        let envelope = JSend::success(json!({ "ok": true }));
        let resp = envelope.into_response();
        assert_eq!(resp.status().as_u16(), 200);
    }

    #[tokio::test]
    async fn into_response_fail_is_400() {
        let envelope = JSend::fail(json!({ "field": "bad" }));
        let resp = envelope.into_response();
        assert_eq!(resp.status().as_u16(), 400);
    }

    #[tokio::test]
    async fn into_response_error_is_500_by_default() {
        let envelope: JSend = JSend::error("boom");
        let resp = envelope.into_response();
        assert_eq!(resp.status().as_u16(), 500);
    }

    #[tokio::test]
    async fn into_response_error_uses_code_when_set() {
        let envelope: JSend = JSend::error("nope").with_code(503);
        let resp = envelope.into_response();
        assert_eq!(resp.status().as_u16(), 503);
    }

    #[tokio::test]
    async fn jsend_response_overrides_status() {
        let envelope = JSend::success(json!({ "id": "abc" }));
        let resp = JSendResponse::new(StatusCode::CREATED, envelope).into_response();
        assert_eq!(resp.status().as_u16(), 201);
        let body = body_json(JSendResponse::new(
            StatusCode::CREATED,
            JSend::success(json!({ "id": "abc" })),
        ))
        .await;
        assert_eq!(body, json!({ "status": "success", "data": { "id": "abc" } }));
    }
}
