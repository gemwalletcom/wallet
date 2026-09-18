use crate::duration_formatter::GemDurationPart;
use crate::formatted_number::GemFormattedNumber;
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
    pub invite_reward_points_text: String,
    pub referral_code: Option<String>,
    pub referral_link: Option<String>,
    pub used_referral_code: Option<String>,
    pub pending_countdown: Vec<GemDurationPart>,
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
