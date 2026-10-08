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

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::routing::get;
    use http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    async fn slow() -> StatusCode {
        tokio::time::sleep(Duration::from_millis(200)).await;
        StatusCode::OK
    }

    async fn panicking() -> StatusCode {
        panic!("boom")
    }

    fn router() -> Router {
        with_security_headers(
            Router::new()
                .route("/slow", get(slow))
                .route("/panic", get(panicking))
                .layer(timeout_layer(Duration::from_millis(20)))
                .layer(catch_panic_layer()),
        )
    }

    #[tokio::test]
    async fn test_timeout_answers_504_with_security_headers() {
        let response = router().oneshot(Request::get("/slow").body(Body::empty()).unwrap()).await.unwrap();

        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(response.headers().get(header::X_CONTENT_TYPE_OPTIONS).unwrap(), "nosniff");
        assert_eq!(response.headers().get(header::X_FRAME_OPTIONS).unwrap(), "SAMEORIGIN");
    }

    #[tokio::test]
    async fn test_panic_answers_json_500() {
        let response = router().oneshot(Request::get("/panic").body(Body::empty()).unwrap()).await.unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.headers().get(header::CONTENT_TYPE).unwrap(), "application/json");
        assert_eq!(response.into_body().collect().await.unwrap().to_bytes().as_ref(), br#"{"error":{"message":"Internal server error"}}"#);
    }
}
