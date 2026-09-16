use std::sync::Arc;

use super::GemSettingsService;
use crate::services::preferences::{GemPreferencesService, testkit::MemoryPreferencesStore};

impl GemSettingsService {
    pub fn mock() -> Self {
        Self::new(Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default()))))
    }
}
