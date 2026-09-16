use crate::formatted_number::GemFormattedNumber;
use chrono::{DateTime, Utc};
use primitives::RewardRedemptionOption;

#[derive(Debug, Clone, Default, PartialEq, uniffi::Record)]
pub struct GemRewardsState {
    pub has_referral_code: bool,
    pub has_used_referral_code: bool,
    pub can_invite: bool,
    pub can_use_referral_code: bool,
    pub shows_info: bool,
    pub is_unverified: bool,
    pub has_pending_referral: bool,
    pub can_activate_pending_referral: bool,
    pub invite_reward_points: i32,
    pub referral_code: Option<String>,
    pub referral_link: Option<String>,
    pub used_referral_code: Option<String>,
    pub verify_after: Option<DateTime<Utc>>,
    pub disable_reason: Option<String>,
    pub referral_count_text: String,
    pub points_text: String,
    pub redemptions: Vec<GemRewardsRedemption>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRewardsRedemption {
    pub option: RewardRedemptionOption,
    pub can_redeem: bool,
    pub points_text: String,
    pub value: GemFormattedNumber,
}
