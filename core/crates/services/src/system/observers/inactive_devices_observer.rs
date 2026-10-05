use std::error::Error;
use std::sync::Arc;

use crate::system::repository::Repository;
use cacher::{ThrottleCacher, ThrottledTask};
use localizer::LanguageLocalizer;
use primitives::{Asset, Chain};
use push_notification::{GorushNotification, PushNotification};
use streamer::{NotificationsPayload, StreamProducerQueue};

pub struct InactiveDevicesObserver {
    repository: Arc<dyn Repository>,
    throttle: Arc<dyn ThrottleCacher>,
    stream_producer: Arc<dyn StreamProducerQueue>,
}

impl InactiveDevicesObserver {
    pub(crate) fn new(repository: Arc<dyn Repository>, throttle: Arc<dyn ThrottleCacher>, stream_producer: Arc<dyn StreamProducerQueue>) -> Self {
        Self { repository, throttle, stream_producer }
    }

    pub async fn observe(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let devices = self.repository.inactive_devices(10, 14, Some(true)).await?;
        for device in &devices {
            let subscriptions = self.repository.device_subscriptions(device.id.clone()).await?;
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
