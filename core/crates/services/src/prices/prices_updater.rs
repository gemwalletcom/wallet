use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;

use crate::ConfigCacher;
use crate::prices::PriceClient;
use cacher::PriceMetadataCacher;
use config_keys::ConfigKey;
use gem_tracing::info_with_fields;
use prices::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProviderAsset};
use primitives::{AssetId, PriceData, PriceId};
use storage::AssetFilter;
use streamer::{PricesPayload, StreamProducerQueue};

use super::repository::Repository;

const BATCH_SIZE: usize = 1000;

pub struct PricesUpdater {
    provider: Arc<dyn PriceAssetsProvider>,
    repository: Arc<dyn Repository>,
    price_client: PriceClient,
    stream_producer: Arc<dyn StreamProducerQueue>,
}

impl PricesUpdater {
    pub(crate) fn new(provider: Arc<dyn PriceAssetsProvider>, repository: Arc<dyn Repository>, price_client: PriceClient, stream_producer: Arc<dyn StreamProducerQueue>) -> Self {
        Self {
            provider,
            repository,
            price_client,
            stream_producer,
        }
    }

    pub async fn update_assets(&self, limit: usize) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if limit == 0 {
            return Ok(0);
        }
        self.save_assets(self.provider.get_assets(limit).await?).await
    }

    pub async fn update_assets_new(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.save_assets(self.provider.get_assets_new().await?).await
    }

    pub async fn publish_assets_metadata(&self, cooldowns: &dyn PriceMetadataCacher, config: &ConfigCacher) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.provider.provider();
        let (mappings, enabled) = self.repository.provider_price_assets(provider, vec![AssetFilter::IsEnabled(true)]).await?;
        let retry = config.get_duration(ConfigKey::PriceMetadataRetryInterval).await?;
        let mut ids: Vec<_> = mappings.into_iter().filter(|mapping| enabled.contains(&mapping.asset_id)).map(|mapping| mapping.price_id).collect();
        ids.sort_by_cached_key(PriceId::id);
        ids.dedup();
        let cooling_down = cooldowns.cooling_down(&ids).await?;
        let ids: Vec<_> = ids.into_iter().filter(|id| !cooling_down.contains(id)).take(config.get_usize(ConfigKey::PriceMetadataBatchSize).await?).collect();
        for id in &ids {
            cooldowns.start_cooldown(id, retry).await?;
            if !self.stream_producer.publish_fetch_prices_metadata(id.clone()).await? {
                return Err(format!("Metadata publish rejected for {id}").into());
            }
        }
        info_with_fields!("publish prices metadata", provider = provider.id(), count = ids.len());
        Ok(ids.len())
    }

    pub async fn update_prices_all(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.provider.provider();
        let mappings = self.repository.price_mappings(provider, None).await?;
        self.update_prices(mappings).await
    }

    pub async fn update_prices_window(&self, offset: usize, limit: usize) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.provider.provider();
        let mappings = self.repository.price_mappings(provider, Some((offset, limit))).await?;
        self.update_prices(mappings).await
    }

    pub async fn update_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if mappings.is_empty() {
            return Ok(0);
        }
        self.publish_prices(self.provider.get_prices(mappings).await?).await
    }

    async fn save_assets(&self, assets: Vec<PriceProviderAsset>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.provider.provider();
        let mut saved = 0;
        let mut queued = 0;

        for chunk in assets.chunks(BATCH_SIZE) {
            let asset_ids: Vec<AssetId> = chunk.iter().map(|a| a.mapping.asset_id.clone()).collect();
            let existing: HashSet<AssetId> = self.repository.asset_ids(vec![AssetFilter::Ids(asset_ids.iter().map(ToString::to_string).collect())]).await?.into_iter().collect();
            let (known, missing): (Vec<&PriceProviderAsset>, Vec<&PriceProviderAsset>) = chunk.iter().partition(|asset| existing.contains(&asset.mapping.asset_id));

            if !missing.is_empty() {
                self.stream_producer.publish_fetch_assets(missing.iter().map(|a| a.mapping.asset_id.clone()).collect()).await?;
                queued += missing.len();
            }
            if known.is_empty() {
                continue;
            }

            let assets_by_id: HashMap<String, PriceProviderAsset> = known.iter().map(|a| (a.mapping.asset_id.to_string(), (*a).clone())).collect();
            let prices: Vec<AssetPriceFull> = assets_by_id.values().cloned().map(|a| AssetPriceFull::from_provider_asset(a, provider)).collect();
            saved += self.price_client.save_prices(provider, &prices).await?;
        }

        info_with_fields!("update prices assets", provider = provider.id(), saved = saved, queued_for_fetch = queued);
        Ok(saved)
    }

    async fn publish_prices(&self, prices: Vec<AssetPriceFull>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if prices.is_empty() {
            return Ok(0);
        }
        let provider = self.provider.provider();

        let payload: Vec<PriceData> = prices.iter().map(AssetPriceFull::as_price_data).map(|data| (data.id.clone(), data)).collect::<HashMap<_, _>>().into_values().collect();
        let count = payload.len();
        for chunk in payload.chunks(BATCH_SIZE) {
            self.stream_producer.publish_prices(PricesPayload::new(chunk.to_vec())).await?;
        }

        info_with_fields!("update prices", provider = provider.id(), count = count);
        Ok(count)
    }
}
