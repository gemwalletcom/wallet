use std::error::Error;
use std::time::Duration;

use primitives::{StreamEvent, device_stream_channel, unix_timestamp};

use crate::{CacheKey, CacherClient};

impl CacherClient {
    pub async fn publish_event(&self, device_id: &str, event: &StreamEvent) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.publish(&device_stream_channel(device_id), event).await
    }

    pub async fn take_events(&self, device_id: &str, retention: Duration) -> Result<Vec<(String, f64)>, Box<dyn Error + Send + Sync>> {
        self.take_sorted_set(events_key(device_id, retention)).await
    }

    pub async fn add_events(&self, device_id: &str, retention: Duration, events: &[(String, f64)]) -> Result<(), Box<dyn Error + Send + Sync>> {
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
