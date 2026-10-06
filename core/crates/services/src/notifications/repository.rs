use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::{Asset, AssetId, Chain, Device, NotificationData, ScanAddress, TransactionType};
use storage::{AssetsRepository, Database, DatabaseError, DeviceFieldUpdate, DevicesRepository, NewNotification, NotificationsRepository, ScanAddressesRepository, TransactionsRepository, WalletsRepository};

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError>;
    async fn create_notification(&self, notification: NewNotification) -> Result<Vec<Device>, DatabaseError>;
    async fn device_notifications(&self, device_id: String, since: Option<NaiveDateTime>, limit: usize) -> Result<(Device, Vec<NotificationData>), DatabaseError>;
    async fn mark_all_as_read(&self, device_id: String) -> Result<usize, DatabaseError>;
    async fn update_device_fields(&self, device_ids: Vec<String>, updates: Vec<DeviceFieldUpdate>) -> Result<usize, DatabaseError>;
    async fn scan_address(&self, chain: Chain, address: String) -> Result<ScanAddress, DatabaseError>;
    async fn addresses_with_transactions(&self, chain: Chain, kinds: Vec<TransactionType>, since: NaiveDateTime) -> Result<Vec<String>, DatabaseError>;
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
    async fn asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError> {
        self.database.run(move |client| client.get_asset(&asset_id)).await
    }

    async fn create_notification(&self, notification: NewNotification) -> Result<Vec<Device>, DatabaseError> {
        self.database
            .run(move |client| {
                let wallet_id = notification.wallet_id;
                client.create_notifications(vec![notification])?;
                client.get_devices_by_wallet_id(wallet_id)
            })
            .await
    }

    async fn device_notifications(&self, device_id: String, since: Option<NaiveDateTime>, limit: usize) -> Result<(Device, Vec<NotificationData>), DatabaseError> {
        self.database
            .run(move |client| {
                let device = client.get_device(&device_id)?;
                let notifications = client.get_notifications_by_device_id(&device_id, since, limit)?;
                Ok((device, notifications))
            })
            .await
    }

    async fn mark_all_as_read(&self, device_id: String) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.mark_all_as_read(&device_id)).await
    }

    async fn update_device_fields(&self, device_ids: Vec<String>, updates: Vec<DeviceFieldUpdate>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.update_device_fields(device_ids, updates)).await
    }

    async fn scan_address(&self, chain: Chain, address: String) -> Result<ScanAddress, DatabaseError> {
        self.database.run(move |client| client.get_scan_address(chain, &address)).await
    }

    async fn addresses_with_transactions(&self, chain: Chain, kinds: Vec<TransactionType>, since: NaiveDateTime) -> Result<Vec<String>, DatabaseError> {
        self.database.run(move |client| client.get_addresses_by_chain_and_kind(chain.as_ref(), kinds, since)).await
    }
}
