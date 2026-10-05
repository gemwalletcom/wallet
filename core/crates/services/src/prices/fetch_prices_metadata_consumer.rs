use std::error::Error;
use std::sync::Arc;

use crate::ConfigCacher;
use async_trait::async_trait;
use cacher::PriceMetadataCacher;
use config_keys::{ConfigKey, ConfigParamKey};
use gem_tracing::info_with_fields;
use prices::{AssetPriceMapping, PriceProviders};
use primitives::PriceId;
use storage::AssetFilter;
use streamer::consumer::MessageConsumer;

use super::repository::Repository;

pub struct FetchPricesMetadataConsumer {
    pub(crate) repository: Arc<dyn Repository>,
    pub cooldowns: Arc<dyn PriceMetadataCacher>,
    pub config: Arc<ConfigCacher>,
    pub providers: PriceProviders,
}

#[async_trait]
impl MessageConsumer<PriceId, usize> for FetchPricesMetadataConsumer {
    async fn should_consume(&self, price_id: &PriceId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let providers = self.repository.price_providers().await?;
        Ok(providers.into_iter().any(|provider| provider.provider == price_id.provider && provider.enabled))
    }

    async fn consume(&self, price_id: PriceId) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.providers.get(&price_id.provider).ok_or_else(|| format!("Metadata provider unavailable: {}", price_id.provider))?;
        let id = price_id.to_string();
        let retry = self.config.get_duration(ConfigKey::PriceMetadataRetryInterval).await?;
        self.cooldowns.start_cooldown(&price_id, retry).await?;
        let mappings: Vec<_> = self
            .repository
            .price_asset_ids(id.clone(), vec![AssetFilter::IsEnabled(true)])
            .await?
            .into_iter()
            .map(|asset_id| AssetPriceMapping::new(asset_id, price_id.provider_price_id.clone()))
            .collect();
        if mappings.is_empty() {
            return Ok(0);
        }
        let metadata = provider.get_assets_metadata(mappings).await?;
        let count = metadata.len();
        self.repository.update_assets_metadata(metadata).await?;
        let cooldown = if count == 0 {
            self.config.get_duration(ConfigKey::PriceMissingCooldown).await?
        } else {
            self.config.get_param_duration(&ConfigParamKey::PriceProviderAssetsMetadataDuration(price_id.provider)).await?
        };
        self.cooldowns.start_cooldown(&price_id, cooldown).await?;
        info_with_fields!("update price metadata", price_id = id, count = count);
        Ok(count)
    }
}
