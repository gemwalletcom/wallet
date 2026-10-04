use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use primitives::{AssetId, AssetPriceInfo, FiatRate, PriceProvider};

use crate::{CacheKey, CacherClient};

pub fn price_channel(asset_id: &AssetId) -> String {
    CacheKey::Price(&asset_id.to_string()).key()
}

#[async_trait]
pub trait PriceCacher: Send + Sync {
    async fn set_fiat_rates(&self, rates: &[FiatRate]) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn fiat_rates(&self) -> Result<Option<Vec<FiatRate>>, Box<dyn Error + Send + Sync>>;
    async fn set_prices(&self, prices: &[AssetPriceInfo], ttl_seconds: i64) -> Result<usize, Box<dyn Error + Send + Sync>>;
    async fn prices(&self, asset_ids: &[AssetId]) -> Result<Vec<AssetPriceInfo>, Box<dyn Error + Send + Sync>>;
    async fn price(&self, asset_id: &AssetId) -> Result<Option<AssetPriceInfo>, Box<dyn Error + Send + Sync>>;
    async fn is_mapping_missing(&self, provider: PriceProvider, asset_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn set_mapping_missing(&self, provider: PriceProvider, asset_id: &str, cooldown: Duration) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl PriceCacher for CacherClient {
    async fn set_fiat_rates(&self, rates: &[FiatRate]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_cached(CacheKey::FiatRates, &rates).await
    }

    async fn fiat_rates(&self) -> Result<Option<Vec<FiatRate>>, Box<dyn Error + Send + Sync>> {
        self.get_cached_optional(CacheKey::FiatRates).await
    }

    async fn set_prices(&self, prices: &[AssetPriceInfo], ttl_seconds: i64) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let values = prices.iter().filter_map(|price| serde_json::to_string(price).ok().map(|value| (price_channel(&price.asset_id), value))).collect();
        self.set_values_with_publish(values, ttl_seconds).await
    }

    async fn prices(&self, asset_ids: &[AssetId]) -> Result<Vec<AssetPriceInfo>, Box<dyn Error + Send + Sync>> {
        let keys = asset_ids.iter().map(price_channel).collect();
        self.get_values(keys).await
    }

    async fn price(&self, asset_id: &AssetId) -> Result<Option<AssetPriceInfo>, Box<dyn Error + Send + Sync>> {
        self.get_cached_optional(CacheKey::Price(&asset_id.to_string())).await
    }

    async fn is_mapping_missing(&self, provider: PriceProvider, asset_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(self.get_cached_optional::<bool>(CacheKey::PriceMissingMapping(provider.id(), asset_id, 0)).await?.is_some())
    }

    async fn set_mapping_missing(&self, provider: PriceProvider, asset_id: &str, cooldown: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_cached(CacheKey::PriceMissingMapping(provider.id(), asset_id, cooldown.as_secs()), &true).await
    }
}
