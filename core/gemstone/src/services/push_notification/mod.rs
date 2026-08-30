pub mod model;
pub mod rules;

pub use model::GemPushNotification;

#[derive(Debug, Default, uniffi::Object)]
pub struct GemPushNotificationService {}

#[uniffi::export]
impl GemPushNotificationService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, notification_type: String, data: Option<String>) -> Option<GemPushNotification> {
        rules::notification(&notification_type, data.as_deref())
    }
}
