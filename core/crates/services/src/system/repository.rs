use std::collections::HashSet;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::{ChainAddress, Device, PlatformStore, Release};
use storage::{Database, DatabaseError, DevicesRepository, ReleasesRepository, TransactionsRepository, WalletRecord, WalletsRepository};

pub(crate) struct TransactionCleanupResult {
    pub(crate) addresses: usize,
    pub(crate) transactions_addresses: usize,
    pub(crate) transactions_deleted: usize,
}

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn delete_subscriptions_after_days(&self, days: i64) -> Result<usize, DatabaseError>;
    async fn cleanup_heavy_addresses(&self, min_count: i64, limit: i64, since: NaiveDateTime) -> Result<Option<TransactionCleanupResult>, DatabaseError>;
    async fn is_update_enabled(&self, store: PlatformStore) -> Result<bool, DatabaseError>;
    async fn set_release_version(&self, store: PlatformStore, version: String) -> Result<(), DatabaseError>;
    async fn inactive_devices(&self, min_days: i64, max_days: i64, push_enabled: Option<bool>) -> Result<Vec<Device>, DatabaseError>;
    async fn device_subscriptions(&self, device_id: String) -> Result<Vec<(WalletRecord, ChainAddress)>, DatabaseError>;
}

pub(crate) struct PostgresRepository {
    database: Database,
}

impl PostgresRepository {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn delete_subscriptions_after_days(&self, days: i64) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.delete_devices_subscriptions_after_days(days)).await
    }

    async fn cleanup_heavy_addresses(&self, min_count: i64, limit: i64, since: NaiveDateTime) -> Result<Option<TransactionCleanupResult>, DatabaseError> {
        self.database
            .run(move |client| {
                let heavy_addresses = client.get_transactions_addresses(min_count, limit, since)?;
                if heavy_addresses.is_empty() {
                    return Ok(None);
                }
                client.add_subscriptions_exclude_addresses(heavy_addresses.clone())?;
                let addresses = heavy_addresses.len();
                let affected_transaction_ids = client.delete_transactions_addresses(heavy_addresses)?;
                let transactions_addresses = affected_transaction_ids.len();
                let unique_ids: Vec<i64> = affected_transaction_ids.into_iter().collect::<HashSet<_>>().into_iter().collect();
                let transactions_deleted = client.delete_orphaned_transactions(unique_ids)?;
                Ok(Some(TransactionCleanupResult {
                    addresses,
                    transactions_addresses,
                    transactions_deleted,
                }))
            })
            .await
    }

    async fn is_update_enabled(&self, store: PlatformStore) -> Result<bool, DatabaseError> {
        self.database.run(move |client| client.is_update_enabled(store)).await
    }

    async fn set_release_version(&self, store: PlatformStore, version: String) -> Result<(), DatabaseError> {
        self.database
            .run(move |client| {
                let current = client.get_releases()?.into_iter().find(|release| release.store == store).map(|release| release.version);
                if current.as_ref() != Some(&version) {
                    client.update_release(Release::new(store, version, false))?;
                }
                Ok(())
            })
            .await
    }

    async fn inactive_devices(&self, min_days: i64, max_days: i64, push_enabled: Option<bool>) -> Result<Vec<Device>, DatabaseError> {
        self.database.run(move |client| client.devices_inactive_days(min_days, max_days, push_enabled)).await
    }

    async fn device_subscriptions(&self, device_id: String) -> Result<Vec<(WalletRecord, ChainAddress)>, DatabaseError> {
        self.database
            .run(move |client| {
                let device_row_id = client.get_device_row_id(&device_id)?;
                client.get_subscriptions(device_row_id)
            })
            .await
    }
}
