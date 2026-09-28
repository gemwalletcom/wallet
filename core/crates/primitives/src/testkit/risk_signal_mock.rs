use chrono::{TimeDelta, Utc};

use crate::{Platform, RiskSignal};

impl RiskSignal {
    pub fn mock(referrer_username: &str, fingerprint: &str, ip_address: &str, ip_isp: &str, device_model: &str, device_id: i32) -> Self {
        Self {
            fingerprint: fingerprint.to_string(),
            referrer_username: referrer_username.to_string(),
            device_id,
            device_platform: Platform::IOS,
            device_model: device_model.to_string(),
            ip_address: ip_address.to_string(),
            ip_isp: ip_isp.to_string(),
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
