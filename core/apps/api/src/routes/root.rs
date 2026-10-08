use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use http::StatusCode;
use http::header::CONTENT_TYPE;
use primitives::unix_milliseconds;
use serde::Serialize;

use crate::metrics::Metrics;
use crate::request::ClientIp;
use crate::response::ApiResponse;
use crate::state::AppState;

#[derive(Serialize)]
pub struct Status {
    time: u64,
    ipv4: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_status)).route("/health", get(get_health)).route("/metrics", get(get_metrics))
}

async fn get_status(ClientIp(ip): ClientIp) -> ApiResponse<Status> {
    Status {
        time: unix_milliseconds().unwrap_or_default(),
        ipv4: ip.to_string(),
    }
    .into()
}

async fn get_health() -> StatusCode {
    StatusCode::OK
}

async fn get_metrics(State(metrics): State<Arc<Metrics>>) -> Response {
    ([(CONTENT_TYPE, "text/plain; charset=utf-8")], metrics.encode()).into_response()
}
