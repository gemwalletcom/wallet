use std::error::Error;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use config_keys::{ConfigKey, ConfigParamKey};
use gem_tracing::info_with_fields;
use prices::{AssetPriceMapping, PriceProviders};
use primitives::PriceId;
use storage::database::assets::AssetFilter;
use storage::{AssetUpdate, AssetsLinksRepository, AssetsRepository, ConfigCacher, Database, PricesProvidersRepository, PricesRepository};
use streamer::consumer::MessageConsumer;

pub struct FetchPricesMetadataConsumer {
    pub database: Database,
    pub cacher: CacherClient,
    pub config: ConfigCacher,
    pub providers: PriceProviders,
}

#[async_trait]
impl MessageConsumer<PriceId, usize> for FetchPricesMetadataConsumer {
    async fn should_process(&self, price_id: &PriceId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(self.database.prices_providers()?.get_prices_providers()?.into_iter().any(|provider| provider.id.0 == price_id.provider && provider.enabled))
    }

    async fn process(&self, price_id: PriceId) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.providers.get(&price_id.provider).ok_or_else(|| format!("Metadata provider unavailable: {}", price_id.provider))?;
        let id = price_id.to_string();
        let retry = self.config.get_duration(ConfigKey::PriceMetadataRetryInterval)?.as_secs();
        self.cacher.set_cached(CacheKey::PriceMetadata(&id, retry), &price_id).await?;
        let asset_ids = self.database.prices()?.get_prices_assets_for_price_ids(vec![id.clone()])?.into_iter().map(|row| row.asset_id.to_string()).collect();
        let mappings: Vec<_> = self
            .database
            .assets()?
            .get_asset_ids_by_filter(vec![AssetFilter::Ids(asset_ids), AssetFilter::IsEnabled(true)])?
            .into_iter()
            .map(|asset_id| AssetPriceMapping::new(asset_id, price_id.provider_price_id.clone()))
            .collect();
        if mappings.is_empty() {
            return Ok(0);
        }
        let metadata = provider.get_assets_metadata(mappings).await?;
        for asset in &metadata {
            self.database.assets()?.update_assets(vec![asset.asset_id.clone()], vec![AssetUpdate::Rank(asset.rank)])?;
            self.database.assets_links()?.add_assets_links(&asset.asset_id, asset.links.clone())?;
        }
        let cooldown = if metadata.is_empty() {
            self.config.get_duration(ConfigKey::PriceMissingCooldown)?
        } else {
            self.config.get_param_duration(&ConfigParamKey::PriceProviderAssetsMetadataDuration(price_id.provider))?
        };
        self.cacher.set_cached(CacheKey::PriceMetadata(&id, cooldown.as_secs()), &price_id).await?;
        info_with_fields!("update price metadata", price_id = id, count = metadata.len());
        Ok(metadata.len())
    }
}
