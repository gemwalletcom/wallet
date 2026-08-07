use std::error::Error;
use std::time::Duration;

use cacher::{CacheKey, CacherClient};
use gem_tracing::{error_fields, info_with_fields};
use primitives::{StreamEvent, WebSocketPricePayload, unix_timestamp};
use rocket::futures::StreamExt;
use rocket::serde::json::serde_json;
use rocket_ws::stream::DuplexStream;

use super::client::StreamObserverClient;

pub async fn new_stream(redis_url: &str, cacher_client: &CacherClient, retention: Duration, history_limit: usize, observer: &mut StreamObserverClient, stream: DuplexStream) {
    let Ok((mut stream, mut redis_connection, mut rx)) = crate::websocket::setup_ws_resources(redis_url, stream).await else {
        error_fields!("websocket failed to setup redis connection");
        return;
    };
    info_with_fields!("websocket device stream connected", status = "ok");

    if let Err(e) = observer.subscribe_device_channel(&mut redis_connection).await {
        error_fields!("websocket failed to subscribe device channel", message = format!("{e:?}"));
        return;
    }
    if let Err(e) = flush_device_stream_events(observer, cacher_client, retention, history_limit, &mut stream).await {
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
            Some(message) = rx.recv() => {
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

async fn flush_device_stream_events(
    observer: &StreamObserverClient,
    cacher_client: &CacherClient,
    retention: Duration,
    history_limit: usize,
    stream: &mut DuplexStream,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let now = unix_timestamp() as f64;
    let cache_key = CacheKey::DeviceStreamEvents(observer.device_id(), retention.as_secs());
    let cached_events = cacher_client.take_sorted_set_with_scores(&cache_key.key()).await?;
    let mut pending_events = cached_events
        .into_iter()
        .filter(|(_, expires_at)| *expires_at > now)
        .filter_map(|(value, expires_at)| match serde_json::from_str::<StreamEvent>(&value) {
            Ok(event) => Some((value, expires_at, event)),
            Err(error) => {
                error_fields!("invalid cached device stream event", message = format!("{error:?}"));
                None
            }
        })
        .collect::<Vec<_>>();
    pending_events.drain(..pending_events.len().saturating_sub(history_limit));
    pending_events.sort_by(|(_, left_expiration, left_event), (_, right_expiration, right_event)| {
        let priority = |event: &StreamEvent| match event {
            StreamEvent::Transactions(_) => 0,
            StreamEvent::Balances(_) => 1,
            StreamEvent::Prices(_)
            | StreamEvent::PriceAlerts(_)
            | StreamEvent::Nft(_)
            | StreamEvent::Perpetual(_)
            | StreamEvent::InAppNotification(_)
            | StreamEvent::FiatTransaction(_)
            | StreamEvent::Support(_) => 0,
        };
        left_expiration.total_cmp(right_expiration).then_with(|| priority(left_event).cmp(&priority(right_event)))
    });
    for (index, (_, _, event)) in pending_events.iter().enumerate() {
        if let Err(error) = observer.send_event(stream, event.clone()).await {
            let remaining_events = pending_events[index..]
                .iter()
                .map(|(value, expires_at, _)| (value.clone(), *expires_at))
                .collect::<Vec<_>>();
            cacher_client.add_to_sorted_set_cached(cache_key, &remaining_events).await?;
            return Err(error);
        }
    }
    Ok(())
}
