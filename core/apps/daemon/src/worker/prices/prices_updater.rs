use gem_tracing::info_with_fields;
use pricer::PriceClient;
use prices::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProviderAsset, PriceProviderAssetMetadata};
use primitives::{AssetId, PriceData};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use storage::database::prices::PriceFilter;
use storage::models::{AssetRow, PriceRow};
use storage::{AssetUpdate, AssetsLinksRepository, AssetsRepository, Database, PricesRepository};
use streamer::{PricesPayload, StreamProducer, StreamProducerQueue};

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

    pub async fn update_assets(&self, limit: usize) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        if limit == 0 {
            return Ok(0);
        }
        self.save_assets(self.provider.get_assets(limit).await?).await
    }

    pub async fn update_assets_new(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        self.save_assets(self.provider.get_assets_new().await?).await
    }

    pub async fn update_assets_metadata(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.provider.provider();
        let prices = self.database.prices()?.get_prices_by_filter(vec![PriceFilter::Provider(provider)])?;
        let mappings = self.get_enabled_asset_price_mappings(prices)?;
        if mappings.is_empty() {
            return Ok(0);
        }

        let mut updated = 0;
        for batch in metadata_batches(mappings) {
            updated += self.save_assets_metadata(self.provider.get_assets_metadata(batch).await?)?;
        }

        info_with_fields!("update prices assets metadata", provider = provider.id(), count = updated);
        Ok(updated)
    }

    pub async fn update_prices_all(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.provider.provider();
        let prices = self.database.prices()?.get_prices_by_filter(vec![PriceFilter::Provider(provider)])?;
        let mappings = self.get_asset_price_mappings(prices)?;
        self.update_prices(mappings).await
    }

    pub async fn update_prices_window(&self, offset: usize, limit: usize) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.provider.provider();
        let prices: Vec<PriceRow> = self
            .database
            .prices()?
            .get_prices_by_filter(vec![PriceFilter::Provider(provider)])?
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        if prices.is_empty() {
            return Ok(0);
        }
        let mappings = self.get_asset_price_mappings(prices)?;
        self.update_prices(mappings).await
    }

    pub async fn update_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        if mappings.is_empty() {
            return Ok(0);
        }
        self.publish_prices(self.provider.get_prices(mappings).await?).await
    }

    fn get_asset_price_mappings(&self, prices: Vec<PriceRow>) -> Result<Vec<AssetPriceMapping>, Box<dyn std::error::Error + Send + Sync>> {
        if prices.is_empty() {
            return Ok(vec![]);
        }

        let provider_price_ids_by_price_id: HashMap<String, String> = prices.into_iter().map(|price| (price.id.to_string(), price.provider_price_id().to_string())).collect();
        let asset_rows = self
            .database
            .prices()?
            .get_prices_assets_for_price_ids(provider_price_ids_by_price_id.keys().cloned().collect())?;

        Ok(asset_rows
            .into_iter()
            .filter_map(|row| {
                provider_price_ids_by_price_id
                    .get(&row.price_id.to_string())
                    .cloned()
                    .map(|provider_price_id| AssetPriceMapping::new(row.asset_id.0, provider_price_id))
            })
            .collect())
    }

    fn get_enabled_asset_price_mappings(&self, prices: Vec<PriceRow>) -> Result<Vec<AssetPriceMapping>, Box<dyn std::error::Error + Send + Sync>> {
        let mappings = self.get_asset_price_mappings(prices)?;
        let asset_ids = mappings.iter().map(|mapping| mapping.asset_id.clone()).collect();
        let enabled: HashSet<AssetId> = self
            .database
            .assets()?
            .get_assets_rows(asset_ids)?
            .into_iter()
            .filter(|asset| asset.is_enabled)
            .map(|asset| asset.as_asset_id())
            .collect();
        Ok(mappings.into_iter().filter(|mapping| enabled.contains(&mapping.asset_id)).collect())
    }

    async fn save_assets(&self, assets: Vec<PriceProviderAsset>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.provider.provider();
        let mut saved = 0;
        let mut queued = 0;

        for chunk in assets.chunks(BATCH_SIZE) {
            let asset_ids: Vec<AssetId> = chunk.iter().map(|a| a.mapping.asset_id.clone()).collect();
            let existing: HashMap<String, AssetRow> = self.database.assets()?.get_assets_rows(asset_ids)?.into_iter().map(|a| (a.id.clone(), a)).collect();
            let (known, missing): (Vec<&PriceProviderAsset>, Vec<&PriceProviderAsset>) = chunk.iter().partition(|a| existing.contains_key(&a.mapping.asset_id.to_string()));

            if !missing.is_empty() {
                self.stream_producer
                    .publish_fetch_assets(missing.iter().map(|a| a.mapping.asset_id.clone()).collect())
                    .await?;
                queued += missing.len();
            }
            if known.is_empty() {
                continue;
            }

            let assets_by_id: HashMap<String, PriceProviderAsset> = known.iter().map(|a| (a.mapping.asset_id.to_string(), (*a).clone())).collect();
            let prices: Vec<AssetPriceFull> = assets_by_id.values().cloned().map(|a| AssetPriceFull::from_provider_asset(a, provider)).collect();
            saved += self.price_client.save_prices(provider, &prices)?;

            let supply_updates: Vec<_> = assets_by_id.iter().filter_map(|(id, asset)| asset_supply_update(asset, existing.get(id)?)).collect();
            self.store_asset_updates(supply_updates)?;
        }

        info_with_fields!("update prices assets", provider = provider.id(), saved = saved, queued_for_fetch = queued);
        Ok(saved)
    }

    async fn publish_prices(&self, prices: Vec<AssetPriceFull>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        if prices.is_empty() {
            return Ok(0);
        }
        let provider = self.provider.provider();

        let payload: Vec<PriceData> = prices
            .iter()
            .map(AssetPriceFull::as_price_data)
            .map(|data| (data.id.clone(), data))
            .collect::<HashMap<_, _>>()
            .into_values()
            .collect();
        let count = payload.len();
        for chunk in payload.chunks(BATCH_SIZE) {
            self.stream_producer.publish_prices(PricesPayload::new(chunk.to_vec())).await?;
        }

        info_with_fields!("update prices", provider = provider.id(), count = count);
        Ok(count)
    }

    fn save_assets_metadata(&self, metadata: Vec<PriceProviderAssetMetadata>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let metadata_by_asset_id: HashMap<String, PriceProviderAssetMetadata> =
            metadata.into_iter().map(|asset_metadata| (asset_metadata.asset_id.to_string(), asset_metadata)).collect();

        let mut updated = 0;
        for asset_metadata in metadata_by_asset_id.values() {
            self.database
                .assets()?
                .update_assets(vec![asset_metadata.asset_id.clone()], vec![AssetUpdate::Rank(asset_metadata.rank)])?;
            self.database.assets_links()?.add_assets_links(&asset_metadata.asset_id, asset_metadata.links.clone())?;
            updated += 1;
        }
        Ok(updated)
    }

    fn store_asset_updates(&self, updates: Vec<(AssetId, AssetUpdate)>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (asset_id, update) in updates {
            self.database.assets()?.update_assets(vec![asset_id], vec![update])?;
        }
        Ok(())
    }
}

fn metadata_batches(mappings: Vec<AssetPriceMapping>) -> Vec<Vec<AssetPriceMapping>> {
    let grouped = mappings.into_iter().fold(BTreeMap::<String, Vec<AssetPriceMapping>>::new(), |mut grouped, mapping| {
        grouped.entry(mapping.provider_price_id.clone()).or_default().push(mapping);
        grouped
    });
    grouped
        .into_values()
        .collect::<Vec<_>>()
        .chunks(BATCH_SIZE)
        .map(|groups| groups.iter().flatten().cloned().collect())
        .collect()
}

fn asset_supply_update(asset: &PriceProviderAsset, current: &AssetRow) -> Option<(AssetId, AssetUpdate)> {
    let market = asset.market.as_ref()?;
    let circulating = market.circulating_supply.filter(|v| *v > 0.0).or(current.circulating_supply);
    let total = market.total_supply.filter(|v| *v > 0.0).or(current.total_supply);
    let max = market.max_supply.filter(|v| *v > 0.0).or(current.max_supply);
    if circulating == current.circulating_supply && total == current.total_supply && max == current.max_supply {
        return None;
    }
    Some((asset.mapping.asset_id.clone(), AssetUpdate::supply(circulating, total, max)?))
}

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use super::*;

    #[test]
    fn test_metadata_batches_keep_provider_price_ids_together() {
        let repeated = (0..=BATCH_SIZE).map(|index| AssetPriceMapping::new(AssetId::from_token(Chain::Ethereum, &format!("0x{index:x}")), "shared".to_string()));
        let unique = (0..BATCH_SIZE).map(|index| {
            let id = format!("price-{index}");
            AssetPriceMapping::new(AssetId::from_token(Chain::Ethereum, &format!("0x1{index:x}")), id)
        });

        let batches = metadata_batches(repeated.chain(unique).collect());
        let shared_batches = batches.iter().filter(|batch| batch.iter().any(|mapping| mapping.provider_price_id == "shared")).count();
        let unique_ids: HashSet<&str> = batches.iter().flatten().map(|mapping| mapping.provider_price_id.as_str()).collect();

        assert_eq!(batches.len(), 2);
        assert_eq!(shared_batches, 1);
        assert_eq!(unique_ids.len(), BATCH_SIZE + 1);
    }
}
