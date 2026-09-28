use std::error::Error;

use primitives::Device;
use push_notification::{GorushNotification, PushNotification, PushNotificationTypes};
use pusher::PusherClient;
use storage::{Database, DatabaseError, DeviceRecord, DevicesRepository, PriceAlertsRepository, WalletRecord, WalletsRepository};

use super::admin_device::AdminDevice;
use super::wallets_client::WalletsClient;

pub enum DeviceWalletLookup {
    Found(DeviceRecord, WalletRecord),
    DeviceNotFound,
    WalletNotFound,
    WalletUnavailable,
}

#[derive(Clone)]
pub struct DevicesClient {
    database: Database,
    pusher: PusherClient,
}

impl DevicesClient {
    pub fn new(database: Database, pusher: PusherClient) -> Self {
        Self { database, pusher }
    }

    pub async fn add_device(&self, device: Device) -> Result<Device, Box<dyn Error + Send + Sync>> {
        Ok(self.database.run(move |client| client.add_device(device)).await?)
    }

    pub async fn get_device(&self, device_id: &str) -> Result<Device, Box<dyn Error + Send + Sync>> {
        let device_id = device_id.to_string();
        Ok(self.database.run(move |client| client.get_device(&device_id)).await?)
    }

    pub async fn get_admin_device(&self, device_id: &str, wallets: &WalletsClient) -> Result<AdminDevice, Box<dyn Error + Send + Sync>> {
        let device_id = device_id.to_string();
        let (device, price_alert_count) = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let device = client.get_device_record(&device_id)?;
                let price_alert_count = client.count_price_alerts_for_device_id(device.id)?;
                Ok((device, price_alert_count))
            })
            .await?;
        Ok(AdminDevice {
            price_alert_count,
            wallets: wallets.get_wallet_overviews(device.id).await?,
            device: device.device,
        })
    }

    pub async fn update_device(&self, device: Device) -> Result<Device, Box<dyn Error + Send + Sync>> {
        Ok(self.database.run(move |client| client.update_device(device)).await?)
    }

    pub async fn send_push_notification_device(&self, device_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let device = self.get_device(device_id).await?;
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
        let device_id = device_id.to_string();
        Ok(self.database.run(move |client| client.get_device_exist(&device_id)).await?)
    }

    pub async fn find_device_record(&self, device_id: &str) -> Result<Option<DeviceRecord>, DatabaseError> {
        let device_id = device_id.to_string();
        self.database.run(move |client| Ok(client.get_device_record(&device_id).ok())).await
    }

    pub async fn find_device_wallet(&self, device_id: &str, wallet_id: &str) -> Result<DeviceWalletLookup, DatabaseError> {
        let device_id = device_id.to_string();
        let wallet_id = wallet_id.to_string();
        self.database
            .run(move |client| {
                let Ok(device) = client.get_device_record(&device_id) else {
                    return Ok(DeviceWalletLookup::DeviceNotFound);
                };
                Ok(match client.get_wallet_by_device_and_identifier(device.id, &wallet_id) {
                    Ok(wallet) => DeviceWalletLookup::Found(device, wallet),
                    Err(error) if error.is_not_found() => DeviceWalletLookup::WalletNotFound,
                    Err(_) => DeviceWalletLookup::WalletUnavailable,
                })
            })
            .await
    }
}
