use std::collections::HashSet;
use std::error::Error;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use primitives::PriceProvider;

#[async_trait]
pub trait ChartsHistoryStore: Send + Sync {
    async fn synced_prices(&self, provider: PriceProvider) -> Result<HashSet<String>, Box<dyn Error + Send + Sync>>;
    async fn add_synced_price(&self, provider: PriceProvider, price_id: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn remove_synced_prices(&self, provider: PriceProvider, price_ids: &[String]) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl ChartsHistoryStore for CacherClient {
    async fn synced_prices(&self, provider: PriceProvider) -> Result<HashSet<String>, Box<dyn Error + Send + Sync>> {
        Ok(self.get_set_members_cached(vec![CacheKey::ChartsHistory(provider.id()).key()]).await?.into_iter().collect())
    }

    async fn add_synced_price(&self, provider: PriceProvider, price_id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.add_to_set_cached(CacheKey::ChartsHistory(provider.id()), &[price_id.to_string()]).await?;
        Ok(())
    }

    async fn remove_synced_prices(&self, provider: PriceProvider, price_ids: &[String]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.remove_from_set_cached(CacheKey::ChartsHistory(provider.id()), price_ids).await?;
        Ok(())
    }
}
