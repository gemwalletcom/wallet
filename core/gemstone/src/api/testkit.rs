use std::sync::Arc;

use super::GemDeviceApiClient;
use crate::alien::AlienProvider;
use crate::services::device::GemDeviceKeyService;

impl GemDeviceApiClient {
    pub fn mock(provider: Arc<dyn AlienProvider>) -> Self {
        Self::new(provider, Arc::new(GemDeviceKeyService::mock()))
    }
}
