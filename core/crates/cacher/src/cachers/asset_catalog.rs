use std::{error::Error, time::Duration};

use async_trait::async_trait;
use primitives::fiat_assets::AssetCatalog;

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait AssetCatalogCacher: Send + Sync {
    async fn asset_catalog(&self) -> Result<Option<AssetCatalog>, Box<dyn Error + Send + Sync>>;
    async fn set_asset_catalog(&self, catalog: &AssetCatalog, ttl: Duration) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn delete_asset_catalog(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl AssetCatalogCacher for CacherClient {
    async fn asset_catalog(&self) -> Result<Option<AssetCatalog>, Box<dyn Error + Send + Sync>> {
        self.get(CacheKey::AssetCatalog(0)).await
    }

    async fn set_asset_catalog(&self, catalog: &AssetCatalog, ttl: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(CacheKey::AssetCatalog(ttl.as_secs()), catalog).await
    }

    async fn delete_asset_catalog(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.delete(&[CacheKey::AssetCatalog(0)]).await?;
        Ok(())
    }
}
