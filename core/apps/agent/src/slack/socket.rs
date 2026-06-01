use std::time::Duration;

use crate::Result;
use futures_util::{SinkExt, StreamExt};
use gem_tracing::tracing::{debug, error, info, warn};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::AppState;
use crate::slack::dispatch;

const APPS_CONNECTIONS_OPEN: &str = "https://slack.com/api/apps.connections.open";

#[derive(Debug, Deserialize)]
struct OpenResponse {
    ok: bool,
    url: Option<String>,
    error: Option<String>,
}

pub async fn run_forever(state: AppState) -> Result<()> {
    let mut backoff = Duration::from_secs(1);
    loop {
        match connect_once(&state).await {
            Ok(()) => {
                info!("socket closed cleanly, reconnecting");
                backoff = Duration::from_secs(1);
            }
            Err(e) => {
                error!(error = %e, backoff_secs = backoff.as_secs(), "socket failed; backing off");
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(30));
            }
        }
    }
}

async fn connect_once(state: &AppState) -> Result<()> {
    let url = open_socket(&state.settings.slack.app.token).await.map_err(|e| format!("apps.connections.open: {e}"))?;
    let (ws, _) = connect_async(&url).await.map_err(|e| format!("websocket connect: {e}"))?;
    let (mut tx, mut rx) = ws.split();
    info!("socket mode connected");

    while let Some(msg) = rx.next().await {
        match msg.map_err(|e| format!("ws recv: {e}"))? {
            Message::Text(text) => {
                let v: Value = match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(error = %e, "non-json frame");
                        continue;
                    }
                };
                let env_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
                let envelope_id = v.get("envelope_id").and_then(|e| e.as_str()).map(String::from);
                debug!(env_type, ?envelope_id, "envelope");

                match env_type {
                    "hello" => continue,
                    "disconnect" => {
                        let reason = v.get("reason").and_then(|r| r.as_str()).unwrap_or("");
                        info!(reason, "slack asked us to reconnect");
                        return Ok(());
                    }
                    "events_api" => {
                        if let Some(eid) = envelope_id {
                            tx.send(Message::Text(json!({"envelope_id": eid}).to_string().into())).await?;
                        }
                        let payload = v.get("payload").cloned().unwrap_or(Value::Null);
                        let state = state.clone();
                        tokio::spawn(async move {
                            if let Err(e) = dispatch::handle_event(state, payload).await {
                                error!(error = %e, "event handler failed");
                            }
                        });
                    }
                    other => warn!(other, "unhandled envelope type"),
                }
            }
            Message::Ping(p) => tx.send(Message::Pong(p)).await?,
            Message::Close(_) => {
                info!("slack closed the socket");
                return Ok(());
            }
            _ => {}
        }
    }
    Ok(())
}

async fn open_socket(app_token: &str) -> Result<String> {
    let resp: OpenResponse = reqwest::Client::new().post(APPS_CONNECTIONS_OPEN).bearer_auth(app_token).send().await?.json().await?;
    if !resp.ok {
        return Err(format!("apps.connections.open: {}", resp.error.unwrap_or_default()).into());
    }
    resp.url.ok_or_else(|| "no url in apps.connections.open response".into())
}
