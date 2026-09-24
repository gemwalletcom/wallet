use std::error::Error;
use std::time::Duration;

use cacher::{CacheKey, CacherClient};
use gem_tracing::error_fields;
use primitives::{StreamEvent, unix_timestamp};

pub struct PendingStreamEvent {
    value: String,
    expires_at: f64,
    pub event: StreamEvent,
}

#[derive(Clone)]
pub struct DeviceStreamClient {
    cacher: CacherClient,
    retention: Duration,
    history_limit: usize,
}

impl DeviceStreamClient {
    pub fn new(cacher: CacherClient, retention: Duration, history_limit: usize) -> Self {
        Self { cacher, retention, history_limit }
    }

    pub async fn take_pending_events(&self, device_id: &str) -> Result<Vec<PendingStreamEvent>, Box<dyn Error + Send + Sync>> {
        let now = unix_timestamp() as f64;
        let cached_events = self.cacher.take_sorted_set_with_scores(&self.cache_key(device_id).key()).await?;
        let mut pending_events = cached_events
            .into_iter()
            .filter(|(_, expires_at)| *expires_at > now)
            .filter_map(|(value, expires_at)| match serde_json::from_str::<StreamEvent>(&value) {
                Ok(event) => Some(PendingStreamEvent { value, expires_at, event }),
                Err(error) => {
                    error_fields!("invalid cached device stream event", message = format!("{error:?}"));
                    None
                }
            })
            .collect::<Vec<_>>();
        pending_events.drain(..pending_events.len().saturating_sub(self.history_limit));
        pending_events.sort_by(|left, right| left.expires_at.total_cmp(&right.expires_at).then_with(|| delivery_priority(&left.event).cmp(&delivery_priority(&right.event))));
        Ok(pending_events)
    }

    pub async fn restore_events(&self, device_id: &str, events: &[PendingStreamEvent]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let entries = events.iter().map(|event| (event.value.clone(), event.expires_at)).collect::<Vec<_>>();
        self.cacher.add_to_sorted_set_cached(self.cache_key(device_id), &entries).await?;
        Ok(())
    }

    fn cache_key<'a>(&self, device_id: &'a str) -> CacheKey<'a> {
        CacheKey::DeviceStreamEvents(device_id, self.retention.as_secs())
    }
}

fn delivery_priority(event: &StreamEvent) -> u8 {
    match event {
        StreamEvent::Transactions(_) => 0,
        StreamEvent::Balances(_) => 1,
        StreamEvent::Prices(_) | StreamEvent::PriceAlerts(_) | StreamEvent::Nft(_) | StreamEvent::Perpetual(_) | StreamEvent::InAppNotification(_) | StreamEvent::FiatTransaction(_) | StreamEvent::Support(_) | StreamEvent::Error(_) => 0,
    }
}
