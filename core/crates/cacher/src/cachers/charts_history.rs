use std::collections::HashSet;
use std::error::Error;

use primitives::PriceProvider;

use crate::{CacheKey, CacherClient};

impl CacherClient {
    pub async fn synced_prices(&self, provider: PriceProvider) -> Result<HashSet<String>, Box<dyn Error + Send + Sync>> {
        Ok(self.set_members(&[CacheKey::ChartsHistory(provider.id())]).await?.into_iter().flatten().collect())
    }

    pub async fn add_synced_price(&self, provider: PriceProvider, price_id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.add_to_set(CacheKey::ChartsHistory(provider.id()), &[price_id.to_string()]).await?;
        Ok(())
    }

    pub async fn remove_synced_prices(&self, provider: PriceProvider, price_ids: &[String]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.remove_from_set(CacheKey::ChartsHistory(provider.id()), price_ids).await?;
        Ok(())
    }
}
