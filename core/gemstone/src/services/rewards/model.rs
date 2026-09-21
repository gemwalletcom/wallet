use crate::formatted_number::GemFormattedNumber;
use crate::models::list::GemListRow;
use primitives::RewardRedemptionOption;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRewardsState {
    pub has_referral_code: bool,
    pub can_invite: bool,
    pub can_use_referral_code: bool,
    pub shows_info: bool,
    pub error_notice: Option<GemListRow>,
    pub status_notice: Option<GemListRow>,
    pub shows_pending_activation: bool,
    pub can_activate_pending_referral: bool,
    pub invite_reward_points: GemFormattedNumber,
    pub referral_code: Option<String>,
    pub referral_link: Option<String>,
    pub used_referral_code: Option<String>,
    pub info_rows: Vec<GemListRow>,
    pub redemptions: Vec<GemRewardsRedemption>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemIncomingCode {
    Activate { code: String },
    Confirm { code: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRewardsRedemption {
    pub option: RewardRedemptionOption,
    pub can_redeem: bool,
    pub points: GemFormattedNumber,
    pub value: GemFormattedNumber,
}

impl Default for GemRewardsState {
    fn default() -> Self {
        Self {
            has_referral_code: false,
            can_invite: false,
            can_use_referral_code: false,
            shows_info: false,
            error_notice: None,
            status_notice: None,
            shows_pending_activation: false,
            can_activate_pending_referral: false,
            invite_reward_points: GemFormattedNumber::count(0),
            referral_code: None,
            referral_link: None,
            used_referral_code: None,
            info_rows: Vec::new(),
            redemptions: Vec::new(),
        }
    }
}
