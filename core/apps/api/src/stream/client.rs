use std::error::Error;

use axum::extract::ws::{CloseFrame, Message, WebSocket, close_code};
use futures::StreamExt;
use gem_tracing::{error_fields, info_with_fields};
use http_server::ShutdownReceiver;
use primitives::{StreamEvent, WebSocketPricePayload};
use redis::PushKind;
use services::devices::DeviceStreamClient;
use tokio_tungstenite::tungstenite::Error as WsError;

use super::observer::StreamObserver;
use super::redis::connect;
use super::{MISSED_PONGS_LIMIT, PING_INTERVAL};

pub async fn run(redis_url: &str, device_stream: &DeviceStreamClient, observer: &mut StreamObserver, mut socket: WebSocket, mut shutdown: ShutdownReceiver) {
    let Ok((mut redis_connection, mut rx)) = connect(redis_url).await else {
        error_fields!("websocket failed to setup redis connection");
        return;
    };
    info_with_fields!("websocket device stream connected", status = "ok");

    if let Err(error) = observer.subscribe_device_channel(&mut redis_connection).await {
        error_fields!("websocket failed to subscribe device channel", message = format!("{error:?}"));
        return;
    }
    if let Err(error) = flush_device_stream_events(observer, device_stream, &mut socket).await {
        error_fields!("websocket failed to flush device stream events", message = format!("{error:?}"));
        return;
    }

    let mut ping_interval = tokio::time::interval(PING_INTERVAL);
    ping_interval.tick().await;
    let mut unanswered_pings = 0;
    loop {
        tokio::select! {
            biased;
            _ = shutdown.changed() => {
                let _ = socket.send(Message::Close(Some(CloseFrame { code: close_code::AWAY, reason: "server shutting down".into() }))).await;
                break;
            }
            _ = ping_interval.tick() => {
                if unanswered_pings >= MISSED_PONGS_LIMIT {
                    info_with_fields!("websocket client stopped answering pings", device_id = observer.device_id());
                    break;
                }
                unanswered_pings += 1;
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break;
                }
            }
            _ = observer.next_price_interval() => {
                let prices = observer.take_prices();
                if prices.is_empty() {
                    continue;
                }

                let payload = WebSocketPricePayload { prices, rates: vec![] };
                match observer.send_event(&mut socket, StreamEvent::Prices(payload)).await {
                    Ok(_) => {
                        info_with_fields!("websocket tick notified prices", status = "ok");
                    }
                    Err(error) => {
                        error_fields!("websocket send error on tick", message = format!("{error:?}"));
                        break;
                    }
                }
            }
            message = rx.recv() => {
                let Some(message) = message else {
                    error_fields!("websocket redis push channel closed");
                    break;
                };
                if message.kind == PushKind::Disconnection {
                    error_fields!("websocket redis connection lost");
                    break;
                }
                match observer.receive_redis_message(&message) {
                    Ok(Some(event)) => {
                        if let Err(error) = observer.send_event(&mut socket, event).await {
                            error_fields!("websocket send event error", message = format!("{error:?}"));
                            break;
                        }
                    }
                    Ok(None) => { }
                    Err(error) => {
                        error_fields!("websocket redis message handler error", message = format!("{error:?}"));
                    }
                }
            }
            message = socket.next() => {
                match message {
                    Some(Ok(message)) => {
                        if matches!(message, Message::Pong(_)) {
                            unanswered_pings = 0;
                        }
                        if let Err(error) = observer.respond_to_ws_message(message, &mut redis_connection, &mut socket).await {
                            error_fields!("websocket message handler error", message = format!("{error:?}"));
                            break;
                        }
                    }
                    Some(Err(error)) => {
                        if !is_disconnect_error(&error) {
                            error_fields!("websocket stream error", message = format!("{error:?}"));
                        }
                        break;
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }
    info_with_fields!("websocket device stream disconnected", status = "ok");
}

fn is_disconnect_error(error: &axum::Error) -> bool {
    let mut source: Option<&(dyn Error + 'static)> = Some(error);
    while let Some(current) = source {
        if let Some(error) = current.downcast_ref::<WsError>() {
            return matches!(error, WsError::Protocol(_) | WsError::ConnectionClosed | WsError::AlreadyClosed);
        }
        source = current.source();
    }
    false
}

async fn flush_device_stream_events(observer: &StreamObserver, device_stream: &DeviceStreamClient, socket: &mut WebSocket) -> Result<(), Box<dyn Error + Send + Sync>> {
    let pending_events = device_stream.take_pending_events(observer.device_id()).await?;
    for (index, pending_event) in pending_events.iter().enumerate() {
        if let Err(error) = observer.send_event(socket, pending_event.event.clone()).await {
            device_stream.restore_events(observer.device_id(), &pending_events[index..]).await?;
            return Err(error);
        }
    }
    Ok(())
}
