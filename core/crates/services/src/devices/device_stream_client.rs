use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use gem_tracing::error_fields;
use primitives::{StreamEvent, device_stream_channel, unix_timestamp};

#[async_trait]
pub trait DeviceStreamCacher: Send + Sync {
    async fn publish_event(&self, device_id: &str, event: &StreamEvent) -> Result<usize, Box<dyn Error + Send + Sync>>;
    async fn events(&self, device_id: &str, retention: Duration) -> Result<Vec<(String, f64)>, Box<dyn Error + Send + Sync>>;
    async fn take_events(&self, device_id: &str, retention: Duration) -> Result<Vec<(String, f64)>, Box<dyn Error + Send + Sync>>;
    async fn add_events(&self, device_id: &str, retention: Duration, events: &[(String, f64)]) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn remove_events(&self, device_id: &str, retention: Duration, events: &[String]) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl DeviceStreamCacher for CacherClient {
    async fn publish_event(&self, device_id: &str, event: &StreamEvent) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.publish(&device_stream_channel(device_id), event).await
    }

    async fn events(&self, device_id: &str, retention: Duration) -> Result<Vec<(String, f64)>, Box<dyn Error + Send + Sync>> {
        self.sorted_set_range_with_scores(&events_key(device_id, retention).key(), 0, -1).await
    }

    async fn take_events(&self, device_id: &str, retention: Duration) -> Result<Vec<(String, f64)>, Box<dyn Error + Send + Sync>> {
        self.take_sorted_set_with_scores(&events_key(device_id, retention).key()).await
    }

    async fn add_events(&self, device_id: &str, retention: Duration, events: &[(String, f64)]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.add_to_sorted_set_cached(events_key(device_id, retention), events).await?;
        Ok(())
    }

    async fn remove_events(&self, device_id: &str, retention: Duration, events: &[String]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.remove_from_sorted_set_cached(events_key(device_id, retention), events).await?;
        Ok(())
    }
}

fn events_key(device_id: &str, retention: Duration) -> CacheKey<'_> {
    CacheKey::DeviceStreamEvents(device_id, retention.as_secs())
}

pub struct PendingStreamEvent {
    value: String,
    expires_at: f64,
    pub event: StreamEvent,
}

#[derive(Clone)]
pub struct DeviceStreamClient {
    events: Arc<dyn DeviceStreamCacher>,
    retention: Duration,
    history_limit: usize,
}

impl DeviceStreamClient {
    pub fn new(events: Arc<dyn DeviceStreamCacher>, retention: Duration, history_limit: usize) -> Self {
        Self { events, retention, history_limit }
    }

    pub async fn take_pending_events(&self, device_id: &str) -> Result<Vec<PendingStreamEvent>, Box<dyn Error + Send + Sync>> {
        let now = unix_timestamp() as f64;
        let cached_events = self.events.take_events(device_id, self.retention).await?;
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
        self.events.add_events(device_id, self.retention, &entries).await
    }
}

fn delivery_priority(event: &StreamEvent) -> u8 {
    match event {
        StreamEvent::Transactions(_) => 0,
        StreamEvent::Balances(_) => 1,
        StreamEvent::Prices(_)
        | StreamEvent::PriceAlerts(_)
        | StreamEvent::Nft(_)
        | StreamEvent::Perpetual(_)
        | StreamEvent::InAppNotification(_)
        | StreamEvent::FiatTransaction(_)
        | StreamEvent::WalletConfiguration(_)
        | StreamEvent::Support(_)
        | StreamEvent::Error(_) => 0,
    }
}
