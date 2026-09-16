use std::sync::Arc;

use super::GemExplorerService;
use crate::services::preferences::{GemPreferencesService, testkit::MemoryPreferencesStore};

impl GemExplorerService {
    pub fn mock() -> Self {
        Self::new(Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default()))))
    }
}
