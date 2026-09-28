use chrono::NaiveDateTime;
use serde_json::Value;

use crate::{IpUsageType, Platform, PlatformStore};

#[derive(Debug, Clone, PartialEq)]
pub struct RiskSignal {
    pub fingerprint: String,
    pub referrer_username: String,
    pub device_id: i32,
    pub device_platform: Platform,
    pub device_model: String,
    pub ip_address: String,
    pub ip_isp: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewRiskSignal {
    pub fingerprint: String,
    pub referrer_username: String,
    pub device_id: i32,
    pub device_platform: Platform,
    pub device_platform_store: PlatformStore,
    pub device_os: String,
    pub device_model: String,
    pub device_locale: String,
    pub device_currency: String,
    pub ip_address: String,
    pub ip_country_code: String,
    pub ip_usage_type: IpUsageType,
    pub ip_isp: String,
    pub ip_abuse_score: i32,
    pub risk_score: i32,
    pub user_agent: String,
    pub metadata: Value,
}
