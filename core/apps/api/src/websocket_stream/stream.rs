use std::error::Error;

use gem_tracing::{error_fields, info_with_fields};
use primitives::{StreamEvent, WebSocketPricePayload};
use redis::PushKind;
use rocket::futures::StreamExt;
use rocket_ws::stream::DuplexStream;
use services::devices::DeviceStreamClient;

use super::client::StreamObserverClient;

pub async fn new_stream(redis_url: &str, device_stream: &DeviceStreamClient, observer: &mut StreamObserverClient, stream: DuplexStream) {
    let Ok((mut stream, mut redis_connection, mut rx)) = crate::websocket::setup_ws_resources(redis_url, stream).await else {
        error_fields!("websocket failed to setup redis connection");
        return;
    };
    info_with_fields!("websocket device stream connected", status = "ok");

    if let Err(e) = observer.subscribe_device_channel(&mut redis_connection).await {
        error_fields!("websocket failed to subscribe device channel", message = format!("{e:?}"));
        return;
    }
    if let Err(e) = flush_device_stream_events(observer, device_stream, &mut stream).await {
        error_fields!("websocket failed to flush device stream events", message = format!("{e:?}"));
        return;
    }

    loop {
        tokio::select! {
            biased;
            _ = observer.next_price_interval() => {
                let prices = observer.take_prices();
                if prices.is_empty() {
                    continue;
                }

                let payload = WebSocketPricePayload { prices, rates: vec![] };
                match observer.send_event(&mut stream, StreamEvent::Prices(payload)).await {
                    Ok(_) => {
                        info_with_fields!("websocket tick notified prices", status = "ok");
                    }
                    Err(e) => {
                        error_fields!("websocket send error on tick", message = format!("{e:?}"));
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
                match observer.handle_redis_message(&message) {
                    Ok(Some(event)) => {
                        if let Err(e) = observer.send_event(&mut stream, event).await {
                            error_fields!("websocket send event error", message = format!("{e:?}"));
                            break;
                        }
                    }
                    Ok(None) => { }
                    Err(e) => {
                        error_fields!("websocket redis message handler error", message = format!("{e:?}"));
                    }
                }
            }
            message = stream.next() => {
                match message {
                    Some(Ok(message)) => {
                        if let Err(e) = observer.handle_ws_message(message, &mut redis_connection, &mut stream).await {
                            error_fields!("websocket message handler error", message = format!("{e:?}"));
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        if !crate::websocket::is_disconnect_error(&e) {
                            error_fields!("websocket stream error", message = format!("{e:?}"));
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

async fn flush_device_stream_events(observer: &StreamObserverClient, device_stream: &DeviceStreamClient, stream: &mut DuplexStream) -> Result<(), Box<dyn Error + Send + Sync>> {
    let pending_events = device_stream.take_pending_events(observer.device_id()).await?;
    for (index, pending_event) in pending_events.iter().enumerate() {
        if let Err(error) = observer.send_event(stream, pending_event.event.clone()).await {
            device_stream.restore_events(observer.device_id(), &pending_events[index..]).await?;
            return Err(error);
        }
    }
    Ok(())
}
