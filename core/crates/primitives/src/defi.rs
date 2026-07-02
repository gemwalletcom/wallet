use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

use crate::{AssetId, Chain};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum DefiProvider {
    Zerion,
    DeBank,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, AsRefStr, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
#[typeshare(swift = "Equatable, Sendable, CaseIterable, Hashable")]
pub enum DefiPositionType {
    Lending,
    Staking,
    LiquidityPool,
    Rewards,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Equatable, Hashable, Sendable, Identifiable")]
pub struct DefiPosition {
    pub id: String,
    #[typeshare(skip)]
    pub provider: DefiProvider,
    pub chain: Chain,
    pub protocol_info: DefiProtocol,
    pub name: String,
    pub position_type: DefiPositionType,
    pub assets: Vec<DefiPositionAsset>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub struct DefiProtocol {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub struct DefiPositionAsset {
    pub asset_id: AssetId,
    pub value: String,
}
