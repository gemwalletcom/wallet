use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use axum::Router;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use gem_tracing::error_fields;
use http::StatusCode;
use http_server::{ErrorBody, HttpMetrics, ServeConfig, ShutdownReceiver, serve};
use tokio::task::JoinHandle;

use crate::metrics::{self, MetricsProvider};

pub struct HealthState {
    ready: AtomicBool,
}

impl Default for HealthState {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthState {
    pub fn new() -> Self {
        Self { ready: AtomicBool::new(false) }
    }

    pub fn set_ready(&self) {
        self.ready.store(true, Ordering::Relaxed);
    }

    pub fn set_not_ready(&self) {
        self.ready.store(false, Ordering::Relaxed);
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }
}

#[derive(Clone)]
struct ServerState {
    health: Arc<HealthState>,
    provider: Arc<dyn MetricsProvider>,
    http_metrics: HttpMetrics,
}

async fn health(State(state): State<ServerState>) -> Response {
    if state.health.is_ready() {
        StatusCode::OK.into_response()
    } else {
        ErrorBody::from_status(StatusCode::SERVICE_UNAVAILABLE).into_response()
    }
}

async fn get_metrics(State(state): State<ServerState>) -> Response {
    metrics::encode(state.provider.as_ref(), &state.http_metrics).into_response()
}

pub fn router(health_state: Arc<HealthState>, provider: Arc<dyn MetricsProvider>) -> Router {
    let http_metrics = HttpMetrics::new();
    let state = ServerState {
        health: health_state,
        provider,
        http_metrics: http_metrics.clone(),
    };
    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(get_metrics))
        .fallback(async || ErrorBody::from_status(StatusCode::NOT_FOUND))
        .with_state(state)
        .layer(http_metrics.layer())
}

pub fn spawn_server(provider: Arc<dyn MetricsProvider>, bind: SocketAddr, config: ServeConfig, shutdown: ShutdownReceiver) -> (Arc<HealthState>, JoinHandle<()>) {
    let state = Arc::new(HealthState::new());
    let router = router(state.clone(), provider);
    let handle = tokio::spawn(async move {
        if let Err(error) = serve(router, bind, config, shutdown).await {
            error_fields!("health server failed", error = error.to_string());
        }
    });
    (state, handle)
}
