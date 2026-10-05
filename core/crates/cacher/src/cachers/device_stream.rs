use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use primitives::{StreamEvent, device_stream_channel, unix_timestamp};

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait DeviceStreamCacher: Send + Sync {
    async fn publish_event(&self, device_id: &str, event: &StreamEvent) -> Result<usize, Box<dyn Error + Send + Sync>>;
    async fn take_events(&self, device_id: &str, retention: Duration) -> Result<Vec<(String, f64)>, Box<dyn Error + Send + Sync>>;
    async fn add_events(&self, device_id: &str, retention: Duration, events: &[(String, f64)]) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl DeviceStreamCacher for CacherClient {
    async fn publish_event(&self, device_id: &str, event: &StreamEvent) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.publish(&device_stream_channel(device_id), event).await
    }

    async fn take_events(&self, device_id: &str, retention: Duration) -> Result<Vec<(String, f64)>, Box<dyn Error + Send + Sync>> {
        self.take_sorted_set(events_key(device_id, retention)).await
    }

    async fn add_events(&self, device_id: &str, retention: Duration, events: &[(String, f64)]) -> Result<(), Box<dyn Error + Send + Sync>> {
        if events.is_empty() {
            return Ok(());
        }
        self.remove_from_sorted_set_up_to(events_key(device_id, retention), unix_timestamp() as f64).await?;
        self.add_to_sorted_set(events_key(device_id, retention), events).await?;
        Ok(())
    }
}

fn events_key(device_id: &str, retention: Duration) -> CacheKey<'_> {
    CacheKey::DeviceStreamEvents(device_id, retention.as_secs())
}
