use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::{Asset, AssetId, Chain, Device, NotificationData, ScanAddress, TransactionType};
use storage::{AssetsRepository, Database, DatabaseError, DeviceFilter, DeviceUpdate, DevicesRepository, NewNotification, NotificationsRepository, ScanAddressesRepository, TransactionsRepository, WalletsRepository};

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn get_asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError>;
    async fn add_notification(&self, notification: NewNotification) -> Result<Vec<Device>, DatabaseError>;
    async fn get_device_notifications(&self, device_id: String, since: Option<NaiveDateTime>, limit: usize) -> Result<(Device, Vec<NotificationData>), DatabaseError>;
    async fn set_notifications_read(&self, device_id: String) -> Result<usize, DatabaseError>;
    async fn update_devices(&self, filters: Vec<DeviceFilter>, updates: Vec<DeviceUpdate>) -> Result<usize, DatabaseError>;
    async fn get_scan_address(&self, chain: Chain, address: String) -> Result<ScanAddress, DatabaseError>;
    async fn get_addresses_with_transactions(&self, chain: Chain, kinds: Vec<TransactionType>, since: NaiveDateTime) -> Result<Vec<String>, DatabaseError>;
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
    async fn get_asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError> {
        self.database.run(move |client| client.get_asset(&asset_id)).await
    }

    async fn add_notification(&self, notification: NewNotification) -> Result<Vec<Device>, DatabaseError> {
        self.database
            .run(move |client| {
                let wallet_id = notification.wallet_id;
                client.add_notifications(vec![notification])?;
                client.get_devices_by_wallet_id(wallet_id)
            })
            .await
    }

    async fn get_device_notifications(&self, device_id: String, since: Option<NaiveDateTime>, limit: usize) -> Result<(Device, Vec<NotificationData>), DatabaseError> {
        self.database
            .run(move |client| {
                let device = client.get_device(&device_id)?;
                let notifications = client.get_notifications_by_device_id(&device_id, since, limit)?;
                Ok((device, notifications))
            })
            .await
    }

    async fn set_notifications_read(&self, device_id: String) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.set_notifications_read(&device_id)).await
    }

    async fn update_devices(&self, filters: Vec<DeviceFilter>, updates: Vec<DeviceUpdate>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.update_devices(filters, updates)).await
    }

    async fn get_scan_address(&self, chain: Chain, address: String) -> Result<ScanAddress, DatabaseError> {
        self.database.run(move |client| client.get_scan_address(chain, &address)).await
    }

    async fn get_addresses_with_transactions(&self, chain: Chain, kinds: Vec<TransactionType>, since: NaiveDateTime) -> Result<Vec<String>, DatabaseError> {
        self.database.run(move |client| client.get_addresses_by_chain_and_kind(chain.as_ref(), kinds, since)).await
    }
}
