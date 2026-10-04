use std::error::Error;
use std::sync::Arc;

use cacher::{ThrottleCacher, ThrottledTask};
use localizer::LanguageLocalizer;
use primitives::{Asset, Chain};
use push_notification::{GorushNotification, PushNotification};
use storage::{Database, DatabaseError, DevicesRepository, WalletsRepository};
use streamer::{NotificationsPayload, StreamProducerQueue};

pub struct InactiveDevicesObserver {
    database: Database,
    throttle: Arc<dyn ThrottleCacher>,
    stream_producer: Arc<dyn StreamProducerQueue>,
}

impl InactiveDevicesObserver {
    pub fn new(database: Database, throttle: Arc<dyn ThrottleCacher>, stream_producer: Arc<dyn StreamProducerQueue>) -> Self {
        Self { database, throttle, stream_producer }
    }

    pub async fn observe(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let devices = self.database.run(|client| client.devices_inactive_days(10, 14, Some(true))).await?;
        for device in &devices {
            let device_id = device.id.clone();
            let subscriptions = self
                .database
                .run(move |client| -> Result<_, DatabaseError> {
                    let device_row_id = client.get_device_row_id(&device_id)?;
                    client.get_subscriptions(device_row_id)
                })
                .await?;
            if subscriptions.is_empty() {
                continue;
            }
            if !self.throttle.try_start(ThrottledTask::InactiveDeviceObservation { device_id: &device.id }).await? {
                continue;
            }
            let language_localizer = LanguageLocalizer::new_with_language(device.locale.as_ref());
            let asset = Asset::from_chain(Chain::Bitcoin);
            let (title, description) = language_localizer.notification_onboarding_buy_asset(&asset.name);
            if let Some(notification) = GorushNotification::from_device(device.clone(), title, description, PushNotification::new_buy_asset(asset.id)) {
                self.stream_producer.publish_notifications_observers(NotificationsPayload::new(vec![notification])).await?;
            }
        }

        Ok(devices.len())
    }
}
