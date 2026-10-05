use std::{error::Error, sync::Arc};

use cacher::AssetCatalogCacher;
use config_keys::ConfigKey;
use gem_tracing::warn_with_fields;
use primitives::fiat_assets::AssetCatalog;
use tokio::sync::Semaphore;

use super::repository::Repository;
use crate::ConfigCacher;

pub struct AssetCatalogClient {
    repository: Arc<dyn Repository>,
    cacher: Arc<dyn AssetCatalogCacher>,
    config: Arc<ConfigCacher>,
    refresh: Semaphore,
}

impl AssetCatalogClient {
    pub(crate) fn new(repository: Arc<dyn Repository>, cacher: Arc<dyn AssetCatalogCacher>, config: Arc<ConfigCacher>) -> Self {
        Self {
            repository,
            cacher,
            config,
            refresh: Semaphore::new(1),
        }
    }

    pub async fn get(&self) -> Result<AssetCatalog, Box<dyn Error + Send + Sync>> {
        if let Some(catalog) = self.cached().await {
            return Ok(catalog);
        }

        let _permit = self.refresh.acquire().await?;
        if let Some(catalog) = self.cached().await {
            return Ok(catalog);
        }

        let catalog = self.repository.asset_catalog().await?;
        match self.config.get_duration(ConfigKey::AssetsCatalogCacheDuration).await {
            Ok(ttl) if !ttl.is_zero() => {
                if let Err(error) = self.cacher.set_asset_catalog(&catalog, ttl).await {
                    warn_with_fields!("asset catalog cache write failed", error = error.as_ref());
                }
            }
            Ok(_) => {}
            Err(error) => warn_with_fields!("asset catalog cache duration unavailable", error = &error),
        }
        Ok(catalog)
    }

    async fn cached(&self) -> Option<AssetCatalog> {
        match self.cacher.asset_catalog().await {
            Ok(catalog) => catalog,
            Err(error) => {
                warn_with_fields!("asset catalog cache read failed", error = error.as_ref());
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use async_trait::async_trait;
    use cacher::AssetCatalogCacher;
    use primitives::fiat_assets::AssetCatalog;

    use super::*;
    use crate::testkit::{MemoryAssetRepository, MemoryConfigRepository};

    #[derive(Default)]
    struct MemoryCacher {
        catalog: Mutex<Option<AssetCatalog>>,
        ttl: Mutex<Option<Duration>>,
    }

    #[async_trait]
    impl AssetCatalogCacher for MemoryCacher {
        async fn asset_catalog(&self) -> Result<Option<AssetCatalog>, Box<dyn Error + Send + Sync>> {
            Ok(self.catalog.lock().unwrap().clone())
        }

        async fn set_asset_catalog(&self, catalog: &AssetCatalog, ttl: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
            *self.catalog.lock().unwrap() = Some(catalog.clone());
            *self.ttl.lock().unwrap() = Some(ttl);
            Ok(())
        }

        async fn delete_asset_catalog(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
            *self.catalog.lock().unwrap() = None;
            Ok(())
        }
    }

    #[tokio::test]
    async fn caches_catalog_after_first_database_read() {
        let catalog = AssetCatalog::new(vec!["bitcoin".into()], vec!["ethereum".into()], vec!["solana".into()]);
        let repository = Arc::new(MemoryAssetRepository::new(vec![]).with_catalog(catalog.clone()));
        let cacher = Arc::new(MemoryCacher::default());
        let client = AssetCatalogClient::new(repository.clone(), cacher.clone(), Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::new()))));

        assert_eq!(client.get().await.unwrap(), catalog);
        assert_eq!(client.get().await.unwrap(), catalog);
        assert_eq!(repository.catalog_reads(), 1);
        assert_eq!(*cacher.ttl.lock().unwrap(), Some(Duration::from_secs(15 * 60)));
    }
}
