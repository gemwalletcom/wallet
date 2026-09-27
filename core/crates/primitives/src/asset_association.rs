use model_derive::Model;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

use crate::AssetId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Model)]
#[model(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct AssetAssociation {
    pub asset_id: AssetId,
    #[serde(rename = "type")]
    pub association_type: AssetAssociationType,
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString, PartialEq, Eq, Model)]
#[model(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AssetAssociationType {
    Official,
    Bridged,
    Wrapped,
}
