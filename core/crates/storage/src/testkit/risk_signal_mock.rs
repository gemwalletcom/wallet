use chrono::{TimeDelta, Utc};
use primitives::{IpUsageType, Platform, PlatformStore};

use crate::models::RiskSignalRow;

impl RiskSignalRow {
    pub fn mock(referrer_username: &str, fingerprint: &str, ip_address: &str, ip_isp: &str, device_model: &str, device_id: i32) -> Self {
        Self {
            id: 1,
            fingerprint: fingerprint.to_string(),
            referrer_username: referrer_username.to_string(),
            device_id,
            device_platform: Platform::IOS.into(),
            device_platform_store: PlatformStore::AppStore.into(),
            device_os: "18.0".to_string(),
            device_model: device_model.to_string(),
            device_locale: "en-US".to_string(),
            device_currency: "USD".to_string(),
            ip_address: ip_address.to_string(),
            ip_country_code: "US".to_string(),
            ip_usage_type: IpUsageType::Isp.into(),
            ip_isp: ip_isp.to_string(),
            ip_abuse_score: 0,
            risk_score: 0,
            user_agent: String::new(),
            metadata: None,
            created_at: Utc::now().naive_utc() - TimeDelta::hours(1),
        }
    }

    pub fn mock_recent(referrer_username: &str, seconds_ago: i64) -> Self {
        Self {
            created_at: Utc::now().naive_utc() - TimeDelta::seconds(seconds_ago),
            ..Self::mock(referrer_username, "fp", "10.0.0.1", "ISP", "Model", 2)
        }
    }
}
