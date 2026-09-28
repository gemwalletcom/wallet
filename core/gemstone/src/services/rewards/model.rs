use crate::formatted_number::GemFormattedNumber;
use crate::models::list::{GemListRow, GemListSection};
use crate::models::state::GemLoadState;
use primitives::{AssetId, Rewards, Wallet, WalletId};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemRewardsIntroItem {
    InviteFriends,
    EarnPoints,
    GetRewards,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemRewardsInviteAction {
    CreateCode,
    Share,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemRewardsPendingReferral {
    pub code: String,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRewardsState {
    pub intro: Vec<GemRewardsIntroItem>,
    pub invite_action: Option<GemRewardsInviteAction>,
    pub can_use_referral_code: bool,
    pub pending_referral: Option<GemRewardsPendingReferral>,
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

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemRewardsWallets {
    pub wallets: Vec<Wallet>,
    pub can_choose: bool,
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
            intro: vec![GemRewardsIntroItem::InviteFriends, GemRewardsIntroItem::EarnPoints, GemRewardsIntroItem::GetRewards],
            invite_action: None,
            can_use_referral_code: false,
            pending_referral: None,
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
