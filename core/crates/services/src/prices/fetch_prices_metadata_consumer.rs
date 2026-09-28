use std::error::Error;
use std::sync::Arc;

use crate::ConfigCacher;
use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use config_keys::{ConfigKey, ConfigParamKey};
use gem_tracing::info_with_fields;
use prices::{AssetPriceMapping, PriceProviders};
use primitives::PriceId;
use storage::{AssetFilter, AssetUpdate, AssetsLinksRepository, AssetsRepository, Database, DatabaseError, PricesProvidersRepository, PricesRepository};
use streamer::consumer::MessageConsumer;

pub struct FetchPricesMetadataConsumer {
    pub database: Database,
    pub cacher: CacherClient,
    pub config: Arc<ConfigCacher>,
    pub providers: PriceProviders,
}

#[async_trait]
impl MessageConsumer<PriceId, usize> for FetchPricesMetadataConsumer {
    async fn should_process(&self, price_id: &PriceId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let providers = self.database.run(PricesProvidersRepository::get_prices_providers).await?;
        Ok(providers.into_iter().any(|provider| provider.provider == price_id.provider && provider.enabled))
    }

    async fn process(&self, price_id: PriceId) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.providers.get(&price_id.provider).ok_or_else(|| format!("Metadata provider unavailable: {}", price_id.provider))?;
        let id = price_id.to_string();
        let retry = self.config.get_duration(ConfigKey::PriceMetadataRetryInterval).await?.as_secs();
        self.cacher.set_cached(CacheKey::PriceMetadata(&id, retry), &price_id).await?;
        let price_ids = vec![id.clone()];
        let mappings: Vec<_> = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let asset_ids = client.get_prices_assets_for_price_ids(price_ids)?.into_iter().map(|row| row.asset_id.to_string()).collect();
                client.get_asset_ids_by_filter(vec![AssetFilter::Ids(asset_ids), AssetFilter::IsEnabled(true)])
            })
            .await?
            .into_iter()
            .map(|asset_id| AssetPriceMapping::new(asset_id, price_id.provider_price_id.clone()))
            .collect();
        if mappings.is_empty() {
            return Ok(0);
        }
        let metadata = provider.get_assets_metadata(mappings).await?;
        let count = metadata.len();
        self.database
            .run(move |client| -> Result<_, DatabaseError> {
                for asset in metadata {
                    client.update_assets(vec![asset.asset_id.clone()], vec![AssetUpdate::Rank(asset.rank)])?;
                    client.add_assets_links(&asset.asset_id, asset.links)?;
                }
                Ok(())
            })
            .await?;
        let cooldown = if count == 0 {
            self.config.get_duration(ConfigKey::PriceMissingCooldown).await?
        } else {
            self.config.get_param_duration(&ConfigParamKey::PriceProviderAssetsMetadataDuration(price_id.provider)).await?
        };
        self.cacher.set_cached(CacheKey::PriceMetadata(&id, cooldown.as_secs()), &price_id).await?;
        info_with_fields!("update price metadata", price_id = id, count = count);
        Ok(count)
    }
}
