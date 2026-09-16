use primitives::{MINUTE, WEEK};

use crate::worker::rewards::rewards_abuse_checker::AbuseDetectionConfig;

impl AbuseDetectionConfig {
    pub fn mock() -> Self {
        Self {
            disable_threshold: 200,
            attempt_penalty: 15,
            verified_threshold_multiplier: 2.0,
            lookback: WEEK,
            min_referrals_to_evaluate: 2,
            country_rotation_threshold: 2,
            country_rotation_penalty: 50,
            ring_referrers_per_device_threshold: 2,
            ring_referrers_per_fingerprint_threshold: 2,
            ring_penalty: 80,
            device_farming_threshold: 5,
            device_farming_penalty: 10,
            velocity_window: MINUTE * 5,
            velocity_divisor: 2,
            velocity_penalty: 100,
            referral_per_user_daily: 5,
            verified_multiplier: 2,
            trusted_multiplier: 3,
            disabled_referrer_penalty: 80,
        }
    }
}
