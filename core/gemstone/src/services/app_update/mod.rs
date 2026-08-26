pub mod error;
pub mod rules;

use std::sync::Arc;

use primitives::{PlatformStore, Release};

use crate::services::config::GemConfigService;
use crate::services::preferences::GemPreferencesService;

pub use error::GemAppUpdateError;

#[derive(uniffi::Object)]
pub struct GemAppUpdateService {
    config: Arc<GemConfigService>,
    preferences: Arc<GemPreferencesService>,
}

#[uniffi::export]
impl GemAppUpdateService {
    #[uniffi::constructor]
    pub fn new(config: Arc<GemConfigService>, preferences: Arc<GemPreferencesService>) -> Self {
        Self { config, preferences }
    }

    pub async fn newest(&self, store: PlatformStore, current_version: String) -> Result<Option<Release>, GemAppUpdateError> {
        if store == PlatformStore::Local {
            return Ok(None);
        }
        let config = self.config.get_config().await?;
        Ok(rules::newest_release(&config.releases, store, &current_version))
    }

    pub async fn check(&self, store: PlatformStore, current_version: String) -> Result<Option<Release>, GemAppUpdateError> {
        if store == PlatformStore::Local {
            return Ok(None);
        }
        let config = self.config.get_config().await?;
        let skipped_version = self.preferences.get_skipped_app_version()?;
        Ok(rules::available_update(&config.releases, store, &current_version, skipped_version.as_deref()))
    }

    pub fn skip(&self, version: String) -> Result<(), GemAppUpdateError> {
        Ok(self.preferences.set_skipped_app_version(version)?)
    }
}
