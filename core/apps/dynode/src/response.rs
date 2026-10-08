use axum::body::Body;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use http_server::ErrorBody;

use crate::proxy::ProxyResponse;

impl IntoResponse for ProxyResponse {
    fn into_response(self) -> Response {
        let ProxyResponse { status, headers, body, .. } = self;
        let mut response = Response::new(Body::from(body));
        *response.status_mut() = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY);
        *response.headers_mut() = headers;
        response
    }
}

pub(crate) struct ProxyError {
    status: StatusCode,
    message: String,
}

impl ProxyError {
    pub(crate) fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self { status, message: message.into() }
    }

    pub(crate) fn with_code(status: u16, message: impl Into<String>) -> Self {
        Self::new(StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY), message)
    }
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        ErrorBody::new(self.status, self.message).into_response()
    }
}
