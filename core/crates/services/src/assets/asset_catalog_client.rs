use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use cacher::AssetCatalogCacher;
use config_keys::ConfigKey;
use gem_tracing::warn_with_fields;
use primitives::fiat_assets::AssetCatalog;
use storage::{AssetsRepository, Database, DatabaseError};
use tokio::sync::Semaphore;

use crate::ConfigCacher;

#[async_trait]
trait Repository: Send + Sync {
    async fn asset_catalog(&self) -> Result<AssetCatalog, DatabaseError>;
}

struct PostgresRepository {
    database: Database,
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn asset_catalog(&self) -> Result<AssetCatalog, DatabaseError> {
        self.database.run(AssetsRepository::get_asset_catalog).await
    }
}

pub struct AssetCatalogClient {
    repository: Arc<dyn Repository>,
    cacher: Arc<dyn AssetCatalogCacher>,
    config: Arc<ConfigCacher>,
    refresh: Semaphore,
}

impl AssetCatalogClient {
    pub(crate) fn new(database: Database, cacher: Arc<dyn AssetCatalogCacher>, config: Arc<ConfigCacher>) -> Self {
        Self::new_with_repository(Arc::new(PostgresRepository { database }), cacher, config)
    }

    fn new_with_repository(repository: Arc<dyn Repository>, cacher: Arc<dyn AssetCatalogCacher>, config: Arc<ConfigCacher>) -> Self {
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
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };
    use std::time::Duration;

    use async_trait::async_trait;
    use cacher::AssetCatalogCacher;
    use primitives::fiat_assets::AssetCatalog;

    use super::*;
    use crate::testkit::MemoryConfigRepository;

    struct MemoryRepository {
        catalog: AssetCatalog,
        reads: AtomicUsize,
    }

    #[async_trait]
    impl Repository for MemoryRepository {
        async fn asset_catalog(&self) -> Result<AssetCatalog, DatabaseError> {
            self.reads.fetch_add(1, Ordering::Relaxed);
            Ok(self.catalog.clone())
        }
    }

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
        let repository = Arc::new(MemoryRepository {
            catalog: catalog.clone(),
            reads: AtomicUsize::new(0),
        });
        let cacher = Arc::new(MemoryCacher::default());
        let client = AssetCatalogClient::new_with_repository(repository.clone(), cacher.clone(), Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::new()))));

        assert_eq!(client.get().await.unwrap(), catalog);
        assert_eq!(client.get().await.unwrap(), catalog);
        assert_eq!(repository.reads.load(Ordering::Relaxed), 1);
        assert_eq!(*cacher.ttl.lock().unwrap(), Some(Duration::from_secs(15 * 60)));
    }
}
