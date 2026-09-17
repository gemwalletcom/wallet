use chrono::{DateTime, Utc};
use num_bigint::BigUint;

use crate::{Asset, RedemptionResult, RedemptionStatus, RewardRedemption, RewardRedemptionOption, RewardRedemptionType, RewardStatus, Rewards};

impl Rewards {
    pub fn mock(code: Option<&str>, status: RewardStatus) -> Self {
        Self {
            code: code.map(str::to_string),
            status,
            ..Self::default()
        }
    }

    pub fn mock_pending(verify_after: DateTime<Utc>) -> Self {
        Self {
            used_referral_code: Some("friend".to_string()),
            verify_after: Some(verify_after),
            ..Self::mock(None, RewardStatus::Pending)
        }
    }
}

impl RewardRedemptionOption {
    pub fn mock(asset: Option<Asset>) -> Self {
        Self {
            id: "option-1".to_string(),
            redemption_type: RewardRedemptionType::Asset,
            points: 100,
            asset,
            value: BigUint::from(1u32),
            remaining: None,
        }
    }
}

impl RedemptionResult {
    pub fn mock(asset: Option<Asset>) -> Self {
        Self {
            redemption: RewardRedemption {
                id: 7,
                option: RewardRedemptionOption::mock(asset),
                status: RedemptionStatus::Completed,
                transaction_id: None,
                created_at: DateTime::UNIX_EPOCH,
            },
        }
    }
}
