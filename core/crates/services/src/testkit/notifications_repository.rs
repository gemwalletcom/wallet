use std::sync::Mutex;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::{Asset, AssetId, Chain, Device, NotificationData, ScanAddress, TransactionType};
use storage::{DatabaseError, DeviceFilter, DeviceUpdate, NewNotification};

use crate::notifications::repository::Repository;

#[derive(Default)]
pub(crate) struct MemoryNotificationsRepository {
    device_updates: Mutex<Vec<(Vec<DeviceFilter>, Vec<DeviceUpdate>)>>,
}

impl MemoryNotificationsRepository {
    pub(crate) fn device_updates(&self) -> Vec<(Vec<DeviceFilter>, Vec<DeviceUpdate>)> {
        self.device_updates.lock().unwrap().clone()
    }
}

#[async_trait]
impl Repository for MemoryNotificationsRepository {
    async fn get_asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError> {
        Err(DatabaseError::not_found("Asset", asset_id.to_string()))
    }

    async fn add_notification(&self, _notification: NewNotification) -> Result<Vec<Device>, DatabaseError> {
        Ok(vec![])
    }

    async fn get_device_notifications(&self, device_id: String, _since: Option<NaiveDateTime>, _limit: usize) -> Result<(Device, Vec<NotificationData>), DatabaseError> {
        Err(DatabaseError::not_found("Device", device_id))
    }

    async fn set_notifications_read(&self, _device_id: String) -> Result<usize, DatabaseError> {
        Ok(0)
    }

    async fn update_devices(&self, filters: Vec<DeviceFilter>, updates: Vec<DeviceUpdate>) -> Result<usize, DatabaseError> {
        self.device_updates.lock().unwrap().push((filters, updates));
        Ok(1)
    }

    async fn get_scan_address(&self, _chain: Chain, address: String) -> Result<ScanAddress, DatabaseError> {
        Err(DatabaseError::not_found("ScanAddress", address))
    }

    async fn get_addresses_with_transactions(&self, _chain: Chain, _kinds: Vec<TransactionType>, _since: NaiveDateTime) -> Result<Vec<String>, DatabaseError> {
        Ok(vec![])
    }
}
