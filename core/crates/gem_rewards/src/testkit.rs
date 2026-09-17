use primitives::rewards::RewardStatus;
use primitives::{IpUsageType, Platform, PlatformStore};

use crate::{IpCheckResult, RiskScoringInput, RiskSignalInput};

impl RiskScoringInput {
    pub fn mock() -> Self {
        Self {
            username: "user1".to_string(),
            device_id: 1,
            device_platform: Platform::IOS,
            device_platform_store: PlatformStore::AppStore,
            device_os: "18.0".to_string(),
            device_model: "iPhone15,2".to_string(),
            device_locale: "en-US".to_string(),
            device_currency: "USD".to_string(),
            ip_result: IpCheckResult {
                ip_address: "192.168.1.1".to_string(),
                country_code: "US".to_string(),
                confidence_score: 0,
                is_tor: false,
                is_vpn: false,
                usage_type: IpUsageType::Isp,
                isp: "Comcast".to_string(),
            },
            referrer_status: RewardStatus::Unverified,
            referrer_referral_count: 2,
            user_agent: String::new(),
        }
    }
}

impl RiskSignalInput {
    pub fn mock() -> Self {
        RiskScoringInput::mock().to_signal_input()
    }
}
