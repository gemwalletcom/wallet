use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;

use crate::ConfigCacher;
use crate::prices::PriceClient;
use cacher::{CacheKey, CacherClient};
use config_keys::ConfigKey;
use gem_tracing::info_with_fields;
use prices::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProviderAsset};
use primitives::{AssetId, PriceData, PriceId};
use storage::{AssetFilter, AssetSupply, AssetUpdate, AssetsRepository, Database, DatabaseClient, DatabaseError, PriceFilter, PricesRepository};
use streamer::{PricesPayload, QueueName, StreamProducer, StreamProducerQueue};

const BATCH_SIZE: usize = 1000;

pub struct PricesUpdater {
    provider: Arc<dyn PriceAssetsProvider>,
    database: Database,
    price_client: PriceClient,
    stream_producer: StreamProducer,
}

impl PricesUpdater {
    pub fn new(provider: Arc<dyn PriceAssetsProvider>, database: Database, price_client: PriceClient, stream_producer: StreamProducer) -> Self {
        Self {
            provider,
            database,
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

    pub async fn publish_assets_metadata(&self, cacher: &CacherClient, config: &ConfigCacher) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.provider.provider();
        let (mappings, enabled) = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let mappings = client.get_prices_assets_by_provider(provider)?;
                let asset_ids = mappings.iter().map(|mapping| mapping.asset_id.to_string()).collect();
                let enabled: HashSet<_> = client.get_asset_ids_by_filter(vec![AssetFilter::Ids(asset_ids), AssetFilter::IsEnabled(true)])?.into_iter().collect();
                Ok((mappings, enabled))
            })
            .await?;
        let retry = config.get_duration(ConfigKey::PriceMetadataRetryInterval).await?.as_secs();
        let mut ids: Vec<_> = mappings.into_iter().filter(|mapping| enabled.contains(&mapping.asset_id)).map(|mapping| mapping.price_id).collect();
        ids.sort_by_cached_key(PriceId::id);
        ids.dedup();
        let keys = ids.iter().map(|id| CacheKey::PriceMetadata(&id.to_string(), retry).key()).collect();
        let cooling_down: HashSet<PriceId> = cacher.get_values(keys).await?;
        let ids: Vec<_> = ids.into_iter().filter(|id| !cooling_down.contains(id)).take(config.get_usize(ConfigKey::PriceMetadataBatchSize).await?).collect();
        for id in &ids {
            cacher.set_cached(CacheKey::PriceMetadata(&id.to_string(), retry), id).await?;
            if !self.stream_producer.publish(QueueName::FetchPricesMetadata, id).await? {
                return Err(format!("Metadata publish rejected for {id}").into());
            }
        }
        info_with_fields!("publish prices metadata", provider = provider.id(), count = ids.len());
        Ok(ids.len())
    }

    pub async fn update_prices_all(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.provider.provider();
        let mappings = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let prices = client.get_prices_by_filter(vec![PriceFilter::Provider(provider)])?;
                asset_price_mappings(client, prices)
            })
            .await?;
        self.update_prices(mappings).await
    }

    pub async fn update_prices_window(&self, offset: usize, limit: usize) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = self.provider.provider();
        let mappings = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let prices: Vec<PriceData> = client.get_prices_by_filter(vec![PriceFilter::Provider(provider)])?.into_iter().skip(offset).take(limit).collect();
                asset_price_mappings(client, prices)
            })
            .await?;
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
            let existing: HashMap<String, AssetSupply> = self
                .database
                .run(move |client| client.get_assets_supply(asset_ids))
                .await?
                .into_iter()
                .map(|(asset_id, supply)| (asset_id.to_string(), supply))
                .collect();
            let (known, missing): (Vec<&PriceProviderAsset>, Vec<&PriceProviderAsset>) = chunk.iter().partition(|a| existing.contains_key(&a.mapping.asset_id.to_string()));

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

            let supply_updates: Vec<_> = assets_by_id.iter().filter_map(|(id, asset)| asset_supply_update(asset, existing.get(id)?)).collect();
            self.store_asset_updates(supply_updates).await?;
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

    async fn store_asset_updates(&self, updates: Vec<(AssetId, AssetUpdate)>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.database
            .run(move |client| -> Result<(), DatabaseError> {
                for (asset_id, update) in updates {
                    client.update_assets(vec![asset_id], vec![update])?;
                }
                Ok(())
            })
            .await?;
        Ok(())
    }
}

fn asset_price_mappings(client: &mut DatabaseClient, prices: Vec<PriceData>) -> Result<Vec<AssetPriceMapping>, DatabaseError> {
    if prices.is_empty() {
        return Ok(vec![]);
    }

    let price_ids = prices.into_iter().map(|price| price.id.to_string()).collect();
    Ok(client
        .get_prices_assets_for_price_ids(price_ids)?
        .into_iter()
        .map(|mapping| AssetPriceMapping::new(mapping.asset_id, mapping.price_id.provider_price_id))
        .collect())
}

fn asset_supply_update(asset: &PriceProviderAsset, current: &AssetSupply) -> Option<(AssetId, AssetUpdate)> {
    let market = asset.market.as_ref()?;
    let circulating = market.circulating_supply.filter(|v| *v > 0.0).or(current.circulating);
    let total = market.total_supply.filter(|v| *v > 0.0).or(current.total);
    let max = market.max_supply.filter(|v| *v > 0.0).or(current.max);
    if circulating == current.circulating && total == current.total && max == current.max {
        return None;
    }
    Some((asset.mapping.asset_id.clone(), AssetUpdate::supply(circulating, total, max)?))
}
