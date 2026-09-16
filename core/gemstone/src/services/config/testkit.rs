use std::sync::Arc;

use super::GemConfigService;
use crate::alien::AlienProvider;
use crate::api::GemApiClient;
use crate::services::preferences::{GemPreferencesService, testkit::MemoryPreferencesStore};

impl GemConfigService {
    pub fn mock(provider: Arc<dyn AlienProvider>) -> Self {
        Self::new(
            Arc::new(GemApiClient::new(provider)),
            Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default()))),
        )
    }
}
