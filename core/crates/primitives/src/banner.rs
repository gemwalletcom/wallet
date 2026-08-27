use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

use crate::{Asset, WalletId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Banner {
    pub wallet_id: Option<WalletId>,
    pub asset: Option<Asset>,
    pub event: BannerEvent,
    pub state: BannerState,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, EnumString, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum BannerEvent {
    Stake,
    AccountActivation,
    EnableNotifications,
    AccountBlockedMultiSignature,
    ActivateAsset,
    SuspiciousAsset,
    Onboarding,
    TradePerpetuals,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, EnumString, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum BannerState {
    Active,
    Cancelled,
    AlwaysActive,
}
