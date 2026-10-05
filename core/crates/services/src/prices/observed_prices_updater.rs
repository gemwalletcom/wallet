use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use super::repository::Repository;
use crate::prices::PriceClient;
use cacher::ObservedAssetsCacher;
use prices::AssetPriceMapping;
use primitives::{AssetId, PriceProvider};
use streamer::StreamProducerQueue;

use super::{AssetsProviders, PricesUpdater};

#[derive(Clone, Copy)]
pub struct ObservedPricesConfig {
    pub max_assets: usize,
    pub min_observers: usize,
    pub primary_price_max_age: Duration,
}

pub struct ObservedPricesUpdater {
    observed: Arc<dyn ObservedAssetsCacher>,
    repository: Arc<dyn Repository>,
    price_client: PriceClient,
    providers: AssetsProviders,
    stream_producer: Arc<dyn StreamProducerQueue>,
    config: ObservedPricesConfig,
}

impl ObservedPricesUpdater {
    pub(crate) fn new(observed: Arc<dyn ObservedAssetsCacher>, repository: Arc<dyn Repository>, price_client: PriceClient, providers: AssetsProviders, stream_producer: Arc<dyn StreamProducerQueue>, config: ObservedPricesConfig) -> Self {
        Self {
            observed,
            repository,
            price_client,
            providers,
            stream_producer,
            config,
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let asset_ids: Vec<AssetId> = self.get_observed_assets().await?.into_iter().filter_map(|id| AssetId::new(&id)).collect();
        if asset_ids.is_empty() {
            return Ok(0);
        }

        let mut by_provider: HashMap<PriceProvider, Vec<AssetPriceMapping>> = HashMap::new();
        let primary_prices = self.repository.primary_prices(asset_ids, self.config.primary_price_max_age).await?;
        for (asset_id, price) in primary_prices {
            by_provider.entry(price.provider).or_default().push(AssetPriceMapping::new(asset_id, price.provider_price_id));
        }

        let mut total = 0;
        for (provider, mappings) in by_provider {
            let Some(instance) = self.providers.get(&provider).cloned() else {
                continue;
            };
            total += PricesUpdater::new(instance, self.repository.clone(), self.price_client.clone(), self.stream_producer.clone()).update_prices(mappings).await?;
        }
        Ok(total)
    }

    async fn get_observed_assets(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        self.observed.observed_assets(self.config.min_observers, self.config.max_assets).await
    }
}
