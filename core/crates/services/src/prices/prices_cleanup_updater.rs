use crate::ConfigCacher;
use cacher::CacherClient;
use config_keys::ConfigKey;
use primitives::{DateTime, PriceProvider};
use std::error::Error;
use std::sync::Arc;
use storage::PriceFilter;

use super::repository::Repository;

pub struct PricesCleanupUpdater {
    repository: Arc<dyn Repository>,
    history: CacherClient,
    config: Arc<ConfigCacher>,
    provider: PriceProvider,
}

impl PricesCleanupUpdater {
    pub(crate) fn new(repository: Arc<dyn Repository>, history: CacherClient, config: Arc<ConfigCacher>, provider: PriceProvider) -> Self {
        Self { repository, history, config, provider }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let cutoff = DateTime::now() - self.config.get_duration(ConfigKey::PriceOutdated).await?;
        let (ids, deleted) = self.repository.delete_prices(vec![PriceFilter::Provider(self.provider), PriceFilter::UpdatedBefore(cutoff)]).await?;
        if ids.is_empty() {
            return Ok(0);
        }
        self.history.remove_synced_prices(self.provider, &ids).await?;
        Ok(deleted)
    }
}
