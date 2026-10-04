use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use primitives::PriceId;

#[async_trait]
pub trait PriceMetadataCacher: Send + Sync {
    async fn cooling_down(&self, ids: &[PriceId]) -> Result<HashSet<PriceId>, Box<dyn Error + Send + Sync>>;
    async fn start_cooldown(&self, id: &PriceId, duration: Duration) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl PriceMetadataCacher for CacherClient {
    async fn cooling_down(&self, ids: &[PriceId]) -> Result<HashSet<PriceId>, Box<dyn Error + Send + Sync>> {
        let keys = ids.iter().map(|id| CacheKey::PriceMetadata(&id.to_string(), 0).key()).collect();
        self.get_values(keys).await
    }

    async fn start_cooldown(&self, id: &PriceId, duration: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_cached(CacheKey::PriceMetadata(&id.to_string(), duration.as_secs()), id).await
    }
}
