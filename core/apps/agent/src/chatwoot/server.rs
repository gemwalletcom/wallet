use std::net::SocketAddr;

use crate::Result;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use gem_tracing::tracing::{error, info};
use serde_json::Value;

use crate::AppState;
use crate::chatwoot::dispatch;

pub async fn run_forever(state: AppState) -> Result<()> {
    let webhook = &state.settings.chatwoot.bot.webhook;
    let port = webhook.port;
    let path = webhook.path.clone();

    let app = Router::new().route("/health", get(|| async { "ok" })).route(&path, post(handle_webhook)).with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("binding chatwoot webhook listener on {addr}: {e}"))?;
    info!(port, path = %path, "chatwoot webhook listening");
    axum::serve(listener, app).await.map_err(|e| format!("chatwoot webhook server exited: {e}").into())
}

async fn handle_webhook(State(state): State<AppState>, body: String) -> impl IntoResponse {
    let payload: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            error!(error = %e, "chatwoot webhook: non-json body");
            return (StatusCode::BAD_REQUEST, "invalid json");
        }
    };
    tokio::spawn(async move {
        if let Err(e) = dispatch::handle_event(state, payload).await {
            error!(error = %e, "chatwoot dispatch failed");
        }
    });
    (StatusCode::OK, "ok")
}
