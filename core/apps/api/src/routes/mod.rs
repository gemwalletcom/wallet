pub mod admin;
pub mod devices;
pub mod fiat_rates;
pub mod public;
mod root;

use std::time::Duration;

use axum::extract::{DefaultBodyLimit, Request};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::{Router, body::Body};
use gem_tracing::{error_fields, info_with_fields};
use http::header::{CONTENT_TYPE, USER_AGENT};
use http::{Method, StatusCode};
use http_server::{ErrorBody, HttpMetrics, catch_panic_layer, timeout_layer, with_security_headers};

use crate::error::{ApiError, ErrorContext};
use crate::state::AppState;

const DEFAULT_BODY_LIMIT: usize = 16 * 1024 * 1024;
const WEBHOOK_PATH_PREFIX: &str = "/v1/webhooks/";
const WEBHOOK_SECRET_SEGMENT: usize = 5;

pub fn router(state: AppState, admin_enabled: bool, request_timeout: Duration, http_metrics: &HttpMetrics) -> Router {
    let mut router = Router::new()
        .merge(root::router())
        .nest("/v1", public::router())
        .nest("/v2/devices", devices::v2_router(&state))
        .nest("/v3/devices", devices::v3_router(&state));
    if admin_enabled {
        router = router.nest("/v1/admin", admin::router());
    }
    let router = router
        .fallback(not_found)
        .method_not_allowed_fallback(not_found)
        .with_state(state)
        .layer(timeout_layer(request_timeout))
        .layer(DefaultBodyLimit::max(DEFAULT_BODY_LIMIT))
        .layer(catch_panic_layer())
        .layer(middleware::map_response(json_error_net))
        .layer(middleware::from_fn(log_failed_requests))
        .layer(http_metrics.layer());
    with_security_headers(router)
}

pub async fn not_found() -> ApiError {
    ApiError::from_status(StatusCode::NOT_FOUND)
}

pub async fn json_error_net(response: Response) -> Response {
    let status = response.status();
    let is_json = response.headers().get(CONTENT_TYPE).and_then(|value| value.to_str().ok()).is_some_and(|value| value.starts_with("application/json"));
    if status.is_success() || status == StatusCode::SWITCHING_PROTOCOLS || status.is_redirection() || is_json {
        return response;
    }
    let (parts, _) = response.into_parts();
    let error = ErrorBody::from_status(status);
    let context = ErrorContext {
        message: error.message.clone(),
        detail: None,
    };
    let mut replaced = error.into_response();
    replaced.extensions_mut().extend(parts.extensions);
    replaced.extensions_mut().insert(context);
    replaced
}

pub async fn log_failed_requests(request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();
    let uri = redacted_uri(request.method(), request.uri().path(), request.uri().query());
    let user_agent = request.headers().get(USER_AGENT).and_then(|value| value.to_str().ok()).unwrap_or("unknown").to_string();
    let response = next.run(request).await;
    let status = response.status();
    if status.is_success() || status == StatusCode::SWITCHING_PROTOCOLS {
        return response;
    }
    let context = response.extensions().get::<ErrorContext>().cloned();
    if context.is_none() && !status.is_server_error() {
        return response;
    }
    let message = context
        .as_ref()
        .map(|context| context.message.clone())
        .unwrap_or_else(|| format!("{} {}", status.as_u16(), status.canonical_reason().unwrap_or_default()));
    match context.and_then(|context| context.detail) {
        Some(detail) => error_fields!("Request failed", method = method.as_str(), uri = uri, status = status.as_u16(), error = detail, user_agent = user_agent),
        None => info_with_fields!("Request failed", method = method.as_str(), uri = uri, status = status.as_u16(), error = message, user_agent = user_agent),
    }
    response
}

fn redacted_uri(method: &Method, path: &str, query: Option<&str>) -> String {
    let path = if method == Method::POST && path.starts_with(WEBHOOK_PATH_PREFIX) && path.trim_end_matches('/').matches('/').count() >= WEBHOOK_SECRET_SEGMENT {
        let mut segments: Vec<&str> = path.split('/').collect();
        if let Some(secret) = segments.get_mut(WEBHOOK_SECRET_SEGMENT) {
            *secret = "[redacted]";
        }
        segments.join("/")
    } else {
        path.to_string()
    };
    match query {
        Some(query) => format!("{path}?{query}"),
        None => path,
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::response::Response;
    use http::header::CONTENT_TYPE;
    use http::{Method, StatusCode};

    use super::{json_error_net, redacted_uri};
    use crate::error::ErrorContext;

    #[tokio::test]
    async fn test_a_bare_error_response_becomes_a_json_error_we_log() {
        let timed_out = Response::builder().status(StatusCode::GATEWAY_TIMEOUT).body(Body::empty()).unwrap();

        let response = json_error_net(timed_out).await;

        assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), "application/json");
        assert_eq!(response.extensions().get::<ErrorContext>().unwrap().message, "504 Gateway Timeout");
    }

    #[test]
    fn test_redacted_uri() {
        assert_eq!(redacted_uri(&Method::POST, "/v1/webhooks/fiat/moonpay/s3cret", None), "/v1/webhooks/fiat/moonpay/[redacted]");
        assert_eq!(redacted_uri(&Method::POST, "/v1/webhooks/fiat/moonpay", Some("a=1")), "/v1/webhooks/fiat/moonpay?a=1");
        assert_eq!(redacted_uri(&Method::GET, "/v1/assets/ethereum", Some("currency=USD")), "/v1/assets/ethereum?currency=USD");
    }
}
