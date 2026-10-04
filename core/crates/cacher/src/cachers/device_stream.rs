use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use primitives::{StreamEvent, device_stream_channel};

use crate::{CacheKey, CacherClient};

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
