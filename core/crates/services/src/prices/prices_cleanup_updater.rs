use crate::ConfigCacher;
use cacher::{CacheKey, CacherClient};
use chrono::Utc;
use config_keys::ConfigKey;
use primitives::PriceProvider;
use std::error::Error;
use std::sync::Arc;
use storage::{Database, DatabaseError, PriceFilter, PricesRepository};

pub struct PricesCleanupUpdater {
    database: Database,
    cacher: CacherClient,
    config: Arc<ConfigCacher>,
    provider: PriceProvider,
}

impl PricesCleanupUpdater {
    pub fn new(database: Database, cacher: CacherClient, config: Arc<ConfigCacher>, provider: PriceProvider) -> Self {
        Self { database, cacher, config, provider }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let cutoff = (Utc::now() - chrono::Duration::from_std(self.config.get_duration(ConfigKey::PriceOutdated).await?)?).naive_utc();
        let provider = self.provider;
        let (ids, deleted) = self
            .database
            .run(move |client| -> Result<(Vec<String>, usize), DatabaseError> {
                let ids: Vec<String> = client
                    .get_prices_by_filter(vec![PriceFilter::Provider(provider), PriceFilter::UpdatedBefore(cutoff)])?
                    .into_iter()
                    .map(|p| p.id.to_string())
                    .collect();
                if ids.is_empty() {
                    return Ok((ids, 0));
                }
                let deleted = client.delete_prices(ids.clone())?;
                Ok((ids, deleted))
            })
            .await?;
        if ids.is_empty() {
            return Ok(0);
        }
        self.cacher.remove_from_set_cached(CacheKey::ChartsHistory(self.provider.id()), &ids).await?;
        Ok(deleted)
    }
}
