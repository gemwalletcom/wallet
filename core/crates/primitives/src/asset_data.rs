use chrono::{DateTime, Utc};
use model_derive::Model;
use serde::{Deserialize, Serialize};

use crate::{Account, Asset, AssetAssociation, AssetMetaData, Balance, Price, PriceAlert};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Model)]
#[model(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct AssetData {
    pub asset: Asset,
    pub balance: Balance,
    pub account: Account,
    pub price: Option<Price>,
    pub price_alerts: Vec<PriceAlert>,
    pub metadata: AssetMetaData,
    pub associations: Vec<AssetAssociation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Model)]
#[model(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ChainAssetData {
    pub asset_data: AssetData,
    pub fee_asset_data: AssetData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Model)]
#[model(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct RecentAsset {
    pub asset: Asset,
    pub created_at: DateTime<Utc>,
}
