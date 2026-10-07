use std::sync::Mutex;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::{Asset, AssetId, Chain, Device, NotificationData, ScanAddress, TransactionType};
use storage::{DatabaseError, DeviceFieldUpdate, NewNotification};

use crate::notifications::repository::Repository;

#[derive(Default)]
pub(crate) struct MemoryNotificationsRepository {
    device_updates: Mutex<Vec<(Vec<String>, Vec<DeviceFieldUpdate>)>>,
}

impl MemoryNotificationsRepository {
    pub(crate) fn device_updates(&self) -> Vec<(Vec<String>, Vec<DeviceFieldUpdate>)> {
        self.device_updates.lock().unwrap().clone()
    }
}

#[async_trait]
impl Repository for MemoryNotificationsRepository {
    async fn asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError> {
        Err(DatabaseError::not_found("Asset", asset_id.to_string()))
    }

    async fn create_notification(&self, _notification: NewNotification) -> Result<Vec<Device>, DatabaseError> {
        Ok(vec![])
    }

    async fn device_notifications(&self, device_id: String, _since: Option<NaiveDateTime>, _limit: usize) -> Result<(Device, Vec<NotificationData>), DatabaseError> {
        Err(DatabaseError::not_found("Device", device_id))
    }

    async fn mark_all_as_read(&self, _device_id: String) -> Result<usize, DatabaseError> {
        Ok(0)
    }

    async fn update_device_fields(&self, device_ids: Vec<String>, updates: Vec<DeviceFieldUpdate>) -> Result<usize, DatabaseError> {
        let count = device_ids.len();
        self.device_updates.lock().unwrap().push((device_ids, updates));
        Ok(count)
    }

    async fn scan_address(&self, _chain: Chain, address: String) -> Result<ScanAddress, DatabaseError> {
        Err(DatabaseError::not_found("ScanAddress", address))
    }

    async fn addresses_with_transactions(&self, _chain: Chain, _kinds: Vec<TransactionType>, _since: NaiveDateTime) -> Result<Vec<String>, DatabaseError> {
        Ok(vec![])
    }
}
