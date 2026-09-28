pub mod rules;

use crate::services::error::GemServiceError;
use crate::services::localization::GemLocalizedText;
use std::sync::Arc;

use primitives::{PlatformStore, Release, is_version_higher};

use crate::services::config::GemConfigService;
use crate::services::preferences::GemPreferencesService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAppUpdateAction {
    Skip,
    Update,
}

/// The update prompt: what it says and which buttons it offers.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAppUpdateOffer {
    pub version: String,
    pub title: GemLocalizedText,
    pub description: GemLocalizedText,
    pub actions: Vec<GemAppUpdateAction>,
    pub apk_url: Option<String>,
}

#[uniffi::export]
impl GemAppUpdateOffer {
    pub fn can_skip(&self) -> bool {
        self.actions.contains(&GemAppUpdateAction::Skip)
    }
}

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

    pub async fn newest(&self, store: PlatformStore, current_version: String) -> Result<Option<Release>, GemServiceError> {
        if store == PlatformStore::Local {
            return Ok(None);
        }
        let config = self.config.get_config().await?;
        Ok(rules::newest_release(&config.releases, store, &current_version))
    }

    pub async fn check(&self, store: PlatformStore, current_version: String) -> Result<Option<GemAppUpdateOffer>, GemServiceError> {
        if store == PlatformStore::Local {
            return Ok(None);
        }
        let config = self.config.get_config().await?;
        let skipped_version = self.preferences.get_skipped_app_version();
        Ok(rules::available_update(&config.releases, store, &current_version, skipped_version.as_deref()).map(rules::update_offer))
    }

    pub fn skip(&self, offer: GemAppUpdateOffer) -> Result<(), GemServiceError> {
        if !offer.can_skip() {
            return Err(GemServiceError::InvalidInput {
                msg: format!("update {} is required", offer.version),
            });
        }
        self.preferences.set_skipped_app_version(offer.version)
    }

    pub fn is_version_higher(&self, new: String, current: String) -> bool {
        is_version_higher(new, current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::preferences::testkit::MemoryPreferencesStore;

    #[test]
    fn test_a_required_update_cannot_be_skipped() {
        let preferences = Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default())));
        let service = GemAppUpdateService::new(Arc::new(GemConfigService::mock(Arc::new(crate::testkit::TestAlienProvider::with_status(500)))), preferences.clone());

        let required = rules::update_offer(Release::new(PlatformStore::AppStore, "2.0.0".into(), true));
        assert!(service.skip(required).is_err());
        assert_eq!(preferences.get_skipped_app_version(), None);

        service.skip(rules::update_offer(Release::new(PlatformStore::AppStore, "2.1.0".into(), false))).unwrap();
        assert_eq!(preferences.get_skipped_app_version(), Some("2.1.0".to_string()));
    }
}
