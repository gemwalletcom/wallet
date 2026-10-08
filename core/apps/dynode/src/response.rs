use axum::body::Body;
use axum::response::{IntoResponse, Response};
use http::StatusCode;

use crate::proxy::ProxyResponse;

impl IntoResponse for ProxyResponse {
    fn into_response(self) -> Response {
        let ProxyResponse { status, headers, body, .. } = self;
        let mut response = Response::new(Body::from(body));
        *response.status_mut() = upstream_status(status);
        *response.headers_mut() = headers;
        response
    }
}

pub(crate) fn upstream_status(status: u16) -> StatusCode {
    StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY)
}
