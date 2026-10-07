use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use primitives::{AssetId, AssetPriceInfo, FiatRate, PriceProvider, PriceProviderConfig};

use crate::{CacheKey, CacherClient};

pub fn price_channel(asset_id: &AssetId) -> String {
    CacheKey::Price(&asset_id.to_string()).key()
}

#[async_trait]
pub trait PriceCacher: Send + Sync {
    async fn price_providers(&self) -> Result<Option<Vec<PriceProviderConfig>>, Box<dyn Error + Send + Sync>>;
    async fn set_price_providers(&self, providers: &[PriceProviderConfig]) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn set_fiat_rates(&self, rates: &[FiatRate]) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn fiat_rates(&self) -> Result<Option<Vec<FiatRate>>, Box<dyn Error + Send + Sync>>;
    async fn set_prices(&self, prices: &[AssetPriceInfo], ttl: Duration) -> Result<usize, Box<dyn Error + Send + Sync>>;
    async fn prices(&self, asset_ids: &[AssetId]) -> Result<Vec<AssetPriceInfo>, Box<dyn Error + Send + Sync>>;
    async fn price(&self, asset_id: &AssetId) -> Result<Option<AssetPriceInfo>, Box<dyn Error + Send + Sync>>;
    async fn is_mapping_missing(&self, provider: PriceProvider, asset_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn set_mapping_missing(&self, provider: PriceProvider, asset_id: &str, cooldown: Duration) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl PriceCacher for CacherClient {
    async fn price_providers(&self) -> Result<Option<Vec<PriceProviderConfig>>, Box<dyn Error + Send + Sync>> {
        self.get(CacheKey::PriceProviders).await
    }

    async fn set_price_providers(&self, providers: &[PriceProviderConfig]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(CacheKey::PriceProviders, &providers).await
    }

    async fn set_fiat_rates(&self, rates: &[FiatRate]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(CacheKey::FiatRates, &rates).await
    }

    async fn fiat_rates(&self) -> Result<Option<Vec<FiatRate>>, Box<dyn Error + Send + Sync>> {
        self.get(CacheKey::FiatRates).await
    }

    async fn set_prices(&self, prices: &[AssetPriceInfo], ttl: Duration) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let ids = prices.iter().map(|price| price.asset_id.to_string()).collect::<Vec<_>>();
        let entries = ids.iter().zip(prices).map(|(id, price)| (CacheKey::Price(id), price)).collect::<Vec<_>>();
        self.set_many_and_publish(&entries, ttl.as_secs()).await
    }

    async fn prices(&self, asset_ids: &[AssetId]) -> Result<Vec<AssetPriceInfo>, Box<dyn Error + Send + Sync>> {
        let ids = asset_ids.iter().map(ToString::to_string).collect::<Vec<_>>();
        let keys = ids.iter().map(|id| CacheKey::Price(id)).collect::<Vec<_>>();
        self.get_many(&keys).await
    }

    async fn price(&self, asset_id: &AssetId) -> Result<Option<AssetPriceInfo>, Box<dyn Error + Send + Sync>> {
        self.get(CacheKey::Price(&asset_id.to_string())).await
    }

    async fn is_mapping_missing(&self, provider: PriceProvider, asset_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(self.get::<bool>(CacheKey::PriceMissingMapping(provider.id(), asset_id, 0)).await?.is_some())
    }

    async fn set_mapping_missing(&self, provider: PriceProvider, asset_id: &str, cooldown: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(CacheKey::PriceMissingMapping(provider.id(), asset_id, cooldown.as_secs()), &true).await
    }
}
