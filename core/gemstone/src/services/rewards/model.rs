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
}
