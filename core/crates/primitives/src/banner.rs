use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

use crate::{Asset, Chain, Wallet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Banner {
    pub wallet: Option<Wallet>,
    pub asset: Option<Asset>,
    pub chain: Option<Chain>,
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
