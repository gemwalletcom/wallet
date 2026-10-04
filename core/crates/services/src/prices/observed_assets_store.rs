use std::error::Error;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use primitives::AssetId;

#[async_trait]
pub trait ObservedAssetsStore: Send + Sync {
    async fn track_observed_assets(&self, asset_ids: &[AssetId]) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn observed_assets(&self, min_observers: usize, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl ObservedAssetsStore for CacherClient {
    async fn track_observed_assets(&self, asset_ids: &[AssetId]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let key = CacheKey::ObservedAssets;
        let ids: Vec<String> = asset_ids.iter().map(ToString::to_string).collect();
        self.sorted_set_incr_with_expire(&key.key(), &ids, key.ttl() as i64).await
    }

    async fn observed_assets(&self, min_observers: usize, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        self.sorted_set_range_by_score(&CacheKey::ObservedAssets.key(), min_observers as f64, f64::INFINITY, limit).await
    }
}
