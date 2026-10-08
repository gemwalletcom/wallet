mod client;
mod observer;
mod price_handler;
mod redis;

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::extract::{FromRef, State};
use axum::middleware::from_fn_with_state;
use axum::response::Response;
use axum::routing::get;
use gem_tracing::info_with_fields;
use http::StatusCode;
use http_server::{ShutdownReceiver, catch_panic_layer, with_security_headers};
use services::devices::{DeviceStreamClient, DevicesClient};
use services::prices::PriceClient;
use tokio_util::task::TaskTracker;

use crate::auth::device::{AuthenticatedDevice, DeviceAuth, DeviceAuthConfig, SignedPath, device_auth};
use crate::error::ApiError;
use crate::routes::{json_error_net, log_failed_requests, not_found};

const STREAM_BODY_LIMIT: usize = 1024 * 1024;
const MAX_MESSAGE_BYTES: usize = 64 * 1024;
const MAX_WRITE_BUFFER_BYTES: usize = 1024 * 1024;
pub const PING_INTERVAL: Duration = Duration::from_secs(30);
pub const MISSED_PONGS_LIMIT: u32 = 2;

pub struct StreamObserverConfig {
    pub redis_url: String,
    pub device_stream: DeviceStreamClient,
}

#[derive(Clone)]
pub struct StreamState {
    pub auth_config: Arc<DeviceAuthConfig>,
    pub devices: Arc<DevicesClient>,
    pub prices: Arc<PriceClient>,
    pub observer: Arc<StreamObserverConfig>,
    pub shutdown: ShutdownReceiver,
    pub connections: TaskTracker,
}

impl FromRef<StreamState> for Arc<DevicesClient> {
    fn from_ref(state: &StreamState) -> Self {
        state.devices.clone()
    }
}

fn authenticated(state: &StreamState, scheme: SignedPath) -> Router<StreamState> {
    let auth = DeviceAuth {
        config: state.auth_config.clone(),
        limit: STREAM_BODY_LIMIT,
        scheme,
        replay: None,
    };
    Router::new().route("/stream", get(stream)).layer(from_fn_with_state(auth, device_auth))
}

pub fn router(state: StreamState) -> Router {
    let router = Router::new()
        .route("/health", get(health))
        .nest("/v2/devices", authenticated(&state, SignedPath::PathOnly))
        .nest("/v3/devices", authenticated(&state, SignedPath::PathAndQuery))
        .fallback(not_found)
        .method_not_allowed_fallback(not_found)
        .with_state(state)
        .layer(catch_panic_layer())
        .layer(axum::middleware::map_response(json_error_net))
        .layer(axum::middleware::from_fn(log_failed_requests));
    with_security_headers(router)
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn stream(device: AuthenticatedDevice, State(state): State<StreamState>, upgrade: WebSocketUpgrade) -> Result<Response, ApiError> {
    let version = device.version()?;
    let device_id = device.record.device.id;
    let connections = state.connections.clone();
    Ok(upgrade
        .max_message_size(MAX_MESSAGE_BYTES)
        .max_frame_size(MAX_MESSAGE_BYTES)
        .max_write_buffer_size(MAX_WRITE_BUFFER_BYTES)
        .on_failed_upgrade(|error: axum::Error| info_with_fields!("websocket upgrade failed", error = error.to_string()))
        .on_upgrade(move |socket: WebSocket| {
            connections.track_future(async move {
                let mut observer = observer::StreamObserver::new(device_id, version, state.prices.clone());
                client::run(&state.observer.redis_url, &state.observer.device_stream, &mut observer, socket, state.shutdown.clone()).await;
            })
        }))
}
