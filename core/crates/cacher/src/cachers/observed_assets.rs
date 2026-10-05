use std::error::Error;

use async_trait::async_trait;
use primitives::AssetId;

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait ObservedAssetsCacher: Send + Sync {
    async fn track_observed_assets(&self, asset_ids: &[AssetId]) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn observed_assets(&self, min_observers: usize, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl ObservedAssetsCacher for CacherClient {
    async fn track_observed_assets(&self, asset_ids: &[AssetId]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let ids: Vec<String> = asset_ids.iter().map(ToString::to_string).collect();
        self.increment_in_sorted_set(CacheKey::ObservedAssets, &ids).await
    }

    async fn observed_assets(&self, min_observers: usize, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        self.sorted_set_range_by_score(CacheKey::ObservedAssets, min_observers as f64, f64::INFINITY, limit).await
    }
}
