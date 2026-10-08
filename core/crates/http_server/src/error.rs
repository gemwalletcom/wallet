use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use primitives::ResponseResult;

pub struct ErrorBody {
    pub status: StatusCode,
    pub message: String,
}

impl ErrorBody {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self { status, message: message.into() }
    }

    pub fn from_status(status: StatusCode) -> Self {
        Self::new(status, status_message(status))
    }
}

pub fn status_message(status: StatusCode) -> String {
    format!("{} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown"))
}

impl IntoResponse for ErrorBody {
    fn into_response(self) -> Response {
        (self.status, Json(ResponseResult::<()>::error(self.message))).into_response()
    }
}
