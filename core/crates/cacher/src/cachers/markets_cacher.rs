use std::error::Error;

use async_trait::async_trait;
use primitives::Markets;

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait MarketsCacher: Send + Sync {
    async fn markets(&self) -> Result<Option<Markets>, Box<dyn Error + Send + Sync>>;
    async fn set_markets(&self, markets: &Markets) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl MarketsCacher for CacherClient {
    async fn markets(&self) -> Result<Option<Markets>, Box<dyn Error + Send + Sync>> {
        self.get_cached_optional(CacheKey::Markets).await
    }

    async fn set_markets(&self, markets: &Markets) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_cached(CacheKey::Markets, markets).await
    }
}
