use model_derive::Model;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

use crate::{Asset, WalletId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model)]
#[model(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Banner {
    pub wallet_id: Option<WalletId>,
    pub asset: Option<Asset>,
    pub event: BannerEvent,
    pub state: BannerState,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, EnumIter, EnumString, PartialEq, Eq, Hash, Model)]
#[model(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum BannerEvent {
    Stake,
    AccountActivation,
    AccountBlockedMultiSignature,
    ActivateAsset,
    SuspiciousAsset,
    Onboarding,
    TradePerpetuals,
}

impl BannerEvent {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, EnumString, PartialEq, Eq, Hash, Model)]
#[model(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum BannerState {
    Active,
    Cancelled,
    AlwaysActive,
}
