use serde::{Deserialize, Serialize};

use crate::notification::PushNotification;
use primitives::Device;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GorushNotifications {
    pub notifications: Vec<GorushNotification>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GorushNotification {
    pub tokens: Vec<String>,
    pub platform: i32,
    pub title: String,
    pub message: String,
    pub topic: Option<String>,
    pub data: PushNotification,
    pub device_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
}

impl GorushNotification {
    pub fn from_device(device: Device, title: String, message: String, data: PushNotification) -> Option<Self> {
        if !device.can_receive_push_notification() {
            return None;
        }
        Some(Self {
            tokens: vec![device.token.clone()],
            platform: device.platform.as_i32(),
            title,
            message,
            topic: None,
            data,
            device_id: device.id,
            dry_run: None,
        })
    }

    pub fn for_token_validation(token: String, platform: i32) -> Self {
        Self {
            tokens: vec![token],
            platform,
            title: String::new(),
            message: String::new(),
            topic: None,
            data: PushNotification {
                notification_type: crate::notification::PushNotificationTypes::Test,
                data: None,
            },
            device_id: String::new(),
            dry_run: Some(true),
        }
    }

    pub fn with_topic(mut self, topic: Option<String>) -> Self {
        self.topic = topic;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedNotification {
    pub notification: GorushNotification,
    pub error: PushErrorLog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushErrorLog {
    pub token: String,
    pub error: String,
}

impl PushErrorLog {
    pub fn new(token: String, error: String) -> Self {
        Self { token, error }
    }

    pub fn is_device_invalid(&self) -> bool {
        const ERROR_PATTERNS: &[&str] = &[
            "notregistered",
            "unregistered",
            "invalidregistration",
            "baddevicetoken",
            "devicetokennotfortopic",
            "mismatchsenderid",
            "requested entity was not found",
        ];

        let error_lower = self.error.to_lowercase();
        ERROR_PATTERNS.iter().any(|pattern| error_lower.contains(pattern))
    }
}

#[cfg(test)]
mod tests {
    use primitives::{Currency, DeviceLocale, Platform, PlatformStore};

    use super::*;

    fn device(is_push_enabled: bool, token: &str) -> Device {
        Device {
            id: "test-device-id".to_string(),
            platform: Platform::IOS,
            platform_store: PlatformStore::AppStore,
            os: "iOS 17.0".to_string(),
            model: "iPhone 15".to_string(),
            token: token.to_string(),
            locale: DeviceLocale::EN,
            version: "1.0.0".to_string(),
            currency: Currency::USD,
            is_push_enabled,
            is_price_alerts_enabled: Some(true),
            subscriptions_version: 1,
        }
    }

    #[test]
    fn from_device() {
        let result = GorushNotification::from_device(device(true, "test-token-123"), "title".to_string(), "msg".to_string(), PushNotification::mock());
        assert!(result.is_some());

        let notification = result.unwrap();
        assert_eq!(notification.tokens, vec!["test-token-123"]);
        assert_eq!(notification.title, "title");
        assert_eq!(notification.message, "msg");
        assert_eq!(notification.device_id, "test-device-id");

        assert!(GorushNotification::from_device(device(false, "token"), "t".to_string(), "m".to_string(), PushNotification::mock()).is_none());
        assert!(GorushNotification::from_device(device(true, ""), "t".to_string(), "m".to_string(), PushNotification::mock()).is_none());
    }

    #[test]
    fn is_device_invalid() {
        assert!(PushErrorLog::new("test".to_string(), "Unregistered".to_string()).is_device_invalid());
        assert!(PushErrorLog::new("test".to_string(), "InvalidRegistration".to_string()).is_device_invalid());
        assert!(PushErrorLog::new("test".to_string(), "BadDeviceToken".to_string()).is_device_invalid());
        assert!(PushErrorLog::new("test".to_string(), "Devicetokennotfortopic".to_string()).is_device_invalid());
        assert!(PushErrorLog::new("test".to_string(), "Mismatchsenderid".to_string()).is_device_invalid());

        assert!(!PushErrorLog::new("".to_string(), "Good".to_string()).is_device_invalid());
    }
}
