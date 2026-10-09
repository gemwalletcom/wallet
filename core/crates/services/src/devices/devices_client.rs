use std::error::Error;
use std::sync::Arc;

use primitives::Device;
use push_notification::{GorushNotification, PushNotification, PushNotificationTypes};
use pusher::PushProvider;
use storage::{DatabaseError, DeviceRecord, WalletRecord};

use super::admin_device::AdminDevice;
use super::repository::Repository;
use super::wallets_client::WalletsClient;

pub enum DeviceWalletLookup {
    Found(DeviceRecord, WalletRecord),
    DeviceNotFound,
    WalletNotFound,
    WalletUnavailable,
}

#[derive(Clone)]
pub struct DevicesClient {
    repository: Arc<dyn Repository>,
    pusher: Arc<dyn PushProvider>,
}

impl DevicesClient {
    pub(crate) fn new(repository: Arc<dyn Repository>, pusher: Arc<dyn PushProvider>) -> Self {
        Self { repository, pusher }
    }

    pub async fn add_device(&self, device: Device) -> Result<Device, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.add_device(device).await?)
    }

    pub async fn get_admin_device(&self, device_id: &str, wallets: &WalletsClient) -> Result<AdminDevice, Box<dyn Error + Send + Sync>> {
        let (device, price_alert_count) = self.repository.get_device_with_price_alert_count(device_id.to_string()).await?;
        Ok(AdminDevice {
            price_alert_count,
            wallets: wallets.get_wallet_overviews(device.id).await?,
            device: device.device,
        })
    }

    pub async fn update_device(&self, device: Device) -> Result<Device, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.update_device(device).await?)
    }

    pub async fn send_push_notification_device(&self, device_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let device = self.repository.get_device(device_id.to_string()).await?;
        let notifications: Vec<_> = GorushNotification::from_device(
            device,
            "Test Notification".to_string(),
            "Test Message".to_string(),
            PushNotification {
                notification_type: PushNotificationTypes::Test,
                data: None,
            },
        )
        .into_iter()
        .collect();
        Ok(self.pusher.push_notifications(notifications).await?.response.counts > 0)
    }

    pub async fn is_device_registered(&self, device_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.get_device_exists(device_id.to_string()).await?)
    }

    pub async fn find_device_record(&self, device_id: &str) -> Result<Option<DeviceRecord>, DatabaseError> {
        self.repository.get_device_record(device_id.to_string()).await
    }

    pub async fn find_device_wallet(&self, device_id: &str, wallet_id: &str) -> Result<DeviceWalletLookup, DatabaseError> {
        self.repository.get_device_wallet(device_id.to_string(), wallet_id.to_string()).await
    }
}
