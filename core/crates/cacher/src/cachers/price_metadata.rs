use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use primitives::PriceId;

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait PriceMetadataCacher: Send + Sync {
    async fn cooling_down(&self, ids: &[PriceId]) -> Result<HashSet<PriceId>, Box<dyn Error + Send + Sync>>;
    async fn start_cooldown(&self, id: &PriceId, duration: Duration) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl PriceMetadataCacher for CacherClient {
    async fn cooling_down(&self, ids: &[PriceId]) -> Result<HashSet<PriceId>, Box<dyn Error + Send + Sync>> {
        let ids = ids.iter().map(ToString::to_string).collect::<Vec<_>>();
        let keys = ids.iter().map(|id| CacheKey::PriceMetadata(id, 0)).collect::<Vec<_>>();
        Ok(self.get_many(&keys).await?.into_iter().collect())
    }

    async fn start_cooldown(&self, id: &PriceId, duration: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(CacheKey::PriceMetadata(&id.to_string(), duration.as_secs()), id).await
    }
}
