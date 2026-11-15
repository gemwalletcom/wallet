use std::error::Error;
use std::mem;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use cacher::DeviceStreamCacher;
use primitives::StreamEvent;

#[derive(Default)]
pub(crate) struct MemoryDeviceStreamCacher {
    events: Mutex<Vec<(String, f64)>>,
}

#[async_trait]
impl DeviceStreamCacher for MemoryDeviceStreamCacher {
    async fn publish_event(&self, device_id: &str, _event: &StreamEvent) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.events.lock().unwrap().push((device_id.to_string(), 0.0));
        Ok(1)
    }

    async fn take_events(&self, _device_id: &str, _retention: Duration) -> Result<Vec<(String, f64)>, Box<dyn Error + Send + Sync>> {
        Ok(mem::take(&mut *self.events.lock().unwrap()))
    }

    async fn add_events(&self, _device_id: &str, _retention: Duration, events: &[(String, f64)]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.events.lock().unwrap().extend_from_slice(events);
        Ok(())
    }
}
