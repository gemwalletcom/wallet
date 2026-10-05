use std::error::Error;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use config_keys::ConfigKey;
use primitives::asset_score::AssetRank;
use primitives::{Asset, AssetBasic, AssetFull, AssetId};
use storage::AssetFilter;

use super::repository::Repository;
use crate::ConfigCacher;

#[derive(Clone)]
pub struct AssetsClient {
    repository: Arc<dyn Repository>,
    config: Arc<ConfigCacher>,
}

impl AssetsClient {
    pub(crate) fn new(repository: Arc<dyn Repository>, config: Arc<ConfigCacher>) -> Self {
        Self { repository, config }
    }

    pub async fn get_asset(&self, asset_id: &AssetId) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.asset(asset_id.clone()).await?)
    }

    pub async fn get_assets(&self, asset_ids: Vec<AssetId>, rate: f64) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>> {
        let max_age = self.config.get_duration(ConfigKey::PricePrimaryMaxAge).await?;
        let filters = vec![AssetFilter::Ids(asset_ids.iter().map(ToString::to_string).collect())];
        let assets = self.repository.assets_with_prices(filters, max_age).await?;
        Ok(assets.into_iter().map(|asset| asset.asset_basic_with_rate(rate)).collect())
    }

    pub async fn get_asset_full(&self, asset_id: &AssetId) -> Result<AssetFull, Box<dyn Error + Send + Sync>> {
        let max_age = self.config.get_duration(ConfigKey::PricePrimaryMaxAge).await?;
        Ok(self.repository.asset_full(asset_id.clone(), max_age).await?)
    }

    pub async fn get_assets_by_wallet_id(&self, device_id: i32, wallet_id: i32, from_timestamp: Option<u64>) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>> {
        let since = from_timestamp.and_then(|ts| DateTime::<Utc>::from_timestamp(ts as i64, 0).map(|dt| dt.naive_utc()));
        let max_age = self.config.get_duration(ConfigKey::PricePrimaryMaxAge).await?;
        let filters = vec![AssetFilter::IsEnabled(true), AssetFilter::HasPrice(true), AssetFilter::RankGt(AssetRank::Trivial.threshold())];
        let assets = self.repository.wallet_assets_with_prices(device_id, wallet_id, since, filters, max_age).await?;
        Ok(assets.into_iter().map(|asset| asset.asset.asset.id).collect())
    }
}

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use super::*;
    use crate::testkit::{MemoryAssetRepository, MemoryConfigRepository};

    #[tokio::test]
    async fn test_get_assets_queries_requested_ids_with_price_max_age() {
        let repository = Arc::new(MemoryAssetRepository::new(vec![]));
        let config = ConfigCacher::new(Arc::new(MemoryConfigRepository::new().with_value(ConfigKey::PricePrimaryMaxAge.as_ref(), "5m")));
        let client = AssetsClient::new(repository.clone(), Arc::new(config));

        client.get_assets(vec![AssetId::from_chain(Chain::Bitcoin)], 1.0).await.unwrap();

        let queries = repository.price_queries();
        assert_eq!(queries.len(), 1);
        assert_eq!(format!("{:?}", queries[0].0), format!("{:?}", vec![AssetFilter::Ids(vec!["bitcoin".to_string()])]));
        assert_eq!(queries[0].1, std::time::Duration::from_secs(300));
    }
}
