use crate::formatted_number::GemFormattedNumber;
use crate::models::list::{GemListRow, GemListSection};
use crate::models::state::GemLoadState;
use primitives::{AssetId, Rewards, WalletId};

use crate::services::localization::GemLocalizedText;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRewardsResult {
    pub wallet_id: WalletId,
    pub state: GemLoadState,
    pub rewards: Option<Rewards>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRewardsViewState {
    pub state: GemLoadState,
    pub rewards: GemRewardsState,
    pub is_refreshing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemRewardsAction {
    CreateCode,
    Share,
    UseReferralCode,
    ActivatePendingReferral { code: String, is_enabled: bool },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRewardsState {
    pub actions: Vec<GemRewardsAction>,
    pub error_notice: Option<GemListRow>,
    pub status_notice: Option<GemListRow>,
    pub sections: Vec<GemListSection>,
    pub invite_description: GemLocalizedText,
    pub referral_code: Option<String>,
    pub referral_link: Option<String>,
    pub share_text: Option<GemLocalizedText>,
    pub used_referral_code: Option<String>,
    pub redemptions: Vec<GemRewardsRedemption>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemIncomingCode {
    Activate { code: String },
    Confirm { code: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRewardsRedemption {
    pub id: String,
    pub asset_id: AssetId,
    pub icon: crate::services::assets::icon::GemAssetIcon,
    pub title: GemLocalizedText,
    pub can_redeem: bool,
    pub points: GemFormattedNumber,
    pub value: GemFormattedNumber,
    pub confirmation: GemLocalizedText,
}

impl Default for GemRewardsState {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
            error_notice: None,
            status_notice: None,
            sections: Vec::new(),
            invite_description: GemLocalizedText::RewardsInviteDescription { points: GemFormattedNumber::count(0) },
            referral_code: None,
            referral_link: None,
            share_text: None,
            used_referral_code: None,
            redemptions: Vec::new(),
        }
    }
}
