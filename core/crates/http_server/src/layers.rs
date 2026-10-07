use std::any::Any;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::response::{IntoResponse, Response};
use gem_tracing::error_fields;
use http::{HeaderValue, StatusCode, header};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;

use crate::error::ErrorBody;

pub const INTERNAL_ERROR_MESSAGE: &str = "Internal server error";

pub fn timeout_layer(timeout: Duration) -> TimeoutLayer {
    TimeoutLayer::with_status_code(StatusCode::GATEWAY_TIMEOUT, timeout)
}

pub fn catch_panic_layer() -> CatchPanicLayer<fn(Box<dyn Any + Send + 'static>) -> Response<Body>> {
    CatchPanicLayer::custom(panic_response)
}

fn panic_response(error: Box<dyn Any + Send + 'static>) -> Response<Body> {
    let detail = error
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| error.downcast_ref::<&str>().map(ToString::to_string))
        .unwrap_or_else(|| "unknown panic".to_string());
    error_fields!("Request handler panicked", error = detail);
    ErrorBody::new(StatusCode::INTERNAL_SERVER_ERROR, INTERNAL_ERROR_MESSAGE).into_response()
}

pub fn with_security_headers<S: Clone + Send + Sync + 'static>(router: Router<S>) -> Router<S> {
    router
        .layer(SetResponseHeaderLayer::if_not_present(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")))
        .layer(SetResponseHeaderLayer::if_not_present(header::X_FRAME_OPTIONS, HeaderValue::from_static("SAMEORIGIN")))
        .layer(SetResponseHeaderLayer::if_not_present(header::HeaderName::from_static("permissions-policy"), HeaderValue::from_static("interest-cohort=()")))
}
