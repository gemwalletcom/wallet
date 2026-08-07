use api_connector::PusherClient;
use primitives::{AdminDevice, Device, GorushNotification, PushNotification, PushNotificationTypes};
use std::error::Error;
use storage::{Database, DevicesRepository, PriceAlertsRepository, models::UpdateDeviceRow};

use super::clients::WalletsClient;

#[derive(Clone)]
pub struct DevicesClient {
    database: Database,
    pusher: PusherClient,
}

impl DevicesClient {
    pub fn new(database: Database, pusher: PusherClient) -> Self {
        Self { database, pusher }
    }

    pub fn add_device(&self, device: Device) -> Result<Device, Box<dyn Error + Send + Sync>> {
        let add_device = UpdateDeviceRow::from_primitive(device.clone());
        Ok(self.database.devices()?.add_device(add_device)?)
    }

    pub fn get_device(&self, device_id: &str) -> Result<Device, Box<dyn Error + Send + Sync>> {
        Ok(self.database.devices()?.get_device(device_id)?)
    }

    pub fn get_admin_device(&self, device_id: &str, wallets: &WalletsClient) -> Result<AdminDevice, Box<dyn Error + Send + Sync>> {
        let device = self.database.devices()?.get_device_row(device_id)?;
        Ok(AdminDevice {
            price_alert_count: self.database.price_alerts()?.count_price_alerts_for_device_id(device.id)?,
            wallets: wallets.get_wallet_overviews(device.id)?,
            device: device.as_primitive(),
        })
    }

    pub fn update_device(&self, device: Device) -> Result<Device, Box<dyn Error + Send + Sync>> {
        let update_device = UpdateDeviceRow::from_primitive(device);
        Ok(self.database.devices()?.update_device(update_device)?)
    }

    pub async fn send_push_notification_device(&self, device_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let device = self.get_device(device_id)?;
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

    pub fn is_device_registered(&self, device_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(self.database.devices()?.get_device_exist(device_id)?)
    }
}
