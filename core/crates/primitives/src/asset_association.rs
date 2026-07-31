use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

use crate::AssetId;

#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AssetAssociation {
    pub asset_id: AssetId,
    #[serde(rename = "type")]
    pub association_type: AssetAssociationType,
}

#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AssetAssociationType {
    Official,
    Bridged,
    Wrapped,
}
