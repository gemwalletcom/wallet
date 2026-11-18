use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Asset, AssetType, FeePriority, FeeUnitType};

pub type FeeEstimatesByPriority = BTreeMap<FeePriority, FeeEstimate>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainFeeEstimates {
    pub asset: Asset,
    pub rate_unit: FeeUnitType,
    pub block_time: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<AssetType>,
    pub transfer: FeeEstimatesByPriority,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_transfer: Option<FeeEstimatesByPriority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap: Option<FeeEstimatesByPriority>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeEstimate {
    pub base: String,
    pub priority_fee: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fiat_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeEstimateChartValue {
    pub date: DateTime<Utc>,
    pub gas_price: f64,
    pub priority_fee: f64,
}
